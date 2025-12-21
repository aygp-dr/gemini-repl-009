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

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_content_serialization() {
        let content = Content {
            role: "user".to_string(),
            parts: vec![Part {
                text: Some("Hello, world!".to_string()),
                function_call: None,
                function_response: None,
            }],
        };

        let json = serde_json::to_string(&content).unwrap();
        assert!(json.contains("\"role\":\"user\""));
        assert!(json.contains("\"text\":\"Hello, world!\""));

        let deserialized: Content = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.role, "user");
        assert_eq!(
            deserialized.parts[0].text,
            Some("Hello, world!".to_string())
        );
    }

    #[test]
    fn test_function_call_serialization() {
        let fc = FunctionCall {
            name: "read_file".to_string(),
            args: json!({"path": "test.txt"}),
        };

        let json = serde_json::to_string(&fc).unwrap();
        assert!(json.contains("\"name\":\"read_file\""));

        let deserialized: FunctionCall = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.name, "read_file");
        assert_eq!(deserialized.args["path"], "test.txt");
    }

    #[test]
    fn test_response_to_content_text() {
        let response = ApiResponse::Text("Hello!".to_string());
        let content = GeminiClient::response_to_content(&response);

        assert_eq!(content.role, "model");
        assert_eq!(content.parts.len(), 1);
        assert_eq!(content.parts[0].text, Some("Hello!".to_string()));
        assert!(content.parts[0].function_call.is_none());
    }

    #[test]
    fn test_response_to_content_function_call() {
        let fc = FunctionCall {
            name: "list_files".to_string(),
            args: json!({"path": "."}),
        };
        let response = ApiResponse::FunctionCall(fc);
        let content = GeminiClient::response_to_content(&response);

        assert_eq!(content.role, "model");
        assert_eq!(content.parts.len(), 1);
        assert!(content.parts[0].text.is_none());
        assert!(content.parts[0].function_call.is_some());
        assert_eq!(
            content.parts[0].function_call.as_ref().unwrap().name,
            "list_files"
        );
    }

    #[test]
    fn test_response_to_content_text_with_function_call() {
        let fc = FunctionCall {
            name: "write_file".to_string(),
            args: json!({"path": "out.txt", "content": "data"}),
        };
        let response = ApiResponse::TextWithFunctionCall {
            text: "I'll write that file for you.".to_string(),
            function_call: fc,
        };
        let content = GeminiClient::response_to_content(&response);

        assert_eq!(content.role, "model");
        assert_eq!(content.parts.len(), 2);
        assert_eq!(
            content.parts[0].text,
            Some("I'll write that file for you.".to_string())
        );
        assert!(content.parts[1].function_call.is_some());
    }

    #[test]
    fn test_create_function_response_content() {
        let response = json!({"success": true, "data": "file contents"});
        let content = GeminiClient::create_function_response_content("read_file", response.clone());

        assert_eq!(content.role, "function");
        assert_eq!(content.parts.len(), 1);
        assert!(content.parts[0].text.is_none());
        assert!(content.parts[0].function_call.is_none());
        assert!(content.parts[0].function_response.is_some());

        let fr = content.parts[0].function_response.as_ref().unwrap();
        assert_eq!(fr.name, "read_file");
        assert_eq!(fr.response["success"], true);
    }

    #[test]
    fn test_part_with_all_fields() {
        let part = Part {
            text: Some("explanation".to_string()),
            function_call: Some(FunctionCall {
                name: "test".to_string(),
                args: json!({}),
            }),
            function_response: None,
        };

        let json = serde_json::to_string(&part).unwrap();
        let deserialized: Part = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.text, Some("explanation".to_string()));
        assert!(deserialized.function_call.is_some());
        assert!(deserialized.function_response.is_none());
    }

    #[test]
    fn test_gemini_client_creation() {
        let client = GeminiClient::new("test-api-key".to_string(), "gemini-pro".to_string());
        assert!(client.is_ok());
    }
}
