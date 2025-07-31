//! Gemini API client implementation

use anyhow::{bail, Result};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::time::Duration;

/// Gemini API client
pub struct GeminiClient {
    client: Client,
    api_key: String,
    model: String,
    base_url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Content {
    pub role: String,
    pub parts: Vec<Part>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Part {
    pub text: Option<String>,
    pub function_call: Option<FunctionCall>,
    pub function_response: Option<FunctionResponse>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionCall {
    pub name: String,
    pub args: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionResponse {
    pub name: String,
    pub response: Value,
}

#[derive(Debug, Serialize, Deserialize)]
struct GenerateContentRequest {
    contents: Vec<Content>,
    tools: Option<Vec<Value>>,
    system_instruction: Option<SystemInstruction>,
}

#[derive(Debug, Serialize, Deserialize)]
struct SystemInstruction {
    parts: Vec<SystemPart>,
}

#[derive(Debug, Serialize, Deserialize)]
struct SystemPart {
    text: String,
}

#[derive(Debug, Deserialize)]
struct GenerateContentResponse {
    candidates: Vec<Candidate>,
}

#[derive(Debug, Deserialize)]
struct Candidate {
    content: Content,
}

impl GeminiClient {
    /// Create a new Gemini client
    pub fn new(api_key: String, model: String) -> Result<Self> {
        let client = Client::builder()
            .timeout(Duration::from_secs(30))
            .build()?;
        
        Ok(Self {
            client,
            api_key,
            model,
            base_url: "https://generativelanguage.googleapis.com/v1beta".to_string(),
        })
    }
    
    /// Send a message without tools
    pub async fn send_message(&self, conversation: &[Content]) -> Result<String> {
        self.send_message_with_tools(conversation, None).await
    }
    
    /// Send a message with tool definitions
    pub async fn send_message_with_tools(&self, conversation: &[Content], tools: Option<Vec<Value>>) -> Result<String> {
        let url = format!(
            "{}/models/{}:generateContent?key={}",
            self.base_url, self.model, self.api_key
        );
        
        // Add system instruction for function calling
        let system_instruction = if tools.is_some() {
            Some(SystemInstruction {
                parts: vec![SystemPart {
                    text: "You are a helpful AI assistant with access to tools. When the user asks you to perform actions that require tools, use the available function calls to complete the request. Always provide clear explanations of what you're doing and what the results mean.".to_string(),
                }],
            })
        } else {
            None
        };
        
        let request = GenerateContentRequest {
            contents: conversation.to_vec(),
            tools,
            system_instruction,
        };
        
        tracing::debug!("Sending request to Gemini API: {}", serde_json::to_string_pretty(&request)?);
        
        let response = self
            .client
            .post(&url)
            .json(&request)
            .send()
            .await?;
        
        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await?;
            bail!("API request failed with status {}: {}", status, error_text);
        }
        
        let response_text = response.text().await?;
        tracing::debug!("Received response from Gemini API: {}", response_text);
        
        let response: GenerateContentResponse = serde_json::from_str(&response_text)?;
        
        if response.candidates.is_empty() {
            bail!("No candidates in response");
        }
        
        let candidate = &response.candidates[0];
        if candidate.content.parts.is_empty() {
            bail!("No parts in candidate content");
        }
        
        let part = &candidate.content.parts[0];
        part.text.clone().unwrap_or_else(|| "No text in response".to_string())
            .pipe(Ok)
    }
}

// Helper trait for pipeline operations
trait Pipe<T> {
    fn pipe<F, U>(self, f: F) -> U
    where
        F: FnOnce(Self) -> U,
        Self: Sized,
    {
        f(self)
    }
}

impl<T> Pipe<T> for T {}