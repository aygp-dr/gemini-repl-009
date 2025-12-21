//! Gemini API provider implementation

use super::{
    FunctionCall, Message, MessageContent, Provider, ProviderResponse, Role, ToolDefinition,
};
use anyhow::{bail, Result};
use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::time::Duration;

/// Gemini API provider
pub struct GeminiProvider {
    client: Client,
    api_key: String,
    model: String,
    base_url: String,
}

impl GeminiProvider {
    /// Create a new Gemini provider
    pub fn new(api_key: String, model: String, timeout_secs: u64) -> Result<Self> {
        let client = Client::builder()
            .timeout(Duration::from_secs(timeout_secs))
            .build()?;

        Ok(Self {
            client,
            api_key,
            model,
            base_url: "https://generativelanguage.googleapis.com/v1beta".to_string(),
        })
    }

    /// Convert our Message format to Gemini's Content format
    fn to_gemini_content(messages: &[Message]) -> Vec<GeminiContent> {
        messages
            .iter()
            .filter_map(|msg| {
                let role = match msg.role {
                    Role::User => "user",
                    Role::Assistant => "model",
                    Role::Function => "function",
                    Role::System => return None, // System messages handled separately
                };

                let parts = match &msg.content {
                    MessageContent::Text(text) => vec![GeminiPart {
                        text: Some(text.clone()),
                        function_call: None,
                        function_response: None,
                    }],
                    MessageContent::FunctionCall(fc) => vec![GeminiPart {
                        text: None,
                        function_call: Some(GeminiFunctionCall {
                            name: fc.name.clone(),
                            args: fc.arguments.clone(),
                        }),
                        function_response: None,
                    }],
                    MessageContent::FunctionResponse(fr) => vec![GeminiPart {
                        text: None,
                        function_call: None,
                        function_response: Some(GeminiFunctionResponse {
                            name: fr.name.clone(),
                            response: fr.response.clone(),
                        }),
                    }],
                    MessageContent::Parts(parts) => parts
                        .iter()
                        .map(|p| match p {
                            super::MessagePart::Text(t) => GeminiPart {
                                text: Some(t.clone()),
                                function_call: None,
                                function_response: None,
                            },
                            super::MessagePart::FunctionCall(fc) => GeminiPart {
                                text: None,
                                function_call: Some(GeminiFunctionCall {
                                    name: fc.name.clone(),
                                    args: fc.arguments.clone(),
                                }),
                                function_response: None,
                            },
                            super::MessagePart::FunctionResponse(fr) => GeminiPart {
                                text: None,
                                function_call: None,
                                function_response: Some(GeminiFunctionResponse {
                                    name: fr.name.clone(),
                                    response: fr.response.clone(),
                                }),
                            },
                        })
                        .collect(),
                };

                Some(GeminiContent {
                    role: role.to_string(),
                    parts,
                })
            })
            .collect()
    }

    /// Convert tool definitions to Gemini's format
    fn to_gemini_tools(tools: &[ToolDefinition]) -> Vec<Value> {
        vec![serde_json::json!({
            "function_declarations": tools.iter().map(|t| {
                serde_json::json!({
                    "name": t.name,
                    "description": t.description,
                    "parameters": t.parameters
                })
            }).collect::<Vec<_>>()
        })]
    }

    /// Extract system message for Gemini's system_instruction
    fn extract_system_instruction(messages: &[Message]) -> Option<GeminiSystemInstruction> {
        messages
            .iter()
            .find(|m| m.role == Role::System)
            .and_then(|m| m.text())
            .map(|text| GeminiSystemInstruction {
                parts: vec![GeminiSystemPart {
                    text: text.to_string(),
                }],
            })
    }
}

#[async_trait]
impl Provider for GeminiProvider {
    fn name(&self) -> &str {
        "gemini"
    }

    fn model(&self) -> &str {
        &self.model
    }

    fn supports_tools(&self) -> bool {
        true
    }

    fn max_context_tokens(&self) -> usize {
        // Gemini 2.0 Flash has 1M context, but we'll use a conservative limit
        128_000
    }

    async fn generate(
        &self,
        messages: &[Message],
        tools: Option<&[ToolDefinition]>,
    ) -> Result<ProviderResponse> {
        let url = format!(
            "{}/models/{}:generateContent?key={}",
            self.base_url, self.model, self.api_key
        );

        let contents = Self::to_gemini_content(messages);
        let system_instruction = Self::extract_system_instruction(messages).or_else(|| {
            if tools.is_some() {
                Some(GeminiSystemInstruction {
                    parts: vec![GeminiSystemPart {
                        text: "You are a helpful AI assistant with access to tools. When the user asks you to perform actions that require tools, use the available function calls to complete the request.".to_string(),
                    }],
                })
            } else {
                None
            }
        });

        let gemini_tools = tools.map(Self::to_gemini_tools);

        let request = GeminiRequest {
            contents,
            tools: gemini_tools,
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

        let response: GeminiResponse = serde_json::from_str(&response_text)?;

        if response.candidates.is_empty() {
            bail!("No candidates in response");
        }

        let candidate = &response.candidates[0];
        if candidate.content.parts.is_empty() {
            bail!("No parts in candidate content");
        }

        // Parse the response
        let mut text_content = None;
        let mut function_call = None;

        for part in &candidate.content.parts {
            if let Some(fc) = &part.function_call {
                function_call = Some(FunctionCall {
                    name: fc.name.clone(),
                    arguments: fc.args.clone(),
                });
            }
            if let Some(t) = &part.text {
                text_content = Some(t.clone());
            }
        }

        match (text_content, function_call) {
            (Some(text), Some(fc)) => Ok(ProviderResponse::TextWithFunctionCall {
                text,
                function_call: fc,
            }),
            (None, Some(fc)) => Ok(ProviderResponse::FunctionCall(fc)),
            (Some(text), None) => Ok(ProviderResponse::Text(text)),
            (None, None) => Ok(ProviderResponse::Text("No response content".to_string())),
        }
    }

    fn estimate_tokens(&self, text: &str) -> usize {
        // Gemini uses a similar tokenizer to GPT, roughly 4 chars per token
        text.len() / 4
    }
}

// Gemini API types

#[derive(Debug, Serialize)]
struct GeminiRequest {
    contents: Vec<GeminiContent>,
    #[serde(skip_serializing_if = "Option::is_none")]
    tools: Option<Vec<Value>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    system_instruction: Option<GeminiSystemInstruction>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct GeminiContent {
    role: String,
    parts: Vec<GeminiPart>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct GeminiPart {
    #[serde(skip_serializing_if = "Option::is_none")]
    text: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    function_call: Option<GeminiFunctionCall>,
    #[serde(skip_serializing_if = "Option::is_none")]
    function_response: Option<GeminiFunctionResponse>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct GeminiFunctionCall {
    name: String,
    args: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct GeminiFunctionResponse {
    name: String,
    response: Value,
}

#[derive(Debug, Serialize)]
struct GeminiSystemInstruction {
    parts: Vec<GeminiSystemPart>,
}

#[derive(Debug, Serialize)]
struct GeminiSystemPart {
    text: String,
}

#[derive(Debug, Deserialize)]
struct GeminiResponse {
    candidates: Vec<GeminiCandidate>,
}

#[derive(Debug, Deserialize)]
struct GeminiCandidate {
    content: GeminiContent,
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_message_to_gemini_content() {
        let messages = vec![Message::user("Hello"), Message::assistant("Hi there!")];

        let content = GeminiProvider::to_gemini_content(&messages);
        assert_eq!(content.len(), 2);
        assert_eq!(content[0].role, "user");
        assert_eq!(content[1].role, "model");
    }

    #[test]
    fn test_system_message_extraction() {
        let messages = vec![Message::system("You are helpful"), Message::user("Hello")];

        let system = GeminiProvider::extract_system_instruction(&messages);
        assert!(system.is_some());
        assert_eq!(system.unwrap().parts[0].text, "You are helpful");
    }

    #[test]
    fn test_tool_conversion() {
        let tools = vec![ToolDefinition::new(
            "read_file",
            "Read a file",
            json!({
                "type": "object",
                "properties": {
                    "path": {"type": "string"}
                }
            }),
        )];

        let gemini_tools = GeminiProvider::to_gemini_tools(&tools);
        assert_eq!(gemini_tools.len(), 1);
        assert!(gemini_tools[0]["function_declarations"].is_array());
    }

    #[test]
    fn test_provider_creation() {
        let provider = GeminiProvider::new("test-key".to_string(), "gemini-pro".to_string(), 30);
        assert!(provider.is_ok());
        let p = provider.unwrap();
        assert_eq!(p.name(), "gemini");
        assert_eq!(p.model(), "gemini-pro");
        assert!(p.supports_tools());
    }
}
