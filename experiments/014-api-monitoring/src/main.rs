//! Experiment: Monitor and validate actual Gemini API calls
//! This creates observability for our REPL without external dependencies

use anyhow::Result;
use chrono::Utc;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::time::Instant;
use tracing::{debug, info, warn};

#[derive(Debug, Serialize, Deserialize)]
struct ApiObservation {
    timestamp: chrono::DateTime<Utc>,
    request: RequestDetails,
    response: ResponseDetails,
    metrics: Metrics,
    validation: ValidationResult,
}

#[derive(Debug, Serialize, Deserialize)]
struct RequestDetails {
    method: String,
    endpoint: String,
    body: serde_json::Value,
    model: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct ResponseDetails {
    status: u16,
    body: serde_json::Value,
    headers: std::collections::HashMap<String, String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct Metrics {
    duration_ms: u64,
    prompt_tokens: Option<u32>,
    completion_tokens: Option<u32>,
    total_tokens: Option<u32>,
}

#[derive(Debug, Serialize, Deserialize)]
struct ValidationResult {
    is_valid: bool,
    errors: Vec<String>,
    warnings: Vec<String>,
}

struct GeminiMonitor {
    client: Client,
    api_key: String,
    log_dir: String,
}

impl GeminiMonitor {
    fn new(api_key: String) -> Result<Self> {
        let log_dir = format!("logs/gemini/{}", Utc::now().format("%Y-%m-%d"));
        fs::create_dir_all(&log_dir)?;
        
        Ok(Self {
            client: Client::new(),
            api_key,
            log_dir,
        })
    }
    
    async fn call_api(&self, body: serde_json::Value) -> Result<ApiObservation> {
        let model = "gemini-1.5-flash";
        let url = format!(
            "https://generativelanguage.googleapis.com/v1beta/models/{}:generateContent?key={}",
            model, self.api_key
        );
        
        let start = Instant::now();
        
        // Make the actual API call
        let response = self.client
            .post(&url)
            .json(&body)
            .send()
            .await?;
        
        let duration_ms = start.elapsed().as_millis() as u64;
        let status = response.status().as_u16();
        
        // Extract headers for monitoring
        let mut headers = std::collections::HashMap::new();
        for (key, value) in response.headers() {
            headers.insert(
                key.to_string(),
                value.to_str().unwrap_or("<binary>").to_string(),
            );
        }
        
        let response_body: serde_json::Value = response.json().await?;
        
        // Validate the response
        let validation = self.validate_response(&body, &response_body, status);
        
        // Extract metrics
        let metrics = self.extract_metrics(&response_body, duration_ms);
        
        let observation = ApiObservation {
            timestamp: Utc::now(),
            request: RequestDetails {
                method: "POST".to_string(),
                endpoint: format!("models/{}:generateContent", model),
                body: body.clone(),
                model: model.to_string(),
            },
            response: ResponseDetails {
                status,
                body: response_body,
                headers,
            },
            metrics,
            validation,
        };
        
        // Log the observation
        self.log_observation(&observation)?;
        
        Ok(observation)
    }
    
    fn validate_response(
        &self,
        request: &serde_json::Value,
        response: &serde_json::Value,
        status: u16,
    ) -> ValidationResult {
        let mut errors = Vec::new();
        let mut warnings = Vec::new();
        
        // Check status code
        if status != 200 {
            errors.push(format!("Non-200 status code: {}", status));
        }
        
        // Validate response structure
        if status == 200 {
            // Check for required fields
            if response.get("candidates").is_none() {
                errors.push("Missing 'candidates' field in response".to_string());
            }
            
            // Check for usage metadata
            if response.get("usageMetadata").is_none() {
                warnings.push("Missing 'usageMetadata' field".to_string());
            }
            
            // Validate candidates structure
            if let Some(candidates) = response.get("candidates").and_then(|c| c.as_array()) {
                if candidates.is_empty() {
                    errors.push("Empty candidates array".to_string());
                }
                
                for (i, candidate) in candidates.iter().enumerate() {
                    if candidate.get("content").is_none() {
                        errors.push(format!("Candidate {} missing 'content' field", i));
                    }
                }
            }
        }
        
        // Check for error responses
        if let Some(error) = response.get("error") {
            if let Some(message) = error.get("message").and_then(|m| m.as_str()) {
                errors.push(format!("API error: {}", message));
            }
        }
        
        ValidationResult {
            is_valid: errors.is_empty(),
            errors,
            warnings,
        }
    }
    
    fn extract_metrics(&self, response: &serde_json::Value, duration_ms: u64) -> Metrics {
        let mut metrics = Metrics {
            duration_ms,
            prompt_tokens: None,
            completion_tokens: None,
            total_tokens: None,
        };
        
        if let Some(usage) = response.get("usageMetadata") {
            metrics.prompt_tokens = usage.get("promptTokenCount")
                .and_then(|v| v.as_u64())
                .map(|v| v as u32);
            metrics.completion_tokens = usage.get("candidatesTokenCount")
                .and_then(|v| v.as_u64())
                .map(|v| v as u32);
            metrics.total_tokens = usage.get("totalTokenCount")
                .and_then(|v| v.as_u64())
                .map(|v| v as u32);
        }
        
        metrics
    }
    
    fn log_observation(&self, observation: &ApiObservation) -> Result<()> {
        let filename = format!("{}/observations.jsonl", self.log_dir);
        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(filename)?;
        
        writeln!(file, "{}", serde_json::to_string(observation)?)?;
        
        // Also log summary to console
        info!(
            "API call completed: status={}, duration={}ms, tokens={:?}, valid={}",
            observation.response.status,
            observation.metrics.duration_ms,
            observation.metrics.total_tokens,
            observation.validation.is_valid
        );
        
        if !observation.validation.errors.is_empty() {
            warn!("Validation errors: {:?}", observation.validation.errors);
        }
        
        Ok(())
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_env_filter("debug")
        .init();
    
    // Load API key
    let api_key = std::env::var("GEMINI_API_KEY")
        .expect("GEMINI_API_KEY must be set");
    
    let monitor = GeminiMonitor::new(api_key)?;
    
    // Test different request patterns
    let test_requests = vec![
        // Valid request
        json!({
            "contents": [{
                "role": "user",
                "parts": [{"text": "Hello, how are you?"}]
            }]
        }),
        
        // Multi-turn conversation
        json!({
            "contents": [
                {
                    "role": "user",
                    "parts": [{"text": "What is 2+2?"}]
                },
                {
                    "role": "model",
                    "parts": [{"text": "2+2 equals 4."}]
                },
                {
                    "role": "user",
                    "parts": [{"text": "Double it"}]
                }
            ]
        }),
        
        // Invalid request (missing role)
        json!({
            "contents": [{
                "parts": [{"text": "This should fail"}]
            }]
        }),
    ];
    
    for (i, request) in test_requests.into_iter().enumerate() {
        println!("\n=== Test {} ===", i + 1);
        debug!("Request: {}", serde_json::to_string_pretty(&request)?);
        
        match monitor.call_api(request).await {
            Ok(observation) => {
                println!("Status: {}", observation.response.status);
                println!("Valid: {}", observation.validation.is_valid);
                if !observation.validation.errors.is_empty() {
                    println!("Errors: {:?}", observation.validation.errors);
                }
                if !observation.validation.warnings.is_empty() {
                    println!("Warnings: {:?}", observation.validation.warnings);
                }
                println!("Metrics: {:?}", observation.metrics);
            }
            Err(e) => {
                println!("Request failed: {}", e);
            }
        }
        
        // Rate limit courtesy delay
        tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
    }
    
    println!("\n=== Summary ===");
    println!("Observations logged to: {}/observations.jsonl", monitor.log_dir);
    
    Ok(())
}