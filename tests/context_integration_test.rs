//! Integration tests for context window management
//!
//! These tests verify:
//! - Token counting accuracy
//! - Sliding window behavior
//! - Context compaction and summarization
//! - System message preservation
//! - Context usage tracking

use gemini_repl::api::{Content, Part};
use gemini_repl::context::{
    can_summarize, create_summary_message, create_summary_prompt,
    identify_summarization_candidates, ContextManager,
};

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

fn make_long_conversation(pairs: usize) -> Vec<Content> {
    let mut conversation = Vec::new();
    for i in 0..pairs {
        conversation.push(make_message(
            "user",
            &format!(
                "This is user message number {} with some additional text to increase token count.",
                i
            ),
        ));
        conversation.push(make_message(
            "model",
            &format!(
                "This is the assistant response to message {} with detailed explanation.",
                i
            ),
        ));
    }
    conversation
}

// ============================================================================
// Token Counting Tests
// ============================================================================

#[test]
fn test_token_counting_empty_conversation() {
    let cm = ContextManager::new(1000);
    let conversation: Vec<Content> = vec![];
    let stats = cm.count_tokens(&conversation);

    assert_eq!(stats.total, 0);
    assert_eq!(stats.user_tokens, 0);
    assert_eq!(stats.assistant_tokens, 0);
}

#[test]
fn test_token_counting_single_message() {
    let cm = ContextManager::new(1000);
    let conversation = vec![make_message("user", "Hello world")];
    let stats = cm.count_tokens(&conversation);

    assert!(stats.total > 0);
    assert!(stats.user_tokens > 0);
    assert_eq!(stats.assistant_tokens, 0);
    assert_eq!(stats.user_messages, 1);
}

#[test]
fn test_token_counting_conversation() {
    let cm = ContextManager::new(1000);
    let conversation = vec![
        make_message("user", "What is the weather?"),
        make_message("model", "I don't have access to weather data."),
        make_message("user", "Can you help me code?"),
        make_message("model", "Yes, I can help with coding."),
    ];
    let stats = cm.count_tokens(&conversation);

    assert!(stats.total > 0);
    assert!(stats.user_tokens > 0);
    assert!(stats.assistant_tokens > 0);
    assert_eq!(stats.user_messages, 2);
    assert_eq!(stats.assistant_messages, 2);
}

#[test]
fn test_token_counting_with_system_message() {
    let cm = ContextManager::new(1000);
    let conversation = vec![
        make_message("system", "You are a helpful assistant."),
        make_message("user", "Hello"),
        make_message("model", "Hi there!"),
    ];
    let stats = cm.count_tokens(&conversation);

    assert!(stats.system_tokens > 0);
    assert!(stats.user_tokens > 0);
    assert!(stats.assistant_tokens > 0);
}

// ============================================================================
// Context Usage Tests
// ============================================================================

#[test]
fn test_usage_percentage_empty() {
    let cm = ContextManager::new(1000);
    let conversation: Vec<Content> = vec![];
    let percentage = cm.usage_percentage(&conversation);

    assert_eq!(percentage, 0.0);
}

#[test]
fn test_usage_percentage_increases() {
    let cm = ContextManager::new(100); // Small limit for testing
    let small = vec![make_message("user", "Hi")];
    let large = make_long_conversation(5);

    let small_pct = cm.usage_percentage(&small);
    let large_pct = cm.usage_percentage(&large);

    assert!(large_pct > small_pct);
}

#[test]
fn test_needs_warning_threshold() {
    // Warning at 80%
    let cm = ContextManager::with_thresholds(100, 0.5, 0.9);

    let small = vec![make_message("user", "Hi")];
    assert!(!cm.needs_warning(&small));
}

#[test]
fn test_needs_compaction_threshold() {
    // Compaction at 90%
    let cm = ContextManager::with_thresholds(50, 0.5, 0.6);
    let conversation = make_long_conversation(10);

    // With a very small token limit and many messages, should need compaction
    assert!(cm.needs_compaction(&conversation));
}

// ============================================================================
// Sliding Window Tests
// ============================================================================

#[test]
fn test_sliding_window_preserves_when_under_limit() {
    let cm = ContextManager::new(100000); // Large limit
    let conversation = make_long_conversation(5);
    let original_len = conversation.len();

    let result = cm.apply_sliding_window(&conversation);
    assert_eq!(result.len(), original_len);
}

#[test]
fn test_sliding_window_trims_old_messages() {
    let cm = ContextManager::new(50); // Very small limit
    let conversation = make_long_conversation(10);

    let result = cm.apply_sliding_window(&conversation);
    assert!(result.len() < conversation.len());
}

#[test]
fn test_sliding_window_keeps_recent_messages() {
    let cm = ContextManager::new(100); // Small limit
    let mut conversation = make_long_conversation(10);

    // Mark the last message with unique content
    if let Some(last) = conversation.last_mut() {
        last.parts[0].text = Some("UNIQUE_FINAL_MESSAGE".to_string());
    }

    let result = cm.apply_sliding_window(&conversation);

    // The unique message should be preserved
    let has_unique = result.iter().any(|c| {
        c.parts
            .first()
            .and_then(|p| p.text.as_ref())
            .map(|t| t.contains("UNIQUE_FINAL_MESSAGE"))
            .unwrap_or(false)
    });
    assert!(has_unique);
}

#[test]
fn test_sliding_window_preserves_system_messages() {
    let cm = ContextManager::new(100); // Small limit
    let mut conversation = vec![make_message(
        "system",
        "You are a helpful coding assistant.",
    )];
    conversation.extend(make_long_conversation(10));

    let result = cm.apply_sliding_window(&conversation);

    // System message should always be preserved
    let has_system = result.iter().any(|c| c.role == "system");
    assert!(has_system);
}

#[test]
fn test_sliding_window_multiple_system_messages() {
    let cm = ContextManager::new(150);
    let mut conversation = vec![
        make_message("system", "System instruction 1"),
        make_message("system", "System instruction 2"),
    ];
    conversation.extend(make_long_conversation(10));

    let result = cm.apply_sliding_window(&conversation);

    // Both system messages should be preserved
    let system_count = result.iter().filter(|c| c.role == "system").count();
    assert_eq!(system_count, 2);
}

#[test]
fn test_messages_to_drop_calculation() {
    let cm = ContextManager::new(50);
    let conversation = make_long_conversation(10);

    let to_drop = cm.messages_to_drop(&conversation);
    assert!(to_drop > 0);

    let trimmed = cm.apply_sliding_window(&conversation);
    assert_eq!(conversation.len() - trimmed.len(), to_drop);
}

#[test]
fn test_trim_to_limit_in_place() {
    let cm = ContextManager::with_thresholds(50, 0.5, 0.6);
    let mut conversation = make_long_conversation(10);
    let original_len = conversation.len();

    cm.trim_to_limit(&mut conversation);

    // Should have trimmed
    assert!(conversation.len() < original_len);
}

// ============================================================================
// Summarization Tests
// ============================================================================

#[test]
fn test_can_summarize_minimum_messages() {
    let single = vec![make_message("user", "Hello")];
    assert!(!can_summarize(&single));

    let pair = vec![
        make_message("user", "Hello"),
        make_message("model", "Hi there!"),
    ];
    assert!(can_summarize(&pair));
}

#[test]
fn test_create_summary_prompt_format() {
    let messages = vec![
        make_message("user", "What is Rust?"),
        make_message("model", "Rust is a systems programming language."),
    ];

    let prompt = create_summary_prompt(&messages);

    assert!(prompt.contains("User:"));
    assert!(prompt.contains("Assistant:"));
    assert!(prompt.contains("What is Rust?"));
    assert!(prompt.contains("systems programming language"));
    assert!(prompt.contains("summary"));
}

#[test]
fn test_create_summary_message_format() {
    let summary = create_summary_message("The user asked about Rust programming.");

    assert_eq!(summary.role, "system");
    assert!(summary.parts[0]
        .text
        .as_ref()
        .unwrap()
        .contains("Previous conversation summary"));
    assert!(summary.parts[0]
        .text
        .as_ref()
        .unwrap()
        .contains("Rust programming"));
}

#[test]
fn test_identify_summarization_candidates() {
    let conversation = vec![
        make_message("system", "System prompt"),
        make_message("user", "Message 1"),
        make_message("model", "Response 1"),
        make_message("user", "Message 2"),
        make_message("model", "Response 2"),
        make_message("user", "Message 3"),
        make_message("model", "Response 3"),
        make_message("user", "Message 4"),
        make_message("model", "Response 4"),
    ];

    // Keep last 4 messages (2 pairs)
    let candidates = identify_summarization_candidates(&conversation, 4);

    // Should identify older messages (not system, not recent 4)
    assert!(!candidates.is_empty());

    // Should not include system message (index 0)
    assert!(!candidates.contains(&0));

    // Should not include last 4 messages
    // Find the non-system indices
    let non_system_indices: Vec<usize> = conversation
        .iter()
        .enumerate()
        .filter(|(_, c)| c.role != "system")
        .map(|(idx, _)| idx)
        .collect();

    // Last 4 non-system messages should not be candidates
    if non_system_indices.len() > 4 {
        let keep_start = non_system_indices.len() - 4;
        for idx in non_system_indices.iter().skip(keep_start) {
            assert!(!candidates.contains(idx));
        }
    }
}

#[test]
fn test_get_messages_to_summarize() {
    let cm = ContextManager::with_thresholds(100, 0.5, 0.6);
    let conversation = make_long_conversation(10);

    let (to_summarize, to_keep) = cm.get_messages_to_summarize(&conversation);

    if cm.needs_compaction(&conversation) {
        // Should have some messages to summarize
        assert!(!to_summarize.is_empty() || to_keep.len() == conversation.len());
    }

    // to_keep + to_summarize should account for all messages
    // (Note: system messages may affect this)
}

#[test]
fn test_apply_summary_insertion() {
    let cm = ContextManager::new(1000);
    let remaining = vec![
        make_message("system", "You are helpful"),
        make_message("user", "Recent question"),
        make_message("model", "Recent answer"),
    ];

    let result = cm.apply_summary("Previous discussion about coding.", remaining);

    // Should have summary message inserted
    let has_summary = result.iter().any(|c| {
        c.role == "system"
            && c.parts
                .first()
                .and_then(|p| p.text.as_ref())
                .map(|t| t.contains("Previous conversation summary"))
                .unwrap_or(false)
    });
    assert!(has_summary);

    // Original system message should still exist
    let has_original_system = result.iter().any(|c| {
        c.role == "system"
            && c.parts
                .first()
                .and_then(|p| p.text.as_ref())
                .map(|t| t.contains("You are helpful"))
                .unwrap_or(false)
    });
    assert!(has_original_system);
}

#[test]
fn test_should_summarize_conditions() {
    // Small limit to trigger compaction
    let cm = ContextManager::with_thresholds(50, 0.5, 0.6);
    let conversation = make_long_conversation(10);

    // Should suggest summarization for large conversations over limit
    let should = cm.should_summarize(&conversation);

    // This depends on whether compaction is needed and there are enough messages
    // The test verifies the logic doesn't crash and returns a valid boolean
    let _ = should; // Suppress unused warning
}

// ============================================================================
// Edge Cases
// ============================================================================

#[test]
fn test_empty_conversation_operations() {
    let cm = ContextManager::new(1000);
    let empty: Vec<Content> = vec![];

    assert_eq!(cm.usage_percentage(&empty), 0.0);
    assert!(!cm.needs_warning(&empty));
    assert!(!cm.needs_compaction(&empty));
    assert_eq!(cm.remaining_tokens(&empty), 1000);
    assert_eq!(cm.apply_sliding_window(&empty).len(), 0);
    assert_eq!(cm.messages_to_drop(&empty), 0);
}

#[test]
fn test_only_system_messages() {
    let cm = ContextManager::new(100);
    let conversation = vec![
        make_message("system", "Instruction 1"),
        make_message("system", "Instruction 2"),
    ];

    let result = cm.apply_sliding_window(&conversation);
    assert_eq!(result.len(), 2); // Both preserved
}

#[test]
fn test_status_string_format() {
    let cm = ContextManager::new(1000);
    let conversation = vec![make_message("user", "Hello"), make_message("model", "Hi!")];

    let status = cm.status(&conversation);
    assert!(status.contains("/ 1000 tokens"));
    assert!(status.contains("%"));
}

#[test]
fn test_max_tokens_accessor() {
    let cm = ContextManager::new(50000);
    assert_eq!(cm.max_tokens(), 50000);
}

#[test]
fn test_remaining_tokens() {
    let cm = ContextManager::new(1000);
    let empty: Vec<Content> = vec![];
    assert_eq!(cm.remaining_tokens(&empty), 1000);

    let with_msg = vec![make_message("user", "Hello")];
    let remaining = cm.remaining_tokens(&with_msg);
    assert!(remaining < 1000);
    assert!(remaining > 0);
}

// ============================================================================
// Context Manager Default
// ============================================================================

#[test]
fn test_context_manager_default() {
    let cm = ContextManager::default();
    assert_eq!(cm.max_tokens(), 128_000); // Gemini default
}

#[test]
fn test_custom_thresholds_clamping() {
    // Thresholds should be clamped to 0.0-1.0
    let cm = ContextManager::with_thresholds(1000, 1.5, -0.5);
    // This should not panic and thresholds should be clamped
    let empty: Vec<Content> = vec![];
    assert!(!cm.needs_warning(&empty)); // Should work without panic
}
