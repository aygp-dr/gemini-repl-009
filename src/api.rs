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
    #[serde(rename = "systemInstruction", skip_serializing_if = "Option::is_none")]
    pub system_instruction: Option<Content>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Content {
    pub role: String,
    pub parts: Vec<Part>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Part {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,
    #[serde(rename = "functionCall", skip_serializing_if = "Option::is_none")]
    pub function_call: Option<FunctionCall>,
    #[serde(rename = "functionResponse", skip_serializing_if = "Option::is_none")]
    pub function_response: Option<FunctionResponse>,
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
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,
    #[serde(rename = "functionCall", skip_serializing_if = "Option::is_none")]
    pub function_call: Option<FunctionCall>,
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
    /// Creates a new Gemini client.
    /// 
    /// # Errors
    /// 
    /// Returns an error if the HTTP client cannot be created.
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
    
    /// Sends a message to the Gemini API.
    /// 
    /// # Errors
    /// 
    /// Returns an error if the API request fails or returns an error response.
    pub async fn send_message(&self, messages: &[Content]) -> Result<String> {
        self.send_message_with_tools(messages, None).await
    }
    
    /// Sends a message to the Gemini API with optional function calling tools.
    /// 
    /// # Errors
    /// 
    /// Returns an error if the API request fails or returns an error response.
    pub async fn send_message_with_tools(&self, messages: &[Content], tools: Option<Vec<Tool>>) -> Result<String> {
        // Add system instruction if tools are provided
        let system_instruction = if tools.is_some() {
            Some(Content {
                role: "system".to_string(),
                parts: vec![Part {
                    text: Some("You have access to function calling tools. When the user asks about files, directories, or code, you MUST use the appropriate tool: read_file for reading files, list_files for listing directories, write_file for creating files, and search_code for searching. Always prefer using tools over explaining that you cannot access files.".to_string()),
                    function_call: None,
                    function_response: None,
                }],
            })
        } else {
            None
        };
        
        let request = GenerateRequest {
            contents: messages.to_vec(),
            tools,
            system_instruction,
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
                    // Check for function call
                    if let Some(function_call) = &part.function_call {
                        return Ok(format!("FUNCTION_CALL: {} with args: {}", 
                            function_call.name, 
                            function_call.args.as_ref()
                                .map_or_else(|| "{}".to_string(), std::string::ToString::to_string)
                        ));
                    }
                    
                    // Otherwise return text
                    if let Some(text) = &part.text {
                        return Ok(text.trim().to_string());
                    }
                }
            }
        }
        
        Err(anyhow!("Empty response from API"))
    }
}