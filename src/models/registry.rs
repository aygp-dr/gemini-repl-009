//! Model Registry
//! 
//! Maintains a registry of available models and their configurations.
//! Handles model registration, lookup, and lifecycle management.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

use super::{ModelCapabilities, ModelConfig, ModelError, ModelResult};

/// Registry for managing available models
#[derive(Debug, Clone)]
pub struct ModelRegistry {
    models: HashMap<String, RegisteredModel>,
    provider_models: HashMap<String, Vec<String>>, // provider_id -> model_ids
}

/// A model registered in the system
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegisteredModel {
    /// Unique model identifier
    pub id: String,
    
    /// Human-readable model name
    pub name: String,
    
    /// Provider that hosts this model
    pub provider_id: String,
    
    /// Model capabilities
    pub capabilities: ModelCapabilities,
    
    /// Default configuration for this model
    pub default_config: ModelConfig,
    
    /// When this model was registered
    pub created_at: chrono::DateTime<chrono::Utc>,
    
    /// Model metadata
    pub metadata: HashMap<String, String>,
}

impl ModelRegistry {
    /// Create a new empty model registry
    pub fn new() -> Self {
        Self {
            models: HashMap::new(),
            provider_models: HashMap::new(),
        }
    }
    
    /// Register a new model
    pub fn register_model(&mut self, model: RegisteredModel) -> ModelResult<()> {
        // Check if model already exists
        if self.models.contains_key(&model.id) {
            return Err(ModelError::ModelAlreadyExists(model.id.clone()));
        }
        
        // Add to provider mapping
        self.provider_models
            .entry(model.provider_id.clone())
            .or_insert_with(Vec::new)
            .push(model.id.clone());
        
        // Register the model
        self.models.insert(model.id.clone(), model);
        
        Ok(())
    }
    
    /// Unregister a model
    pub fn unregister_model(&mut self, model_id: &str) -> ModelResult<RegisteredModel> {
        let model = self.models.remove(model_id)
            .ok_or_else(|| ModelError::ModelNotFound(model_id.to_string()))?;
        
        // Remove from provider mapping
        if let Some(provider_models) = self.provider_models.get_mut(&model.provider_id) {
            provider_models.retain(|id| id != model_id);
            
            // Remove provider entry if no models left
            if provider_models.is_empty() {
                self.provider_models.remove(&model.provider_id);
            }
        }
        
        Ok(model)
    }
    
    /// Get a model by ID
    pub fn get_model(&self, model_id: &str) -> Option<&RegisteredModel> {
        self.models.get(model_id)
    }
    
    /// Get a mutable reference to a model
    pub fn get_model_mut(&mut self, model_id: &str) -> Option<&mut RegisteredModel> {
        self.models.get_mut(model_id)
    }
    
    /// List all registered models
    pub fn list_models(&self) -> Vec<RegisteredModel> {
        self.models.values().cloned().collect()
    }
    
    /// List models by provider
    pub fn list_models_by_provider(&self, provider_id: &str) -> Vec<RegisteredModel> {
        self.provider_models
            .get(provider_id)
            .map(|model_ids| {
                model_ids.iter()
                    .filter_map(|id| self.models.get(id))
                    .cloned()
                    .collect()
            })
            .unwrap_or_default()
    }
    
    /// Search models by capability
    pub fn find_models_with_capability(&self, capability: &str) -> Vec<RegisteredModel> {
        self.models
            .values()
            .filter(|model| model.capabilities.has_capability(capability))
            .cloned()
            .collect()
    }
    
    /// Search models by name pattern
    pub fn search_models(&self, query: &str) -> Vec<RegisteredModel> {
        let query_lower = query.to_lowercase();
        self.models
            .values()
            .filter(|model| {
                model.name.to_lowercase().contains(&query_lower) ||
                model.id.to_lowercase().contains(&query_lower) ||
                model.metadata.values().any(|v| v.to_lowercase().contains(&query_lower))
            })
            .cloned()
            .collect()
    }
    
    /// Get registry statistics
    pub fn get_stats(&self) -> RegistryStats {
        let total_models = self.models.len();
        let total_providers = self.provider_models.len();
        
        let mut capabilities_count = HashMap::new();
        for model in self.models.values() {
            for capability in &model.capabilities.supported_capabilities {
                *capabilities_count.entry(capability.clone()).or_insert(0) += 1;
            }
        }
        
        RegistryStats {
            total_models,
            total_providers,
            capabilities_distribution: capabilities_count,
            oldest_registration: self.models.values()
                .map(|m| m.created_at)
                .min(),
            newest_registration: self.models.values()
                .map(|m| m.created_at)
                .max(),
        }
    }
    
    /// Update model configuration
    pub fn update_model_config(&mut self, model_id: &str, config: ModelConfig) -> ModelResult<()> {
        let model = self.models.get_mut(model_id)
            .ok_or_else(|| ModelError::ModelNotFound(model_id.to_string()))?;
        
        // Validate new configuration against capabilities
        model.capabilities.validate_config(&config)?;
        
        model.default_config = config;
        Ok(())
    }
    
    /// Update model metadata
    pub fn update_model_metadata(&mut self, model_id: &str, metadata: HashMap<String, String>) -> ModelResult<()> {
        let model = self.models.get_mut(model_id)
            .ok_or_else(|| ModelError::ModelNotFound(model_id.to_string()))?;
        
        model.metadata = metadata;
        Ok(())
    }
    
    /// Remove all models from a specific provider
    pub fn unregister_provider_models(&mut self, provider_id: &str) -> ModelResult<Vec<RegisteredModel>> {
        let model_ids = self.provider_models.remove(provider_id)
            .unwrap_or_default();
        
        let mut removed_models = Vec::new();
        for model_id in model_ids {
            if let Some(model) = self.models.remove(&model_id) {
                removed_models.push(model);
            }
        }
        
        Ok(removed_models)
    }
    
    /// Check if a model exists
    pub fn has_model(&self, model_id: &str) -> bool {
        self.models.contains_key(model_id)
    }
    
    /// Check if a provider has any models
    pub fn has_provider(&self, provider_id: &str) -> bool {
        self.provider_models.contains_key(provider_id)
    }
    
    /// Get all provider IDs
    pub fn get_provider_ids(&self) -> Vec<String> {
        self.provider_models.keys().cloned().collect()
    }
    
    /// Validate registry consistency
    pub fn validate(&self) -> ModelResult<()> {
        // Check that all models in provider_models exist in models
        for (provider_id, model_ids) in &self.provider_models {
            for model_id in model_ids {
                let model = self.models.get(model_id)
                    .ok_or_else(|| ModelError::RegistryInconsistent(
                        format!("Model {} referenced by provider {} not found in registry", model_id, provider_id)
                    ))?;
                
                if model.provider_id != *provider_id {
                    return Err(ModelError::RegistryInconsistent(
                        format!("Model {} has provider_id {}, but is listed under provider {}", 
                               model_id, model.provider_id, provider_id)
                    ));
                }
            }
        }
        
        // Check that all models have corresponding provider entries
        for (model_id, model) in &self.models {
            let provider_models = self.provider_models.get(&model.provider_id)
                .ok_or_else(|| ModelError::RegistryInconsistent(
                    format!("Model {} has provider_id {}, but provider not found in registry", 
                           model_id, model.provider_id)
                ))?;
            
            if !provider_models.contains(model_id) {
                return Err(ModelError::RegistryInconsistent(
                    format!("Model {} not listed under its provider {}", model_id, model.provider_id)
                ));
            }
        }
        
        Ok(())
    }
}

/// Registry statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegistryStats {
    pub total_models: usize,
    pub total_providers: usize,
    pub capabilities_distribution: HashMap<String, usize>,
    pub oldest_registration: Option<chrono::DateTime<chrono::Utc>>,
    pub newest_registration: Option<chrono::DateTime<chrono::Utc>>,
}

impl Default for ModelRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::capabilities::{ModelCapabilities, CapabilityType};
    
    fn create_test_model(id: &str, provider_id: &str) -> RegisteredModel {
        RegisteredModel {
            id: id.to_string(),
            name: format!("Test Model {}", id),
            provider_id: provider_id.to_string(),
            capabilities: ModelCapabilities::new(vec![CapabilityType::TextGeneration]),
            default_config: ModelConfig::default(),
            created_at: chrono::Utc::now(),
            metadata: HashMap::new(),
        }
    }
    
    #[test]
    fn test_register_and_get_model() {
        let mut registry = ModelRegistry::new();
        let model = create_test_model("test-1", "provider-1");
        
        registry.register_model(model.clone()).unwrap();
        
        let retrieved = registry.get_model("test-1").unwrap();
        assert_eq!(retrieved.id, "test-1");
        assert_eq!(retrieved.provider_id, "provider-1");
    }
    
    #[test]
    fn test_register_duplicate_model() {
        let mut registry = ModelRegistry::new();
        let model = create_test_model("test-1", "provider-1");
        
        registry.register_model(model.clone()).unwrap();
        
        let result = registry.register_model(model);
        assert!(matches!(result, Err(ModelError::ModelAlreadyExists(_))));
    }
    
    #[test]
    fn test_unregister_model() {
        let mut registry = ModelRegistry::new();
        let model = create_test_model("test-1", "provider-1");
        
        registry.register_model(model.clone()).unwrap();
        
        let removed = registry.unregister_model("test-1").unwrap();
        assert_eq!(removed.id, "test-1");
        
        assert!(registry.get_model("test-1").is_none());
    }
    
    #[test]
    fn test_list_models_by_provider() {
        let mut registry = ModelRegistry::new();
        
        registry.register_model(create_test_model("model-1", "provider-1")).unwrap();
        registry.register_model(create_test_model("model-2", "provider-1")).unwrap();
        registry.register_model(create_test_model("model-3", "provider-2")).unwrap();
        
        let provider1_models = registry.list_models_by_provider("provider-1");
        assert_eq!(provider1_models.len(), 2);
        
        let provider2_models = registry.list_models_by_provider("provider-2");
        assert_eq!(provider2_models.len(), 1);
    }
    
    #[test]
    fn test_search_models() {
        let mut registry = ModelRegistry::new();
        
        let mut model1 = create_test_model("gpt-4", "openai");
        model1.name = "GPT-4 Turbo".to_string();
        
        let mut model2 = create_test_model("claude-3", "anthropic");
        model2.name = "Claude 3 Sonnet".to_string();
        
        registry.register_model(model1).unwrap();
        registry.register_model(model2).unwrap();
        
        let gpt_models = registry.search_models("gpt");
        assert_eq!(gpt_models.len(), 1);
        assert_eq!(gpt_models[0].id, "gpt-4");
        
        let turbo_models = registry.search_models("turbo");
        assert_eq!(turbo_models.len(), 1);
    }
    
    #[test]
    fn test_registry_validation() {
        let mut registry = ModelRegistry::new();
        
        registry.register_model(create_test_model("test-1", "provider-1")).unwrap();
        registry.register_model(create_test_model("test-2", "provider-1")).unwrap();
        
        // Registry should be valid
        registry.validate().unwrap();
        
        // Manually corrupt the registry
        registry.provider_models.insert("provider-2".to_string(), vec!["nonexistent".to_string()]);
        
        // Registry should now be invalid
        assert!(registry.validate().is_err());
    }
    
    #[test]
    fn test_unregister_provider_models() {
        let mut registry = ModelRegistry::new();
        
        registry.register_model(create_test_model("model-1", "provider-1")).unwrap();
        registry.register_model(create_test_model("model-2", "provider-1")).unwrap();
        registry.register_model(create_test_model("model-3", "provider-2")).unwrap();
        
        let removed = registry.unregister_provider_models("provider-1").unwrap();
        assert_eq!(removed.len(), 2);
        
        assert!(registry.get_model("model-1").is_none());
        assert!(registry.get_model("model-2").is_none());
        assert!(registry.get_model("model-3").is_some());
        
        assert!(!registry.has_provider("provider-1"));
        assert!(registry.has_provider("provider-2"));
    }
}