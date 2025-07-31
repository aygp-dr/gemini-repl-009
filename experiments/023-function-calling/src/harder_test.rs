#!/usr/bin/env rust
//! HARDER test - Multiple function calling scenarios

use anyhow::Result;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::HashMap;
use std::env;
use std::fs;
use tracing::{info, warn, debug};

#[derive(Serialize, Deserialize, Debug, Clone)]
struct GenerateRequest {
    contents: Vec<Content>,
    #[serde(skip_serializing_if = "Option::is_none")]
    tools: Option<Vec<Tool>>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct Content {
    role: String,
    parts: Vec<Part>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(untagged)]
enum Part {
    Text { text: String },
    FunctionCall { 
        #[serde(rename = "functionCall")]
        function_call: FunctionCall 
    },
    FunctionResponse { 
        #[serde(rename = "functionResponse")]
        function_response: FunctionResponse 
    },
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct Tool {
    #[serde(rename = "functionDeclarations")]
    function_declarations: Vec<FunctionDeclaration>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct FunctionDeclaration {
    name: String,
    description: String,
    parameters: FunctionParameters,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct FunctionParameters {
    #[serde(rename = "type")]
    param_type: String,
    properties: HashMap<String, ParameterProperty>,
    required: Vec<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct ParameterProperty {
    #[serde(rename = "type")]
    prop_type: String,
    description: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    items: Option<Box<ParameterProperty>>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct FunctionCall {
    name: String,
    args: Value,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct FunctionResponse {
    name: String,
    response: Value,
}

fn create_all_tools() -> Vec<Tool> {
    vec![Tool {
        function_declarations: vec![
            FunctionDeclaration {
                name: "read_file".to_string(),
                description: "Read file contents from the filesystem".to_string(),
                parameters: FunctionParameters {
                    param_type: "object".to_string(),
                    properties: HashMap::from([
                        ("file_path".to_string(), ParameterProperty {
                            prop_type: "string".to_string(),
                            description: "Path to the file to read".to_string(),
                            items: None,
                        }),
                    ]),
                    required: vec!["file_path".to_string()],
                },
            },
            FunctionDeclaration { 
                name: "list_files".to_string(),
                description: "List files matching a pattern".to_string(),
                parameters: FunctionParameters {
                    param_type: "object".to_string(),
                    properties: HashMap::from([
                        ("pattern".to_string(), ParameterProperty {
                            prop_type: "string".to_string(),
                            description: "Glob pattern (e.g., '*.rs', '*.md')".to_string(),
                            items: None,
                        }),
                    ]),
                    required: vec!["pattern".to_string()],
                },
            },
            FunctionDeclaration {
                name: "write_file".to_string(),
                description: "Write content to a file".to_string(),
                parameters: FunctionParameters {
                    param_type: "object".to_string(),
                    properties: HashMap::from([
                        ("file_path".to_string(), ParameterProperty {
                            prop_type: "string".to_string(),
                            description: "Path to write the file".to_string(),
                            items: None,
                        }),
                        ("content".to_string(), ParameterProperty {
                            prop_type: "string".to_string(),
                            description: "Content to write".to_string(),
                            items: None,
                        }),
                    ]),
                    required: vec!["file_path".to_string(), "content".to_string()],
                },
            },
        ],
    }]
}

async fn test_function_call(client: &Client, model: &str, api_key: &str, prompt: &str, expected_function: &str) -> Result<bool> {
    let tools = create_all_tools();
    
    let request = GenerateRequest {
        contents: vec![
            Content {
                role: "user".to_string(),
                parts: vec![Part::Text { 
                    text: "You have file system tools: read_file, list_files, write_file. Use them when asked about files. Do not claim you cannot access files.".to_string() 
                }],
            },
            Content {
                role: "model".to_string(),
                parts: vec![Part::Text { 
                    text: "I understand. I will use the file system tools when appropriate.".to_string() 
                }],
            },
            Content {
                role: "user".to_string(),
                parts: vec![Part::Text { text: prompt.to_string() }],
            }
        ],
        tools: Some(tools),
    };
    
    let url = format!(
        "https://generativelanguage.googleapis.com/v1beta/models/{}:generateContent?key={}",
        model, api_key
    );
    
    let response = client.post(&url).json(&request).send().await?;
    let text = response.text().await?;
    
    if let Ok(response_json) = serde_json::from_str::<Value>(&text) {
        if let Some(candidates) = response_json["candidates"].as_array() {
            for candidate in candidates {
                if let Some(parts) = candidate["content"]["parts"].as_array() {
                    for part in parts {
                        if let Some(func_call) = part.get("functionCall") {
                            let function_name = func_call["name"].as_str().unwrap_or("");
                            info!("✅ Function called: {} (expected: {})", function_name, expected_function);
                            return Ok(function_name == expected_function);
                        }
                    }
                }
            }
        }
    }
    
    warn!("❌ No function call detected for: {}", prompt);
    Ok(false)
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_target(false)
        .with_env_filter("harder_test=debug,info")
        .init();

    info!("=== HARDER FUNCTION CALLING TESTS ===");
    
    let api_key = env::var("GEMINI_API_KEY")?;
    let model = "gemini-2.0-flash-lite";
    let client = Client::new();
    
    info!("Model: {}", model);
    info!("Running comprehensive function calling tests...");
    
    let test_cases = vec![
        ("Use read_file to read Cargo.toml", "read_file"),
        ("List all Rust files using list_files with pattern '*.rs'", "list_files"),
        ("Create a test file called hello.txt with write_file", "write_file"),
        ("What's in the README.md file? Use read_file", "read_file"),
        ("Show me all markdown files with list_files", "list_files"),
        ("Write 'Hello World' to test.txt using write_file", "write_file"),
        ("Read the Makefile using the read_file tool", "read_file"),
        ("Use list_files to find all files ending in .toml", "list_files"),
    ];
    
    let mut successes = 0;
    let mut total = test_cases.len();
    
    for (i, (prompt, expected_function)) in test_cases.iter().enumerate() {
        info!("\n--- Test {}/{}: {} ---", i + 1, total, prompt);
        
        match test_function_call(&client, model, &api_key, prompt, expected_function).await {
            Ok(true) => {
                info!("🎉 SUCCESS!");
                successes += 1;
            },
            Ok(false) => {
                warn!("❌ FAILED - wrong or no function call");
            },
            Err(e) => {
                warn!("❌ ERROR: {}", e);
            }
        }
        
        // Small delay between tests
        tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
    }
    
    info!("\n=== FINAL RESULTS ===");
    info!("✅ Successes: {}/{} ({:.1}%)", successes, total, successes as f64 / total as f64 * 100.0);
    
    if successes == total {
        info!("🏆 PERFECT SCORE! Function calling is working flawlessly!");
    } else if successes > total / 2 {
        info!("🎯 Good performance! Function calling is mostly working.");
    } else {
        warn!("🚨 Poor performance. Function calling needs improvement.");
    }
    
    Ok(())
}