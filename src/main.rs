//! Gemini REPL - A secure, performant REPL for AI conversations

use anyhow::Result;
use clap::Parser;
use rustyline::DefaultEditor;
use rustyline::error::ReadlineError;
use std::env;

mod api;
mod logging;
mod functions;

use api::{GeminiClient, Content, Part};
use logging::{init_logging, is_debug_mode};
use functions::get_available_tools;

#[derive(Parser, Debug)]
#[command(name = "gemini-repl")]
#[command(version, about = "A secure, performant REPL for AI conversations", long_about = None)]
struct Args {
    /// API key for Gemini (can also use `GEMINI_API_KEY` env var)
    #[arg(short, long, env = "GEMINI_API_KEY", hide_env_values = true)]
    api_key: Option<String>,
    
    /// Model to use
    #[arg(short, long, default_value = "gemini-2.0-flash-exp")]
    model: String,
    
    /// Enable debug logging
    #[arg(short, long)]
    debug: bool,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();
    
    // Initialize logging with our custom module
    init_logging(args.debug || is_debug_mode())?;
    
    tracing::info!("Starting Gemini REPL v{}", env!("CARGO_PKG_VERSION"));
    
    // Check for noop mode
    let noop_mode = env::var("NOOP_MODE")
        .map(|v| v.to_lowercase() == "true" || v == "1")
        .unwrap_or(false);
    
    // Initialize API client if not in noop mode and API key is available
    let client = if !noop_mode && args.api_key.is_some() {
        Some(GeminiClient::new(args.api_key.clone().unwrap(), args.model.clone())?)
    } else {
        None
    };
    
    // Print welcome message
    println!("Gemini REPL v{} - Type /help for commands, /exit to quit", env!("CARGO_PKG_VERSION"));
    
    if noop_mode {
        println!("Running in NOOP mode (no API calls will be made)");
    } else if client.is_none() {
        println!("Note: No API key provided. Set GEMINI_API_KEY or use --api-key");
        println!("Running in noop mode");
    } else {
        println!("Connected to Gemini API (model: {})", args.model);
    }
    
    // Conversation history
    let mut conversation: Vec<Content> = Vec::new();
    
    // Initialize readline
    let mut rl = DefaultEditor::new()?;
    
    // Main REPL loop
    loop {
        match rl.readline("gemini> ") {
            Ok(line) => {
                // Add to history
                let _ = rl.add_history_entry(line.as_str());
                
                // Handle commands
                let trimmed = line.trim();
                
                if trimmed.is_empty() {
                    continue;
                }
                
                match trimmed {
                    "/exit" | "/quit" => {
                        println!("Goodbye!");
                        break;
                    }
                    "/help" => {
                        print_help();
                    }
                    "/model" => {
                        println!("Current model: {}", args.model);
                    }
                    "/clear" => {
                        // Clear screen
                        print!("\x1B[2J\x1B[1;1H");
                    }
                    "/context" => {
                        // Show conversation context
                        if conversation.is_empty() {
                            println!("No conversation history yet");
                        } else {
                            println!("Conversation history ({} messages):", conversation.len());
                            for (i, content) in conversation.iter().enumerate() {
                                let role = if i % 2 == 0 { "User" } else { "Assistant" };
                                if let Some(text) = &content.parts[0].text {
                                    println!("{role}: {text}");
                                }
                            }
                        }
                    }
                    input if input.starts_with('/') => {
                        println!("Unknown command: {input}. Type /help for available commands.");
                    }
                    input => {
                        // Handle user input
                        if let Some(client) = &client {
                            // Add user message to conversation
                            conversation.push(Content {
                                role: "user".to_string(),
                                parts: vec![Part { text: Some(input.to_string()), function_call: None, function_response: None }],
                            });
                            
                            // Make API call with tools
                            let tools = get_available_tools();
                            match client.send_message_with_tools(&conversation, Some(tools)).await {
                                Ok(response) => {
                                    println!("{response}");
                                    
                                    // Add assistant response to conversation
                                    conversation.push(Content {
                                        role: "model".to_string(),
                                        parts: vec![Part { text: Some(response), function_call: None, function_response: None }],
                                    });
                                }
                                Err(e) => {
                                    eprintln!("Error: {e}");
                                }
                            }
                        } else {
                            // Noop mode - echo input back
                            println!("You said: {input}");
                            println!("(Running in noop mode - no API calls made)");
                        }
                    }
                }
            }
            Err(ReadlineError::Interrupted) => {
                println!("^C");
            }
            Err(ReadlineError::Eof) => {
                println!("^D");
                break;
            }
            Err(err) => {
                println!("Error: {err:?}");
                break;
            }
        }
    }
    
    tracing::info!("Gemini REPL shutting down");
    Ok(())
}

fn print_help() {
    println!("Available commands:");
    println!("  /help    - Show this help message");
    println!("  /exit    - Exit the REPL (/quit also works)");
    println!("  /model   - Show current model");
    println!("  /clear   - Clear the screen");
    println!("  /context - Show conversation history");
    println!();
    println!("Signal handling:");
    println!("  Ctrl+C   - Cancel current input");
    println!("  Ctrl+D   - Exit the REPL");
}