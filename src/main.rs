//! Gemini REPL - A secure, performant REPL for AI conversations with self-modification capabilities

use anyhow::Result;
use clap::Parser;
use rustyline::error::ReadlineError;
use rustyline::DefaultEditor;
use std::env;

mod api;
mod errors;
mod logging;
mod models;
mod self_modification;
mod session;
mod tools;
mod utils;

use api::{ApiResponse, Content, GeminiClient, Part};
use logging::{init_logging, is_debug_mode};
use session::{Session, SessionManager, SessionStats};
use tools::ToolRegistry;

/// Maximum number of tool call iterations to prevent infinite loops
const MAX_TOOL_ITERATIONS: usize = 10;

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

    // Initialize session manager
    let session_manager = SessionManager::new()?;

    // Run the REPL
    run_repl(client, &args, tool_registry, session_manager).await?;

    tracing::info!("Gemini REPL shutting down");
    Ok(())
}

async fn run_repl(
    client: Option<GeminiClient>,
    args: &Args,
    tool_registry: ToolRegistry,
    session_manager: SessionManager,
) -> Result<()> {
    // Conversation history
    let mut conversation: Vec<Content> = Vec::new();
    // Current session name (None if unsaved)
    let mut current_session: Option<String> = None;

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

                if let Some(should_break) = handle_command(
                    trimmed,
                    args,
                    &mut conversation,
                    &tool_registry,
                    &session_manager,
                    &mut current_session,
                ) {
                    if should_break {
                        break;
                    }
                } else {
                    // Handle user input
                    handle_user_input(trimmed, client.as_ref(), &mut conversation, &tool_registry)
                        .await;
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

fn handle_command(
    trimmed: &str,
    args: &Args,
    conversation: &mut Vec<Content>,
    tool_registry: &ToolRegistry,
    session_manager: &SessionManager,
    current_session: &mut Option<String>,
) -> Option<bool> {
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
        "/reset" => {
            conversation.clear();
            *current_session = None;
            println!("Conversation reset");
            Some(false)
        }
        "/stats" => {
            let stats = SessionStats::from_conversation(conversation, current_session.clone());
            print_stats(&stats);
            Some(false)
        }
        "/sessions" => {
            print_sessions(session_manager);
            Some(false)
        }
        input if input.starts_with("/save") => {
            let name = input.strip_prefix("/save").unwrap().trim();
            let session_name = if name.is_empty() {
                current_session
                    .clone()
                    .unwrap_or_else(|| session_manager.generate_name())
            } else {
                name.to_string()
            };
            save_session(
                session_manager,
                &session_name,
                &args.model,
                conversation,
                current_session,
            );
            Some(false)
        }
        input if input.starts_with("/load") => {
            let name = input.strip_prefix("/load").unwrap().trim();
            if name.is_empty() {
                println!("Usage: /load <session_name>");
                println!("Use /sessions to list available sessions");
            } else {
                load_session(
                    session_manager,
                    name,
                    &args.model,
                    conversation,
                    current_session,
                );
            }
            Some(false)
        }
        input if input.starts_with("/delete") => {
            let name = input.strip_prefix("/delete").unwrap().trim();
            if name.is_empty() {
                println!("Usage: /delete <session_name>");
            } else {
                delete_session(session_manager, name, current_session);
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

fn print_stats(stats: &SessionStats) {
    println!("Session Statistics:");
    if let Some(name) = &stats.session_name {
        println!("  Session name: {}", name);
    } else {
        println!("  Session name: (unsaved)");
    }
    println!("  Total messages: {}", stats.message_count);
    println!("  User messages: {}", stats.user_messages);
    println!("  Assistant messages: {}", stats.assistant_messages);
    println!("  Total characters: {}", stats.total_chars);
}

fn print_sessions(session_manager: &SessionManager) {
    match session_manager.list() {
        Ok(sessions) => {
            if sessions.is_empty() {
                println!("No saved sessions found");
                println!(
                    "Sessions are stored in: {}",
                    session_manager.sessions_dir().display()
                );
            } else {
                println!("Saved sessions:");
                for session in sessions {
                    println!(
                        "  {} - {} messages ({})",
                        session.name,
                        session.message_count,
                        session.updated_at.format("%Y-%m-%d %H:%M")
                    );
                }
            }
        }
        Err(e) => {
            eprintln!("Error listing sessions: {}", e);
        }
    }
}

fn save_session(
    session_manager: &SessionManager,
    name: &str,
    model: &str,
    conversation: &[Content],
    current_session: &mut Option<String>,
) {
    if conversation.is_empty() {
        println!("No conversation to save");
        return;
    }

    let session = Session::from_conversation(name, model, conversation.to_vec());
    match session_manager.save(&session) {
        Ok(path) => {
            *current_session = Some(name.to_string());
            println!("Session saved: {}", name);
            tracing::debug!("Session saved to: {}", path.display());
        }
        Err(e) => {
            eprintln!("Error saving session: {}", e);
        }
    }
}

fn load_session(
    session_manager: &SessionManager,
    name: &str,
    expected_model: &str,
    conversation: &mut Vec<Content>,
    current_session: &mut Option<String>,
) {
    match session_manager.load(name) {
        Ok(session) => {
            if session.model != expected_model {
                println!(
                    "Warning: Session was created with model '{}', but current model is '{}'",
                    session.model, expected_model
                );
            }
            *conversation = session.conversation;
            *current_session = Some(session.name.clone());
            println!(
                "Loaded session '{}' ({} messages)",
                session.name, session.message_count
            );
        }
        Err(e) => {
            eprintln!("Error loading session: {}", e);
        }
    }
}

fn delete_session(
    session_manager: &SessionManager,
    name: &str,
    current_session: &mut Option<String>,
) {
    match session_manager.delete(name) {
        Ok(()) => {
            println!("Deleted session: {}", name);
            // Clear current session if it was the deleted one
            if current_session.as_deref() == Some(name) {
                *current_session = None;
            }
        }
        Err(e) => {
            eprintln!("Error deleting session: {}", e);
        }
    }
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

        // Make API call with tools - may require multiple iterations for tool calls
        let tools = tool_registry.get_tool_definitions();

        for iteration in 0..MAX_TOOL_ITERATIONS {
            match client
                .send_message_with_tools(conversation, Some(tools.clone()))
                .await
            {
                Ok(response) => {
                    match &response {
                        ApiResponse::Text(text) => {
                            // Simple text response - print and add to conversation
                            println!("{text}");
                            conversation.push(GeminiClient::response_to_content(&response));
                            break;
                        }
                        ApiResponse::FunctionCall(fc) => {
                            // Execute the function and continue the conversation
                            tracing::info!("Executing tool: {}", fc.name);
                            println!("[Calling tool: {}]", fc.name);

                            // Add the function call to conversation
                            conversation.push(GeminiClient::response_to_content(&response));

                            // Execute the tool
                            match tool_registry.execute_tool(&fc.name, fc.args.clone()).await {
                                Ok(result) => {
                                    tracing::debug!("Tool result: {}", result);
                                    // Add function response to conversation
                                    let func_response =
                                        GeminiClient::create_function_response_content(
                                            &fc.name, result,
                                        );
                                    conversation.push(func_response);
                                    // Continue loop to get model's response to the tool result
                                }
                                Err(e) => {
                                    eprintln!("Tool execution error: {e}");
                                    // Add error as function response
                                    let error_response =
                                        GeminiClient::create_function_response_content(
                                            &fc.name,
                                            serde_json::json!({
                                                "error": e.to_string()
                                            }),
                                        );
                                    conversation.push(error_response);
                                }
                            }
                        }
                        ApiResponse::TextWithFunctionCall {
                            text,
                            function_call,
                        } => {
                            // Print the text, then handle the function call
                            println!("{text}");
                            tracing::info!("Executing tool: {}", function_call.name);
                            println!("[Calling tool: {}]", function_call.name);

                            // Add the response to conversation
                            conversation.push(GeminiClient::response_to_content(&response));

                            // Execute the tool
                            match tool_registry
                                .execute_tool(&function_call.name, function_call.args.clone())
                                .await
                            {
                                Ok(result) => {
                                    tracing::debug!("Tool result: {}", result);
                                    let func_response =
                                        GeminiClient::create_function_response_content(
                                            &function_call.name,
                                            result,
                                        );
                                    conversation.push(func_response);
                                }
                                Err(e) => {
                                    eprintln!("Tool execution error: {e}");
                                    let error_response =
                                        GeminiClient::create_function_response_content(
                                            &function_call.name,
                                            serde_json::json!({
                                                "error": e.to_string()
                                            }),
                                        );
                                    conversation.push(error_response);
                                }
                            }
                        }
                    }
                }
                Err(e) => {
                    eprintln!("Error: {e}");
                    break;
                }
            }

            if iteration == MAX_TOOL_ITERATIONS - 1 {
                eprintln!("Warning: Maximum tool iterations reached");
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
    println!("  /help          - Show this help message");
    println!("  /exit          - Exit the REPL (/quit also works)");
    println!("  /model         - Show current model");
    println!("  /clear         - Clear the screen");
    println!("  /context       - Show conversation history");
    println!("  /tools         - List available tools");

    println!();
    println!("Session management:");
    println!("  /save [name]   - Save session (auto-generates name if not provided)");
    println!("  /load <name>   - Load a saved session");
    println!("  /sessions      - List all saved sessions");
    println!("  /delete <name> - Delete a saved session");
    println!("  /reset         - Clear conversation history");
    println!("  /stats         - Show session statistics");

    if self_modification_enabled {
        println!();
        println!("Self-modification:");
        println!("  /capabilities  - Show self-modification capabilities");
    }

    println!();
    println!("Signal handling:");
    println!("  Ctrl+C         - Cancel current input");
    println!("  Ctrl+D         - Exit the REPL");
}
