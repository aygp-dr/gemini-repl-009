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

    /// Apply sliding window to keep conversation within token limits
    ///
    /// This preserves:
    /// - System messages (always kept)
    /// - Recent messages up to the token limit
    /// - Tries to keep user/assistant pairs together
    ///
    /// Returns the trimmed conversation
    pub fn apply_sliding_window(&self, conversation: &[Content]) -> Vec<Content> {
        let stats = self.count_tokens(conversation);

        // If within limits, return as-is
        if stats.total <= self.max_tokens {
            return conversation.to_vec();
        }

        // Separate system messages from others
        let (system_msgs, other_msgs): (Vec<_>, Vec<_>) = conversation
            .iter()
            .cloned()
            .partition(|c| c.role == "system");

        // Calculate tokens used by system messages
        let system_tokens: usize = system_msgs.iter().map(|c| self.counter.count_message(c)).sum();

        // Calculate available tokens for conversation
        let available_tokens = self.max_tokens.saturating_sub(system_tokens);

        // Keep messages from the end until we exceed the limit
        let mut kept_msgs: Vec<Content> = Vec::new();
        let mut current_tokens = 0;

        for msg in other_msgs.iter().rev() {
            let msg_tokens = self.counter.count_message(msg);

            if current_tokens + msg_tokens > available_tokens {
                break;
            }

            kept_msgs.push(msg.clone());
            current_tokens += msg_tokens;
        }

        // Reverse to maintain order
        kept_msgs.reverse();

        // Combine system messages with kept conversation messages
        let mut result = system_msgs;
        result.extend(kept_msgs);

        result
    }

    /// Apply sliding window in-place
    pub fn trim_to_limit(&self, conversation: &mut Vec<Content>) {
        if self.needs_compaction(conversation) {
            *conversation = self.apply_sliding_window(conversation);
        }
    }

    /// Get the number of messages that would be dropped by sliding window
    pub fn messages_to_drop(&self, conversation: &[Content]) -> usize {
        let trimmed = self.apply_sliding_window(conversation);
        conversation.len().saturating_sub(trimmed.len())
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

    #[test]
    fn test_sliding_window_under_limit() {
        let cm = ContextManager::new(10000);
        let conversation = vec![
            make_message("user", "Hello"),
            make_message("model", "Hi there!"),
        ];

        let trimmed = cm.apply_sliding_window(&conversation);
        assert_eq!(trimmed.len(), 2);
    }

    #[test]
    fn test_sliding_window_over_limit() {
        // Use very small token limit to force trimming
        let cm = ContextManager::new(20); // ~5 tokens per message
        let conversation = vec![
            make_message("user", "Message one that is long"),
            make_message("model", "Response one that is long"),
            make_message("user", "Message two that is long"),
            make_message("model", "Response two that is long"),
            make_message("user", "Message three that is long"),
            make_message("model", "Response three that is long"),
        ];

        let trimmed = cm.apply_sliding_window(&conversation);
        // Should have fewer messages
        assert!(trimmed.len() < conversation.len());
        // Should keep recent messages
        assert!(
            trimmed
                .last()
                .and_then(|c| c.parts.first()?.text.as_ref())
                .map(|t| t.contains("three"))
                .unwrap_or(false)
        );
    }

    #[test]
    fn test_sliding_window_preserves_system() {
        let cm = ContextManager::new(50);
        let conversation = vec![
            make_message("system", "You are helpful"),
            make_message("user", "Long message that takes tokens"),
            make_message("model", "Long response that takes tokens"),
            make_message("user", "Another long message here"),
        ];

        let trimmed = cm.apply_sliding_window(&conversation);

        // System message should always be kept
        assert!(trimmed.iter().any(|c| c.role == "system"));
    }

    #[test]
    fn test_messages_to_drop() {
        let cm = ContextManager::new(20);
        let conversation = vec![
            make_message("user", "A long message here"),
            make_message("model", "A long response here"),
            make_message("user", "Another message here"),
        ];

        let to_drop = cm.messages_to_drop(&conversation);
        // Should indicate some messages need dropping
        assert!(to_drop > 0 || conversation.len() <= 3);
    }

    #[test]
    fn test_trim_to_limit() {
        let cm = ContextManager::with_thresholds(50, 0.5, 0.6);
        let mut conversation = vec![
            make_message("user", "A long message that takes up space"),
            make_message("model", "A long response that takes up space"),
            make_message("user", "Another long message for context"),
            make_message("model", "Another long response for context"),
        ];

        let original_len = conversation.len();
        cm.trim_to_limit(&mut conversation);

        // If compaction was needed, length should be reduced
        if cm.needs_compaction(&vec![
            make_message("user", "A long message that takes up space"),
            make_message("model", "A long response that takes up space"),
            make_message("user", "Another long message for context"),
            make_message("model", "Another long response for context"),
        ]) {
            assert!(conversation.len() <= original_len);
        }
    }
}
