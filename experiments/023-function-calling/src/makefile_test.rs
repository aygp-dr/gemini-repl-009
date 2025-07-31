//! Focused test for Makefile dependency queries
//! 
//! This validates that queries about Makefile targets trigger read_file

use anyhow::Result;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::HashMap;
use std::env;
use std::fs;
use tracing::{info, warn, debug};

// Import types from main (in real impl, these would be in a shared module)
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

fn create_file_tools() -> Vec<Tool> {
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
        ],
    }]
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_target(false)
        .with_env_filter("makefile_test=debug,info")
        .init();

    info!("=== Makefile Dependency Test ===");
    
    // Test cases that should trigger read_file for Makefile
    let test_prompts = vec![
        "Use the read_file tool to read Makefile and tell me what targets are defined",
        "What are the target dependencies of Makefile? Use read_file to check",
        "Show me the targets in the Makefile by reading it with read_file",
        "Read the Makefile file and tell me what it contains",
        "Use read_file with file_path='Makefile' to see what I can build",
    ];
    
    let tools = create_file_tools();
    
    for prompt in test_prompts {
        info!("\n--- Testing prompt: \"{}\" ---", prompt);
        
        let request = GenerateRequest {
            contents: vec![
                Content {
                    role: "user".to_string(),
                    parts: vec![Part::Text { 
                        text: "You have access to file system tools including read_file. Use read_file immediately when asked about file contents. You must use the tools provided rather than claiming you cannot access files.".to_string() 
                    }],
                },
                Content {
                    role: "model".to_string(),
                    parts: vec![Part::Text { 
                        text: "I understand. I will use the read_file tool when asked about file contents.".to_string() 
                    }],
                },
                Content {
                    role: "user".to_string(),
                    parts: vec![Part::Text { text: prompt.to_string() }],
                }
            ],
            tools: Some(tools.clone()),
        };
        
        info!("Expected behavior: Should call read_file with file_path='Makefile'");
        
        // Show the request that would be sent to Gemini
        debug!("Request JSON:");
        println!("{}", serde_json::to_string_pretty(&request)?);
        
        // Simulate what the expected response should look like
        let expected_response = json!({
            "candidates": [{
                "content": {
                    "role": "model",
                    "parts": [{
                        "functionCall": {
                            "name": "read_file",
                            "args": {
                                "file_path": "Makefile"
                            }
                        }
                    }]
                }
            }]
        });
        
        info!("Expected response structure:");
        println!("{}", serde_json::to_string_pretty(&expected_response)?);
        
        // Try actual API call if configured
        if let Ok(api_key) = env::var("GEMINI_API_KEY") {
            if api_key != "mock-api-key" && api_key != "your-api-key-here" {
                info!("\nMaking actual API call...");
                match make_api_call(&request, &api_key).await {
                    Ok(response_text) => {
                        info!("API Response:");
                        
                        // Try to parse and check for function call
                        if let Ok(response) = serde_json::from_str::<Value>(&response_text) {
                            if let Some(candidates) = response["candidates"].as_array() {
                                for candidate in candidates {
                                    if let Some(parts) = candidate["content"]["parts"].as_array() {
                                        for part in parts {
                                            if let Some(func_call) = part.get("functionCall") {
                                                info!("✅ Function call detected!");
                                                info!("  Function: {}", func_call["name"]);
                                                info!("  Args: {}", func_call["args"]);
                                                
                                                // Verify it's calling read_file with Makefile
                                                if func_call["name"] == "read_file" &&
                                                   func_call["args"]["file_path"] == "Makefile" {
                                                    info!("✅ CORRECT: Would read Makefile!");
                                                    
                                                    // Execute the function to show result
                                                    if let Ok(result) = execute_function_call(func_call) {
                                                        info!("Function result preview:");
                                                        if let Some(content) = result["content"].as_str() {
                                                            let preview = content.lines().take(5).collect::<Vec<_>>().join("\n");
                                                            info!("First 5 lines of Makefile:\n{}", preview);
                                                        }
                                                    }
                                                } else {
                                                    warn!("⚠️  Unexpected function or args");
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                            
                            // Pretty print the response
                            println!("{}", serde_json::to_string_pretty(&response)?);
                        }
                    }
                    Err(e) => {
                        warn!("API call failed: {}", e);
                    }
                }
            }
        }
    }
    
    info!("\n=== Test Complete ===");
    info!("\nFor the query 'What are the target dependencies of Makefile?':");
    info!("1. Model should recognize this requires reading Makefile");
    info!("2. Model should call read_file with file_path='Makefile'");
    info!("3. After receiving file contents, model can parse and answer about dependencies");
    
    Ok(())
}

async fn make_api_call(request: &GenerateRequest, api_key: &str) -> Result<String> {
    let client = Client::new();
    let model = env::var("GEMINI_MODEL").unwrap_or_else(|_| "gemini-2.0-flash-lite".to_string());
    
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
    let text = response.text().await?;
    
    if !status.is_success() {
        return Err(anyhow::anyhow!("API error {}: {}", status, text));
    }
    
    Ok(text)
}

// Execute function call locally to show what would happen
fn execute_function_call(func_call: &Value) -> Result<Value> {
    let name = func_call["name"].as_str().unwrap_or("unknown");
    let args = &func_call["args"];
    
    match name {
        "read_file" => {
            let file_path = args["file_path"].as_str().unwrap_or("unknown");
            info!("Executing read_file for: {}", file_path);
            
            match fs::read_to_string(file_path) {
                Ok(content) => {
                    info!("✅ Successfully read {} bytes from {}", content.len(), file_path);
                    Ok(json!({
                        "content": content,
                        "size": content.len(),
                        "exists": true,
                        "file_path": file_path
                    }))
                },
                Err(e) => {
                    warn!("❌ Failed to read {}: {}", file_path, e);
                    Ok(json!({
                        "content": null,
                        "exists": false,
                        "error": format!("{}", e),
                        "file_path": file_path
                    }))
                }
            }
        },
        _ => Err(anyhow::anyhow!("Unknown function: {}", name))
    }
}