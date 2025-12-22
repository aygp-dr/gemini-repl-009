//! Ollama API provider implementation
//!
//! Provides integration with locally-running Ollama instances.
//! Ollama is the default provider when available.

use super::{
    FunctionCall, Message, MessageContent, Provider, ProviderResponse, Role, StreamChunk,
    ToolDefinition,
};
use anyhow::{bail, Context, Result};
use async_trait::async_trait;
use futures::stream::{self, Stream};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::time::Duration;

/// Ollama API provider
pub struct OllamaProvider {
    client: Client,
    model: String,
    base_url: String,
}

impl OllamaProvider {
    /// Create a new Ollama provider
    pub fn new(model: String, base_url: Option<String>, timeout_secs: u64) -> Result<Self> {
        let client = Client::builder()
            .timeout(Duration::from_secs(timeout_secs))
            .build()?;

        Ok(Self {
            client,
            model,
            base_url: base_url.unwrap_or_else(|| "http://localhost:11434".to_string()),
        })
    }

    /// Check if Ollama is available at the configured URL
    pub async fn is_available(&self) -> bool {
        let url = format!("{}/api/tags", self.base_url);
        match self.client.get(&url).send().await {
            Ok(resp) => resp.status().is_success(),
            Err(_) => false,
        }
    }

    /// List available models
    pub async fn list_models(&self) -> Result<Vec<String>> {
        let url = format!("{}/api/tags", self.base_url);
        let response = self.client.get(&url).send().await?;

        if !response.status().is_success() {
            bail!("Failed to list models: {}", response.status());
        }

        let data: OllamaTagsResponse = response.json().await?;
        Ok(data.models.into_iter().map(|m| m.name).collect())
    }

    /// Convert our Message format to Ollama's format
    fn to_ollama_messages(messages: &[Message]) -> Vec<OllamaMessage> {
        messages
            .iter()
            .map(|msg| {
                let role = match msg.role {
                    Role::User => "user",
                    Role::Assistant => "assistant",
                    Role::System => "system",
                    Role::Function => "assistant", // Ollama doesn't have function role
                };

                let content = match &msg.content {
                    MessageContent::Text(text) => text.clone(),
                    MessageContent::FunctionCall(fc) => {
                        format!("[Function call: {}]\n{}", fc.name, fc.arguments)
                    }
                    MessageContent::FunctionResponse(fr) => {
                        format!("[Function response: {}]\n{}", fr.name, fr.response)
                    }
                    MessageContent::Parts(parts) => parts
                        .iter()
                        .map(|p| match p {
                            super::MessagePart::Text(t) => t.clone(),
                            super::MessagePart::FunctionCall(fc) => {
                                format!("[Function call: {}]", fc.name)
                            }
                            super::MessagePart::FunctionResponse(fr) => {
                                format!("[Function response: {}]", fr.name)
                            }
                        })
                        .collect::<Vec<_>>()
                        .join("\n"),
                };

                OllamaMessage {
                    role: role.to_string(),
                    content,
                }
            })
            .collect()
    }

    /// Convert tool definitions to system prompt additions
    fn tools_to_system_prompt(tools: &[ToolDefinition]) -> String {
        if tools.is_empty() {
            return String::new();
        }

        let mut prompt = String::from("\n\nYou have access to the following tools:\n\n");

        for tool in tools {
            prompt.push_str(&format!("## {}\n", tool.name));
            prompt.push_str(&format!("{}\n", tool.description));
            prompt.push_str(&format!(
                "Parameters: {}\n\n",
                serde_json::to_string_pretty(&tool.parameters).unwrap_or_default()
            ));
        }

        prompt.push_str(
            r#"
To use a tool, respond with a JSON object in this exact format:
```json
{"tool": "tool_name", "arguments": {...}}
```

Only use a tool when necessary. If you can answer directly, do so without using tools.
"#,
        );

        prompt
    }

    /// Try to parse a function call from the response
    fn parse_function_call(text: &str) -> Option<FunctionCall> {
        // Look for JSON tool call pattern
        if let Some(start) = text.find("```json") {
            if let Some(end) = text[start..]
                .find("```\n")
                .or_else(|| text[start..].rfind("```"))
            {
                let json_str = &text[start + 7..start + end].trim();
                if let Ok(parsed) = serde_json::from_str::<Value>(json_str) {
                    if let (Some(tool), Some(args)) = (parsed.get("tool"), parsed.get("arguments"))
                    {
                        if let Some(name) = tool.as_str() {
                            return Some(FunctionCall {
                                name: name.to_string(),
                                arguments: args.clone(),
                            });
                        }
                    }
                }
            }
        }

        // Also try inline JSON
        if let Some(start) = text
            .find(r#"{"tool":"#)
            .or_else(|| text.find(r#"{"tool" :"#))
        {
            // Find matching closing brace
            let mut depth = 0;
            let mut end = start;
            for (i, c) in text[start..].char_indices() {
                match c {
                    '{' => depth += 1,
                    '}' => {
                        depth -= 1;
                        if depth == 0 {
                            end = start + i + 1;
                            break;
                        }
                    }
                    _ => {}
                }
            }

            if end > start {
                let json_str = &text[start..end];
                if let Ok(parsed) = serde_json::from_str::<Value>(json_str) {
                    if let (Some(tool), Some(args)) = (parsed.get("tool"), parsed.get("arguments"))
                    {
                        if let Some(name) = tool.as_str() {
                            return Some(FunctionCall {
                                name: name.to_string(),
                                arguments: args.clone(),
                            });
                        }
                    }
                }
            }
        }

        None
    }
}

#[async_trait]
impl Provider for OllamaProvider {
    fn name(&self) -> &str {
        "ollama"
    }

    fn model(&self) -> &str {
        &self.model
    }

    fn supports_tools(&self) -> bool {
        // Ollama supports tools via prompt engineering
        true
    }

    fn max_context_tokens(&self) -> usize {
        // Most Ollama models have 4K-8K context, some have more
        // Use conservative default
        8192
    }

    async fn generate(
        &self,
        messages: &[Message],
        tools: Option<&[ToolDefinition]>,
    ) -> Result<ProviderResponse> {
        let url = format!("{}/api/chat", self.base_url);

        let mut ollama_messages = Self::to_ollama_messages(messages);

        // Add tool instructions to system message if tools provided
        if let Some(tools) = tools {
            if !tools.is_empty() {
                let tool_prompt = Self::tools_to_system_prompt(tools);

                // Find or create system message
                if let Some(sys_msg) = ollama_messages.iter_mut().find(|m| m.role == "system") {
                    sys_msg.content.push_str(&tool_prompt);
                } else {
                    ollama_messages.insert(
                        0,
                        OllamaMessage {
                            role: "system".to_string(),
                            content: format!(
                                "You are a helpful AI assistant with access to tools.{}",
                                tool_prompt
                            ),
                        },
                    );
                }
            }
        }

        let request = OllamaChatRequest {
            model: self.model.clone(),
            messages: ollama_messages,
            stream: false,
            options: None,
        };

        tracing::debug!(
            "Sending request to Ollama API: {}",
            serde_json::to_string_pretty(&request)?
        );

        let response = self
            .client
            .post(&url)
            .json(&request)
            .send()
            .await
            .context("Failed to connect to Ollama")?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await?;
            bail!(
                "Ollama API request failed with status {}: {}",
                status,
                error_text
            );
        }

        let response_text = response.text().await?;
        tracing::debug!("Received response from Ollama API: {}", response_text);

        let response: OllamaChatResponse =
            serde_json::from_str(&response_text).context("Failed to parse Ollama response")?;

        let text = response.message.content;

        // Try to parse function call from response
        if let Some(fc) = Self::parse_function_call(&text) {
            // Extract text before the function call
            let text_before = if let Some(idx) = text.find("```json") {
                text[..idx].trim().to_string()
            } else if let Some(idx) = text.find(r#"{"tool":"#) {
                text[..idx].trim().to_string()
            } else {
                String::new()
            };

            if text_before.is_empty() {
                Ok(ProviderResponse::FunctionCall(fc))
            } else {
                Ok(ProviderResponse::TextWithFunctionCall {
                    text: text_before,
                    function_call: fc,
                })
            }
        } else {
            Ok(ProviderResponse::Text(text))
        }
    }

    async fn generate_stream(
        &self,
        messages: &[Message],
        tools: Option<&[ToolDefinition]>,
    ) -> Result<Box<dyn Stream<Item = Result<StreamChunk>> + Send + Unpin>> {
        let url = format!("{}/api/chat", self.base_url);

        let mut ollama_messages = Self::to_ollama_messages(messages);

        if let Some(tools) = tools {
            if !tools.is_empty() {
                let tool_prompt = Self::tools_to_system_prompt(tools);
                if let Some(sys_msg) = ollama_messages.iter_mut().find(|m| m.role == "system") {
                    sys_msg.content.push_str(&tool_prompt);
                } else {
                    ollama_messages.insert(
                        0,
                        OllamaMessage {
                            role: "system".to_string(),
                            content: format!(
                                "You are a helpful AI assistant with access to tools.{}",
                                tool_prompt
                            ),
                        },
                    );
                }
            }
        }

        let request = OllamaChatRequest {
            model: self.model.clone(),
            messages: ollama_messages,
            stream: true,
            options: None,
        };

        let response = self
            .client
            .post(&url)
            .json(&request)
            .send()
            .await
            .context("Failed to connect to Ollama")?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await?;
            bail!("Ollama API request failed: {} - {}", status, error_text);
        }

        // For now, return a simple non-streaming implementation
        // Full streaming would require parsing NDJSON
        let text = response.text().await?;
        let mut chunks = Vec::new();

        for line in text.lines() {
            if line.is_empty() {
                continue;
            }
            if let Ok(chunk) = serde_json::from_str::<OllamaChatResponse>(line) {
                if !chunk.message.content.is_empty() {
                    chunks.push(Ok(StreamChunk::Text(chunk.message.content)));
                }
                if chunk.done {
                    chunks.push(Ok(StreamChunk::Done));
                    break;
                }
            }
        }

        Ok(Box::new(stream::iter(chunks)))
    }

    fn estimate_tokens(&self, text: &str) -> usize {
        // Ollama models vary, but ~4 chars per token is reasonable
        text.len() / 4
    }
}

// Ollama API types

#[derive(Debug, Serialize)]
struct OllamaChatRequest {
    model: String,
    messages: Vec<OllamaMessage>,
    stream: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    options: Option<OllamaOptions>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct OllamaMessage {
    role: String,
    content: String,
}

#[derive(Debug, Serialize)]
struct OllamaOptions {
    #[serde(skip_serializing_if = "Option::is_none")]
    temperature: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    num_ctx: Option<usize>,
}

#[derive(Debug, Deserialize)]
struct OllamaChatResponse {
    message: OllamaMessage,
    #[serde(default)]
    done: bool,
}

#[derive(Debug, Deserialize)]
struct OllamaTagsResponse {
    models: Vec<OllamaModel>,
}

#[derive(Debug, Deserialize)]
struct OllamaModel {
    name: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_provider_creation() {
        let provider = OllamaProvider::new("llama2".to_string(), None, 30);
        assert!(provider.is_ok());
        let p = provider.unwrap();
        assert_eq!(p.name(), "ollama");
        assert_eq!(p.model(), "llama2");
        assert!(p.supports_tools());
    }

    #[test]
    fn test_custom_base_url() {
        let provider = OllamaProvider::new(
            "llama2".to_string(),
            Some("http://remote:11434".to_string()),
            30,
        )
        .unwrap();
        assert_eq!(provider.base_url, "http://remote:11434");
    }

    #[test]
    fn test_message_conversion() {
        let messages = vec![
            Message::system("You are helpful"),
            Message::user("Hello"),
            Message::assistant("Hi there!"),
        ];

        let ollama_messages = OllamaProvider::to_ollama_messages(&messages);
        assert_eq!(ollama_messages.len(), 3);
        assert_eq!(ollama_messages[0].role, "system");
        assert_eq!(ollama_messages[1].role, "user");
        assert_eq!(ollama_messages[2].role, "assistant");
    }

    #[test]
    fn test_parse_function_call_json_block() {
        let text = r#"I'll read that file for you.
```json
{"tool": "read_file", "arguments": {"path": "test.txt"}}
```"#;

        let fc = OllamaProvider::parse_function_call(text);
        assert!(fc.is_some());
        let fc = fc.unwrap();
        assert_eq!(fc.name, "read_file");
        assert_eq!(fc.arguments["path"], "test.txt");
    }

    #[test]
    fn test_parse_function_call_inline() {
        let text = r#"Let me check that: {"tool": "list_files", "arguments": {"directory": "."}}"#;

        let fc = OllamaProvider::parse_function_call(text);
        assert!(fc.is_some());
        let fc = fc.unwrap();
        assert_eq!(fc.name, "list_files");
    }

    #[test]
    fn test_parse_function_call_none() {
        let text = "This is just a regular response without any tool calls.";
        let fc = OllamaProvider::parse_function_call(text);
        assert!(fc.is_none());
    }

    #[test]
    fn test_tools_to_system_prompt() {
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

        let prompt = OllamaProvider::tools_to_system_prompt(&tools);
        assert!(prompt.contains("read_file"));
        assert!(prompt.contains("Read a file"));
        assert!(prompt.contains(r#"{"tool": "tool_name"#));
    }

    #[test]
    fn test_empty_tools() {
        let tools: Vec<ToolDefinition> = vec![];
        let prompt = OllamaProvider::tools_to_system_prompt(&tools);
        assert!(prompt.is_empty());
    }
}
