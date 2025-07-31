//! Error Handling Experiment

use anyhow::Result;
use thiserror::Error;
use tracing::{error, info, warn};

#[derive(Error, Debug)]
pub enum ReplError {
    #[error("Configuration error: {message}")]
    Config { message: String },
    
    #[error("API error: {status} - {message}")]
    Api { status: u16, message: String },
    
    #[error("IO error")]
    Io(#[from] std::io::Error),
    
    #[error("Parse error: {input}")]
    Parse { input: String },
}

fn test_error_scenarios() -> Result<()> {
    info!("Testing error handling scenarios");
    
    // Test different error types
    let errors = vec![
        ReplError::Config { message: "Invalid API key".to_string() },
        ReplError::Api { status: 401, message: "Unauthorized".to_string() },
        ReplError::Parse { input: "invalid command".to_string() },
    ];
    
    for (i, err) in errors.iter().enumerate() {
        error!("Test error {}: {}", i + 1, err);
        
        // Test error chain and context
        match err {
            ReplError::Config { message } => {
                warn!("Config issue detected: {}", message);
            }
            ReplError::Api { status, message } => {
                warn!("API failure: HTTP {} - {}", status, message);
            }
            ReplError::Parse { input } => {
                warn!("Parse failure for input: '{}'", input);
            }
            ReplError::Io(e) => {
                warn!("IO error: {}", e);
            }
        }
    }
    
    info!("✓ Error handling test complete");
    Ok(())
}

fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();
    
    info!("=== Error Handling Test ===");
    
    test_error_scenarios()?;
    
    // Test actual IO error
    match std::fs::read("nonexistent_file.txt") {
        Ok(_) => info!("File read unexpectedly succeeded"),
        Err(e) => {
            let repl_err = ReplError::Io(e);
            error!("Expected IO error: {}", repl_err);
            info!("✓ IO error handling works");
        }
    }
    
    info!("=== Error Handling Complete ===");
    Ok(())
}