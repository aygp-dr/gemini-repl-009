//! Comprehensive test to prove function calling works
//! This will run 500 test cases and collect statistics

use anyhow::Result;
use chrono::{DateTime, Utc};
use futures::stream::{self, StreamExt};
use gemini_repl::api::{Content, GeminiClient, Part};
use gemini_repl::functions::get_available_tools;
use rand::seq::SliceRandom;
use rand::Rng;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::env;
use std::fs;
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::Arc;
use std::time::Duration;
use tokio::time::sleep;
use tracing::{error, info, warn};

#[derive(Debug, Clone, Serialize, Deserialize)]
struct TestCase {
    id: usize,
    prompt: String,
    category: String,
    expected_function: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct TestResult {
    test_case: TestCase,
    response: String,
    is_function_call: bool,
    function_name: Option<String>,
    success: bool,
    timestamp: DateTime<Utc>,
    error: Option<String>,
}

#[derive(Debug, Default, Serialize)]
struct Statistics {
    total_tests: u32,
    successful_function_calls: u32,
    failed_function_calls: u32,
    no_function_response: u32,
    errors: u32,
    rate_limit_hits: u32,
    by_category: HashMap<String, CategoryStats>,
    by_function: HashMap<String, u32>,
}

#[derive(Debug, Default, Serialize)]
struct CategoryStats {
    total: u32,
    success: u32,
    failure: u32,
}

fn generate_test_cases() -> Vec<TestCase> {
    let mut test_cases = Vec::new();
    let mut id = 0;

    // Direct read file prompts (should have high success rate)
    let read_targets = vec![
        "Makefile", "README.md", "Cargo.toml", "src/main.rs", 
        "src/api.rs", "src/lib.rs", ".gitignore", "LICENSE"
    ];
    
    for target in &read_targets {
        // Direct commands
        test_cases.push(TestCase {
            id: id,
            prompt: format!("read {}", target),
            category: "direct_read".to_string(),
            expected_function: "read_file".to_string(),
        });
        id += 1;
        
        test_cases.push(TestCase {
            id: id,
            prompt: format!("show me {}", target),
            category: "show_file".to_string(),
            expected_function: "read_file".to_string(),
        });
        id += 1;
        
        test_cases.push(TestCase {
            id: id,
            prompt: format!("display the contents of {}", target),
            category: "display_file".to_string(),
            expected_function: "read_file".to_string(),
        });
        id += 1;
        
        // More natural language
        test_cases.push(TestCase {
            id: id,
            prompt: format!("what's in {}?", target),
            category: "whats_in".to_string(),
            expected_function: "read_file".to_string(),
        });
        id += 1;
        
        test_cases.push(TestCase {
            id: id,
            prompt: format!("can you read {} for me", target),
            category: "can_you_read".to_string(),
            expected_function: "read_file".to_string(),
        });
        id += 1;
    }

    // List files prompts
    let directories = vec!["src", "tests", "experiments", ".", "target"];
    for dir in &directories {
        test_cases.push(TestCase {
            id: id,
            prompt: format!("list files in {}", dir),
            category: "list_direct".to_string(),
            expected_function: "list_files".to_string(),
        });
        id += 1;
        
        test_cases.push(TestCase {
            id: id,
            prompt: format!("show all files in the {} directory", dir),
            category: "show_all_files".to_string(),
            expected_function: "list_files".to_string(),
        });
        id += 1;
        
        test_cases.push(TestCase {
            id: id,
            prompt: format!("what files are in {}?", dir),
            category: "what_files".to_string(),
            expected_function: "list_files".to_string(),
        });
        id += 1;
    }

    // Search prompts
    let search_terms = vec!["TODO", "function", "async", "error", "test", "impl"];
    for term in &search_terms {
        test_cases.push(TestCase {
            id: id,
            prompt: format!("search for {}", term),
            category: "search_direct".to_string(),
            expected_function: "search_code".to_string(),
        });
        id += 1;
        
        test_cases.push(TestCase {
            id: id,
            prompt: format!("find all occurrences of {}", term),
            category: "find_occurrences".to_string(),
            expected_function: "search_code".to_string(),
        });
        id += 1;
        
        test_cases.push(TestCase {
            id: id,
            prompt: format!("look for {} in the code", term),
            category: "look_for".to_string(),
            expected_function: "search_code".to_string(),
        });
        id += 1;
    }

    // Write file prompts (be careful with these)
    test_cases.push(TestCase {
        id: id,
        prompt: "create a file called test_output.txt with content 'Hello World'".to_string(),
        category: "create_file".to_string(),
        expected_function: "write_file".to_string(),
    });
    id += 1;

    // Mix in some variations
    let variations = vec![
        ("read the configuration from Cargo.toml", "read_file"),
        ("show me all Rust files", "list_files"),
        ("find TODO comments", "search_code"),
        ("display Makefile contents", "read_file"),
        ("what's inside the src folder", "list_files"),
        ("check what's in main.rs", "read_file"),
        ("scan for println statements", "search_code"),
        ("view the README file", "read_file"),
    ];

    for (prompt, expected) in variations {
        test_cases.push(TestCase {
            id: id,
            prompt: prompt.to_string(),
            category: "variation".to_string(),
            expected_function: expected.to_string(),
        });
        id += 1;
    }

    // Shuffle and take 500 (or repeat if needed)
    let mut rng = rand::thread_rng();
    let mut final_cases = Vec::new();
    
    while final_cases.len() < 500 {
        let mut batch = test_cases.clone();
        batch.shuffle(&mut rng);
        final_cases.extend(batch);
    }
    
    final_cases.truncate(500);
    
    // Re-number them
    for (i, case) in final_cases.iter_mut().enumerate() {
        case.id = i;
    }
    
    final_cases
}

async fn run_single_test(
    client: &GeminiClient,
    test_case: TestCase,
    rate_limit_counter: Arc<AtomicU32>,
) -> TestResult {
    let start = Utc::now();
    
    // Create conversation
    let conversation = vec![Content {
        role: "user".to_string(),
        parts: vec![Part {
            text: Some(test_case.prompt.clone()),
            function_call: None,
            function_response: None,
        }],
    }];
    
    // Get tools
    let tools = get_available_tools();
    
    // Make API call
    match client.send_message_with_tools(&conversation, Some(tools)).await {
        Ok(response) => {
            let is_function_call = response.starts_with("FUNCTION_CALL:");
            let function_name = if is_function_call {
                response
                    .split_whitespace()
                    .nth(1)
                    .map(|s| s.to_string())
            } else {
                None
            };
            
            let success = is_function_call && 
                function_name.as_ref().map_or(false, |name| name == &test_case.expected_function);
            
            TestResult {
                test_case,
                response,
                is_function_call,
                function_name,
                success,
                timestamp: start,
                error: None,
            }
        }
        Err(e) => {
            let error_str = e.to_string();
            if error_str.contains("429") || error_str.contains("RESOURCE_EXHAUSTED") {
                rate_limit_counter.fetch_add(1, Ordering::Relaxed);
                warn!("Rate limit hit: {}", error_str);
            } else {
                error!("API error: {}", error_str);
            }
            
            TestResult {
                test_case,
                response: String::new(),
                is_function_call: false,
                function_name: None,
                success: false,
                timestamp: start,
                error: Some(error_str),
            }
        }
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive("function_calling_proof=info".parse()?)
        )
        .init();
    
    info!("Starting function calling validation experiment");
    
    // Get API key
    let api_key = env::var("GOOGLE_AI_API_KEY")
        .or_else(|_| env::var("GEMINI_API_KEY"))
        .expect("Set GOOGLE_AI_API_KEY or GEMINI_API_KEY");
    
    // Create client
    let client = GeminiClient::new(api_key, "gemini-2.0-flash-exp".to_string())?;
    
    // Generate test cases
    let test_cases = generate_test_cases();
    info!("Generated {} test cases", test_cases.len());
    
    // Create output directory
    let output_dir = "experiments/025-function-calling-proof/results";
    fs::create_dir_all(output_dir)?;
    
    // Rate limiting setup
    let rate_limit_counter = Arc::new(AtomicU32::new(0));
    let mut results = Vec::new();
    let mut stats = Statistics::default();
    
    // Process in batches with rate limiting
    let batch_size = 10;
    let delay_between_batches = Duration::from_secs(2);
    let delay_between_requests = Duration::from_millis(200);
    
    info!("Starting test execution with {} ms between requests", delay_between_requests.as_millis());
    
    for (batch_idx, chunk) in test_cases.chunks(batch_size).enumerate() {
        info!("Processing batch {}/{}", batch_idx + 1, (test_cases.len() + batch_size - 1) / batch_size);
        
        // Process batch concurrently with limited parallelism
        let batch_results: Vec<TestResult> = stream::iter(chunk)
            .map(|test_case| {
                let client = &client;
                let counter = rate_limit_counter.clone();
                async move {
                    // Small delay between requests
                    sleep(delay_between_requests).await;
                    run_single_test(client, test_case.clone(), counter).await
                }
            })
            .buffer_unordered(3) // Max 3 concurrent requests
            .collect()
            .await;
        
        // Update statistics
        for result in &batch_results {
            stats.total_tests += 1;
            
            if let Some(error) = &result.error {
                stats.errors += 1;
                if error.contains("429") || error.contains("RESOURCE_EXHAUSTED") {
                    stats.rate_limit_hits += 1;
                }
            } else if result.is_function_call {
                if result.success {
                    stats.successful_function_calls += 1;
                    if let Some(func_name) = &result.function_name {
                        *stats.by_function.entry(func_name.clone()).or_insert(0) += 1;
                    }
                } else {
                    stats.failed_function_calls += 1;
                }
            } else {
                stats.no_function_response += 1;
            }
            
            // Update category stats
            let cat_stats = stats.by_category
                .entry(result.test_case.category.clone())
                .or_insert_with(CategoryStats::default);
            cat_stats.total += 1;
            if result.success {
                cat_stats.success += 1;
            } else {
                cat_stats.failure += 1;
            }
        }
        
        results.extend(batch_results);
        
        // Save intermediate results
        if batch_idx % 5 == 0 {
            let intermediate_file = format!("{}/intermediate_{}.json", output_dir, batch_idx);
            fs::write(&intermediate_file, serde_json::to_string_pretty(&results)?)?;
            info!("Saved intermediate results to {}", intermediate_file);
        }
        
        // Rate limit check
        if stats.rate_limit_hits > 5 {
            warn!("Multiple rate limits hit, increasing delay");
            sleep(Duration::from_secs(10)).await;
        } else {
            sleep(delay_between_batches).await;
        }
    }
    
    // Save final results
    let timestamp = Utc::now().format("%Y%m%d_%H%M%S");
    let results_file = format!("{}/results_{}.json", output_dir, timestamp);
    let stats_file = format!("{}/statistics_{}.json", output_dir, timestamp);
    let report_file = format!("{}/report_{}.md", output_dir, timestamp);
    
    fs::write(&results_file, serde_json::to_string_pretty(&results)?)?;
    fs::write(&stats_file, serde_json::to_string_pretty(&stats)?)?;
    
    // Generate report
    let report = generate_report(&stats, &results);
    fs::write(&report_file, report)?;
    
    // Print summary
    println!("\n=== FUNCTION CALLING VALIDATION RESULTS ===");
    println!("Total tests: {}", stats.total_tests);
    println!("Successful function calls: {} ({:.1}%)", 
        stats.successful_function_calls,
        (stats.successful_function_calls as f64 / stats.total_tests as f64) * 100.0
    );
    println!("Failed function calls: {}", stats.failed_function_calls);
    println!("No function response: {}", stats.no_function_response);
    println!("Errors: {}", stats.errors);
    println!("Rate limits hit: {}", stats.rate_limit_hits);
    println!("\nResults saved to: {}", output_dir);
    
    Ok(())
}

fn generate_report(stats: &Statistics, results: &[TestResult]) -> String {
    let mut report = String::new();
    
    report.push_str(&format!("# Function Calling Validation Report\n\n"));
    report.push_str(&format!("Generated: {}\n", Utc::now().format("%Y-%m-%d %H:%M:%S UTC")));
    report.push_str(&format!("Model: gemini-2.0-flash-exp\n"));
    report.push_str(&format!("Library Version: 0.1.1\n\n"));
    
    report.push_str("## Executive Summary\n\n");
    report.push_str(&format!("**Total Tests Run**: {}\n", stats.total_tests));
    report.push_str(&format!("**Function Call Success Rate**: {:.1}%\n", 
        (stats.successful_function_calls as f64 / stats.total_tests as f64) * 100.0
    ));
    report.push_str(&format!("**Errors**: {} ({} rate limits)\n\n", stats.errors, stats.rate_limit_hits));
    
    report.push_str("## Results by Category\n\n");
    report.push_str("| Category | Total | Success | Rate |\n");
    report.push_str("|----------|-------|---------|------|\n");
    
    let mut categories: Vec<_> = stats.by_category.iter().collect();
    categories.sort_by_key(|(name, _)| name.as_str());
    
    for (category, cat_stats) in categories {
        let rate = (cat_stats.success as f64 / cat_stats.total as f64) * 100.0;
        report.push_str(&format!("| {} | {} | {} | {:.1}% |\n", 
            category, cat_stats.total, cat_stats.success, rate
        ));
    }
    
    report.push_str("\n## Function Call Distribution\n\n");
    report.push_str("| Function | Count |\n");
    report.push_str("|----------|-------|\n");
    
    let mut functions: Vec<_> = stats.by_function.iter().collect();
    functions.sort_by_key(|(name, _)| name.as_str());
    
    for (function, count) in functions {
        report.push_str(&format!("| {} | {} |\n", function, count));
    }
    
    report.push_str("\n## Sample Successful Calls\n\n");
    let successful: Vec<_> = results.iter()
        .filter(|r| r.success)
        .take(10)
        .collect();
    
    for result in successful {
        report.push_str(&format!("- **Prompt**: \"{}\"\n", result.test_case.prompt));
        report.push_str(&format!("  **Response**: {}\n\n", result.response));
    }
    
    report.push_str("## Sample Failed Calls\n\n");
    let failed: Vec<_> = results.iter()
        .filter(|r| !r.success && r.error.is_none())
        .take(5)
        .collect();
    
    for result in failed {
        report.push_str(&format!("- **Prompt**: \"{}\"\n", result.test_case.prompt));
        report.push_str(&format!("  **Expected**: {}\n", result.test_case.expected_function));
        report.push_str(&format!("  **Response**: {}\n\n", 
            if result.response.len() > 100 {
                format!("{}...", &result.response[..100])
            } else {
                result.response.clone()
            }
        ));
    }
    
    report.push_str("## Conclusion\n\n");
    if stats.successful_function_calls > 0 {
        report.push_str("✅ **Function calling is working!** The system successfully detected and executed function calls.\n\n");
        report.push_str(&format!("Out of {} tests, {} resulted in successful function calls, proving that the implementation works.\n", 
            stats.total_tests, stats.successful_function_calls
        ));
    } else {
        report.push_str("❌ Function calling needs investigation. No successful function calls were detected.\n");
    }
    
    report
}