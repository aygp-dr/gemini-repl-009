//! Context compaction strategies for managing conversation history
//!
//! Provides different strategies for reducing context size while preserving
//! important information.

use crate::api::{Content, Part};

/// Strategy for compacting conversation history
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompactionStrategy {
    /// Simple truncation - drop oldest messages (default, no LLM required)
    Truncate,
    /// Summarize older messages using the LLM
    Summarize,
}

impl Default for CompactionStrategy {
    fn default() -> Self {
        Self::Truncate
    }
}

/// Result of a compaction operation
#[derive(Debug)]
pub struct CompactionResult {
    /// The compacted conversation
    pub conversation: Vec<Content>,
    /// Number of messages that were compacted/removed
    pub messages_compacted: usize,
    /// Whether a summary was created
    pub summary_created: bool,
    /// Estimated tokens saved
    pub tokens_saved: usize,
}

/// Helper to create a summarization prompt for older messages
pub fn create_summary_prompt(messages: &[Content]) -> String {
    let mut prompt = String::from(
        "Please provide a concise summary of the following conversation segment. \
         Focus on key decisions, important information exchanged, and any actions taken. \
         Keep the summary brief but informative.\n\n---\n\n",
    );

    for msg in messages {
        let role = match msg.role.as_str() {
            "user" => "User",
            "model" => "Assistant",
            "function" => "Function",
            "system" => "System",
            _ => &msg.role,
        };

        if let Some(text) = msg.parts.first().and_then(|p| p.text.as_ref()) {
            prompt.push_str(&format!("{}: {}\n\n", role, text));
        } else if let Some(fc) = msg.parts.first().and_then(|p| p.function_call.as_ref()) {
            prompt.push_str(&format!("{}: [Called function {}]\n\n", role, fc.name));
        } else if let Some(fr) = msg.parts.first().and_then(|p| p.function_response.as_ref()) {
            prompt.push_str(&format!(
                "{}: [Function {} returned result]\n\n",
                role, fr.name
            ));
        }
    }

    prompt.push_str("---\n\nProvide a summary:");
    prompt
}

/// Create a summary message to insert into the conversation
pub fn create_summary_message(summary_text: &str) -> Content {
    Content {
        role: "system".to_string(),
        parts: vec![Part {
            text: Some(format!("[Previous conversation summary]\n{}", summary_text)),
            function_call: None,
            function_response: None,
        }],
    }
}

/// Identify messages that should be candidates for summarization
///
/// Returns the indices of messages that can be summarized (excludes system messages
/// and the most recent messages which should be kept verbatim)
pub fn identify_summarization_candidates(
    conversation: &[Content],
    keep_recent: usize,
) -> Vec<usize> {
    let non_system: Vec<usize> = conversation
        .iter()
        .enumerate()
        .filter(|(_, c)| c.role != "system")
        .map(|(i, _)| i)
        .collect();

    // Keep the most recent N messages
    if non_system.len() <= keep_recent {
        return Vec::new();
    }

    non_system[..non_system.len() - keep_recent].to_vec()
}

/// Check if messages are suitable for summarization (have enough content)
pub fn can_summarize(messages: &[Content]) -> bool {
    // Need at least 2 messages with text content
    let text_messages = messages
        .iter()
        .filter(|c| c.parts.first().and_then(|p| p.text.as_ref()).is_some())
        .count();
    text_messages >= 2
}

#[cfg(test)]
mod tests {
    use super::*;

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
    fn test_compaction_strategy_default() {
        assert_eq!(CompactionStrategy::default(), CompactionStrategy::Truncate);
    }

    #[test]
    fn test_create_summary_prompt() {
        let messages = vec![
            make_message("user", "Hello, how are you?"),
            make_message("model", "I'm doing well, thank you!"),
        ];

        let prompt = create_summary_prompt(&messages);
        assert!(prompt.contains("User: Hello, how are you?"));
        assert!(prompt.contains("Assistant: I'm doing well, thank you!"));
        assert!(prompt.contains("summary"));
    }

    #[test]
    fn test_create_summary_message() {
        let summary = create_summary_message("The user asked about weather");
        assert_eq!(summary.role, "system");
        assert!(summary.parts[0]
            .text
            .as_ref()
            .unwrap()
            .contains("Previous conversation summary"));
    }

    #[test]
    fn test_identify_summarization_candidates() {
        let conversation = vec![
            make_message("system", "You are helpful"),
            make_message("user", "Message 1"),
            make_message("model", "Response 1"),
            make_message("user", "Message 2"),
            make_message("model", "Response 2"),
            make_message("user", "Message 3"),
            make_message("model", "Response 3"),
        ];

        // Keep last 4 messages
        let candidates = identify_summarization_candidates(&conversation, 4);
        assert_eq!(candidates.len(), 2); // Indices 1 and 2
        assert_eq!(candidates, vec![1, 2]);
    }

    #[test]
    fn test_identify_summarization_candidates_not_enough() {
        let conversation = vec![make_message("user", "Hello"), make_message("model", "Hi")];

        let candidates = identify_summarization_candidates(&conversation, 4);
        assert!(candidates.is_empty());
    }

    #[test]
    fn test_can_summarize() {
        let messages = vec![make_message("user", "Hello"), make_message("model", "Hi")];
        assert!(can_summarize(&messages));

        let single = vec![make_message("user", "Hello")];
        assert!(!can_summarize(&single));
    }
}
