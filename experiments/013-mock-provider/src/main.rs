//! Experiment: Mock provider that replays recorded requests/responses

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

#[derive(Debug, Serialize, Deserialize)]
struct MockRequest {
    method: String,
    url: String,
    body: serde_json::Value,
}

#[derive(Debug, Serialize, Deserialize)]
struct MockResponse {
    status: u16,
    body: serde_json::Value,
}

#[derive(Debug, Serialize, Deserialize)]
struct Recording {
    request: MockRequest,
    response: MockResponse,
}

struct MockProvider {
    recordings: Vec<Recording>,
    current_index: usize,
}

impl MockProvider {
    fn load_from_jsonl(req_file: &Path, resp_file: &Path) -> Result<Self> {
        let mut recordings = Vec::new();
        
        // Read request logs
        let req_contents = fs::read_to_string(req_file)?;
        let resp_contents = fs::read_to_string(resp_file)?;
        
        let requests: Vec<serde_json::Value> = req_contents
            .lines()
            .filter(|l| !l.is_empty())
            .map(|line| serde_json::from_str(line))
            .collect::<Result<Vec<_>, _>>()?;
        
        let responses: Vec<serde_json::Value> = resp_contents
            .lines()
            .filter(|l| !l.is_empty())
            .map(|line| serde_json::from_str(line))
            .collect::<Result<Vec<_>, _>>()?;
        
        // Match requests with responses by ID
        for (req, resp) in requests.iter().zip(responses.iter()) {
            recordings.push(Recording {
                request: MockRequest {
                    method: req["method"].as_str().unwrap_or("GET").to_string(),
                    url: req["url"].as_str().unwrap_or("").to_string(),
                    body: req["body"].clone(),
                },
                response: MockResponse {
                    status: resp["status"].as_u64().unwrap_or(200) as u16,
                    body: resp["body"].clone(),
                },
            });
        }
        
        Ok(Self {
            recordings,
            current_index: 0,
        })
    }
    
    fn find_matching(&mut self, method: &str, body: &serde_json::Value) -> Option<&Recording> {
        // Simple matching: find first unused recording with matching method
        // In real implementation, would match on URL and body content
        for (i, recording) in self.recordings.iter().enumerate() {
            if i >= self.current_index && recording.request.method == method {
                self.current_index = i + 1;
                return Some(recording);
            }
        }
        None
    }
    
    async fn handle_request(
        &mut self,
        method: &str,
        _url: &str,
        body: &serde_json::Value,
    ) -> Result<serde_json::Value> {
        if let Some(recording) = self.find_matching(method, body) {
            println!("Mock: Found matching recording");
            Ok(recording.response.body.clone())
        } else {
            println!("Mock: No matching recording found");
            Ok(json!({
                "error": "No mock recording found",
                "request": body
            }))
        }
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    // First, create some mock recordings
    let mock_dir = Path::new("mock_data");
    fs::create_dir_all(mock_dir)?;
    
    // Create sample recordings
    let reqs = vec![
        json!({
            "id": "req-1",
            "method": "POST",
            "url": "https://api.example.com/generate",
            "body": {"prompt": "Hello"}
        }),
        json!({
            "id": "req-2",
            "method": "POST",
            "url": "https://api.example.com/generate",
            "body": {"prompt": "What is 2+2?"}
        }),
    ];
    
    let resps = vec![
        json!({
            "request_id": "req-1",
            "status": 200,
            "body": {"response": "Hello! How can I help you?"}
        }),
        json!({
            "request_id": "req-2",
            "status": 200,
            "body": {"response": "2+2 equals 4"}
        }),
    ];
    
    // Write to JSONL files
    let req_file = mock_dir.join("reqs.jsonl");
    let resp_file = mock_dir.join("resps.jsonl");
    
    fs::write(
        &req_file,
        reqs.iter()
            .map(|r| serde_json::to_string(r).unwrap())
            .collect::<Vec<_>>()
            .join("\n"),
    )?;
    
    fs::write(
        &resp_file,
        resps.iter()
            .map(|r| serde_json::to_string(r).unwrap())
            .collect::<Vec<_>>()
            .join("\n"),
    )?;
    
    println!("Created mock data files");
    
    // Load and test the mock provider
    let mut provider = MockProvider::load_from_jsonl(&req_file, &resp_file)?;
    println!("\nLoaded {} recordings", provider.recordings.len());
    
    // Test requests
    let test_requests = vec![
        ("POST", "https://api.example.com/generate", json!({"prompt": "Hello"})),
        ("POST", "https://api.example.com/generate", json!({"prompt": "What is 2+2?"})),
        ("POST", "https://api.example.com/generate", json!({"prompt": "Unknown prompt"})),
    ];
    
    for (method, url, body) in test_requests {
        println!("\nRequest: {} {} - {}", method, url, body);
        let response = provider.handle_request(method, url, &body).await?;
        println!("Response: {}", serde_json::to_string_pretty(&response)?);
    }
    
    Ok(())
}