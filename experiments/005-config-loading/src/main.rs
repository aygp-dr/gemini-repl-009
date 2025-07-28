//! Configuration loading experiment

use anyhow::Result;
use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use std::env;
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub api_key: Option<String>,
    pub model_name: String,
    pub history_file: PathBuf,
    pub log_file: PathBuf,
    pub timeout_ms: u64,
    pub max_history_size: usize,
    pub debug: bool,
    pub proxy: Option<String>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            api_key: None,
            model_name: "gemini-1.5-flash".to_string(),
            history_file: PathBuf::from(".gemini_history"),
            log_file: PathBuf::from(".gemini.log"),
            timeout_ms: 30000,
            max_history_size: 1000,
            debug: false,
            proxy: None,
        }
    }
}

impl Config {
    pub fn load() -> Result<Self> {
        let mut config = Self::default();
        
        // 1. Load from config file if it exists
        if let Some(config_path) = Self::config_file_path() {
            if config_path.exists() {
                let content = fs::read_to_string(&config_path)?;
                let file_config: Config = toml::from_str(&content)?;
                config = file_config;
                println!("✓ Loaded config from: {}", config_path.display());
            } else {
                println!("• No config file found at: {}", config_path.display());
            }
        }
        
        // 2. Override with environment variables
        if let Ok(api_key) = env::var("GEMINI_API_KEY") {
            config.api_key = Some(api_key);
            println!("✓ API key loaded from GEMINI_API_KEY");
        }
        
        if let Ok(model) = env::var("GEMINI_MODEL") {
            config.model_name = model;
            println!("✓ Model loaded from GEMINI_MODEL");
        }
        
        if let Ok(debug) = env::var("DEBUG") {
            config.debug = debug.to_lowercase() == "true";
            println!("✓ Debug mode: {}", config.debug);
        }
        
        if let Ok(proxy) = env::var("HTTPS_PROXY") {
            config.proxy = Some(proxy);
            println!("✓ Proxy loaded from HTTPS_PROXY");
        }
        
        Ok(config)
    }
    
    pub fn config_file_path() -> Option<PathBuf> {
        ProjectDirs::from("com", "aygp-dr", "gemini-repl")
            .map(|proj_dirs| proj_dirs.config_dir().join("config.toml"))
    }
    
    pub fn save(&self) -> Result<()> {
        if let Some(config_path) = Self::config_file_path() {
            if let Some(parent) = config_path.parent() {
                fs::create_dir_all(parent)?;
            }
            let content = toml::to_string_pretty(self)?;
            fs::write(&config_path, content)?;
            println!("✓ Config saved to: {}", config_path.display());
        }
        Ok(())
    }
}

fn main() -> Result<()> {
    println!("=== Configuration Loading Test ===");
    
    // Test loading configuration
    let config = Config::load()?;
    
    println!("\n--- Final Configuration ---");
    println!("API Key: {}", 
        config.api_key.as_ref()
            .map(|k| format!("{}...", &k[..10.min(k.len())]))
            .unwrap_or_else(|| "None".to_string())
    );
    println!("Model: {}", config.model_name);
    println!("History file: {}", config.history_file.display());
    println!("Debug: {}", config.debug);
    println!("Proxy: {}", config.proxy.as_deref().unwrap_or("None"));
    
    // Test saving configuration
    println!("\n--- Testing Config Save ---");
    config.save()?;
    
    println!("\n=== Configuration Test Complete ===");
    Ok(())
}