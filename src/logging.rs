//! Logging infrastructure for debugging and request/response capture

use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;

/// Request/Response log entry for JSONL format
#[derive(Debug, Serialize, Deserialize)]
pub struct ApiLogEntry {
    pub timestamp: DateTime<Utc>,
    pub host: String,
    pub path: String,
    pub method: String,
    pub headers: std::collections::HashMap<String, String>,
    pub body: serde_json::Value,
    pub response_status: Option<u16>,
    pub response_body: Option<serde_json::Value>,
    pub duration_ms: Option<u64>,
}

/// Logger for API requests/responses
pub struct ApiLogger {
    base_dir: PathBuf,
    enabled: bool,
}

impl ApiLogger {
    /// Creates a new API logger.
    ///
    /// # Errors
    ///
    /// Returns an error if the log directory cannot be created.
    pub fn new(base_dir: impl AsRef<Path>, enabled: bool) -> Result<Self> {
        let base_dir = base_dir.as_ref().to_path_buf();
        if enabled {
            fs::create_dir_all(&base_dir)?;
        }
        Ok(Self { base_dir, enabled })
    }

    /// Log a request (before sending).
    ///
    /// # Errors
    ///
    /// Returns an error if the log file cannot be written.
    pub fn log_request(
        &self,
        host: &str,
        path: &str,
        method: &str,
        headers: &[(String, String)],
        body: &serde_json::Value,
    ) -> Result<String> {
        if !self.enabled {
            return Ok(String::new());
        }

        let request_id = uuid::Uuid::new_v4().to_string();
        let timestamp = Utc::now();

        // Create directory structure: logs/{host}/{path}/
        let log_dir = self
            .base_dir
            .join(host.replace(':', "_"))
            .join(path.trim_start_matches('/').replace('/', "_"));
        fs::create_dir_all(&log_dir)?;

        // Log to reqs.jsonl
        let entry = ApiLogEntry {
            timestamp,
            host: host.to_string(),
            path: path.to_string(),
            method: method.to_string(),
            headers: headers.iter().cloned().collect(),
            body: body.clone(),
            response_status: None,
            response_body: None,
            duration_ms: None,
        };

        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(log_dir.join("reqs.jsonl"))?;

        writeln!(file, "{}", serde_json::to_string(&entry)?)?;

        Ok(request_id)
    }

    /// Log a response (after receiving).
    ///
    /// # Errors
    ///
    /// Returns an error if the log file cannot be written.
    pub fn log_response(
        &self,
        host: &str,
        path: &str,
        status: u16,
        body: &serde_json::Value,
        duration_ms: u64,
    ) -> Result<()> {
        if !self.enabled {
            return Ok(());
        }

        let log_dir = self
            .base_dir
            .join(host.replace(':', "_"))
            .join(path.trim_start_matches('/').replace('/', "_"));

        // Log to resps.jsonl
        let entry = serde_json::json!({
            "timestamp": Utc::now(),
            "status": status,
            "body": body,
            "duration_ms": duration_ms,
        });

        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(log_dir.join("resps.jsonl"))?;

        writeln!(file, "{}", serde_json::to_string(&entry)?)?;

        Ok(())
    }
}

/// Initialize logging based on environment
/// Initializes the logging system.
///
/// # Errors
///
/// Returns an error if the logging system cannot be initialized.
pub fn init_logging(debug: bool) -> Result<()> {
    let filter = if debug {
        tracing_subscriber::EnvFilter::from_default_env()
            .add_directive("gemini_repl=debug".parse()?)
            .add_directive("reqwest=debug".parse()?)
    } else {
        tracing_subscriber::EnvFilter::from_default_env().add_directive("gemini_repl=info".parse()?)
    };

    // Console logging
    let fmt_layer = tracing_subscriber::fmt::layer()
        .with_target(true)
        .with_level(true)
        .with_thread_ids(false)
        .with_thread_names(false);

    // File logging if debug mode
    if debug {
        let file_appender = tracing_appender::rolling::daily("logs", "gemini-repl.log");
        let (non_blocking, _guard) = tracing_appender::non_blocking(file_appender);
        let file_layer = tracing_subscriber::fmt::layer()
            .with_writer(non_blocking)
            .with_ansi(false)
            .json();

        tracing_subscriber::registry()
            .with(filter)
            .with(fmt_layer)
            .with(file_layer)
            .init();
    } else {
        tracing_subscriber::registry()
            .with(filter)
            .with(fmt_layer)
            .init();
    }

    Ok(())
}

/// Check if debug mode is enabled
#[must_use]
pub fn is_debug_mode() -> bool {
    std::env::var("DEBUG")
        .map(|v| v.to_lowercase() == "true" || v == "1")
        .unwrap_or(false)
}
