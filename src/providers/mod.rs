//! Provider abstraction for multiple LLM backends
//!
//! This module provides a trait-based abstraction for LLM providers,
//! enabling support for Gemini, Ollama, OpenAI, and other backends.

#![allow(dead_code)]

pub mod gemini;
pub mod ollama;
pub mod types;

use anyhow::Result;
use async_trait::async_trait;
use reqwest::Client;
use std::time::Duration;

pub use gemini::GeminiProvider;
pub use ollama::OllamaProvider;
pub use types::*;

/// Provider trait for LLM backends
///
/// All LLM providers must implement this trait to be usable with the REPL.
#[async_trait]
pub trait Provider: Send + Sync {
    /// Get the provider name (e.g., "gemini", "ollama", "openai")
    fn name(&self) -> &str;

    /// Get the current model name
    fn model(&self) -> &str;

    /// Check if this provider supports function/tool calling
    fn supports_tools(&self) -> bool;

    /// Get the maximum context window size in tokens
    fn max_context_tokens(&self) -> usize;

    /// Generate a response from the model
    ///
    /// # Arguments
    /// * `messages` - The conversation history
    /// * `tools` - Optional tool definitions for function calling
    ///
    /// # Returns
    /// A ProviderResponse containing either text or a function call
    async fn generate(
        &self,
        messages: &[Message],
        tools: Option<&[ToolDefinition]>,
    ) -> Result<ProviderResponse>;

    /// Generate a streaming response (optional, not all providers support this)
    async fn generate_stream(
        &self,
        _messages: &[Message],
        _tools: Option<&[ToolDefinition]>,
    ) -> Result<Box<dyn futures::Stream<Item = Result<StreamChunk>> + Send + Unpin>> {
        // Default implementation: not supported
        anyhow::bail!("Streaming not supported by provider {}", self.name())
    }

    /// Estimate token count for a message (approximate)
    fn estimate_tokens(&self, text: &str) -> usize {
        // Default: ~4 characters per token (rough estimate)
        text.len() / 4
    }
}

/// Provider configuration
#[derive(Debug, Clone)]
pub struct ProviderConfig {
    /// Provider type
    pub provider_type: ProviderType,
    /// API key (if required)
    pub api_key: Option<String>,
    /// Base URL for API
    pub base_url: Option<String>,
    /// Model name
    pub model: String,
    /// Request timeout in seconds
    pub timeout_secs: u64,
}

impl Default for ProviderConfig {
    fn default() -> Self {
        Self {
            provider_type: ProviderType::Gemini,
            api_key: None,
            base_url: None,
            model: "gemini-2.0-flash-exp".to_string(),
            timeout_secs: 30,
        }
    }
}

/// Supported provider types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProviderType {
    Gemini,
    Ollama,
    OpenAI,
}

impl std::fmt::Display for ProviderType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ProviderType::Gemini => write!(f, "gemini"),
            ProviderType::Ollama => write!(f, "ollama"),
            ProviderType::OpenAI => write!(f, "openai"),
        }
    }
}

impl std::str::FromStr for ProviderType {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self> {
        match s.to_lowercase().as_str() {
            "gemini" => Ok(ProviderType::Gemini),
            "ollama" => Ok(ProviderType::Ollama),
            "openai" => Ok(ProviderType::OpenAI),
            _ => anyhow::bail!("Unknown provider: {}. Use gemini, ollama, or openai", s),
        }
    }
}

/// Create a provider from configuration
pub fn create_provider(config: ProviderConfig) -> Result<Box<dyn Provider>> {
    match config.provider_type {
        ProviderType::Gemini => {
            let api_key = config
                .api_key
                .ok_or_else(|| anyhow::anyhow!("Gemini requires an API key"))?;
            Ok(Box::new(GeminiProvider::new(
                api_key,
                config.model,
                config.timeout_secs,
            )?))
        }
        ProviderType::Ollama => Ok(Box::new(OllamaProvider::new(
            config.model,
            config.base_url,
            config.timeout_secs,
        )?)),
        ProviderType::OpenAI => {
            anyhow::bail!("OpenAI provider not yet implemented")
        }
    }
}

/// Check if Ollama is available at the given URL
pub async fn is_ollama_available(base_url: Option<&str>) -> bool {
    let url = base_url.unwrap_or("http://localhost:11434");
    let client = match Client::builder().timeout(Duration::from_secs(2)).build() {
        Ok(c) => c,
        Err(_) => return false,
    };

    match client.get(format!("{}/api/tags", url)).send().await {
        Ok(resp) => resp.status().is_success(),
        Err(_) => false,
    }
}

/// Auto-detect the best available provider
///
/// Priority order:
/// 1. Ollama (if running locally)
/// 2. Gemini (if API key available)
/// 3. None
pub async fn detect_provider(
    gemini_api_key: Option<String>,
    ollama_url: Option<String>,
) -> Option<ProviderConfig> {
    // Check Ollama first (preferred for local/private use)
    if is_ollama_available(ollama_url.as_deref()).await {
        tracing::info!(
            "Detected Ollama at {}",
            ollama_url.as_deref().unwrap_or("localhost:11434")
        );
        return Some(ProviderConfig {
            provider_type: ProviderType::Ollama,
            api_key: None,
            base_url: ollama_url,
            model: "llama3.2".to_string(), // Good default model
            timeout_secs: 120,             // Ollama can be slower
        });
    }

    // Fall back to Gemini if API key available
    if let Some(api_key) = gemini_api_key {
        tracing::info!("Using Gemini provider");
        return Some(ProviderConfig {
            provider_type: ProviderType::Gemini,
            api_key: Some(api_key),
            base_url: None,
            model: "gemini-2.0-flash-exp".to_string(),
            timeout_secs: 30,
        });
    }

    None
}

/// Get default model for a provider type
pub fn default_model_for_provider(provider_type: ProviderType) -> &'static str {
    match provider_type {
        ProviderType::Ollama => "llama3.2",
        ProviderType::Gemini => "gemini-2.0-flash-exp",
        ProviderType::OpenAI => "gpt-4o",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_provider_type_from_str() {
        assert_eq!(
            "gemini".parse::<ProviderType>().unwrap(),
            ProviderType::Gemini
        );
        assert_eq!(
            "ollama".parse::<ProviderType>().unwrap(),
            ProviderType::Ollama
        );
        assert_eq!(
            "openai".parse::<ProviderType>().unwrap(),
            ProviderType::OpenAI
        );
        assert_eq!(
            "GEMINI".parse::<ProviderType>().unwrap(),
            ProviderType::Gemini
        );
        assert!("unknown".parse::<ProviderType>().is_err());
    }

    #[test]
    fn test_provider_type_display() {
        assert_eq!(ProviderType::Gemini.to_string(), "gemini");
        assert_eq!(ProviderType::Ollama.to_string(), "ollama");
        assert_eq!(ProviderType::OpenAI.to_string(), "openai");
    }

    #[test]
    fn test_provider_config_default() {
        let config = ProviderConfig::default();
        assert_eq!(config.provider_type, ProviderType::Gemini);
        assert_eq!(config.model, "gemini-2.0-flash-exp");
        assert_eq!(config.timeout_secs, 30);
    }
}
