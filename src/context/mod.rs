//! Context management for conversation history
//!
//! This module handles token counting, context window management,
//! and automatic compaction of conversation history.

#![allow(dead_code)]

pub mod tokenizer;

pub use tokenizer::{TokenCounter, TokenStats};

use crate::api::Content;

/// Context manager for handling conversation history within token limits
pub struct ContextManager {
    /// Maximum tokens allowed in context
    max_tokens: usize,
    /// Token counter
    counter: TokenCounter,
    /// Threshold percentage for compaction warning (0.0 - 1.0)
    warning_threshold: f32,
    /// Threshold percentage for automatic compaction (0.0 - 1.0)
    compaction_threshold: f32,
}

impl ContextManager {
    /// Create a new context manager with default settings
    pub fn new(max_tokens: usize) -> Self {
        Self {
            max_tokens,
            counter: TokenCounter::new(),
            warning_threshold: 0.8,
            compaction_threshold: 0.9,
        }
    }

    /// Create with custom thresholds
    pub fn with_thresholds(
        max_tokens: usize,
        warning_threshold: f32,
        compaction_threshold: f32,
    ) -> Self {
        Self {
            max_tokens,
            counter: TokenCounter::new(),
            warning_threshold: warning_threshold.clamp(0.0, 1.0),
            compaction_threshold: compaction_threshold.clamp(0.0, 1.0),
        }
    }

    /// Count tokens in a conversation
    pub fn count_tokens(&self, conversation: &[Content]) -> TokenStats {
        self.counter.count_conversation(conversation)
    }

    /// Get the percentage of context used (0.0 - 1.0)
    pub fn usage_percentage(&self, conversation: &[Content]) -> f32 {
        let stats = self.count_tokens(conversation);
        stats.total as f32 / self.max_tokens as f32
    }

    /// Check if we're approaching the token limit
    pub fn needs_warning(&self, conversation: &[Content]) -> bool {
        self.usage_percentage(conversation) >= self.warning_threshold
    }

    /// Check if compaction is needed
    pub fn needs_compaction(&self, conversation: &[Content]) -> bool {
        self.usage_percentage(conversation) >= self.compaction_threshold
    }

    /// Get remaining tokens
    pub fn remaining_tokens(&self, conversation: &[Content]) -> usize {
        let stats = self.count_tokens(conversation);
        self.max_tokens.saturating_sub(stats.total)
    }

    /// Get a formatted status string
    pub fn status(&self, conversation: &[Content]) -> String {
        let stats = self.count_tokens(conversation);
        let percentage = (stats.total as f32 / self.max_tokens as f32) * 100.0;
        format!(
            "{} / {} tokens ({:.1}%)",
            stats.total, self.max_tokens, percentage
        )
    }

    /// Get max tokens
    pub fn max_tokens(&self) -> usize {
        self.max_tokens
    }
}

impl Default for ContextManager {
    fn default() -> Self {
        // Default to 128K tokens (Gemini 2.0 Flash conservative limit)
        Self::new(128_000)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::api::Part;

    fn make_message(role: &str, text: &str) -> Content {
        Content {
            role: role.to_string(),
            parts: vec![Part {
                text: Some(text.to_string()),
                function_call: None,
                function_response: None,
            }],
        }
    }

    #[test]
    fn test_context_manager_creation() {
        let cm = ContextManager::new(10000);
        assert_eq!(cm.max_tokens(), 10000);
    }

    #[test]
    fn test_usage_percentage() {
        let cm = ContextManager::new(100);
        let conversation = vec![
            make_message("user", "Hello world"), // ~3 tokens
        ];

        let percentage = cm.usage_percentage(&conversation);
        assert!(percentage > 0.0);
        assert!(percentage < 0.1); // Should be a small percentage
    }

    #[test]
    fn test_needs_warning() {
        let cm = ContextManager::with_thresholds(100, 0.5, 0.9);

        // Small conversation - no warning
        let small = vec![make_message("user", "Hi")];
        assert!(!cm.needs_warning(&small));
    }

    #[test]
    fn test_status_format() {
        let cm = ContextManager::new(1000);
        let conversation = vec![make_message("user", "Test message")];
        let status = cm.status(&conversation);

        assert!(status.contains("/ 1000 tokens"));
        assert!(status.contains("%"));
    }
}
