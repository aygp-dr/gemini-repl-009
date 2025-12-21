//! Token counting for context management
//!
//! Provides approximate token counting for conversations.
//! Uses a simple heuristic (4 characters ≈ 1 token) which is
//! reasonably accurate for English text with modern LLM tokenizers.

use crate::api::Content;
use serde_json::Value;

/// Token counter for estimating context usage
pub struct TokenCounter {
    /// Characters per token (approximate)
    chars_per_token: f32,
}

impl TokenCounter {
    /// Create a new token counter with default settings
    pub fn new() -> Self {
        Self {
            chars_per_token: 4.0,
        }
    }

    /// Create with custom characters per token ratio
    pub fn with_ratio(chars_per_token: f32) -> Self {
        Self {
            chars_per_token: chars_per_token.max(1.0),
        }
    }

    /// Estimate tokens in a string
    pub fn count_text(&self, text: &str) -> usize {
        (text.len() as f32 / self.chars_per_token).ceil() as usize
    }

    /// Estimate tokens in a JSON value
    pub fn count_json(&self, value: &Value) -> usize {
        let json_str = value.to_string();
        self.count_text(&json_str)
    }

    /// Count tokens in a single message
    pub fn count_message(&self, content: &Content) -> usize {
        let mut tokens = 0;

        // Role overhead (approximately 2-4 tokens for role markup)
        tokens += 3;

        // Count each part
        for part in &content.parts {
            if let Some(text) = &part.text {
                tokens += self.count_text(text);
            }
            if let Some(fc) = &part.function_call {
                tokens += self.count_text(&fc.name);
                tokens += self.count_json(&fc.args);
                tokens += 5; // Overhead for function call structure
            }
            if let Some(fr) = &part.function_response {
                tokens += self.count_text(&fr.name);
                tokens += self.count_json(&fr.response);
                tokens += 5; // Overhead for function response structure
            }
        }

        tokens
    }

    /// Count tokens in a conversation
    pub fn count_conversation(&self, conversation: &[Content]) -> TokenStats {
        let mut stats = TokenStats::default();

        for content in conversation {
            let message_tokens = self.count_message(content);
            stats.total += message_tokens;

            match content.role.as_str() {
                "user" => {
                    stats.user_tokens += message_tokens;
                    stats.user_messages += 1;
                }
                "model" => {
                    stats.assistant_tokens += message_tokens;
                    stats.assistant_messages += 1;
                }
                "function" => {
                    stats.function_tokens += message_tokens;
                    stats.function_messages += 1;
                }
                "system" => {
                    stats.system_tokens += message_tokens;
                }
                _ => {}
            }
        }

        stats
    }
}

impl Default for TokenCounter {
    fn default() -> Self {
        Self::new()
    }
}

/// Statistics about token usage in a conversation
#[derive(Debug, Clone, Default)]
pub struct TokenStats {
    /// Total tokens in the conversation
    pub total: usize,
    /// Tokens from user messages
    pub user_tokens: usize,
    /// Tokens from assistant messages
    pub assistant_tokens: usize,
    /// Tokens from function responses
    pub function_tokens: usize,
    /// Tokens from system messages
    pub system_tokens: usize,
    /// Number of user messages
    pub user_messages: usize,
    /// Number of assistant messages
    pub assistant_messages: usize,
    /// Number of function messages
    pub function_messages: usize,
}

impl TokenStats {
    /// Get a formatted summary
    pub fn summary(&self) -> String {
        format!(
            "Total: {} tokens ({} user, {} assistant, {} function, {} system) | Messages: {} user, {} assistant, {} function",
            self.total,
            self.user_tokens,
            self.assistant_tokens,
            self.function_tokens,
            self.system_tokens,
            self.user_messages,
            self.assistant_messages,
            self.function_messages
        )
    }

    /// Get average tokens per message
    pub fn avg_tokens_per_message(&self) -> f32 {
        let total_messages = self.user_messages + self.assistant_messages + self.function_messages;
        if total_messages == 0 {
            0.0
        } else {
            self.total as f32 / total_messages as f32
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::api::Part;
    use serde_json::json;

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
    fn test_count_text() {
        let counter = TokenCounter::new();

        // 4 chars per token
        assert_eq!(counter.count_text("test"), 1); // 4 chars = 1 token
        assert_eq!(counter.count_text("testing"), 2); // 7 chars = 2 tokens (ceil)
        assert_eq!(counter.count_text("a longer test string"), 5); // 20 chars = 5 tokens
    }

    #[test]
    fn test_count_text_empty() {
        let counter = TokenCounter::new();
        assert_eq!(counter.count_text(""), 0);
    }

    #[test]
    fn test_count_message() {
        let counter = TokenCounter::new();
        let message = make_message("user", "Hello world");

        let tokens = counter.count_message(&message);
        // 3 (role overhead) + 3 (11 chars / 4 = 2.75, ceil = 3)
        assert!(tokens >= 5);
    }

    #[test]
    fn test_count_conversation() {
        let counter = TokenCounter::new();
        let conversation = vec![
            make_message("user", "Hello"),
            make_message("model", "Hi there!"),
            make_message("user", "How are you?"),
        ];

        let stats = counter.count_conversation(&conversation);

        assert!(stats.total > 0);
        assert_eq!(stats.user_messages, 2);
        assert_eq!(stats.assistant_messages, 1);
        assert!(stats.user_tokens > 0);
        assert!(stats.assistant_tokens > 0);
    }

    #[test]
    fn test_count_json() {
        let counter = TokenCounter::new();
        let value = json!({"path": "test.txt", "content": "hello"});

        let tokens = counter.count_json(&value);
        assert!(tokens > 5); // JSON structure adds overhead
    }

    #[test]
    fn test_token_stats_summary() {
        let stats = TokenStats {
            total: 100,
            user_tokens: 40,
            assistant_tokens: 50,
            function_tokens: 10,
            system_tokens: 0,
            user_messages: 2,
            assistant_messages: 2,
            function_messages: 1,
        };

        let summary = stats.summary();
        assert!(summary.contains("100 tokens"));
        assert!(summary.contains("40 user"));
        assert!(summary.contains("50 assistant"));
    }

    #[test]
    fn test_avg_tokens_per_message() {
        let stats = TokenStats {
            total: 100,
            user_messages: 2,
            assistant_messages: 2,
            function_messages: 1,
            ..Default::default()
        };

        let avg = stats.avg_tokens_per_message();
        assert!((avg - 20.0).abs() < 0.01); // 100 / 5 = 20
    }

    #[test]
    fn test_avg_tokens_empty() {
        let stats = TokenStats::default();
        assert_eq!(stats.avg_tokens_per_message(), 0.0);
    }

    #[test]
    fn test_custom_ratio() {
        let counter = TokenCounter::with_ratio(3.0);
        // 12 chars with 3 chars/token = 4 tokens
        assert_eq!(counter.count_text("Hello world!"), 4);
    }
}
