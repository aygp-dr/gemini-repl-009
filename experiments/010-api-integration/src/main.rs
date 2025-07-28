//! API Integration Experiment - Full Gemini API integration

use anyhow::Result;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::env;
use futures::StreamExt;

#[derive(Serialize)]
struct GenerateRequest {
    contents: Vec<Content>,
}

#[derive(Serialize)]
struct Content {
    parts: Vec<Part>,
}

#[derive(Serialize)]
struct Part {
    text: String,
}

#[derive(Deserialize, Debug)]
struct GenerateResponse {
    candidates: Option<Vec<Candidate>>,
    error: Option<ApiError>,
}

#[derive(Deserialize, Debug)]
struct Candidate {
    content: ContentResponse,
}

#[derive(Deserialize, Debug)]
struct ContentResponse {
    parts: Vec<PartResponse>,
}

#[derive(Deserialize, Debug)]
struct PartResponse {
    text: String,
}

#[derive(Deserialize, Debug)]
struct ApiError {
    code: i32,
    message: String,
}

#[tokio::main]
async fn main() -> Result<()> {
    println!("=== API Integration Test ===");
    
    let api_key = env::var("GEMINI_API_KEY")
        .unwrap_or_else(|_| "test-key-for-development".to_string());
    
    println!("Using API key: {}...", &api_key[..10.min(api_key.len())]);
    
    // Create client with proxy support
    let mut client_builder = Client::builder();
    if let Ok(proxy) = env::var("HTTPS_PROXY") {
        println!("Using proxy: {}", proxy);
        client_builder = client_builder.proxy(reqwest::Proxy::https(&proxy)?);
    }
    
    let client = client_builder.build()?;
    
    // Test API integration
    let test_cases = vec![
        "What is 2 + 40?",
        "Explain quantum computing in one sentence.",
        "Write hello world in Rust",
    ];
    
    for (i, prompt) in test_cases.iter().enumerate() {
        println!("\n--- Test {} ---", i + 1);
        println!("Prompt: {}", prompt);
        
        match send_request(&client, &api_key, prompt).await {
            Ok(response) => {
                if let Some(error) = response.error {
                    println!("✗ API Error: {} ({})", error.message, error.code);
                } else if let Some(candidates) = response.candidates {
                    if let Some(candidate) = candidates.first() {
                        let answer = &candidate.content.parts[0].text;
                        println!("✓ Response: {}", answer.trim());
                        
                        // Special check for math problem
                        if prompt.contains("2 + 40") && answer.contains("42") {
                            println!("✓ Correct math answer!");
                        }
                    }
                } else {
                    println!("✗ Empty response");
                }
            }
            Err(e) => {
                println!("✗ Request failed: {}", e);
            }
        }
    }
    
    println!("\n=== API Integration Test Complete ===");
    Ok(())
}

async fn send_request(client: &Client, api_key: &str, prompt: &str) -> Result<GenerateResponse> {
    let request = GenerateRequest {
        contents: vec![Content {
            parts: vec![Part {
                text: prompt.to_string(),
            }],
        }],
    };
    
    let url = format!(
        "https://generativelanguage.googleapis.com/v1beta/models/gemini-1.5-flash:generateContent?key={}",
        api_key
    );
    
    let response = client
        .post(&url)
        .json(&request)
        .send()
        .await?;
    
    let response_text = response.text().await?;
    let parsed: GenerateResponse = serde_json::from_str(&response_text)?;
    
    Ok(parsed)
}