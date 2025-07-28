//! Rust API Client Test for Gemini

use anyhow::Result;
use clap::Parser;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::env;

#[derive(Parser)]
struct Args {
    /// API key (or use GEMINI_API_KEY env var)
    #[arg(short, long, env = "GEMINI_API_KEY")]
    api_key: Option<String>,
    
    /// Use proxy
    #[arg(short, long)]
    proxy: bool,
}

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
    status: String,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();
    
    let api_key = args.api_key.or_else(|| env::var("GEMINI_API_KEY").ok())
        .ok_or_else(|| anyhow::anyhow!("API key required"))?;
    
    println!("=== Rust Gemini API Test ===");
    println!("Using API Key: {}...", &api_key[..10.min(api_key.len())]);
    
    // Create HTTP client
    let mut client_builder = Client::builder();
    
    // Add proxy if requested
    if args.proxy {
        if let Ok(proxy_url) = env::var("HTTPS_PROXY") {
            println!("Using proxy: {}", proxy_url);
            client_builder = client_builder.proxy(reqwest::Proxy::https(&proxy_url)?);
        }
    }
    
    let client = client_builder.build()?;
    
    // Test 1: Generate content
    println!("\n1. Testing generate content...");
    
    let request = GenerateRequest {
        contents: vec![Content {
            parts: vec![Part {
                text: "What is 2 + 40? Just give the number.".to_string(),
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
    
    let status = response.status();
    println!("Status: {}", status);
    
    let response_text = response.text().await?;
    let parsed: GenerateResponse = serde_json::from_str(&response_text)?;
    
    if let Some(error) = parsed.error {
        println!("✗ API Error: {} ({})", error.message, error.code);
    } else if let Some(candidates) = parsed.candidates {
        if let Some(candidate) = candidates.first() {
            let answer = &candidate.content.parts[0].text;
            println!("✓ API Response: {}", answer.trim());
            
            if answer.contains("42") {
                println!("✓ Correct answer received!");
            }
        }
    }
    
    println!("\n=== Test Complete ===");
    Ok(())
}