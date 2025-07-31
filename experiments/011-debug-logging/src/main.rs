//! Experiment: Debug logging with environment-based configuration

use anyhow::Result;
use tracing::{debug, error, info, warn};
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;

fn init_logging() -> Result<()> {
    // Check DEBUG environment variable
    let debug_mode = std::env::var("DEBUG")
        .map(|v| v == "true" || v == "1")
        .unwrap_or(false);
    
    println!("Debug mode: {}", debug_mode);
    
    let filter = if debug_mode {
        tracing_subscriber::EnvFilter::new("debug")
    } else {
        tracing_subscriber::EnvFilter::new("info")
    };
    
    // Console output
    let fmt_layer = tracing_subscriber::fmt::layer()
        .with_target(true)
        .with_thread_ids(true)
        .with_line_number(true);
    
    if debug_mode {
        // Also log to file in debug mode
        std::fs::create_dir_all("logs")?;
        let file_appender = tracing_appender::rolling::daily("logs", "debug.log");
        let (non_blocking, _guard) = tracing_appender::non_blocking(file_appender);
        
        let file_layer = tracing_subscriber::fmt::layer()
            .with_writer(non_blocking)
            .json()
            .with_current_span(true);
        
        tracing_subscriber::registry()
            .with(filter)
            .with(fmt_layer)
            .with(file_layer)
            .init();
        
        info!("Logging to console and logs/debug.log");
    } else {
        tracing_subscriber::registry()
            .with(filter)
            .with(fmt_layer)
            .init();
        
        info!("Logging to console only");
    }
    
    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    // Load .env if exists
    if let Ok(contents) = std::fs::read_to_string(".env") {
        for line in contents.lines() {
            if let Some((key, value)) = line.split_once('=') {
                std::env::set_var(key.trim(), value.trim());
            }
        }
    }
    
    init_logging()?;
    
    // Test different log levels
    error!("This is an error message");
    warn!("This is a warning");
    info!("This is info");
    debug!("This is debug - only visible in debug mode");
    
    // Structured logging
    info!(
        request_id = "123-456",
        user = "test-user",
        "Processing request"
    );
    
    // Simulate API call logging
    let start = std::time::Instant::now();
    info!("Sending API request");
    debug!(url = "https://api.example.com", method = "POST", "Request details");
    
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
    
    let duration = start.elapsed();
    info!(
        duration_ms = duration.as_millis() as u64,
        status = 200,
        "API request completed"
    );
    
    Ok(())
}