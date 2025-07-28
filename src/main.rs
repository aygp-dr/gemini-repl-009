//! Gemini REPL - A secure, performant REPL for AI conversations
//! 
//! Phase 1: Basic noop REPL with signal handling

use anyhow::Result;
use clap::Parser;
use rustyline::DefaultEditor;
use rustyline::error::ReadlineError;
use tracing_subscriber::EnvFilter;

#[derive(Parser, Debug)]
#[command(name = "gemini-repl")]
#[command(version, about = "A secure, performant REPL for AI conversations", long_about = None)]
struct Args {
    /// API key for Gemini (can also use GEMINI_API_KEY env var)
    #[arg(short, long, env = "GEMINI_API_KEY", hide_env_values = true)]
    api_key: Option<String>,
    
    /// Model to use
    #[arg(short, long, default_value = "gemini-1.5-flash")]
    model: String,
    
    /// Enable debug logging
    #[arg(short, long)]
    debug: bool,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();
    
    // Setup logging
    let filter = if args.debug {
        EnvFilter::from_default_env()
            .add_directive("gemini_repl=debug".parse()?)
    } else {
        EnvFilter::from_default_env()
            .add_directive("gemini_repl=info".parse()?)
    };
    
    tracing_subscriber::fmt()
        .with_env_filter(filter)
        .init();
    
    tracing::info!("Starting Gemini REPL v{}", env!("CARGO_PKG_VERSION"));
    
    // Print welcome message
    println!("Gemini REPL v{} - Type /help for commands, /exit to quit", env!("CARGO_PKG_VERSION"));
    println!("Running in noop mode (Phase 1 implementation)");
    
    if args.api_key.is_none() {
        println!("Note: No API key provided. Set GEMINI_API_KEY or use --api-key");
    }
    
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
                    input if input.starts_with('/') => {
                        println!("Unknown command: {}. Type /help for available commands.", input);
                    }
                    input => {
                        // Echo input back (noop mode)
                        println!("You said: {}", input);
                        println!("(Running in noop mode - no API calls made)");
                    }
                }
            }
            Err(ReadlineError::Interrupted) => {
                println!("^C");
                continue;
            }
            Err(ReadlineError::Eof) => {
                println!("^D");
                break;
            }
            Err(err) => {
                println!("Error: {:?}", err);
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
    println!();
    println!("Signal handling:");
    println!("  Ctrl+C   - Cancel current input");
    println!("  Ctrl+D   - Exit the REPL");
}