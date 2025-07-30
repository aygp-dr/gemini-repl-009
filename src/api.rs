//! Gemini API client module

use anyhow::{Result, anyhow};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::env;
use crate::logging::ApiLogger;

#[derive(Serialize)]
pub struct GenerateRequest {
    pub contents: Vec<Content>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tools: Option<Vec<Tool>>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Content {
    pub role: String,
    pub parts: Vec<Part>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Part {
    pub text: String,
}

#[derive(Deserialize, Debug)]
pub struct GenerateResponse {
    pub candidates: Option<Vec<Candidate>>,
    pub error: Option<ApiError>,
}

#[derive(Deserialize, Debug)]
pub struct Candidate {
    pub content: ContentResponse,
}

#[derive(Deserialize, Debug)]
pub struct ContentResponse {
    pub parts: Vec<PartResponse>,
}

#[derive(Deserialize, Debug)]
pub struct PartResponse {
    pub text: String,
}

#[derive(Deserialize, Debug)]
pub struct ApiError {
    pub code: i32,
    pub message: String,
}

// Tool definitions for function calling
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Tool {
    #[serde(rename = "functionDeclarations")]
    pub function_declarations: Vec<FunctionDeclaration>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct FunctionParameters {
    #[serde(rename = "type")]
    pub param_type: String,
    pub properties: HashMap<String, ParameterProperty>,
    pub required: Vec<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ParameterProperty {
    #[serde(rename = "type")]
    pub prop_type: String,
    pub description: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub items: Option<Box<ParameterProperty>>,
}

// Function calling support
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct FunctionDeclaration {
    pub name: String,
    pub description: String,
    pub parameters: FunctionParameters,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct FunctionCall {
    pub name: String,
    pub args: Option<serde_json::Value>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct FunctionResponse {
    pub name: String,
    pub response: serde_json::Value,
}

pub struct GeminiClient {
    client: Client,
    api_key: String,
    model: String,
    logger: Option<ApiLogger>,
}

impl GeminiClient {
    pub fn new(api_key: String, model: String) -> Result<Self> {
        let mut client_builder = Client::builder();
        
        // Add proxy support if configured
        if let Ok(proxy_url) = env::var("HTTPS_PROXY") {
            tracing::info!("Using proxy: {}", proxy_url);
            client_builder = client_builder.proxy(reqwest::Proxy::https(&proxy_url)?);
        }
        
        let client = client_builder.build()?;
        
        // Create logger if logging is enabled
        let logger = if env::var("GEMINI_API_LOGGING").unwrap_or_default() == "true" {
            let log_dir = env::var("GEMINI_LOG_DIR").unwrap_or_else(|_| "logs".to_string());
            Some(ApiLogger::new(log_dir, true)?)
        } else {
            None
        };
        
        Ok(Self {
            client,
            api_key,
            model,
            logger,
        })
    }
    
    pub async fn send_message(&self, messages: &[Content]) -> Result<String> {
        self.send_message_with_tools(messages, None).await
    }
    
    pub async fn send_message_with_tools(&self, messages: &[Content], tools: Option<Vec<Tool>>) -> Result<String> {
        let request = GenerateRequest {
            contents: messages.to_vec(),
            tools,
        };
        
        let url = format!(
            "https://generativelanguage.googleapis.com/v1beta/models/{}:generateContent?key={}",
            self.model, self.api_key
        );
        
        // Log request if logger is enabled
        if let Some(ref logger) = self.logger {
            let host = "generativelanguage.googleapis.com";
            let path = format!("/v1beta/models/{}:generateContent", self.model);
            logger.log_request(
                host,
                &path,
                "POST",
                &[("Content-Type".to_string(), "application/json".to_string())],
                &serde_json::to_value(&request).unwrap_or_default(),
            )?;
        }
        
        let response = self.client
            .post(&url)
            .json(&request)
            .send()
            .await?;
        
        let status = response.status();
        let response_text = response.text().await?;
        
        // Log response if logger is enabled
        if let Some(ref logger) = self.logger {
            let host = "generativelanguage.googleapis.com";
            let path = format!("/v1beta/models/{}:generateContent", self.model);
            logger.log_response(
                host,
                &path,
                status.as_u16(),
                &serde_json::from_str::<serde_json::Value>(&response_text).unwrap_or_default(),
                100, // TODO: track actual duration
            )?;
        }
        
        let parsed: GenerateResponse = serde_json::from_str(&response_text)?;
        
        if let Some(error) = parsed.error {
            return Err(anyhow!("API Error ({}): {}", error.code, error.message));
        }
        
        if let Some(candidates) = parsed.candidates {
            if let Some(candidate) = candidates.first() {
                if let Some(part) = candidate.content.parts.first() {
                    return Ok(part.text.trim().to_string());
                }
            }
        }
        
        Err(anyhow!("Empty response from API"))
    }
}