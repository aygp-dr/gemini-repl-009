//! Gemini API client implementation

use anyhow::{bail, Result};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::Value;
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

/// Response from the Gemini API that may contain text or a function call
#[derive(Debug, Clone)]
pub enum ApiResponse {
    /// A text response
    Text(String),
    /// A function call request from the model
    FunctionCall(FunctionCall),
    /// A response with both text and function call (model explaining what it's doing)
    TextWithFunctionCall {
        text: String,
        function_call: FunctionCall,
    },
}

impl GeminiClient {
    /// Create a new Gemini client
    pub fn new(api_key: String, model: String) -> Result<Self> {
        let client = Client::builder().timeout(Duration::from_secs(30)).build()?;

        Ok(Self {
            client,
            api_key,
            model,
            base_url: "https://generativelanguage.googleapis.com/v1beta".to_string(),
        })
    }

    /// Send a message without tools (returns text only)
    #[allow(dead_code)]
    pub async fn send_message(&self, conversation: &[Content]) -> Result<String> {
        let response = self.send_message_with_tools(conversation, None).await?;
        match response {
            ApiResponse::Text(text) => Ok(text),
            ApiResponse::FunctionCall(_) => {
                Ok("(Function call received but no tools available)".to_string())
            }
            ApiResponse::TextWithFunctionCall { text, .. } => Ok(text),
        }
    }

    /// Send a message with tool definitions (returns raw ApiResponse)
    pub async fn send_message_with_tools(
        &self,
        conversation: &[Content],
        tools: Option<Vec<Value>>,
    ) -> Result<ApiResponse> {
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

        tracing::debug!(
            "Sending request to Gemini API: {}",
            serde_json::to_string_pretty(&request)?
        );

        let response = self.client.post(&url).json(&request).send().await?;

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

        // Parse the response - check for function calls and text
        let mut text_content = None;
        let mut function_call = None;

        for part in &candidate.content.parts {
            if let Some(fc) = &part.function_call {
                function_call = Some(fc.clone());
            }
            if let Some(t) = &part.text {
                text_content = Some(t.clone());
            }
        }

        match (text_content, function_call) {
            (Some(text), Some(fc)) => Ok(ApiResponse::TextWithFunctionCall {
                text,
                function_call: fc,
            }),
            (None, Some(fc)) => Ok(ApiResponse::FunctionCall(fc)),
            (Some(text), None) => Ok(ApiResponse::Text(text)),
            (None, None) => Ok(ApiResponse::Text("No response content".to_string())),
        }
    }

    /// Get the full Content from the response (for adding to conversation history)
    pub fn response_to_content(response: &ApiResponse) -> Content {
        match response {
            ApiResponse::Text(text) => Content {
                role: "model".to_string(),
                parts: vec![Part {
                    text: Some(text.clone()),
                    function_call: None,
                    function_response: None,
                }],
            },
            ApiResponse::FunctionCall(fc) => Content {
                role: "model".to_string(),
                parts: vec![Part {
                    text: None,
                    function_call: Some(fc.clone()),
                    function_response: None,
                }],
            },
            ApiResponse::TextWithFunctionCall {
                text,
                function_call,
            } => Content {
                role: "model".to_string(),
                parts: vec![
                    Part {
                        text: Some(text.clone()),
                        function_call: None,
                        function_response: None,
                    },
                    Part {
                        text: None,
                        function_call: Some(function_call.clone()),
                        function_response: None,
                    },
                ],
            },
        }
    }

    /// Create a function response content for the conversation
    pub fn create_function_response_content(name: &str, response: Value) -> Content {
        Content {
            role: "function".to_string(),
            parts: vec![Part {
                text: None,
                function_call: None,
                function_response: Some(FunctionResponse {
                    name: name.to_string(),
                    response,
                }),
            }],
        }
    }
}
