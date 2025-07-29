//! Gemini API client module

use anyhow::{Result, anyhow};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::env;

#[derive(Serialize)]
pub struct GenerateRequest {
    pub contents: Vec<Content>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Content {
    pub role: String,
    pub parts: Vec<Part>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Part {
    pub text: String,
}

#[derive(Deserialize, Debug)]
pub struct GenerateResponse {
    pub candidates: Option<Vec<Candidate>>,
    pub error: Option<ApiError>,
}

#[derive(Deserialize, Debug)]
pub struct Candidate {
    pub content: ContentResponse,
}

#[derive(Deserialize, Debug)]
pub struct ContentResponse {
    pub parts: Vec<PartResponse>,
}

#[derive(Deserialize, Debug)]
pub struct PartResponse {
    pub text: String,
}

#[derive(Deserialize, Debug)]
pub struct ApiError {
    pub code: i32,
    pub message: String,
}

pub struct GeminiClient {
    client: Client,
    api_key: String,
    model: String,
}

impl GeminiClient {
    pub fn new(api_key: String, model: String) -> Result<Self> {
        let mut client_builder = Client::builder();
        
        // Add proxy support if configured
        if let Ok(proxy_url) = env::var("HTTPS_PROXY") {
            tracing::info!("Using proxy: {}", proxy_url);
            client_builder = client_builder.proxy(reqwest::Proxy::https(&proxy_url)?);
        }
        
        let client = client_builder.build()?;
        
        Ok(Self {
            client,
            api_key,
            model,
        })
    }
    
    pub async fn send_message(&self, messages: &[Content]) -> Result<String> {
        let request = GenerateRequest {
            contents: messages.to_vec(),
        };
        
        let url = format!(
            "https://generativelanguage.googleapis.com/v1beta/models/{}:generateContent?key={}",
            self.model, self.api_key
        );
        
        let response = self.client
            .post(&url)
            .json(&request)
            .send()
            .await?;
        
        let _status = response.status();
        let response_text = response.text().await?;
        let parsed: GenerateResponse = serde_json::from_str(&response_text)?;
        
        if let Some(error) = parsed.error {
            return Err(anyhow!("API Error ({}): {}", error.code, error.message));
        }
        
        if let Some(candidates) = parsed.candidates {
            if let Some(candidate) = candidates.first() {
                if let Some(part) = candidate.content.parts.first() {
                    return Ok(part.text.trim().to_string());
                }
            }
        }
        
        Err(anyhow!("Empty response from API"))
    }
}