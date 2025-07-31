//! Model Service Module
//! 
//! Provides abstraction layer for different AI model providers and manages
//! model configurations, capabilities, and lifecycle.

pub mod provider;
pub mod registry;
pub mod capabilities;
pub mod config;
pub mod errors;

pub use provider::{ModelProvider, ProviderInfo};
pub use registry::{ModelRegistry, RegisteredModel};
pub use capabilities::{ModelCapabilities, CapabilityType};
pub use config::{ModelConfig, ProviderConfig};
pub use errors::{ModelError, ModelResult};

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

/// Core model service interface
#[async_trait]
pub trait ModelService: Send + Sync {
    /// Generate text using the specified model
    async fn generate(&self, model_id: &str, prompt: &str, config: Option<ModelConfig>) -> ModelResult<String>;
    
    /// Stream text generation (for real-time responses)
    async fn generate_stream(&self, model_id: &str, prompt: &str, config: Option<ModelConfig>) -> ModelResult<Box<dyn futures::Stream<Item = ModelResult<String>> + Unpin + Send>>;
    
    /// List available models
    fn list_models(&self) -> Vec<RegisteredModel>;
    
    /// Get model information
    fn get_model_info(&self, model_id: &str) -> Option<&RegisteredModel>;
    
    /// Register a new model provider
    async fn register_provider(&mut self, provider: Box<dyn ModelProvider>) -> ModelResult<()>;
    
    /// Unregister a model provider
    fn unregister_provider(&mut self, provider_id: &str) -> ModelResult<()>;
    
    /// Validate model configuration
    fn validate_config(&self, model_id: &str, config: &ModelConfig) -> ModelResult<()>;
    
    /// Get provider health status
    async fn health_check(&self, provider_id: &str) -> ModelResult<ProviderHealth>;
}

/// Provider health information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderHealth {
    pub provider_id: String,
    pub status: HealthStatus,
    pub latency_ms: Option<u64>,
    pub error_message: Option<String>,
    pub last_checked: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum HealthStatus {
    Healthy,
    Degraded,
    Unhealthy,
    Unknown,
}

/// Default model service implementation
pub struct DefaultModelService {
    registry: ModelRegistry,
    providers: HashMap<String, Box<dyn ModelProvider>>,
    config: ServiceConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceConfig {
    pub default_timeout_ms: u64,
    pub max_concurrent_requests: usize,
    pub health_check_interval_ms: u64,
    pub retry_attempts: u32,
}

impl Default for ServiceConfig {
    fn default() -> Self {
        Self {
            default_timeout_ms: 30000,
            max_concurrent_requests: 10,
            health_check_interval_ms: 60000,
            retry_attempts: 3,
        }
    }
}

impl DefaultModelService {
    pub fn new(config: ServiceConfig) -> Self {
        Self {
            registry: ModelRegistry::new(),
            providers: HashMap::new(),
            config,
        }
    }
    
    pub fn with_default_config() -> Self {
        Self::new(ServiceConfig::default())
    }
}

#[async_trait]
impl ModelService for DefaultModelService {
    async fn generate(&self, model_id: &str, prompt: &str, config: Option<ModelConfig>) -> ModelResult<String> {
        let model = self.registry.get_model(model_id)
            .ok_or_else(|| ModelError::ModelNotFound(model_id.to_string()))?;
            
        let provider = self.providers.get(&model.provider_id)
            .ok_or_else(|| ModelError::ProviderNotFound(model.provider_id.clone()))?;
            
        let effective_config = config.unwrap_or_else(|| model.default_config.clone());
        
        provider.generate(prompt, &effective_config).await
    }
    
    async fn generate_stream(&self, model_id: &str, prompt: &str, config: Option<ModelConfig>) -> ModelResult<Box<dyn futures::Stream<Item = ModelResult<String>> + Unpin + Send>> {
        let model = self.registry.get_model(model_id)
            .ok_or_else(|| ModelError::ModelNotFound(model_id.to_string()))?;
            
        let provider = self.providers.get(&model.provider_id)
            .ok_or_else(|| ModelError::ProviderNotFound(model.provider_id.clone()))?;
            
        let effective_config = config.unwrap_or_else(|| model.default_config.clone());
        
        provider.generate_stream(prompt, &effective_config).await
    }
    
    fn list_models(&self) -> Vec<RegisteredModel> {
        self.registry.list_models()
    }
    
    fn get_model_info(&self, model_id: &str) -> Option<&RegisteredModel> {
        self.registry.get_model(model_id)
    }
    
    async fn register_provider(&mut self, provider: Box<dyn ModelProvider>) -> ModelResult<()> {
        let provider_info = provider.get_info();
        let provider_id = provider_info.id.clone();
        
        // Validate provider
        provider.validate_config(&provider_info.default_config).await?;
        
        // Register models from this provider
        for model_info in provider_info.supported_models {
            let registered_model = RegisteredModel {
                id: model_info.id.clone(),
                name: model_info.name,
                provider_id: provider_id.clone(),
                capabilities: model_info.capabilities,
                default_config: model_info.default_config,
                created_at: chrono::Utc::now(),
                metadata: model_info.metadata,
            };
            
            self.registry.register_model(registered_model)?;
        }
        
        // Store provider
        self.providers.insert(provider_id, provider);
        
        Ok(())
    }
    
    fn unregister_provider(&mut self, provider_id: &str) -> ModelResult<()> {
        // Remove all models from this provider
        self.registry.unregister_provider_models(provider_id)?;
        
        // Remove provider
        self.providers.remove(provider_id)
            .ok_or_else(|| ModelError::ProviderNotFound(provider_id.to_string()))?;
            
        Ok(())
    }
    
    fn validate_config(&self, model_id: &str, config: &ModelConfig) -> ModelResult<()> {
        let model = self.registry.get_model(model_id)
            .ok_or_else(|| ModelError::ModelNotFound(model_id.to_string()))?;
            
        let provider = self.providers.get(&model.provider_id)
            .ok_or_else(|| ModelError::ProviderNotFound(model.provider_id.clone()))?;
            
        // Validate configuration against model capabilities
        model.capabilities.validate_config(config)?;
        
        Ok(())
    }
    
    async fn health_check(&self, provider_id: &str) -> ModelResult<ProviderHealth> {
        let provider = self.providers.get(provider_id)
            .ok_or_else(|| ModelError::ProviderNotFound(provider_id.to_string()))?;
            
        let start_time = std::time::Instant::now();
        
        match provider.health_check().await {
            Ok(_) => Ok(ProviderHealth {
                provider_id: provider_id.to_string(),
                status: HealthStatus::Healthy,
                latency_ms: Some(start_time.elapsed().as_millis() as u64),
                error_message: None,
                last_checked: chrono::Utc::now(),
            }),
            Err(e) => Ok(ProviderHealth {
                provider_id: provider_id.to_string(),
                status: HealthStatus::Unhealthy,
                latency_ms: Some(start_time.elapsed().as_millis() as u64),
                error_message: Some(e.to_string()),
                last_checked: chrono::Utc::now(),
            }),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_model_service_creation() {
        let service = DefaultModelService::with_default_config();
        assert_eq!(service.list_models().len(), 0);
    }
    
    #[tokio::test]
    async fn test_model_not_found_error() {
        let service = DefaultModelService::with_default_config();
        let result = service.generate("nonexistent", "test prompt", None).await;
        
        assert!(matches!(result, Err(ModelError::ModelNotFound(_))));
    }
}