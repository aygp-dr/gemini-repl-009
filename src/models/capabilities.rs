//! Model Capabilities
//! 
//! Defines the capabilities that models can support and provides
//! validation logic for configurations against these capabilities.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::{ModelConfig, ModelError, ModelResult};

/// Represents the capabilities of a model
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelCapabilities {
    /// List of capabilities this model supports
    pub supported_capabilities: Vec<CapabilityType>,
    
    /// Parameter constraints for this model
    pub parameter_constraints: ParameterConstraints,
    
    /// Context window limits
    pub context_limits: ContextLimits,
    
    /// Rate limits
    pub rate_limits: Option<RateLimits>,
    
    /// Cost information
    pub cost_info: Option<CostInfo>,
    
    /// Additional capability metadata
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Types of capabilities a model can support
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CapabilityType {
    /// Basic text generation
    TextGeneration,
    
    /// Text completion (continuing existing text)
    TextCompletion,
    
    /// Text summarization
    TextSummarization,
    
    /// Question answering
    QuestionAnswering,
    
    /// Code generation
    CodeGeneration,
    
    /// Code explanation and analysis
    CodeAnalysis,
    
    /// Language translation
    Translation,
    
    /// Sentiment analysis
    SentimentAnalysis,
    
    /// Text classification
    TextClassification,
    
    /// Named entity recognition
    NamedEntityRecognition,
    
    /// Function/tool calling
    FunctionCalling,
    
    /// Image understanding
    ImageUnderstanding,
    
    /// Image generation
    ImageGeneration,
    
    /// Audio processing
    AudioProcessing,
    
    /// Video processing
    VideoProcessing,
    
    /// Multimodal processing
    MultimodalProcessing,
    
    /// Structured data extraction
    StructuredDataExtraction,
    
    /// Creative writing
    CreativeWriting,
    
    /// Mathematical reasoning
    MathematicalReasoning,
    
    /// Logical reasoning
    LogicalReasoning,
    
    /// Conversational AI
    ConversationalAI,
    
    /// Custom capability (with name)
    Custom(String),
}

impl CapabilityType {
    /// Get the string representation of this capability
    pub fn as_str(&self) -> &str {
        match self {
            CapabilityType::TextGeneration => "text_generation",
            CapabilityType::TextCompletion => "text_completion",
            CapabilityType::TextSummarization => "text_summarization",
            CapabilityType::QuestionAnswering => "question_answering",
            CapabilityType::CodeGeneration => "code_generation",
            CapabilityType::CodeAnalysis => "code_analysis",
            CapabilityType::Translation => "translation",
            CapabilityType::SentimentAnalysis => "sentiment_analysis",
            CapabilityType::TextClassification => "text_classification",
            CapabilityType::NamedEntityRecognition => "named_entity_recognition",
            CapabilityType::FunctionCalling => "function_calling",
            CapabilityType::ImageUnderstanding => "image_understanding",
            CapabilityType::ImageGeneration => "image_generation",
            CapabilityType::AudioProcessing => "audio_processing",
            CapabilityType::VideoProcessing => "video_processing",
            CapabilityType::MultimodalProcessing => "multimodal_processing",
            CapabilityType::StructuredDataExtraction => "structured_data_extraction",
            CapabilityType::CreativeWriting => "creative_writing",
            CapabilityType::MathematicalReasoning => "mathematical_reasoning",
            CapabilityType::LogicalReasoning => "logical_reasoning",
            CapabilityType::ConversationalAI => "conversational_ai",
            CapabilityType::Custom(name) => name,
        }
    }
    
    /// Parse capability from string
    pub fn from_str(s: &str) -> Self {
        match s {
            "text_generation" => CapabilityType::TextGeneration,
            "text_completion" => CapabilityType::TextCompletion,
            "text_summarization" => CapabilityType::TextSummarization,
            "question_answering" => CapabilityType::QuestionAnswering,
            "code_generation" => CapabilityType::CodeGeneration,
            "code_analysis" => CapabilityType::CodeAnalysis,
            "translation" => CapabilityType::Translation,
            "sentiment_analysis" => CapabilityType::SentimentAnalysis,
            "text_classification" => CapabilityType::TextClassification,
            "named_entity_recognition" => CapabilityType::NamedEntityRecognition,
            "function_calling" => CapabilityType::FunctionCalling,
            "image_understanding" => CapabilityType::ImageUnderstanding,
            "image_generation" => CapabilityType::ImageGeneration,
            "audio_processing" => CapabilityType::AudioProcessing,
            "video_processing" => CapabilityType::VideoProcessing,
            "multimodal_processing" => CapabilityType::MultimodalProcessing,
            "structured_data_extraction" => CapabilityType::StructuredDataExtraction,
            "creative_writing" => CapabilityType::CreativeWriting,
            "mathematical_reasoning" => CapabilityType::MathematicalReasoning,
            "logical_reasoning" => CapabilityType::LogicalReasoning,
            "conversational_ai" => CapabilityType::ConversationalAI,
            custom => CapabilityType::Custom(custom.to_string()),
        }
    }
}

/// Parameter constraints for model configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParameterConstraints {
    /// Temperature constraints
    pub temperature: Option<ParameterRange<f64>>,
    
    /// Top-p constraints
    pub top_p: Option<ParameterRange<f64>>,
    
    /// Top-k constraints
    pub top_k: Option<ParameterRange<u32>>,
    
    /// Maximum tokens constraints
    pub max_tokens: Option<ParameterRange<u32>>,
    
    /// Presence penalty constraints
    pub presence_penalty: Option<ParameterRange<f64>>,
    
    /// Frequency penalty constraints
    pub frequency_penalty: Option<ParameterRange<f64>>,
    
    /// Custom parameter constraints
    pub custom: HashMap<String, serde_json::Value>,
}

/// Range constraint for a parameter
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParameterRange<T> {
    pub min: Option<T>,
    pub max: Option<T>,
    pub default: Option<T>,
    pub step: Option<T>,
}

impl<T> ParameterRange<T>
where
    T: PartialOrd + Copy,
{
    /// Check if a value is within this range
    pub fn contains(&self, value: T) -> bool {
        if let Some(min) = self.min {
            if value < min {
                return false;
            }
        }
        
        if let Some(max) = self.max {
            if value > max {
                return false;
            }
        }
        
        true
    }
    
    /// Get the default value or a reasonable fallback
    pub fn get_default(&self) -> Option<T> {
        self.default
    }
}

/// Context window limits
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextLimits {
    /// Maximum input tokens
    pub max_input_tokens: Option<u32>,
    
    /// Maximum output tokens
    pub max_output_tokens: Option<u32>,
    
    /// Maximum total tokens (input + output)
    pub max_total_tokens: Option<u32>,
    
    /// Context window size
    pub context_window: Option<u32>,
    
    /// Whether the model supports unlimited context
    pub unlimited_context: bool,
}

/// Rate limit information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimits {
    /// Requests per minute
    pub requests_per_minute: Option<u32>,
    
    /// Requests per hour
    pub requests_per_hour: Option<u32>,
    
    /// Requests per day
    pub requests_per_day: Option<u32>,
    
    /// Tokens per minute
    pub tokens_per_minute: Option<u32>,
    
    /// Tokens per hour
    pub tokens_per_hour: Option<u32>,
    
    /// Tokens per day
    pub tokens_per_day: Option<u32>,
}

/// Cost information for model usage
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CostInfo {
    /// Cost per input token (in USD)
    pub cost_per_input_token: Option<f64>,
    
    /// Cost per output token (in USD)
    pub cost_per_output_token: Option<f64>,
    
    /// Fixed cost per request (in USD)
    pub cost_per_request: Option<f64>,
    
    /// Currency code
    pub currency: String,
    
    /// Billing model
    pub billing_model: BillingModel,
}

/// Different billing models
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BillingModel {
    /// Pay per token
    PayPerToken,
    
    /// Pay per request
    PayPerRequest,
    
    /// Subscription based
    Subscription,
    
    /// Free tier with limits
    FreeTier { daily_limit: u32 },
    
    /// Custom billing model
    Custom(String),
}

impl ModelCapabilities {
    /// Create new capabilities with basic text generation
    pub fn new(capabilities: Vec<CapabilityType>) -> Self {
        Self {
            supported_capabilities: capabilities,
            parameter_constraints: ParameterConstraints::default(),
            context_limits: ContextLimits::default(),
            rate_limits: None,
            cost_info: None,
            metadata: HashMap::new(),
        }
    }
    
    /// Create basic text generation capabilities
    pub fn text_generation() -> Self {
        Self::new(vec![CapabilityType::TextGeneration])
    }
    
    /// Create conversational AI capabilities
    pub fn conversational() -> Self {
        Self::new(vec![
            CapabilityType::TextGeneration,
            CapabilityType::ConversationalAI,
            CapabilityType::QuestionAnswering,
        ])
    }
    
    /// Create code-focused capabilities
    pub fn code_assistant() -> Self {
        Self::new(vec![
            CapabilityType::TextGeneration,
            CapabilityType::CodeGeneration,
            CapabilityType::CodeAnalysis,
            CapabilityType::QuestionAnswering,
        ])
    }
    
    /// Check if this model has a specific capability
    pub fn has_capability(&self, capability: &str) -> bool {
        self.supported_capabilities.iter()
            .any(|cap| cap.as_str() == capability)
    }
    
    /// Check if this model has a capability type
    pub fn has_capability_type(&self, capability: &CapabilityType) -> bool {
        self.supported_capabilities.contains(capability)
    }
    
    /// Add a capability
    pub fn add_capability(&mut self, capability: CapabilityType) {
        if !self.supported_capabilities.contains(&capability) {
            self.supported_capabilities.push(capability);
        }
    }
    
    /// Remove a capability
    pub fn remove_capability(&mut self, capability: &CapabilityType) {
        self.supported_capabilities.retain(|cap| cap != capability);
    }
    
    /// Validate a model configuration against these capabilities
    pub fn validate_config(&self, config: &ModelConfig) -> ModelResult<()> {
        // Validate temperature
        if let Some(temp) = config.temperature {
            if let Some(temp_range) = &self.parameter_constraints.temperature {
                if !temp_range.contains(temp) {
                    return Err(ModelError::InvalidParameter(
                        format!("Temperature {} is outside allowed range", temp)
                    ));
                }
            }
        }
        
        // Validate top_p
        if let Some(top_p) = config.top_p {
            if let Some(top_p_range) = &self.parameter_constraints.top_p {
                if !top_p_range.contains(top_p) {
                    return Err(ModelError::InvalidParameter(
                        format!("Top-p {} is outside allowed range", top_p)
                    ));
                }
            }
        }
        
        // Validate max_tokens
        if let Some(max_tokens) = config.max_tokens {
            if let Some(max_tokens_range) = &self.parameter_constraints.max_tokens {
                if !max_tokens_range.contains(max_tokens) {
                    return Err(ModelError::InvalidParameter(
                        format!("Max tokens {} is outside allowed range", max_tokens)
                    ));
                }
            }
            
            // Check against context limits
            if let Some(max_output) = self.context_limits.max_output_tokens {
                if max_tokens > max_output {
                    return Err(ModelError::InvalidParameter(
                        format!("Max tokens {} exceeds model limit of {}", max_tokens, max_output)
                    ));
                }
            }
        }
        
        // Validate presence_penalty
        if let Some(penalty) = config.presence_penalty {
            if let Some(penalty_range) = &self.parameter_constraints.presence_penalty {
                if !penalty_range.contains(penalty) {
                    return Err(ModelError::InvalidParameter(
                        format!("Presence penalty {} is outside allowed range", penalty)
                    ));
                }
            }
        }
        
        // Validate frequency_penalty
        if let Some(penalty) = config.frequency_penalty {
            if let Some(penalty_range) = &self.parameter_constraints.frequency_penalty {
                if !penalty_range.contains(penalty) {
                    return Err(ModelError::InvalidParameter(
                        format!("Frequency penalty {} is outside allowed range", penalty)
                    ));
                }
            }
        }
        
        Ok(())
    }
    
    /// Get capability compatibility score with another set of capabilities
    pub fn compatibility_score(&self, required_capabilities: &[CapabilityType]) -> f64 {
        if required_capabilities.is_empty() {
            return 1.0;
        }
        
        let matching = required_capabilities.iter()
            .filter(|cap| self.has_capability_type(cap))
            .count();
            
        matching as f64 / required_capabilities.len() as f64
    }
    
    /// Estimate cost for a given usage
    pub fn estimate_cost(&self, input_tokens: u32, output_tokens: u32) -> Option<f64> {
        self.cost_info.as_ref().map(|cost| {
            let input_cost = cost.cost_per_input_token.unwrap_or(0.0) * input_tokens as f64;
            let output_cost = cost.cost_per_output_token.unwrap_or(0.0) * output_tokens as f64;
            let request_cost = cost.cost_per_request.unwrap_or(0.0);
            
            input_cost + output_cost + request_cost
        })
    }
}

impl Default for ParameterConstraints {
    fn default() -> Self {
        Self {
            temperature: Some(ParameterRange {
                min: Some(0.0),
                max: Some(2.0),
                default: Some(0.7),
                step: Some(0.1),
            }),
            top_p: Some(ParameterRange {
                min: Some(0.0),
                max: Some(1.0),
                default: Some(0.9),
                step: Some(0.01),
            }),
            top_k: Some(ParameterRange {
                min: Some(1),
                max: Some(100),
                default: Some(40),
                step: Some(1),
            }),
            max_tokens: Some(ParameterRange {
                min: Some(1),
                max: Some(4096),
                default: Some(1024),
                step: Some(1),
            }),
            presence_penalty: Some(ParameterRange {
                min: Some(-2.0),
                max: Some(2.0),
                default: Some(0.0),
                step: Some(0.1),
            }),
            frequency_penalty: Some(ParameterRange {
                min: Some(-2.0),
                max: Some(2.0),
                default: Some(0.0),
                step: Some(0.1),
            }),
            custom: HashMap::new(),
        }
    }
}

impl Default for ContextLimits {
    fn default() -> Self {
        Self {
            max_input_tokens: Some(4096),
            max_output_tokens: Some(4096),
            max_total_tokens: Some(8192),
            context_window: Some(8192),
            unlimited_context: false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_capability_type_string_conversion() {
        let cap = CapabilityType::TextGeneration;
        assert_eq!(cap.as_str(), "text_generation");
        
        let parsed = CapabilityType::from_str("text_generation");
        assert_eq!(parsed, CapabilityType::TextGeneration);
        
        let custom = CapabilityType::Custom("my_custom_capability".to_string());
        assert_eq!(custom.as_str(), "my_custom_capability");
    }
    
    #[test]
    fn test_parameter_range_contains() {
        let range = ParameterRange {
            min: Some(0.0),
            max: Some(1.0),
            default: Some(0.5),
            step: Some(0.1),
        };
        
        assert!(range.contains(0.5));
        assert!(range.contains(0.0));
        assert!(range.contains(1.0));
        assert!(!range.contains(-0.1));
        assert!(!range.contains(1.1));
    }
    
    #[test]
    fn test_capabilities_validation() {
        let capabilities = ModelCapabilities::text_generation();
        let mut config = ModelConfig::default();
        
        // Valid config should pass
        config.temperature = Some(0.7);
        assert!(capabilities.validate_config(&config).is_ok());
        
        // Invalid temperature should fail
        config.temperature = Some(3.0);
        assert!(capabilities.validate_config(&config).is_err());
    }
    
    #[test]
    fn test_capability_compatibility_score() {
        let capabilities = ModelCapabilities::conversational();
        
        let required = vec![CapabilityType::TextGeneration];
        let score = capabilities.compatibility_score(&required);
        assert_eq!(score, 1.0); // Perfect match
        
        let required = vec![
            CapabilityType::TextGeneration,
            CapabilityType::ImageGeneration, // Not supported
        ];
        let score = capabilities.compatibility_score(&required);
        assert_eq!(score, 0.5); // 50% match
    }
    
    #[test]
    fn test_cost_estimation() {
        let mut capabilities = ModelCapabilities::text_generation();
        capabilities.cost_info = Some(CostInfo {
            cost_per_input_token: Some(0.0001),
            cost_per_output_token: Some(0.0002),
            cost_per_request: Some(0.001),
            currency: "USD".to_string(),
            billing_model: BillingModel::PayPerToken,
        });
        
        let cost = capabilities.estimate_cost(1000, 500);
        assert_eq!(cost, Some(0.001 + 0.1 + 0.1)); // request + input + output
    }
}