//! File-based queue system for inter-agent communication
//!
//! Enables other AI agents (Claude Code, Cursor, etc.) to communicate with
//! the REPL via file-based JSON queues, similar to Efrit's queue system.
//!
//! Queue structure:
//! ```
//! ~/.gemini-repl/queues/
//! ├── input/       # Drop JSON request files here
//! ├── output/      # Responses are written here
//! └── archive/     # Processed requests are moved here
//! ```
//!
//! Request format:
//! ```json
//! {
//!   "id": "unique-request-id",
//!   "type": "prompt" | "command",
//!   "content": "your prompt or command",
//!   "context": { ... },  // optional additional context
//!   "created_at": "2025-01-01T00:00:00Z"
//! }
//! ```
//!
//! Response format:
//! ```json
//! {
//!   "id": "same-request-id",
//!   "status": "success" | "error",
//!   "content": "response text",
//!   "error": null | "error message",
//!   "processed_at": "2025-01-01T00:00:00Z"
//! }
//! ```

#![allow(dead_code)]

use crate::config::AppDirs;
use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

/// Request type
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum RequestType {
    /// A prompt to send to the LLM
    Prompt,
    /// A REPL command to execute
    Command,
    /// A ping to check if the REPL is running
    Ping,
}

/// A request from the queue
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueueRequest {
    /// Unique request ID
    pub id: String,
    /// Request type
    #[serde(rename = "type")]
    pub request_type: RequestType,
    /// The content (prompt or command)
    pub content: String,
    /// Optional additional context
    #[serde(default)]
    pub context: Option<serde_json::Value>,
    /// When the request was created
    pub created_at: DateTime<Utc>,
}

impl QueueRequest {
    /// Create a new prompt request
    pub fn prompt(content: &str) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            request_type: RequestType::Prompt,
            content: content.to_string(),
            context: None,
            created_at: Utc::now(),
        }
    }

    /// Create a new command request
    pub fn command(content: &str) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            request_type: RequestType::Command,
            content: content.to_string(),
            context: None,
            created_at: Utc::now(),
        }
    }

    /// Create a ping request
    pub fn ping() -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            request_type: RequestType::Ping,
            content: String::new(),
            context: None,
            created_at: Utc::now(),
        }
    }
}

/// Response status
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ResponseStatus {
    Success,
    Error,
}

/// A response to a queue request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueueResponse {
    /// Request ID this is responding to
    pub id: String,
    /// Response status
    pub status: ResponseStatus,
    /// Response content
    pub content: String,
    /// Error message if status is Error
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
    /// When the response was created
    pub processed_at: DateTime<Utc>,
}

impl QueueResponse {
    /// Create a success response
    pub fn success(id: &str, content: &str) -> Self {
        Self {
            id: id.to_string(),
            status: ResponseStatus::Success,
            content: content.to_string(),
            error: None,
            processed_at: Utc::now(),
        }
    }

    /// Create an error response
    pub fn error(id: &str, error: &str) -> Self {
        Self {
            id: id.to_string(),
            status: ResponseStatus::Error,
            content: String::new(),
            error: Some(error.to_string()),
            processed_at: Utc::now(),
        }
    }
}

/// Queue manager for file-based I/O
pub struct QueueManager {
    dirs: AppDirs,
}

impl QueueManager {
    /// Create a new queue manager
    pub fn new(dirs: AppDirs) -> Self {
        Self { dirs }
    }

    /// Check for pending requests in the input queue
    pub fn poll_requests(&self) -> Result<Vec<(PathBuf, QueueRequest)>> {
        let mut requests = Vec::new();

        for entry in fs::read_dir(self.dirs.queue_input_dir())? {
            let entry = entry?;
            let path = entry.path();

            if path.extension().map(|e| e == "json").unwrap_or(false) {
                match self.read_request(&path) {
                    Ok(req) => requests.push((path, req)),
                    Err(e) => {
                        tracing::warn!("Failed to read queue request {:?}: {}", path, e);
                    }
                }
            }
        }

        // Sort by creation time
        requests.sort_by(|a, b| a.1.created_at.cmp(&b.1.created_at));

        Ok(requests)
    }

    /// Read a request from a file
    fn read_request(&self, path: &PathBuf) -> Result<QueueRequest> {
        let content = fs::read_to_string(path)
            .with_context(|| format!("Failed to read request file: {:?}", path))?;

        let request: QueueRequest = serde_json::from_str(&content)
            .with_context(|| format!("Failed to parse request: {:?}", path))?;

        Ok(request)
    }

    /// Write a response to the output queue
    pub fn write_response(&self, response: &QueueResponse) -> Result<PathBuf> {
        let filename = format!("{}.json", response.id);
        let path = self.dirs.queue_output_dir().join(&filename);

        let content = serde_json::to_string_pretty(response)?;
        fs::write(&path, content)
            .with_context(|| format!("Failed to write response: {:?}", path))?;

        Ok(path)
    }

    /// Archive a processed request
    pub fn archive_request(&self, request_path: &PathBuf) -> Result<()> {
        if let Some(filename) = request_path.file_name() {
            let archive_path = self.dirs.queue_archive_dir().join(filename);
            fs::rename(request_path, &archive_path)
                .with_context(|| format!("Failed to archive request: {:?}", request_path))?;
        }
        Ok(())
    }

    /// Submit a request (for testing or external use)
    pub fn submit_request(&self, request: &QueueRequest) -> Result<PathBuf> {
        let filename = format!("{}.json", request.id);
        let path = self.dirs.queue_input_dir().join(&filename);

        let content = serde_json::to_string_pretty(request)?;
        fs::write(&path, content)
            .with_context(|| format!("Failed to write request: {:?}", path))?;

        Ok(path)
    }

    /// Wait for a response (for external use)
    pub fn wait_for_response(&self, request_id: &str, timeout_secs: u64) -> Result<Option<QueueResponse>> {
        let filename = format!("{}.json", request_id);
        let response_path = self.dirs.queue_output_dir().join(&filename);

        let start = std::time::Instant::now();
        let timeout = std::time::Duration::from_secs(timeout_secs);

        while start.elapsed() < timeout {
            if response_path.exists() {
                let content = fs::read_to_string(&response_path)?;
                let response: QueueResponse = serde_json::from_str(&content)?;
                // Optionally delete the response file after reading
                let _ = fs::remove_file(&response_path);
                return Ok(Some(response));
            }
            std::thread::sleep(std::time::Duration::from_millis(100));
        }

        Ok(None)
    }

    /// Clean up old archived requests (older than N days)
    pub fn cleanup_archive(&self, days: u32) -> Result<usize> {
        let cutoff = Utc::now() - chrono::Duration::days(days as i64);
        let mut removed = 0;

        for entry in fs::read_dir(self.dirs.queue_archive_dir())? {
            let entry = entry?;
            let path = entry.path();

            if let Ok(metadata) = entry.metadata() {
                if let Ok(modified) = metadata.modified() {
                    let modified: DateTime<Utc> = modified.into();
                    if modified < cutoff && fs::remove_file(&path).is_ok() {
                        removed += 1;
                    }
                }
            }
        }

        Ok(removed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    fn setup_test_queue() -> (tempfile::TempDir, QueueManager) {
        let dir = tempdir().unwrap();
        let app_dirs = AppDirs::with_root(dir.path().to_path_buf()).unwrap();
        let manager = QueueManager::new(app_dirs);
        (dir, manager)
    }

    #[test]
    fn test_request_creation() {
        let prompt = QueueRequest::prompt("Hello, world!");
        assert_eq!(prompt.content, "Hello, world!");
        assert!(matches!(prompt.request_type, RequestType::Prompt));

        let cmd = QueueRequest::command("/help");
        assert_eq!(cmd.content, "/help");
        assert!(matches!(cmd.request_type, RequestType::Command));
    }

    #[test]
    fn test_submit_and_poll() {
        let (_dir, manager) = setup_test_queue();

        let request = QueueRequest::prompt("Test prompt");
        let request_id = request.id.clone();

        manager.submit_request(&request).unwrap();

        let requests = manager.poll_requests().unwrap();
        assert_eq!(requests.len(), 1);
        assert_eq!(requests[0].1.id, request_id);
    }

    #[test]
    fn test_response_writing() {
        let (_dir, manager) = setup_test_queue();

        let response = QueueResponse::success("test-id", "Hello!");
        let path = manager.write_response(&response).unwrap();

        assert!(path.exists());

        let content = fs::read_to_string(&path).unwrap();
        let parsed: QueueResponse = serde_json::from_str(&content).unwrap();
        assert_eq!(parsed.content, "Hello!");
    }

    #[test]
    fn test_archive_request() {
        let (dir, manager) = setup_test_queue();

        let request = QueueRequest::prompt("Test");
        let request_path = manager.submit_request(&request).unwrap();

        assert!(request_path.exists());

        manager.archive_request(&request_path).unwrap();

        assert!(!request_path.exists());

        let archive_path = dir
            .path()
            .join("queues")
            .join("archive")
            .join(request_path.file_name().unwrap());
        assert!(archive_path.exists());
    }

    #[test]
    fn test_error_response() {
        let response = QueueResponse::error("test-id", "Something went wrong");
        assert!(matches!(response.status, ResponseStatus::Error));
        assert_eq!(response.error, Some("Something went wrong".to_string()));
    }
}
