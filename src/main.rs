//! Gemini REPL - A secure, performant REPL for AI conversations with self-modification capabilities

use anyhow::Result;
use clap::Parser;
use rustyline::error::ReadlineError;
use rustyline::DefaultEditor;
use std::env;

mod api;
mod context;
mod errors;
mod logging;
mod models;
mod providers;
mod self_modification;
mod session;
mod tools;
mod utils;

use api::{Content, Part};
use logging::{init_logging, is_debug_mode};
use providers::{
    create_provider, default_model_for_provider, detect_provider, Provider, ProviderConfig,
    ProviderType,
};
use session::{ExportFormat, Session, SessionManager, SessionStats};
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

    /// Model to use (provider-specific, e.g., "llama3.2", "gemini-2.0-flash-exp")
    #[arg(short, long)]
    model: Option<String>,

    /// Provider to use: ollama, gemini, openai (auto-detects if not specified)
    #[arg(short, long, env = "LLM_PROVIDER")]
    provider: Option<String>,

    /// Ollama server URL (default: http://localhost:11434)
    #[arg(long, env = "OLLAMA_HOST")]
    ollama_url: Option<String>,

    /// Enable debug logging
    #[arg(short, long)]
    debug: bool,

    /// Enable self-modification features
    #[arg(long)]
    enable_self_modification: bool,

    /// Continue a specific session by name
    #[arg(short = 'C', long = "continue")]
    continue_session: Option<String>,

    /// Continue the most recent session
    #[arg(long)]
    continue_last: bool,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    // Initialize logging with our custom module
    init_logging(args.debug || is_debug_mode())?;

    tracing::info!("Starting Gemini REPL v{}", env!("CARGO_PKG_VERSION"));

    // Initialize provider (with auto-detection)
    let provider = initialize_provider(&args).await;

    // Initialize tool registry
    let mut tool_registry = ToolRegistry::new();
    tool_registry.initialize_default_tools()?;

    if args.enable_self_modification {
        tracing::info!("Self-modification features enabled");
        tool_registry.initialize_self_modification_tools()?;
    }

    // Print welcome message
    print_welcome_v2(&args, provider.as_deref());

    // Initialize session manager
    let session_manager = SessionManager::new()?;

    // Run the REPL
    run_repl_v2(provider, &args, tool_registry, session_manager).await?;

    tracing::info!("Gemini REPL shutting down");
    Ok(())
}

/// Initialize the LLM provider with auto-detection
async fn initialize_provider(args: &Args) -> Option<Box<dyn Provider>> {
    // Check for noop mode
    let noop_mode = env::var("NOOP_MODE")
        .map(|v| v.to_lowercase() == "true" || v == "1")
        .unwrap_or(false);

    if noop_mode {
        return None;
    }

    // If provider explicitly specified, use that
    if let Some(provider_str) = &args.provider {
        let provider_type: ProviderType = match provider_str.parse() {
            Ok(pt) => pt,
            Err(e) => {
                eprintln!("Invalid provider: {}", e);
                return None;
            }
        };

        let model = args
            .model
            .clone()
            .unwrap_or_else(|| default_model_for_provider(provider_type).to_string());

        let config = ProviderConfig {
            provider_type,
            api_key: args.api_key.clone(),
            base_url: args.ollama_url.clone(),
            model,
            timeout_secs: if provider_type == ProviderType::Ollama {
                120
            } else {
                30
            },
        };

        match create_provider(config) {
            Ok(p) => return Some(p),
            Err(e) => {
                eprintln!("Failed to create provider: {}", e);
                return None;
            }
        }
    }

    // Auto-detect provider (Ollama first, then Gemini)
    if let Some(config) = detect_provider(args.api_key.clone(), args.ollama_url.clone()).await {
        let model = args.model.clone().unwrap_or(config.model.clone());
        let config = ProviderConfig { model, ..config };

        match create_provider(config) {
            Ok(p) => return Some(p),
            Err(e) => {
                tracing::warn!("Failed to create auto-detected provider: {}", e);
            }
        }
    }

    None
}

fn print_welcome_v2(args: &Args, provider: Option<&dyn Provider>) {
    println!(
        "Gemini REPL v{} - Type /help for commands, /exit to quit",
        env!("CARGO_PKG_VERSION")
    );

    let noop_mode = env::var("NOOP_MODE")
        .map(|v| v.to_lowercase() == "true" || v == "1")
        .unwrap_or(false);

    if noop_mode {
        println!("Running in NOOP mode (no API calls will be made)");
    } else if let Some(p) = provider {
        println!(
            "Connected to {} (model: {})",
            p.name().to_uppercase(),
            p.model()
        );
        if args.enable_self_modification {
            println!("Self-modification features: ENABLED");
        }
    } else {
        println!("Note: No provider available.");
        println!("  - For Ollama: Start ollama serve");
        println!("  - For Gemini: Set GEMINI_API_KEY or use --api-key");
        println!("Running in noop mode");
    }
}

async fn run_repl_v2(
    provider: Option<Box<dyn Provider>>,
    args: &Args,
    tool_registry: ToolRegistry,
    session_manager: SessionManager,
) -> Result<()> {
    // Conversation history
    let mut conversation: Vec<Content> = Vec::new();
    // Current session name (None if unsaved)
    let mut current_session: Option<String> = None;

    // Get model name for session
    let model_name = provider
        .as_ref()
        .map(|p| p.model().to_string())
        .unwrap_or_else(|| "unknown".to_string());

    // Handle session continuation
    if let Some(session_name) = &args.continue_session {
        match session_manager.load(session_name) {
            Ok(session) => {
                if session.model != model_name {
                    println!(
                        "Warning: Session was created with model '{}', current model is '{}'",
                        session.model, model_name
                    );
                }
                println!(
                    "Continuing session '{}' ({} messages)",
                    session.name, session.message_count
                );
                conversation = session.conversation;
                current_session = Some(session.name);
            }
            Err(e) => {
                eprintln!("Error loading session '{}': {}", session_name, e);
                eprintln!("Starting new session instead.");
            }
        }
    } else if args.continue_last {
        match session_manager.get_last_session() {
            Ok(Some(session)) => {
                if session.model != model_name {
                    println!(
                        "Warning: Session was created with model '{}', current model is '{}'",
                        session.model, model_name
                    );
                }
                println!(
                    "Continuing last session '{}' ({} messages)",
                    session.name, session.message_count
                );
                conversation = session.conversation;
                current_session = Some(session.name);
            }
            Ok(None) => {
                println!("No previous sessions found. Starting new session.");
            }
            Err(e) => {
                eprintln!("Error loading last session: {}", e);
                eprintln!("Starting new session instead.");
            }
        }
    }

    // Initialize readline
    let mut rl = DefaultEditor::new()?;

    // Main REPL loop
    loop {
        match rl.readline("repl> ") {
            Ok(line) => {
                // Add to history
                let _ = rl.add_history_entry(line.as_str());

                // Handle commands
                let trimmed = line.trim();

                if trimmed.is_empty() {
                    continue;
                }

                if let Some(should_break) = handle_command_v2(
                    trimmed,
                    &model_name,
                    &mut conversation,
                    &tool_registry,
                    &session_manager,
                    &mut current_session,
                    args.enable_self_modification,
                ) {
                    if should_break {
                        break;
                    }
                } else {
                    // Handle user input with provider
                    handle_user_input_v2(
                        trimmed,
                        provider.as_deref(),
                        &mut conversation,
                        &tool_registry,
                    )
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

fn handle_command_v2(
    trimmed: &str,
    model: &str,
    conversation: &mut Vec<Content>,
    tool_registry: &ToolRegistry,
    session_manager: &SessionManager,
    current_session: &mut Option<String>,
    self_modification_enabled: bool,
) -> Option<bool> {
    match trimmed {
        "/exit" | "/quit" => {
            println!("Goodbye!");
            Some(true)
        }
        "/help" => {
            print_help(self_modification_enabled);
            Some(false)
        }
        "/model" => {
            println!("Current model: {}", model);
            Some(false)
        }
        "/clear" => {
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
            if self_modification_enabled {
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
        "/tokens" => {
            print_token_stats(conversation);
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
                model,
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
                load_session(session_manager, name, model, conversation, current_session);
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
        "/continue" => {
            continue_last_session(session_manager, model, conversation, current_session);
            Some(false)
        }
        input if input.starts_with("/export") => {
            let args_str = input.strip_prefix("/export").unwrap().trim();
            export_session(
                session_manager,
                args_str,
                model,
                conversation,
                current_session,
            );
            Some(false)
        }
        input if input.starts_with('/') => {
            println!("Unknown command: {input}. Type /help for available commands.");
            Some(false)
        }
        _ => None,
    }
}

async fn handle_user_input_v2(
    input: &str,
    provider: Option<&dyn Provider>,
    conversation: &mut Vec<Content>,
    tool_registry: &ToolRegistry,
) {
    if let Some(provider) = provider {
        // Add user message to conversation
        conversation.push(Content {
            role: "user".to_string(),
            parts: vec![Part {
                text: Some(input.to_string()),
                function_call: None,
                function_response: None,
            }],
        });

        // Convert to provider messages
        let messages: Vec<providers::Message> = conversation
            .iter()
            .map(|c| {
                let role = match c.role.as_str() {
                    "user" => providers::Role::User,
                    "model" => providers::Role::Assistant,
                    "function" => providers::Role::Function,
                    "system" => providers::Role::System,
                    _ => providers::Role::User,
                };
                let content = if let Some(text) = c.parts.first().and_then(|p| p.text.as_ref()) {
                    providers::MessageContent::Text(text.clone())
                } else if let Some(fc) = c.parts.first().and_then(|p| p.function_call.as_ref()) {
                    providers::MessageContent::FunctionCall(providers::FunctionCall {
                        name: fc.name.clone(),
                        arguments: fc.args.clone(),
                    })
                } else if let Some(fr) = c.parts.first().and_then(|p| p.function_response.as_ref())
                {
                    providers::MessageContent::FunctionResponse(providers::FunctionResponse {
                        name: fr.name.clone(),
                        response: fr.response.clone(),
                    })
                } else {
                    providers::MessageContent::Text(String::new())
                };
                providers::Message { role, content }
            })
            .collect();

        // Get tool definitions
        let api_tools = tool_registry.get_tool_definitions();
        let provider_tools: Vec<providers::ToolDefinition> = api_tools
            .iter()
            .filter_map(|t| {
                let name = t.get("name")?.as_str()?.to_string();
                let description = t.get("description")?.as_str()?.to_string();
                let parameters = t.get("parameters")?.clone();
                Some(providers::ToolDefinition {
                    name,
                    description,
                    parameters,
                })
            })
            .collect();

        // Generate response with tools
        for iteration in 0..MAX_TOOL_ITERATIONS {
            match provider.generate(&messages, Some(&provider_tools)).await {
                Ok(response) => {
                    match response {
                        providers::ProviderResponse::Text(text) => {
                            println!("{}", text);
                            conversation.push(Content {
                                role: "model".to_string(),
                                parts: vec![Part {
                                    text: Some(text),
                                    function_call: None,
                                    function_response: None,
                                }],
                            });
                            break;
                        }
                        providers::ProviderResponse::FunctionCall(fc) => {
                            tracing::info!("Executing tool: {}", fc.name);
                            println!("[Calling tool: {}]", fc.name);

                            // Add function call to conversation
                            conversation.push(Content {
                                role: "model".to_string(),
                                parts: vec![Part {
                                    text: None,
                                    function_call: Some(api::FunctionCall {
                                        name: fc.name.clone(),
                                        args: fc.arguments.clone(),
                                    }),
                                    function_response: None,
                                }],
                            });

                            // Execute the tool
                            match tool_registry
                                .execute_tool(&fc.name, fc.arguments.clone())
                                .await
                            {
                                Ok(result) => {
                                    tracing::debug!("Tool result: {}", result);
                                    conversation.push(Content {
                                        role: "function".to_string(),
                                        parts: vec![Part {
                                            text: None,
                                            function_call: None,
                                            function_response: Some(api::FunctionResponse {
                                                name: fc.name.clone(),
                                                response: result,
                                            }),
                                        }],
                                    });
                                }
                                Err(e) => {
                                    eprintln!("Tool execution error: {e}");
                                    conversation.push(Content {
                                        role: "function".to_string(),
                                        parts: vec![Part {
                                            text: None,
                                            function_call: None,
                                            function_response: Some(api::FunctionResponse {
                                                name: fc.name,
                                                response: serde_json::json!({"error": e.to_string()}),
                                            }),
                                        }],
                                    });
                                }
                            }
                        }
                        providers::ProviderResponse::TextWithFunctionCall {
                            text,
                            function_call,
                        } => {
                            println!("{}", text);
                            tracing::info!("Executing tool: {}", function_call.name);
                            println!("[Calling tool: {}]", function_call.name);

                            // Add to conversation
                            conversation.push(Content {
                                role: "model".to_string(),
                                parts: vec![
                                    Part {
                                        text: Some(text),
                                        function_call: None,
                                        function_response: None,
                                    },
                                    Part {
                                        text: None,
                                        function_call: Some(api::FunctionCall {
                                            name: function_call.name.clone(),
                                            args: function_call.arguments.clone(),
                                        }),
                                        function_response: None,
                                    },
                                ],
                            });

                            // Execute the tool
                            match tool_registry
                                .execute_tool(&function_call.name, function_call.arguments.clone())
                                .await
                            {
                                Ok(result) => {
                                    tracing::debug!("Tool result: {}", result);
                                    conversation.push(Content {
                                        role: "function".to_string(),
                                        parts: vec![Part {
                                            text: None,
                                            function_call: None,
                                            function_response: Some(api::FunctionResponse {
                                                name: function_call.name,
                                                response: result,
                                            }),
                                        }],
                                    });
                                }
                                Err(e) => {
                                    eprintln!("Tool execution error: {e}");
                                    conversation.push(Content {
                                        role: "function".to_string(),
                                        parts: vec![Part {
                                            text: None,
                                            function_call: None,
                                            function_response: Some(api::FunctionResponse {
                                                name: function_call.name,
                                                response: serde_json::json!({"error": e.to_string()}),
                                            }),
                                        }],
                                    });
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
        // Noop mode
        println!("You said: {input}");
        println!("(Running in noop mode - no API calls made)");
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

fn print_token_stats(conversation: &[Content]) {
    use context::ContextManager;

    let cm = ContextManager::default();
    let stats = cm.count_tokens(conversation);

    println!("Token Statistics:");
    println!("  {}", cm.status(conversation));
    println!();
    println!("  By role:");
    println!(
        "    User:      {} tokens ({} messages)",
        stats.user_tokens, stats.user_messages
    );
    println!(
        "    Assistant: {} tokens ({} messages)",
        stats.assistant_tokens, stats.assistant_messages
    );
    println!(
        "    Function:  {} tokens ({} messages)",
        stats.function_tokens, stats.function_messages
    );
    if stats.system_tokens > 0 {
        println!("    System:    {} tokens", stats.system_tokens);
    }
    println!();
    println!(
        "  Avg tokens/message: {:.1}",
        stats.avg_tokens_per_message()
    );
    println!(
        "  Remaining capacity: {} tokens",
        cm.remaining_tokens(conversation)
    );

    if cm.needs_warning(conversation) {
        println!();
        println!("  ⚠️  Warning: Approaching context limit!");
    }
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

fn continue_last_session(
    session_manager: &SessionManager,
    expected_model: &str,
    conversation: &mut Vec<Content>,
    current_session: &mut Option<String>,
) {
    match session_manager.get_last_session() {
        Ok(Some(session)) => {
            if session.model != expected_model {
                println!(
                    "Warning: Session was created with model '{}', current model is '{}'",
                    session.model, expected_model
                );
            }
            *conversation = session.conversation;
            *current_session = Some(session.name.clone());
            println!(
                "Loaded last session '{}' ({} messages)",
                session.name, session.message_count
            );
        }
        Ok(None) => {
            println!("No saved sessions found");
        }
        Err(e) => {
            eprintln!("Error loading last session: {}", e);
        }
    }
}

fn export_session(
    session_manager: &SessionManager,
    args_str: &str,
    model: &str,
    conversation: &[Content],
    current_session: &Option<String>,
) {
    // Parse: /export [session_name] [format] [output_file]
    // Default: export current session to json, print to stdout
    let parts: Vec<&str> = args_str.split_whitespace().collect();

    // Determine session to export
    let (session, format, output_file) = match parts.as_slice() {
        [] => {
            // Export current conversation
            if conversation.is_empty() {
                println!("No conversation to export");
                return;
            }
            let name = current_session
                .clone()
                .unwrap_or_else(|| "unsaved".to_string());
            let session = Session::from_conversation(&name, model, conversation.to_vec());
            (session, ExportFormat::Json, None)
        }
        [format] => {
            // Export current conversation in specified format
            if conversation.is_empty() {
                println!("No conversation to export");
                return;
            }
            let fmt = match format.parse::<ExportFormat>() {
                Ok(f) => f,
                Err(e) => {
                    eprintln!("{}", e);
                    return;
                }
            };
            let name = current_session
                .clone()
                .unwrap_or_else(|| "unsaved".to_string());
            let session = Session::from_conversation(&name, model, conversation.to_vec());
            (session, fmt, None)
        }
        [format, output] => {
            // Export current conversation to file
            if conversation.is_empty() {
                println!("No conversation to export");
                return;
            }
            let fmt = match format.parse::<ExportFormat>() {
                Ok(f) => f,
                Err(e) => {
                    eprintln!("{}", e);
                    return;
                }
            };
            let name = current_session
                .clone()
                .unwrap_or_else(|| "unsaved".to_string());
            let session = Session::from_conversation(&name, model, conversation.to_vec());
            (session, fmt, Some(*output))
        }
        [session_name, format, output] => {
            // Export specific session to file
            let session = match session_manager.load(session_name) {
                Ok(s) => s,
                Err(e) => {
                    eprintln!("Error loading session '{}': {}", session_name, e);
                    return;
                }
            };
            let fmt = match format.parse::<ExportFormat>() {
                Ok(f) => f,
                Err(e) => {
                    eprintln!("{}", e);
                    return;
                }
            };
            (session, fmt, Some(*output))
        }
        _ => {
            println!("Usage: /export [format] [output_file]");
            println!("       /export <session_name> <format> <output_file>");
            println!("Formats: json, markdown (or md)");
            return;
        }
    };

    // Export the session
    match session.export(format) {
        Ok(content) => {
            if let Some(path) = output_file {
                match std::fs::write(path, &content) {
                    Ok(()) => {
                        println!("Exported to: {}", path);
                    }
                    Err(e) => {
                        eprintln!("Error writing file: {}", e);
                    }
                }
            } else {
                println!("{}", content);
            }
        }
        Err(e) => {
            eprintln!("Error exporting session: {}", e);
        }
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
    println!("  /continue      - Load the most recent session");
    println!("  /sessions      - List all saved sessions");
    println!("  /delete <name> - Delete a saved session");
    println!("  /export [fmt] [file] - Export session (formats: json, markdown)");
    println!("  /reset         - Clear conversation history");
    println!("  /stats         - Show session statistics");
    println!("  /tokens        - Show token usage and context capacity");

    println!();
    println!("CLI options:");
    println!("  -p, --provider <name>  - Use specific provider (ollama, gemini, openai)");
    println!("  -m, --model <name>     - Use specific model");
    println!("  --ollama-url <url>     - Ollama server URL");
    println!("  -C, --continue <name>  - Start with a specific session");
    println!("  --continue-last        - Start with the most recent session");

    println!();
    println!("Provider auto-detection (default):");
    println!("  1. Ollama (if running at localhost:11434)");
    println!("  2. Gemini (if GEMINI_API_KEY set)");

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
