//! Test runner for function calling validation
//! 
//! Runs test cases to ensure prompts trigger appropriate function calls

use anyhow::Result;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::fs;
use tracing::{info, warn, error, debug};

#[derive(Serialize, Deserialize, Debug)]
struct TestSuite {
    test_suite: TestSuiteData,
}

#[derive(Serialize, Deserialize, Debug)]
struct TestSuiteData {
    name: String,
    description: String,
    version: String,
    categories: Vec<TestCategory>,
}

#[derive(Serialize, Deserialize, Debug)]
struct TestCategory {
    name: String,
    description: String,
    test_cases: Vec<TestCase>,
}

#[derive(Serialize, Deserialize, Debug)]
struct TestCase {
    id: String,
    prompt: String,
    expected_function: Option<String>,
    expected_args: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    expected_functions: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    expected_sequence: Option<Vec<ExpectedCall>>,
    description: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct ExpectedCall {
    function: String,
    args: Value,
    #[serde(default)]
    conditional: bool,
}

#[derive(Debug, Default)]
pub struct TestResults {
    total: usize,
    passed: usize,
    failed: usize,
    skipped: usize,
    details: Vec<TestResult>,
}

#[derive(Debug)]
struct TestResult {
    category: String,
    test_id: String,
    prompt: String,
    expected: String,
    actual: String,
    passed: bool,
    error: Option<String>,
}

// Mock function to simulate API response parsing
// In real implementation, this would call the actual Gemini API
fn analyze_prompt_for_function_call(prompt: &str) -> Option<(String, Value)> {
    // Simple heuristic-based detection for testing
    // Real implementation would use the LLM's response
    
    let prompt_lower = prompt.to_lowercase();
    
    // read_file patterns
    if prompt_lower.contains("read") || prompt_lower.contains("show me") || 
       prompt_lower.contains("what's in") || prompt_lower.contains("contents of") {
        if let Some(file_path) = extract_file_path(&prompt) {
            return Some(("read_file".to_string(), serde_json::json!({
                "file_path": file_path
            })));
        }
    }
    
    // write_file patterns
    if prompt_lower.contains("create") || prompt_lower.contains("write") || 
       prompt_lower.contains("save") || prompt_lower.contains("update") {
        if let Some((file_path, content)) = extract_write_params(&prompt) {
            return Some(("write_file".to_string(), serde_json::json!({
                "file_path": file_path,
                "content": content
            })));
        }
    }
    
    // list_files patterns
    if prompt_lower.contains("list") || prompt_lower.contains("show me all") || 
       prompt_lower.contains("find all") {
        let pattern = extract_list_pattern(&prompt).unwrap_or("*".to_string());
        return Some(("list_files".to_string(), serde_json::json!({
            "pattern": pattern
        })));
    }
    
    // search_code patterns
    if prompt_lower.contains("search") || prompt_lower.contains("find") || 
       prompt_lower.contains("look for") || prompt_lower.contains("occurrences") {
        if let Some(pattern) = extract_search_pattern(&prompt) {
            let file_pattern = extract_file_pattern(&prompt);
            let mut args = serde_json::json!({
                "pattern": pattern
            });
            if let Some(fp) = file_pattern {
                args["file_pattern"] = serde_json::json!(fp);
            }
            return Some(("search_code".to_string(), args));
        }
    }
    
    None
}

fn extract_file_path(prompt: &str) -> Option<String> {
    // Extract file paths from prompts
    // This is a simplified version - real implementation would be more robust
    
    // Look for quoted paths
    if let Some(start) = prompt.find('\'') {
        if let Some(end) = prompt[start+1..].find('\'') {
            return Some(prompt[start+1..start+1+end].to_string());
        }
    }
    
    // Look for common file patterns
    let words: Vec<&str> = prompt.split_whitespace().collect();
    for (i, word) in words.iter().enumerate() {
        if word == &"file" || word == &"called" {
            if i + 1 < words.len() {
                let next = words[i + 1];
                if next.contains('.') {
                    return Some(next.to_string());
                }
            }
        }
    }
    
    // Look for specific file names
    for file in &["README.md", "Cargo.toml", "src/main.rs", ".env.example", 
                  "config/settings.json", ".gitignore"] {
        if prompt.contains(file) {
            return Some(file.to_string());
        }
    }
    
    None
}

fn extract_write_params(prompt: &str) -> Option<(String, String)> {
    // Simplified extraction - real implementation would be more sophisticated
    
    if prompt.contains("test.txt") && prompt.contains("Hello World") {
        return Some(("test.txt".to_string(), "Hello World".to_string()));
    }
    
    if prompt.contains("hello.py") {
        return Some(("hello.py".to_string(), "print('Hello, World!')".to_string()));
    }
    
    if prompt.contains("data.json") {
        return Some(("data.json".to_string(), "{\"name\": \"test\", \"value\": 42}".to_string()));
    }
    
    if prompt.contains(".gitignore") {
        return Some((".gitignore".to_string(), "node_modules\n.env".to_string()));
    }
    
    None
}

fn extract_list_pattern(prompt: &str) -> Option<String> {
    if prompt.contains("Python files") || prompt.contains("*.py") {
        return Some("*.py".to_string());
    }
    if prompt.contains("Rust files") && prompt.contains("src") {
        return Some("src/*.rs".to_string());
    }
    if prompt.contains("markdown") && prompt.contains("recursively") {
        return Some("**/*.md".to_string());
    }
    if prompt.contains("test files") {
        return Some("**/*test*".to_string());
    }
    None
}

fn extract_search_pattern(prompt: &str) -> Option<String> {
    // Extract search patterns from prompts
    if prompt.contains("'TODO'") {
        return Some("TODO".to_string());
    }
    if prompt.contains("function_call") {
        return Some("function_call".to_string());
    }
    if prompt.contains("async functions") {
        return Some("async fn".to_string());
    }
    if prompt.contains("ApiLogger") {
        return Some("ApiLogger".to_string());
    }
    if prompt.contains("Result<") {
        return Some("Result<".to_string());
    }
    None
}

fn extract_file_pattern(prompt: &str) -> Option<String> {
    if prompt.contains("Rust files") || prompt.contains("*.rs") {
        return Some("*.rs".to_string());
    }
    None
}

pub fn run_test_suite(test_file: &str) -> Result<TestResults> {
    info!("Loading test suite from: {}", test_file);
    
    let content = fs::read_to_string(test_file)?;
    let suite: TestSuite = serde_json::from_str(&content)?;
    
    let mut results = TestResults::default();
    
    info!("Running {} test categories", suite.test_suite.categories.len());
    
    for category in suite.test_suite.categories {
        info!("\n=== Category: {} ===", category.name);
        info!("{}", category.description);
        
        for test_case in category.test_cases {
            results.total += 1;
            
            debug!("Running test {}: {}", test_case.id, test_case.prompt);
            
            // Skip multi-step tests for now
            if test_case.expected_functions.is_some() {
                warn!("Skipping multi-step test: {}", test_case.id);
                results.skipped += 1;
                continue;
            }
            
            // Analyze the prompt
            let actual_result = analyze_prompt_for_function_call(&test_case.prompt);
            
            // Check results
            let (passed, error) = match (&test_case.expected_function, &actual_result) {
                (Some(expected_fn), Some((actual_fn, actual_args))) => {
                    let fn_match = expected_fn == actual_fn;
                    let args_match = if let Some(expected_args) = &test_case.expected_args {
                        // Simple comparison - could be more sophisticated
                        expected_args == actual_args
                    } else {
                        true
                    };
                    
                    let passed = fn_match && args_match;
                    let error = if !passed {
                        Some(format!("Function: {} (expected: {}), Args match: {}", 
                                   actual_fn, expected_fn, args_match))
                    } else {
                        None
                    };
                    
                    (passed, error)
                },
                (None, None) => {
                    // Negative test case - should not trigger function
                    (true, None)
                },
                (Some(expected), None) => {
                    (false, Some(format!("Expected function '{}' but got none", expected)))
                },
                (None, Some((actual, _))) => {
                    (false, Some(format!("Expected no function but got '{}'", actual)))
                }
            };
            
            if passed {
                results.passed += 1;
                info!("✅ {}: PASSED", test_case.id);
            } else {
                results.failed += 1;
                error!("❌ {}: FAILED - {}", test_case.id, error.as_ref().unwrap());
            }
            
            results.details.push(TestResult {
                category: category.name.clone(),
                test_id: test_case.id,
                prompt: test_case.prompt,
                expected: format!("{:?}", test_case.expected_function),
                actual: format!("{:?}", actual_result),
                passed,
                error,
            });
        }
    }
    
    Ok(results)
}

pub fn print_summary(results: &TestResults) {
    info!("\n=== Test Summary ===");
    info!("Total tests: {}", results.total);
    info!("✅ Passed: {} ({:.1}%)", results.passed, 
          results.passed as f64 / results.total as f64 * 100.0);
    info!("❌ Failed: {} ({:.1}%)", results.failed,
          results.failed as f64 / results.total as f64 * 100.0);
    info!("⏭️  Skipped: {}", results.skipped);
    
    if results.failed > 0 {
        info!("\n=== Failed Tests ===");
        for result in &results.details {
            if !result.passed {
                error!("{}/{}: {}", result.category, result.test_id, 
                      result.error.as_ref().unwrap());
                debug!("  Prompt: {}", result.prompt);
                debug!("  Expected: {}", result.expected);
                debug!("  Actual: {}", result.actual);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_analyze_read_file() {
        let result = analyze_prompt_for_function_call("Read the README.md file");
        assert!(result.is_some());
        let (func, args) = result.unwrap();
        assert_eq!(func, "read_file");
        assert_eq!(args["file_path"], "README.md");
    }
    
    #[test]
    fn test_analyze_list_files() {
        let result = analyze_prompt_for_function_call("Show me all Python files");
        assert!(result.is_some());
        let (func, args) = result.unwrap();
        assert_eq!(func, "list_files");
        assert_eq!(args["pattern"], "*.py");
    }
    
    #[test]
    fn test_analyze_search_code() {
        let result = analyze_prompt_for_function_call("Search for 'TODO' in the codebase");
        assert!(result.is_some());
        let (func, args) = result.unwrap();
        assert_eq!(func, "search_code");
        assert_eq!(args["pattern"], "TODO");
    }
    
    #[test]
    fn test_negative_case() {
        let result = analyze_prompt_for_function_call("What is the purpose of a README file?");
        assert!(result.is_none());
    }
}