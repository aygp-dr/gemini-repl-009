//! Function Calling Experiment for Gemini API
//! 
//! Implements Gemini API function calling with core file tools (Phase 1)
//! Follows the OpenAPI schema format for function declarations

use anyhow::Result;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::HashMap;
use std::env;
use std::fs;
use std::path::Path;
use tracing::{info, warn, debug};

// === Gemini API Types ===

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

#[derive(Deserialize, Debug)]
struct GenerateResponse {
    candidates: Option<Vec<Candidate>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<ApiError>,
}

#[derive(Deserialize, Debug)]
struct Candidate {
    content: Content,
}

#[derive(Deserialize, Debug)]
struct ApiError {
    code: i32,
    message: String,
}

// === Core File Tools (Phase 1) ===
// Based on gemini-repl-007/src/gemini_repl/tools/codebase_tools.py
// Matches the CODEBASE_TOOL_DECLARATIONS format for compatibility

fn create_core_tools() -> Vec<Tool> {
    vec![Tool {
        function_declarations: vec![
            FunctionDeclaration {
                name: "read_file".to_string(),
                description: "Read the contents of a file from the filesystem.".to_string(),
                parameters: FunctionParameters {
                    param_type: "object".to_string(),
                    properties: HashMap::from([
                        ("file_path".to_string(), ParameterProperty {
                            prop_type: "string".to_string(),
                            description: "Path to the file to read (relative or absolute)".to_string(),
                            items: None,
                        }),
                    ]),
                    required: vec!["file_path".to_string()],
                },
            },
            FunctionDeclaration {
                name: "write_file".to_string(),
                description: "Write content to a file on the filesystem.".to_string(),
                parameters: FunctionParameters {
                    param_type: "object".to_string(),
                    properties: HashMap::from([
                        ("file_path".to_string(), ParameterProperty {
                            prop_type: "string".to_string(),
                            description: "Path to the file to write (relative or absolute)".to_string(),
                            items: None,
                        }),
                        ("content".to_string(), ParameterProperty {
                            prop_type: "string".to_string(),
                            description: "Content to write to the file".to_string(),
                            items: None,
                        }),
                    ]),
                    required: vec!["file_path".to_string(), "content".to_string()],
                },
            },
            FunctionDeclaration {
                name: "list_files".to_string(),
                description: "List files matching a glob pattern (supports ** for recursive).".to_string(),
                parameters: FunctionParameters {
                    param_type: "object".to_string(),
                    properties: HashMap::from([
                        ("pattern".to_string(), ParameterProperty {
                            prop_type: "string".to_string(),
                            description: "Glob pattern to match files (e.g., '*.rs', 'src/**/*.rs')".to_string(),
                            items: None,
                        }),
                    ]),
                    required: vec![],  // pattern is optional with default "*"
                },
            },
            FunctionDeclaration {
                name: "search_code".to_string(),
                description: "Search for patterns in code using ripgrep.".to_string(),
                parameters: FunctionParameters {
                    param_type: "object".to_string(),
                    properties: HashMap::from([
                        ("pattern".to_string(), ParameterProperty {
                            prop_type: "string".to_string(),
                            description: "Regular expression pattern to search for".to_string(),
                            items: None,
                        }),
                        ("file_pattern".to_string(), ParameterProperty {
                            prop_type: "string".to_string(),
                            description: "File pattern to search in (e.g., '*.rs', '*.toml')".to_string(),
                            items: None,
                        }),
                    ]),
                    required: vec!["pattern".to_string()],
                },
            },
        ],
    }]
}

// === Function Execution (NOOP for now) ===

fn execute_function(function_call: &FunctionCall) -> Result<Value> {
    info!("Executing function '{}' with args: {}", 
          function_call.name, 
          serde_json::to_string_pretty(&function_call.args)?);
    
    match function_call.name.as_str() {
        "read_file" => {
            let file_path = function_call.args["file_path"].as_str()
                .ok_or_else(|| anyhow::anyhow!("Missing file_path parameter"))?;
            
            // Read actual file
            match fs::read_to_string(file_path) {
                Ok(content) => {
                    let size = content.len();
                    Ok(json!({
                        "content": content,
                        "size": size,
                        "exists": true,
                        "file_path": file_path
                    }))
                },
                Err(e) => {
                    Ok(json!({
                        "content": null,
                        "exists": false,
                        "error": format!("Failed to read file: {}", e),
                        "file_path": file_path
                    }))
                }
            }
        },
        "write_file" => {
            let file_path = function_call.args["file_path"].as_str()
                .ok_or_else(|| anyhow::anyhow!("Missing file_path parameter"))?;
            let content = function_call.args["content"].as_str()
                .ok_or_else(|| anyhow::anyhow!("Missing content parameter"))?;
            
            // Create parent directories if needed
            if let Some(parent) = Path::new(file_path).parent() {
                fs::create_dir_all(parent)?;
            }
            
            // Write actual file
            match fs::write(file_path, content) {
                Ok(_) => {
                    Ok(json!({
                        "success": true,
                        "bytes_written": content.len(),
                        "file_path": file_path
                    }))
                },
                Err(e) => {
                    Ok(json!({
                        "success": false,
                        "error": format!("Failed to write file: {}", e),
                        "file_path": file_path
                    }))
                }
            }
        },
        "list_files" => {
            let pattern = function_call.args["pattern"].as_str().unwrap_or("*");
            
            // Simple implementation - list current directory
            let mut files = Vec::new();
            let mut directories = Vec::new();
            
            if let Ok(entries) = fs::read_dir(".") {
                for entry in entries.flatten() {
                    if let Ok(metadata) = entry.metadata() {
                        let name = entry.file_name().to_string_lossy().to_string();
                        
                        // Simple pattern matching
                        if pattern == "*" || 
                           (pattern.ends_with("*") && name.starts_with(&pattern[..pattern.len()-1])) ||
                           (pattern.starts_with("*") && name.ends_with(&pattern[1..])) {
                            if metadata.is_dir() {
                                directories.push(name);
                            } else {
                                files.push(name);
                            }
                        }
                    }
                }
            }
            
            Ok(json!({
                "pattern": pattern,
                "files": files,
                "directories": directories,
                "total": files.len() + directories.len()
            }))
        },
        "search_code" => {
            let pattern = function_call.args["pattern"].as_str()
                .ok_or_else(|| anyhow::anyhow!("Missing pattern parameter"))?;
            let file_pattern = function_call.args.get("file_pattern")
                .and_then(|v| v.as_str())
                .unwrap_or("*.rs");
            
            let mut matches = Vec::new();
            
            // Simple search in current directory
            if let Ok(entries) = fs::read_dir(".") {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if path.is_file() {
                        let name = path.to_string_lossy().to_string();
                        
                        // Simple file pattern matching
                        let matches_pattern = file_pattern == "*" || 
                            (file_pattern.starts_with("*") && name.ends_with(&file_pattern[1..]));
                        
                        if matches_pattern {
                            if let Ok(content) = fs::read_to_string(&path) {
                                for (line_num, line) in content.lines().enumerate() {
                                    if line.contains(pattern) {
                                        matches.push(json!({
                                            "file": name.clone(),
                                            "line": line_num + 1,
                                            "content": line.trim()
                                        }));
                                    }
                                }
                            }
                        }
                    }
                }
            }
            
            Ok(json!({
                "pattern": pattern,
                "file_pattern": file_pattern,
                "matches": matches,
                "total_matches": matches.len()
            }))
        },
        _ => Err(anyhow::anyhow!("Unknown function: {}", function_call.name))
    }
}

// === API Testing ===

async fn test_function_calling_flow() -> Result<()> {
    info!("=== Testing Gemini API Function Calling Flow ===");
    
    // Create API client
    let api_key = env::var("GEMINI_API_KEY")
        .unwrap_or_else(|_| "mock-api-key".to_string());
    let model = env::var("GEMINI_MODEL")
        .unwrap_or_else(|_| "gemini-2.0-flash-lite".to_string());
    
    let client = Client::new();
    let tools = create_core_tools();
    
    // Display available tools
    info!("\n--- Available Tools ---");
    for tool in &tools {
        for func in &tool.function_declarations {
            info!("  {} - {}", func.name, func.description);
        }
    }
    
    // Example conversation with function calling - add system instruction
    let mut conversation = vec![
        Content {
            role: "user".to_string(),
            parts: vec![Part::Text { 
                text: "You have access to file system tools including read_file, write_file, list_files, and search_code. Use these tools immediately when asked about file contents or operations. When asked about files like 'What are the target dependencies of Makefile?', use read_file with the appropriate file path.".to_string() 
            }],
        },
        Content {
            role: "model".to_string(),
            parts: vec![Part::Text { 
                text: "I understand. I have access to file system tools and will use them immediately when needed to read files, write files, list files, or search code.".to_string() 
            }],
        },
        Content {
            role: "user".to_string(),
            parts: vec![Part::Text { 
                text: "Read the README.md file and tell me what this project is about.".to_string() 
            }],
        }
    ];
    
    // Create request with tools
    let request = GenerateRequest {
        contents: conversation.clone(),
        tools: Some(tools.clone()),
    };
    
    info!("\n--- Request Structure ---");
    debug!("{}", serde_json::to_string_pretty(&request)?);
    
    // Simulate API response with function call
    let simulated_response = GenerateResponse {
        candidates: Some(vec![Candidate {
            content: Content {
                role: "model".to_string(),
                parts: vec![Part::FunctionCall {
                    function_call: FunctionCall {
                        name: "read_file".to_string(),
                        args: json!({ "file_path": "README.md" }),
                    }
                }],
            },
        }]),
        error: None,
    };
    
    info!("\n--- Simulated API Response ---");
    if let Some(candidates) = &simulated_response.candidates {
        for candidate in candidates {
            for part in &candidate.content.parts {
                match part {
                    Part::FunctionCall { function_call } => {
                        info!("Model requested function: {}", function_call.name);
                        info!("With arguments: {}", serde_json::to_string_pretty(&function_call.args)?);
                        
                        // Execute the function
                        let result = execute_function(function_call)?;
                        info!("\nFunction result: {}", serde_json::to_string_pretty(&result)?);
                        
                        // Add function response to conversation
                        conversation.push(Content {
                            role: "function".to_string(),
                            parts: vec![Part::FunctionResponse {
                                function_response: FunctionResponse {
                                    name: function_call.name.clone(),
                                    response: result,
                                }
                            }],
                        });
                    },
                    Part::Text { text } => {
                        info!("Model said: {}", text);
                    },
                    _ => {}
                }
            }
        }
    }
    
    // Show how to continue the conversation
    info!("\n--- Continuing Conversation ---");
    info!("Would send back to API with function response included");
    info!("Model would then provide final answer based on file content");
    
    // Show the actual API request format
    info!("\n--- Actual API Request Format ---");
    let full_request = GenerateRequest {
        contents: conversation.clone(),
        tools: Some(tools.clone()),
    };
    info!("Request JSON:");
    println!("{}", serde_json::to_string_pretty(&full_request)?);
    
    // Try actual API call if key is available
    if api_key != "mock-api-key" {
        info!("\n--- Making Real API Call ---");
        match make_api_call(&client, &model, &api_key, &full_request).await {
            Ok(response) => {
                info!("API Response: {:?}", response);
            }
            Err(e) => {
                warn!("API call failed (expected in test): {}", e);
            }
        }
    }
    
    Ok(())
}

async fn make_api_call(
    client: &Client,
    model: &str,
    api_key: &str,
    request: &GenerateRequest,
) -> Result<GenerateResponse> {
    let url = format!(
        "https://generativelanguage.googleapis.com/v1beta/models/{}:generateContent?key={}",
        model, api_key
    );
    
    let response = client
        .post(&url)
        .json(request)
        .send()
        .await?;
    
    let status = response.status();
    if !status.is_success() {
        let error_text = response.text().await?;
        return Err(anyhow::anyhow!("API error {}: {}", status, error_text));
    }
    
    let result: GenerateResponse = response.json().await?;
    Ok(result)
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_target(false)
        .with_env_filter("function_calling=debug,info")
        .init();

    info!("=== Gemini Function Calling Experiment ===");
    info!("Implementing core file tools (Phase 1)");
    info!("NOOP mode - functions return mock data\n");

    test_function_calling_flow().await?;
    
    info!("\n=== Experiment Complete ===");
    info!("\nNext steps:");
    info!("1. Implement actual file operations with security");
    info!("2. Add real API calls to Gemini");
    info!("3. Handle multi-turn function calling");
    info!("4. Integrate into main REPL");
    
    Ok(())
}