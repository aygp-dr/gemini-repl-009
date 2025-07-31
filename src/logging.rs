//! Logging configuration and utilities

use anyhow::Result;
use std::env;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

/// Initialize logging with the specified debug level
pub fn init_logging(debug: bool) -> Result<()> {
    let env_filter = if debug {
        EnvFilter::new("debug")
    } else {
        EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"))
    };
    
    tracing_subscriber::registry()
        .with(env_filter)
        .with(tracing_subscriber::fmt::layer().with_target(false))
        .init();
    
    Ok(())
}

/// Check if debug mode is enabled via environment
pub fn is_debug_mode() -> bool {
    env::var("RUST_LOG")
        .map(|v| v.to_lowercase().contains("debug") || v.to_lowercase().contains("trace"))
        .unwrap_or(false)
        || env::var("DEBUG")
            .map(|v| v.to_lowercase() == "true" || v == "1")
            .unwrap_or(false)
}