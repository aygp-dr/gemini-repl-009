//! Gemini REPL - A secure, performant REPL for AI conversations with self-modification capabilities

use anyhow::Result;
use clap::Parser;
use rustyline::error::ReadlineError;
use rustyline::DefaultEditor;
use std::env;

mod api;
mod tools;
mod logging;
mod models;
mod utils;
mod self_modification;
mod errors;

use api::{Content, GeminiClient, Part};
use tools::ToolRegistry;
use logging::{init_logging, is_debug_mode};

#[derive(Parser, Debug)]
#[command(name = "gemini-repl")]
#[command(version, about = "A secure, performant REPL for AI conversations with self-modification capabilities", long_about = None)]
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

    /// Enable self-modification features
    #[arg(long)]
    enable_self_modification: bool,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    // Initialize logging with our custom module
    init_logging(args.debug || is_debug_mode())?;

    tracing::info!("Starting Gemini REPL v{}", env!("CARGO_PKG_VERSION"));

    // Initialize API client
    let client = initialize_client(&args)?;

    // Initialize tool registry
    let mut tool_registry = ToolRegistry::new();
    tool_registry.initialize_default_tools()?;

    if args.enable_self_modification {
        tracing::info!("Self-modification features enabled");
        tool_registry.initialize_self_modification_tools()?;
    }

    // Print welcome message
    print_welcome(&args, client.is_some());

    // Run the REPL
    run_repl(client, &args, tool_registry).await?;

    tracing::info!("Gemini REPL shutting down");
    Ok(())
}

async fn run_repl(client: Option<GeminiClient>, args: &Args, tool_registry: ToolRegistry) -> Result<()> {
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

                if let Some(should_break) = handle_command(trimmed, args, &conversation, &tool_registry) {
                    if should_break {
                        break;
                    }
                } else {
                    // Handle user input
                    handle_user_input(trimmed, client.as_ref(), &mut conversation, &tool_registry).await;
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

    Ok(())
}

fn initialize_client(args: &Args) -> Result<Option<GeminiClient>> {
    // Check for noop mode
    let noop_mode = env::var("NOOP_MODE")
        .map(|v| v.to_lowercase() == "true" || v == "1")
        .unwrap_or(false);

    // Initialize API client if not in noop mode and API key is available
    if !noop_mode && args.api_key.is_some() {
        Ok(Some(GeminiClient::new(
            args.api_key.clone().unwrap(),
            args.model.clone(),
        )?))
    } else {
        Ok(None)
    }
}

fn print_welcome(args: &Args, has_client: bool) {
    println!(
        "Gemini REPL v{} - Type /help for commands, /exit to quit",
        env!("CARGO_PKG_VERSION")
    );

    let noop_mode = env::var("NOOP_MODE")
        .map(|v| v.to_lowercase() == "true" || v == "1")
        .unwrap_or(false);

    if noop_mode {
        println!("Running in NOOP mode (no API calls will be made)");
    } else if !has_client {
        println!("Note: No API key provided. Set GEMINI_API_KEY or use --api-key");
        println!("Running in noop mode");
    } else {
        println!("Connected to Gemini API (model: {})", args.model);
        if args.enable_self_modification {
            println!("Self-modification features: ENABLED");
        }
    }
}

fn handle_command(trimmed: &str, args: &Args, conversation: &[Content], tool_registry: &ToolRegistry) -> Option<bool> {
    match trimmed {
        "/exit" | "/quit" => {
            println!("Goodbye!");
            Some(true)
        }
        "/help" => {
            print_help(args.enable_self_modification);
            Some(false)
        }
        "/model" => {
            println!("Current model: {}", args.model);
            Some(false)
        }
        "/clear" => {
            // Clear screen
            print!("\x1B[2J\x1B[1;1H");
            Some(false)
        }
        "/context" => {
            print_context(conversation);
            Some(false)
        }
        "/tools" => {
            print_tools(tool_registry);
            Some(false)
        }
        "/capabilities" => {
            if args.enable_self_modification {
                print_capabilities();
            } else {
                println!("Self-modification features are disabled. Use --enable-self-modification to enable.");
            }
            Some(false)
        }
        input if input.starts_with('/') => {
            println!("Unknown command: {input}. Type /help for available commands.");
            Some(false)
        }
        _ => None,
    }
}

fn print_context(conversation: &[Content]) {
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

fn print_tools(tool_registry: &ToolRegistry) {
    println!("Available tools:");
    for tool in tool_registry.list_tools() {
        println!("  - {}: {}", tool.name, tool.description);
    }
}

fn print_capabilities() {
    println!("Self-modification capabilities:");
    println!("  - Read and analyze own source code");
    println!("  - Propose code modifications");
    println!("  - Apply patches with validation");
    println!("  - Create new tools dynamically");
    println!("  - Extend functionality through plugins");
}

async fn handle_user_input(
    input: &str,
    client: Option<&GeminiClient>,
    conversation: &mut Vec<Content>,
    tool_registry: &ToolRegistry,
) {
    if let Some(client) = client {
        // Add user message to conversation
        conversation.push(Content {
            role: "user".to_string(),
            parts: vec![Part {
                text: Some(input.to_string()),
                function_call: None,
                function_response: None,
            }],
        });

        // Make API call with tools
        let tools = tool_registry.get_tool_definitions();
        match client
            .send_message_with_tools(conversation, Some(tools))
            .await
        {
            Ok(response) => {
                println!("{response}");

                // Add assistant response to conversation
                conversation.push(Content {
                    role: "model".to_string(),
                    parts: vec![Part {
                        text: Some(response),
                        function_call: None,
                        function_response: None,
                    }],
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

fn print_help(self_modification_enabled: bool) {
    println!("Available commands:");
    println!("  /help       - Show this help message");
    println!("  /exit       - Exit the REPL (/quit also works)");
    println!("  /model      - Show current model");
    println!("  /clear      - Clear the screen");
    println!("  /context    - Show conversation history");
    println!("  /tools      - List available tools");
    
    if self_modification_enabled {
        println!("  /capabilities - Show self-modification capabilities");
    }
    
    println!();
    println!("Signal handling:");
    println!("  Ctrl+C      - Cancel current input");
    println!("  Ctrl+D      - Exit the REPL");
}