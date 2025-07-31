//! Property-based testing for REPL function calling
//! 
//! Uses QuickCheck to verify invariants and properties of the function calling system

use quickcheck::{Arbitrary, Gen, QuickCheck, TestResult};
use quickcheck_macros::quickcheck;
use proptest::prelude::*;
use std::collections::HashSet;

/// Custom types for property testing

#[derive(Debug, Clone)]
struct Prompt(String);

impl Arbitrary for Prompt {
    fn arbitrary(g: &mut Gen) -> Self {
        let choices = vec![
            // File operations
            "read", "show", "display", "view", "open",
            "write", "create", "save", "update", "modify",
            "list", "find", "search", "grep", "locate",
            
            // File paths
            "README.md", "Cargo.toml", "src/main.rs", "test.txt",
            "config.json", ".env", "package.json",
            
            // Patterns
            "*.py", "*.rs", "*.js", "**/*.md",
            
            // Noise words
            "the", "a", "in", "at", "file", "files", "directory",
            "please", "can you", "I need", "show me",
            
            // Conceptual words
            "what", "why", "how", "explain", "describe",
            "purpose", "meaning", "difference", "compare",
        ];
        
        let num_words = g.gen_range(1..10);
        let mut words = Vec::new();
        
        for _ in 0..num_words {
            let idx = g.gen_range(0..choices.len());
            words.push(choices[idx]);
        }
        
        Prompt(words.join(" "))
    }
}

#[derive(Debug, Clone)]
struct FilePath(String);

impl Arbitrary for FilePath {
    fn arbitrary(g: &mut Gen) -> Self {
        let dirs = vec!["", "src/", "tests/", "docs/", "config/"];
        let names = vec!["main", "test", "config", "utils", "lib"];
        let exts = vec![".rs", ".py", ".js", ".md", ".toml", ".json"];
        
        let dir = dirs[g.gen_range(0..dirs.len())];
        let name = names[g.gen_range(0..names.len())];
        let ext = exts[g.gen_range(0..exts.len())];
        
        FilePath(format!("{}{}{}", dir, name, ext))
    }
}

/// Properties to test

#[quickcheck]
fn prop_function_call_deterministic(prompt: String) -> bool {
    // Same prompt should always produce same function call decision
    let result1 = should_trigger_function_call(&prompt);
    let result2 = should_trigger_function_call(&prompt);
    result1 == result2
}

#[quickcheck]
fn prop_read_keywords_trigger_read(file: FilePath) -> bool {
    // Prompts with read keywords + file path should trigger read_file
    let prompts = vec![
        format!("Read the file {}", file.0),
        format!("Show me {}", file.0),
        format!("What's in {}?", file.0),
        format!("Display the contents of {}", file.0),
    ];
    
    prompts.iter().all(|p| {
        matches!(
            analyze_for_function_call(p),
            Some((func, _)) if func == "read_file"
        )
    })
}

#[quickcheck]
fn prop_questions_no_function(question_word: String) -> TestResult {
    let question_words = vec!["what", "why", "how", "when", "where", "who"];
    if !question_words.contains(&question_word.to_lowercase().as_str()) {
        return TestResult::discard();
    }
    
    let prompts = vec![
        format!("{} is Rust?", question_word),
        format!("{} does async work?", question_word),
        format!("{} should I use git?", question_word),
    ];
    
    TestResult::from_bool(
        prompts.iter().all(|p| analyze_for_function_call(p).is_none())
    )
}

#[quickcheck]
fn prop_write_creates_content(path: FilePath, content: String) -> TestResult {
    if content.is_empty() || path.0.is_empty() {
        return TestResult::discard();
    }
    
    let prompts = vec![
        format!("Create {} with content '{}'", path.0, content),
        format!("Write '{}' to {}", content, path.0),
        format!("Save the following to {}: {}", path.0, content),
    ];
    
    TestResult::from_bool(
        prompts.iter().all(|p| {
            matches!(
                analyze_for_function_call(p),
                Some((func, _)) if func == "write_file"
            )
        })
    )
}

#[quickcheck]
fn prop_pattern_preservation(pattern: String) -> TestResult {
    // Patterns in prompts should be preserved in function arguments
    let valid_patterns = vec!["*.py", "*.rs", "**/*.md", "test*", "*_test.js"];
    if !valid_patterns.iter().any(|p| pattern.contains(p)) {
        return TestResult::discard();
    }
    
    let prompt = format!("List all files matching {}", pattern);
    
    if let Some(("list_files", args)) = analyze_for_function_call(&prompt) {
        TestResult::from_bool(args.contains(&pattern))
    } else {
        TestResult::failed()
    }
}

/// Property: Function calls should be prefix-stable
#[quickcheck]
fn prop_prefix_stability(prompt: Prompt, suffix: String) -> bool {
    let base_result = analyze_for_function_call(&prompt.0);
    let extended_result = analyze_for_function_call(&format!("{} {}", prompt.0, suffix));
    
    // If base triggers a function, extended should trigger the same or additional
    match (base_result, extended_result) {
        (None, _) => true, // No function -> any result is valid
        (Some((f1, _)), Some((f2, _))) => f1 == f2, // Same function
        (Some(_), None) => false, // Should not lose function call
    }
}

/// Property: Context length affects accuracy monotonically
#[quickcheck]
fn prop_context_monotonic(prompts: Vec<String>) -> TestResult {
    if prompts.is_empty() || prompts.len() > 10 {
        return TestResult::discard();
    }
    
    let mut confidence_scores = Vec::new();
    let mut context = Vec::new();
    
    for prompt in prompts {
        context.push(prompt.clone());
        let score = get_confidence_score(&context.join(" "));
        confidence_scores.push(score);
    }
    
    // Check if confidence is non-decreasing (with small tolerance for noise)
    let is_monotonic = confidence_scores.windows(2).all(|w| {
        w[1] >= w[0] - 0.05 // Allow 5% decrease for noise
    });
    
    TestResult::from_bool(is_monotonic)
}

/// Property: Tool selection is order-independent for independent operations
#[quickcheck]
fn prop_order_independence(ops: Vec<String>) -> TestResult {
    let operations = vec![
        "Read README.md",
        "List all Python files",
        "Search for TODO",
        "Create test.txt with hello",
    ];
    
    if ops.len() > 4 {
        return TestResult::discard();
    }
    
    let selected_ops: Vec<_> = ops.iter()
        .filter_map(|_| {
            let idx = rand::random::<usize>() % operations.len();
            Some(operations[idx])
        })
        .collect();
    
    if selected_ops.len() < 2 {
        return TestResult::discard();
    }
    
    // Get functions for original order
    let functions1: Vec<_> = selected_ops.iter()
        .filter_map(|op| analyze_for_function_call(op))
        .map(|(f, _)| f)
        .collect();
    
    // Get functions for reversed order
    let functions2: Vec<_> = selected_ops.iter().rev()
        .filter_map(|op| analyze_for_function_call(op))
        .map(|(f, _)| f)
        .collect();
    
    // Sets should be equal (order doesn't matter for independent ops)
    let set1: HashSet<_> = functions1.into_iter().collect();
    let set2: HashSet<_> = functions2.into_iter().collect();
    
    TestResult::from_bool(set1 == set2)
}

/// Stateful property testing with proptest

proptest! {
    #[test]
    fn prop_no_injection(s in ".*") {
        // Function calls should be safe from injection attacks
        let prompt = format!("Read the file {}", s);
        
        if let Some((_, args)) = analyze_for_function_call(&prompt) {
            // Args should be properly escaped/sanitized
            prop_assert!(!args.contains("../"));
            prop_assert!(!args.contains("~"));
            prop_assert!(!args.contains("$"));
            prop_assert!(!args.contains("|"));
            prop_assert!(!args.contains("&&"));
        }
    }
    
    #[test]
    fn prop_file_path_normalization(
        segments in prop::collection::vec("[a-zA-Z0-9_-]+", 1..5),
        ext in prop::sample::select(vec!["rs", "py", "js", "md", "txt"])
    ) {
        let path = format!("{}.{}", segments.join("/"), ext);
        let prompt = format!("Read {}", path);
        
        if let Some(("read_file", args)) = analyze_for_function_call(&prompt) {
            // Path should be normalized
            prop_assert!(!args.contains("//"));
            prop_assert!(!args.contains("./"));
            prop_assert!(!args.starts_with('/'));
        }
    }
}

/// Mock implementations for testing
/// In real implementation, these would call the actual REPL logic

fn should_trigger_function_call(prompt: &str) -> bool {
    let trigger_words = ["read", "write", "create", "list", "search", "find", "show"];
    let question_words = ["what is", "why", "how does", "explain", "describe"];
    
    let lower = prompt.to_lowercase();
    
    // Check if it's a question
    if question_words.iter().any(|w| lower.starts_with(w)) {
        return false;
    }
    
    // Check for trigger words
    trigger_words.iter().any(|w| lower.contains(w))
}

fn analyze_for_function_call(prompt: &str) -> Option<(String, String)> {
    let lower = prompt.to_lowercase();
    
    if lower.contains("read") || lower.contains("show") || lower.contains("display") {
        if let Some(file) = extract_file_path(prompt) {
            return Some(("read_file".to_string(), file));
        }
    }
    
    if lower.contains("write") || lower.contains("create") || lower.contains("save") {
        if let Some(file) = extract_file_path(prompt) {
            return Some(("write_file".to_string(), file));
        }
    }
    
    if lower.contains("list") && lower.contains("file") {
        return Some(("list_files".to_string(), extract_pattern(prompt)));
    }
    
    if lower.contains("search") || lower.contains("find") {
        if let Some(pattern) = extract_search_term(prompt) {
            return Some(("search_code".to_string(), pattern));
        }
    }
    
    None
}

fn extract_file_path(prompt: &str) -> Option<String> {
    // Simple extraction - real implementation would be more sophisticated
    let words: Vec<&str> = prompt.split_whitespace().collect();
    
    for word in words {
        if word.contains('.') && !word.ends_with('.') {
            return Some(word.to_string());
        }
    }
    None
}

fn extract_pattern(prompt: &str) -> String {
    if prompt.contains("*.py") {
        "*.py".to_string()
    } else if prompt.contains("*.rs") {
        "*.rs".to_string()
    } else {
        "*".to_string()
    }
}

fn extract_search_term(prompt: &str) -> Option<String> {
    // Extract quoted terms
    if let Some(start) = prompt.find('\'') {
        if let Some(end) = prompt[start+1..].find('\'') {
            return Some(prompt[start+1..start+1+end].to_string());
        }
    }
    
    if let Some(start) = prompt.find('"') {
        if let Some(end) = prompt[start+1..].find('"') {
            return Some(prompt[start+1..start+1+end].to_string());
        }
    }
    
    None
}

fn get_confidence_score(_context: &str) -> f64 {
    // Mock confidence score - real implementation would use model
    0.8 + (rand::random::<f64>() * 0.2)
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_quickcheck_properties() {
        // Run with more iterations for thorough testing
        let mut qc = QuickCheck::new().tests(1000);
        
        qc.quickcheck(prop_function_call_deterministic as fn(String) -> bool);
        qc.quickcheck(prop_read_keywords_trigger_read as fn(FilePath) -> bool);
    }
}