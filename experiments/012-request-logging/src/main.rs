//! Experiment: Request/Response logging to JSONL files

use anyhow::Result;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::Path;
use std::time::Instant;
uuid::Uuid;

#[derive(Debug, Serialize, Deserialize)]
struct RequestLog {
    id: String,
    timestamp: chrono::DateTime<Utc>,
    method: String,
    url: String,
    headers: std::collections::HashMap<String, String>,
    body: serde_json::Value,
}

#[derive(Debug, Serialize, Deserialize)]
struct ResponseLog {
    request_id: String,
    timestamp: chrono::DateTime<Utc>,
    status: u16,
    headers: std::collections::HashMap<String, String>,
    body: serde_json::Value,
    duration_ms: u64,
}

struct RequestLogger {
    base_dir: String,
}

impl RequestLogger {
    fn new(base_dir: &str) -> Result<Self> {
        fs::create_dir_all(base_dir)?;
        Ok(Self {
            base_dir: base_dir.to_string(),
        })
    }
    
    fn log_request(&self, url: &str, method: &str, body: &serde_json::Value) -> Result<String> {
        let request_id = Uuid::new_v4().to_string();
        let url_parts = url::Url::parse(url)?;
        let host = url_parts.host_str().unwrap_or("unknown");
        let path = url_parts.path().trim_start_matches('/');
        
        // Create directory: logs/{host}/{path}/
        let log_dir = Path::new(&self.base_dir)
            .join(host.replace(':', "_"))
            .join(path.replace('/', "_"));
        fs::create_dir_all(&log_dir)?;
        
        let log_entry = RequestLog {
            id: request_id.clone(),
            timestamp: Utc::now(),
            method: method.to_string(),
            url: url.to_string(),
            headers: std::collections::HashMap::new(),
            body: body.clone(),
        };
        
        // Append to reqs.jsonl
        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(log_dir.join("reqs.jsonl"))?;
        
        writeln!(file, "{}", serde_json::to_string(&log_entry)?)?;
        println!("Logged request: {} to {:?}", request_id, log_dir.join("reqs.jsonl"));
        
        Ok(request_id)
    }
    
    fn log_response(
        &self,
        url: &str,
        request_id: &str,
        status: u16,
        body: &serde_json::Value,
        duration_ms: u64,
    ) -> Result<()> {
        let url_parts = url::Url::parse(url)?;
        let host = url_parts.host_str().unwrap_or("unknown");
        let path = url_parts.path().trim_start_matches('/');
        
        let log_dir = Path::new(&self.base_dir)
            .join(host.replace(':', "_"))
            .join(path.replace('/', "_"));
        
        let log_entry = ResponseLog {
            request_id: request_id.to_string(),
            timestamp: Utc::now(),
            status,
            headers: std::collections::HashMap::new(),
            body: body.clone(),
            duration_ms,
        };
        
        // Append to resps.jsonl
        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(log_dir.join("resps.jsonl"))?;
        
        writeln!(file, "{}", serde_json::to_string(&log_entry)?)?;
        println!("Logged response: {} to {:?}", request_id, log_dir.join("resps.jsonl"));
        
        Ok(())
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let logger = RequestLogger::new("logs")?;
    
    // Simulate API calls to different endpoints
    let endpoints = vec![
        ("https://api.example.com/v1/users", "GET", json!({})),
        ("https://api.example.com/v1/posts", "POST", json!({"title": "Test", "body": "Content"})),
        ("https://generativelanguage.googleapis.com/v1beta/models/gemini-1.5-flash:generateContent", "POST", json!({
            "contents": [{
                "role": "user",
                "parts": [{"text": "Hello"}]
            }]
        })),
    ];
    
    for (url, method, body) in endpoints {
        println!("\nSimulating {} request to {}", method, url);
        
        let start = Instant::now();
        let request_id = logger.log_request(url, method, &body)?;
        
        // Simulate response delay
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
        
        let duration_ms = start.elapsed().as_millis() as u64;
        let response_body = json!({
            "result": "success",
            "data": {"id": 123}
        });
        
        logger.log_response(url, &request_id, 200, &response_body, duration_ms)?;
    }
    
    // Show the created structure
    println!("\nCreated log structure:");
    for entry in fs::read_dir("logs")? {
        let entry = entry?;
        println!("  {}/", entry.file_name().to_string_lossy());
        
        if entry.file_type()?.is_dir() {
            for sub_entry in fs::read_dir(entry.path())? {
                let sub_entry = sub_entry?;
                println!("    {}/", sub_entry.file_name().to_string_lossy());
                
                if sub_entry.file_type()?.is_dir() {
                    for file in fs::read_dir(sub_entry.path())? {
                        let file = file?;
                        println!("      {}", file.file_name().to_string_lossy());
                    }
                }
            }
        }
    }
    
    Ok(())
}