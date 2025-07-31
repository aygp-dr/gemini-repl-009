//! Model Provider Interface
//! 
//! Defines the common interface that all model providers must implement,
//! along with supporting types for provider information and model specifications.

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

use super::{ModelCapabilities, ModelConfig, ModelResult, ModelError};

/// Core interface that all model providers must implement
#[async_trait]
pub trait ModelProvider: Send + Sync {
    /// Get provider information and supported models
    fn get_info(&self) -> ProviderInfo;
    
    /// Generate text using this provider
    async fn generate(&self, prompt: &str, config: &ModelConfig) -> ModelResult<String>;
    
    /// Stream text generation (optional, default implementation returns error)
    async fn generate_stream(&self, prompt: &str, config: &ModelConfig) -> ModelResult<Box<dyn futures::Stream<Item = ModelResult<String>> + Unpin + Send>> {
        Err(ModelError::StreamingNotSupported)
    }
    
    /// Validate provider configuration
    async fn validate_config(&self, config: &ProviderConfig) -> ModelResult<()>;
    
    /// Perform health check
    async fn health_check(&self) -> ModelResult<()>;
    
    /// Get usage statistics (optional)
    fn get_usage_stats(&self) -> Option<UsageStats> {
        None
    }
    
    /// Get rate limit information (optional)
    fn get_rate_limits(&self) -> Option<RateLimits> {
        None
    }
}

/// Information about a model provider
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderInfo {
    /// Unique provider identifier
    pub id: String,
    
    /// Human-readable provider name
    pub name: String,
    
    /// Provider description
    pub description: String,
    
    /// Provider version
    pub version: String,
    
    /// List of models supported by this provider
    pub supported_models: Vec<ModelInfo>,
    
    /// Default configuration for this provider
    pub default_config: ProviderConfig,
    
    /// Provider capabilities and features
    pub capabilities: ProviderCapabilities,
    
    /// Additional metadata
    pub metadata: HashMap<String, String>,
}

/// Information about a specific model
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelInfo {
    /// Unique model identifier
    pub id: String,
    
    /// Human-readable model name
    pub name: String,
    
    /// Model description
    pub description: String,
    
    /// Model version
    pub version: String,
    
    /// Model capabilities
    pub capabilities: ModelCapabilities,
    
    /// Default configuration for this model
    pub default_config: ModelConfig,
    
    /// Model-specific metadata
    pub metadata: HashMap<String, String>,
}

/// Provider-specific configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderConfig {
    /// API endpoint URL
    pub endpoint: String,
    
    /// Authentication configuration
    pub auth: AuthConfig,
    
    /// Connection settings
    pub connection: ConnectionConfig,
    
    /// Provider-specific settings
    pub provider_specific: HashMap<String, serde_json::Value>,
}

/// Authentication configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AuthConfig {
    /// API key authentication
    ApiKey {
        key: String,
        header_name: Option<String>,
    },
    
    /// Bearer token authentication
    BearerToken {
        token: String,
    },
    
    /// OAuth2 authentication
    OAuth2 {
        client_id: String,
        client_secret: String,
        token_url: String,
    },
    
    /// No authentication
    None,
}

/// Connection configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectionConfig {
    /// Request timeout in milliseconds
    pub timeout_ms: u64,
    
    /// Maximum concurrent requests
    pub max_concurrent: usize,
    
    /// Retry configuration
    pub retry: RetryConfig,
    
    /// Connection pool settings
    pub pool_size: Option<usize>,
}

impl Default for ConnectionConfig {
    fn default() -> Self {
        Self {
            timeout_ms: 30000,
            max_concurrent: 5,
            retry: RetryConfig::default(),
            pool_size: Some(10),
        }
    }
}

/// Retry configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetryConfig {
    /// Maximum number of retry attempts
    pub max_attempts: u32,
    
    /// Base delay between retries (milliseconds)
    pub base_delay_ms: u64,
    
    /// Maximum delay between retries (milliseconds)
    pub max_delay_ms: u64,
    
    /// Exponential backoff multiplier
    pub backoff_multiplier: f64,
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            max_attempts: 3,
            base_delay_ms: 1000,
            max_delay_ms: 30000,
            backoff_multiplier: 2.0,
        }
    }
}

/// Provider capabilities
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderCapabilities {
    /// Supports streaming responses
    pub streaming: bool,
    
    /// Supports function calling
    pub function_calling: bool,
    
    /// Supports image input
    pub image_input: bool,
    
    /// Supports image output
    pub image_output: bool,
    
    /// Supports audio input
    pub audio_input: bool,
    
    /// Supports audio output
    pub audio_output: bool,
    
    /// Supports batch processing
    pub batch_processing: bool,
    
    /// Maximum batch size (if batch processing is supported)
    pub max_batch_size: Option<usize>,
    
    /// Supported formats
    pub supported_formats: Vec<String>,
}

impl Default for ProviderCapabilities {
    fn default() -> Self {
        Self {
            streaming: false,
            function_calling: false,
            image_input: false,
            image_output: false,
            audio_input: false,
            audio_output: false,
            batch_processing: false,
            max_batch_size: None,
            supported_formats: vec!["text".to_string()],
        }
    }
}

/// Usage statistics for a provider
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsageStats {
    /// Total requests made
    pub total_requests: u64,
    
    /// Successful requests
    pub successful_requests: u64,
    
    /// Failed requests
    pub failed_requests: u64,
    
    /// Average response time (milliseconds)
    pub avg_response_time_ms: f64,
    
    /// Total tokens consumed (if applicable)
    pub total_tokens: Option<u64>,
    
    /// Total cost (if applicable)
    pub total_cost: Option<f64>,
    
    /// Last updated timestamp
    pub last_updated: chrono::DateTime<chrono::Utc>,
}

/// Rate limit information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimits {
    /// Requests per minute limit
    pub requests_per_minute: Option<u32>,
    
    /// Requests per hour limit
    pub requests_per_hour: Option<u32>,
    
    /// Requests per day limit
    pub requests_per_day: Option<u32>,
    
    /// Tokens per minute limit
    pub tokens_per_minute: Option<u32>,
    
    /// Current request count
    pub current_requests: u32,
    
    /// Current token count
    pub current_tokens: u32,
    
    /// Reset time for current window
    pub reset_time: chrono::DateTime<chrono::Utc>,
}

/// Base provider implementation with common functionality
pub struct BaseProvider {
    pub info: ProviderInfo,
    pub config: ProviderConfig,
    pub client: reqwest::Client,
    pub usage_stats: UsageStats,
}

impl BaseProvider {
    pub fn new(info: ProviderInfo, config: ProviderConfig) -> ModelResult<Self> {
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_millis(config.connection.timeout_ms))
            .build()
            .map_err(|e| ModelError::ProviderError(format!("Failed to create HTTP client: {}", e)))?;
            
        let usage_stats = UsageStats {
            total_requests: 0,
            successful_requests: 0,
            failed_requests: 0,
            avg_response_time_ms: 0.0,
            total_tokens: None,
            total_cost: None,
            last_updated: chrono::Utc::now(),
        };
        
        Ok(Self {
            info,
            config,
            client,
            usage_stats,
        })
    }
    
    /// Update usage statistics
    pub fn update_stats(&mut self, success: bool, response_time_ms: u64, tokens: Option<u64>) {
        self.usage_stats.total_requests += 1;
        
        if success {
            self.usage_stats.successful_requests += 1;
        } else {
            self.usage_stats.failed_requests += 1;
        }
        
        // Update average response time
        let total_time = self.usage_stats.avg_response_time_ms * (self.usage_stats.total_requests - 1) as f64;
        self.usage_stats.avg_response_time_ms = (total_time + response_time_ms as f64) / self.usage_stats.total_requests as f64;
        
        if let Some(tokens) = tokens {
            self.usage_stats.total_tokens = Some(
                self.usage_stats.total_tokens.unwrap_or(0) + tokens
            );
        }
        
        self.usage_stats.last_updated = chrono::Utc::now();
    }
    
    /// Perform HTTP request with retry logic
    pub async fn make_request(&self, request: reqwest::Request) -> ModelResult<reqwest::Response> {
        let mut attempts = 0;
        let retry_config = &self.config.connection.retry;
        
        loop {
            let request_clone = request.try_clone()
                .ok_or_else(|| ModelError::ProviderError("Failed to clone request".to_string()))?;
                
            match self.client.execute(request_clone).await {
                Ok(response) => {
                    if response.status().is_success() {
                        return Ok(response);
                    } else if attempts < retry_config.max_attempts && response.status().is_server_error() {
                        // Retry on server errors
                        attempts += 1;
                        let delay = std::cmp::min(
                            retry_config.base_delay_ms * (retry_config.backoff_multiplier.powi(attempts as i32 - 1) as u64),
                            retry_config.max_delay_ms
                        );
                        tokio::time::sleep(std::time::Duration::from_millis(delay)).await;
                        continue;
                    } else {
                        return Err(ModelError::ProviderError(
                            format!("HTTP error: {} - {}", response.status(), 
                                   response.text().await.unwrap_or_default())
                        ));
                    }
                }
                Err(e) if attempts < retry_config.max_attempts => {
                    attempts += 1;
                    let delay = std::cmp::min(
                        retry_config.base_delay_ms * (retry_config.backoff_multiplier.powi(attempts as i32 - 1) as u64),
                        retry_config.max_delay_ms
                    );
                    tokio::time::sleep(std::time::Duration::from_millis(delay)).await;
                    continue;
                }
                Err(e) => {
                    return Err(ModelError::ProviderError(format!("Request failed: {}", e)));
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_provider_info_serialization() {
        let info = ProviderInfo {
            id: "test-provider".to_string(),
            name: "Test Provider".to_string(),
            description: "A test provider".to_string(),
            version: "1.0.0".to_string(),
            supported_models: vec![],
            default_config: ProviderConfig {
                endpoint: "https://api.example.com".to_string(),
                auth: AuthConfig::None,
                connection: ConnectionConfig::default(),
                provider_specific: HashMap::new(),
            },
            capabilities: ProviderCapabilities::default(),
            metadata: HashMap::new(),
        };
        
        let json = serde_json::to_string(&info).unwrap();
        let deserialized: ProviderInfo = serde_json::from_str(&json).unwrap();
        
        assert_eq!(info.id, deserialized.id);
        assert_eq!(info.name, deserialized.name);
    }
    
    #[test]
    fn test_retry_config_default() {
        let config = RetryConfig::default();
        assert_eq!(config.max_attempts, 3);
        assert_eq!(config.base_delay_ms, 1000);
        assert_eq!(config.backoff_multiplier, 2.0);
    }
}