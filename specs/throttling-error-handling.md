# Throttling and Error Handling Specification

## Overview
Robust rate limiting and error handling for both Gemini and Ollama APIs.

## 1. Rate Limiting Requirements

### Gemini API Limits (Free Tier)
- **Requests per minute (RPM)**: 60
- **Requests per day (RPD)**: 1,500
- **Tokens per minute (TPM)**: 1,000,000

### Implementation Strategy
```rust
// src/rate_limiter.rs
use std::time::{Duration, Instant};
use tokio::sync::Mutex;

pub struct RateLimiter {
    requests_per_minute: u32,
    tokens_per_minute: u32,
    request_times: Mutex<Vec<Instant>>,
    token_usage: Mutex<Vec<(Instant, u32)>>,
}

impl RateLimiter {
    pub async fn check_and_wait(&self, estimated_tokens: u32) -> Result<()> {
        // Remove old entries
        self.cleanup_old_entries().await;
        
        // Check request limit
        let request_wait = self.check_request_limit().await?;
        
        // Check token limit
        let token_wait = self.check_token_limit(estimated_tokens).await?;
        
        // Wait for the longer duration
        let wait_time = request_wait.max(token_wait);
        if wait_time > Duration::ZERO {
            println!("Rate limit approaching, waiting {:.1}s...", wait_time.as_secs_f32());
            tokio::time::sleep(wait_time).await;
        }
        
        Ok(())
    }
}
```

## 2. Error Types and Handling

### Error Categories
```rust
#[derive(Debug, thiserror::Error)]
pub enum ApiError {
    #[error("Rate limit exceeded: {message}")]
    RateLimit { message: String, retry_after: Option<Duration> },
    
    #[error("Invalid request: {0}")]
    BadRequest(String),
    
    #[error("Authentication failed: {0}")]
    Unauthorized(String),
    
    #[error("Model not found: {0}")]
    ModelNotFound(String),
    
    #[error("Service unavailable: {0}")]
    ServiceUnavailable(String),
    
    #[error("Network error: {0}")]
    Network(#[from] reqwest::Error),
    
    #[error("Timeout after {0} seconds")]
    Timeout(u64),
}
```

### Error Response Parsing
```rust
pub fn parse_error_response(status: StatusCode, body: &str) -> ApiError {
    match status {
        StatusCode::TOO_MANY_REQUESTS => {
            // Extract retry-after from headers or body
            let retry_after = extract_retry_after(body);
            ApiError::RateLimit {
                message: "Rate limit exceeded".to_string(),
                retry_after,
            }
        }
        StatusCode::BAD_REQUEST => {
            ApiError::BadRequest(extract_error_message(body))
        }
        StatusCode::UNAUTHORIZED => {
            ApiError::Unauthorized("Invalid API key".to_string())
        }
        StatusCode::SERVICE_UNAVAILABLE => {
            ApiError::ServiceUnavailable("Service temporarily unavailable".to_string())
        }
        _ => ApiError::BadRequest(format!("HTTP {}: {}", status, body))
    }
}
```

## 3. Retry Strategy

### Exponential Backoff with Jitter
```rust
pub struct RetryPolicy {
    max_retries: u32,
    initial_delay: Duration,
    max_delay: Duration,
    jitter_factor: f32,
}

impl RetryPolicy {
    pub async fn execute<F, T>(&self, operation: F) -> Result<T>
    where
        F: Fn() -> Future<Output = Result<T>>,
    {
        let mut attempt = 0;
        let mut delay = self.initial_delay;
        
        loop {
            match operation().await {
                Ok(result) => return Ok(result),
                Err(e) if attempt >= self.max_retries => return Err(e),
                Err(e) if Self::is_retryable(&e) => {
                    attempt += 1;
                    
                    // Add jitter to prevent thundering herd
                    let jitter = rand::random::<f32>() * self.jitter_factor;
                    let wait_time = delay.mul_f32(1.0 + jitter);
                    
                    println!("Attempt {}/{} failed: {}. Retrying in {:.1}s...",
                        attempt, self.max_retries, e, wait_time.as_secs_f32());
                    
                    tokio::time::sleep(wait_time).await;
                    
                    // Exponential backoff
                    delay = (delay * 2).min(self.max_delay);
                }
                Err(e) => return Err(e),
            }
        }
    }
    
    fn is_retryable(error: &ApiError) -> bool {
        matches!(error,
            ApiError::RateLimit { .. } |
            ApiError::ServiceUnavailable(_) |
            ApiError::Network(_) |
            ApiError::Timeout(_)
        )
    }
}
```

## 4. Circuit Breaker Pattern

```rust
pub struct CircuitBreaker {
    failure_threshold: u32,
    recovery_timeout: Duration,
    state: Mutex<CircuitState>,
}

enum CircuitState {
    Closed,
    Open { until: Instant },
    HalfOpen,
}

impl CircuitBreaker {
    pub async fn call<F, T>(&self, operation: F) -> Result<T>
    where
        F: Future<Output = Result<T>>,
    {
        let mut state = self.state.lock().await;
        
        match *state {
            CircuitState::Open { until } if Instant::now() < until => {
                return Err(ApiError::ServiceUnavailable(
                    "Circuit breaker is open".to_string()
                ));
            }
            CircuitState::Open { .. } => {
                *state = CircuitState::HalfOpen;
            }
            _ => {}
        }
        
        drop(state);
        
        match operation.await {
            Ok(result) => {
                self.on_success().await;
                Ok(result)
            }
            Err(e) => {
                self.on_failure().await;
                Err(e)
            }
        }
    }
}
```

## 5. Ollama-Specific Handling

```rust
pub struct OllamaErrorHandler {
    // Ollama runs locally, so different error patterns
    connection_retry_delay: Duration,
    model_load_timeout: Duration,
}

impl OllamaErrorHandler {
    pub async fn handle_error(&self, error: &reqwest::Error) -> ApiError {
        if error.is_connect() {
            ApiError::ServiceUnavailable(
                "Ollama not running. Start with: ollama serve".to_string()
            )
        } else if error.is_timeout() {
            ApiError::Timeout(self.model_load_timeout.as_secs())
        } else {
            ApiError::Network(error.to_string())
        }
    }
}
```

## 6. Usage Tracking and Budgeting

```rust
pub struct UsageTracker {
    daily_limit: u32,
    minute_limit: u32,
    usage_history: Mutex<Vec<(Instant, u32)>>,
}

impl UsageTracker {
    pub async fn check_budget(&self, estimated_tokens: u32) -> Result<()> {
        let usage = self.get_current_usage().await;
        
        if usage.daily_tokens + estimated_tokens > self.daily_limit {
            return Err(ApiError::RateLimit {
                message: "Daily token budget exceeded".to_string(),
                retry_after: Some(self.time_until_reset()),
            });
        }
        
        Ok(())
    }
    
    pub async fn record_usage(&self, tokens: u32) {
        let mut history = self.usage_history.lock().await;
        history.push((Instant::now(), tokens));
        
        // Alert if approaching limits
        let usage = self.calculate_usage(&history);
        if usage.daily_tokens > self.daily_limit * 80 / 100 {
            println!("⚠️  Warning: 80% of daily token limit used");
        }
    }
}
```

## 7. Integration Example

```rust
// In GeminiClient
impl GeminiClient {
    pub async fn send_message_with_retry(&self, messages: &[Content]) -> Result<String> {
        // Check rate limits
        let estimated_tokens = self.estimate_tokens(messages);
        self.rate_limiter.check_and_wait(estimated_tokens).await?;
        
        // Check usage budget
        self.usage_tracker.check_budget(estimated_tokens).await?;
        
        // Execute with retry policy
        let response = self.retry_policy.execute(|| async {
            self.circuit_breaker.call(
                self.send_message_internal(messages)
            ).await
        }).await?;
        
        // Record usage
        if let Some(usage) = response.usage_metadata {
            self.usage_tracker.record_usage(usage.total_tokens).await;
        }
        
        Ok(response.text)
    }
}
```

## 8. Testing Scenarios

1. **Rate Limit Testing**
   - Burst requests to trigger rate limits
   - Verify backoff behavior
   - Test token limit enforcement

2. **Error Recovery Testing**
   - Simulate network failures
   - Test circuit breaker transitions
   - Verify retry exhaustion

3. **Ollama Failover**
   - Test switching to Ollama when Gemini limits hit
   - Verify seamless conversation continuation
   - Test model compatibility handling