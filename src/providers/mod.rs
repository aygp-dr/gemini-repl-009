//! LLM Provider abstraction for supporting multiple backends

use async_trait::async_trait;
use anyhow::Result;
use serde::{Deserialize, Serialize};

pub mod gemini;
pub mod ollama;

/// Common message format for all providers
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub role: String,
    pub content: String,
}

/// Tool/Function declaration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tool {
    pub name: String,
    pub description: String,
    pub parameters: serde_json::Value,
}

/// Provider response
#[derive(Debug)]
pub struct ProviderResponse {
    pub text: Option<String>,
    pub function_call: Option<FunctionCall>,
    pub usage: Option<Usage>,
}

#[derive(Debug, Clone)]
pub struct FunctionCall {
    pub name: String,
    pub arguments: serde_json::Value,
}

#[derive(Debug, Clone)]
pub struct Usage {
    pub prompt_tokens: u32,
    pub completion_tokens: u32,
    pub total_tokens: u32,
}

/// Common interface for LLM providers
#[async_trait]
pub trait LLMProvider: Send + Sync {
    /// Generate a response from messages
    async fn generate(&self, messages: Vec<Message>) -> Result<ProviderResponse>;
    
    /// Generate with tool/function calling support
    async fn generate_with_tools(
        &self,
        messages: Vec<Message>,
        tools: Vec<Tool>,
    ) -> Result<ProviderResponse>;
    
    /// Check if provider is available
    async fn health_check(&self) -> Result<bool>;
    
    /// Get provider name
    fn name(&self) -> &str;
}

/// Provider factory
pub fn create_provider(provider_type: &str) -> Result<Box<dyn LLMProvider>> {
    match provider_type.to_lowercase().as_str() {
        "gemini" => Ok(Box::new(gemini::GeminiProvider::new()?)),
        "ollama" => Ok(Box::new(ollama::OllamaProvider::new()?)),
        _ => Err(anyhow::anyhow!("Unknown provider: {}", provider_type)),
    }
}