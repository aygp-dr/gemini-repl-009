use anyhow::Result;
use serde_json::Value;
use std::env;

/// Debug program to see the actual API response structure
#[tokio::main]
async fn main() -> Result<()> {
    // Get API key
    let api_key = env::var("GOOGLE_AI_API_KEY")
        .or_else(|_| env::var("GEMINI_API_KEY"))
        .expect("Set GOOGLE_AI_API_KEY or GEMINI_API_KEY");
    
    // Create minimal request with tools
    let request = serde_json::json!({
        "contents": [{
            "role": "user",
            "parts": [{
                "text": "Read the file named Makefile and summarize what it does"
            }]
        }],
        "tools": [{
            "functionDeclarations": [{
                "name": "read_file",
                "description": "Read the contents of a file",
                "parameters": {
                    "type": "object",
                    "properties": {
                        "file_path": {
                            "type": "string",
                            "description": "Path to the file to read"
                        }
                    },
                    "required": ["file_path"]
                }
            }]
        }]
    });
    
    println!("=== REQUEST ===");
    println!("{}", serde_json::to_string_pretty(&request)?);
    
    // Make API call
    let client = reqwest::Client::new();
    let url = format!(
        "https://generativelanguage.googleapis.com/v1beta/models/gemini-2.0-flash-lite:generateContent?key={}",
        api_key
    );
    
    let response = client
        .post(&url)
        .json(&request)
        .send()
        .await?;
    
    let status = response.status();
    let response_text = response.text().await?;
    
    println!("\n=== RESPONSE STATUS ===");
    println!("{}", status);
    
    println!("\n=== RAW RESPONSE ===");
    println!("{}", response_text);
    
    // Parse as JSON to pretty print
    let parsed: Value = serde_json::from_str(&response_text)?;
    
    println!("\n=== PARSED RESPONSE ===");
    println!("{}", serde_json::to_string_pretty(&parsed)?);
    
    // Check if there's a function call
    if let Some(candidates) = parsed["candidates"].as_array() {
        for (i, candidate) in candidates.iter().enumerate() {
            println!("\n=== CANDIDATE {} ===", i);
            if let Some(parts) = candidate["content"]["parts"].as_array() {
                for (j, part) in parts.iter().enumerate() {
                    println!("Part {}: {}", j, serde_json::to_string_pretty(part)?);
                    
                    // Check for function call
                    if part.get("functionCall").is_some() {
                        println!("*** FUNCTION CALL DETECTED! ***");
                    }
                }
            }
        }
    }
    
    Ok(())
}