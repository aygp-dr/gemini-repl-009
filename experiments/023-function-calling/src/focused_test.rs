#!/usr/bin/env rust
//! Focused test for function calling with explicit instructions

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

fn create_aggressive_tools() -> Vec<Tool> {
    vec![Tool {
        function_declarations: vec![
            FunctionDeclaration {
                name: "read_file".to_string(),
                description: "IMPORTANT: Use this tool to read file contents. Do not claim you cannot access files.".to_string(),
                parameters: FunctionParameters {
                    param_type: "object".to_string(),
                    properties: HashMap::from([
                        ("file_path".to_string(), ParameterProperty {
                            prop_type: "string".to_string(),
                            description: "Path to the file to read (e.g. 'Makefile', 'README.md')".to_string(),
                            items: None,
                        }),
                    ]),
                    required: vec!["file_path".to_string()],
                },
            },
        ],
    }]
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_target(false)
        .with_env_filter("focused_test=debug,info")
        .init();

    info!("=== FOCUSED FUNCTION CALLING TEST ===");
    
    let api_key = env::var("GEMINI_API_KEY")?;
    let model = "gemini-2.0-flash-lite";
    
    info!("Using model: {}", model);
    info!("Testing with VERY explicit instructions...");
    
    let tools = create_aggressive_tools();
    let client = Client::new();
    
    // Ultra-explicit test case
    let request = GenerateRequest {
        contents: vec![
            Content {
                role: "user".to_string(),
                parts: vec![Part::Text { 
                    text: "SYSTEM: You are a file system assistant. You have a read_file tool available. When I ask about file contents, you MUST use the read_file tool. Do NOT say you cannot access files. Use the tools provided.".to_string() 
                }],
            },
            Content {
                role: "model".to_string(),
                parts: vec![Part::Text { 
                    text: "Understood. I have the read_file tool and will use it to read files when requested. I will not claim I cannot access files.".to_string() 
                }],
            },
            Content {
                role: "user".to_string(),
                parts: vec![Part::Text { 
                    text: "Use the read_file tool to read the Makefile. The file_path parameter should be 'Makefile'.".to_string() 
                }],
            }
        ],
        tools: Some(tools),
    };
    
    info!("Making API call with ultra-explicit instructions...");
    
    let url = format!(
        "https://generativelanguage.googleapis.com/v1beta/models/{}:generateContent?key={}",
        model, api_key
    );
    
    let response = client
        .post(&url)
        .json(&request)
        .send()
        .await?;
    
    let status = response.status();
    let text = response.text().await?;
    
    info!("Response status: {}", status);
    
    if status.is_success() {
        if let Ok(response_json) = serde_json::from_str::<Value>(&text) {
            info!("✅ API call successful!");
            
            // Check for function calls
            if let Some(candidates) = response_json["candidates"].as_array() {
                for candidate in candidates {
                    if let Some(parts) = candidate["content"]["parts"].as_array() {
                        for part in parts {
                            if let Some(func_call) = part.get("functionCall") {
                                info!("🎉 FUNCTION CALL DETECTED!");
                                info!("Function: {}", func_call["name"]);
                                info!("Args: {}", func_call["args"]);
                                
                                if func_call["name"] == "read_file" {
                                    info!("✅ CORRECT: read_file function called!");
                                    
                                    if let Some(file_path) = func_call["args"]["file_path"].as_str() {
                                        info!("File path: {}", file_path);
                                        
                                        // Execute the function
                                        match fs::read_to_string(file_path) {
                                            Ok(content) => {
                                                info!("✅ File read successfully: {} bytes", content.len());
                                                info!("First 100 chars: {}", &content[..content.len().min(100)]);
                                            },
                                            Err(e) => {
                                                warn!("File read error: {}", e);
                                            }
                                        }
                                    }
                                }
                                return Ok(());
                            } else if let Some(text) = part.get("text") {
                                info!("Text response: {}", text.as_str().unwrap_or(""));
                            }
                        }
                    }
                }
            }
            
            warn!("❌ NO FUNCTION CALLS DETECTED");
            info!("Full response:");
            println!("{}", serde_json::to_string_pretty(&response_json)?);
        } else {
            warn!("❌ Invalid JSON response");
            info!("Raw response: {}", text);
        }
    } else {
        warn!("❌ API error {}: {}", status, text);
    }
    
    Ok(())
}