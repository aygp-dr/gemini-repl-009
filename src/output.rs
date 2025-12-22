//! Output formatting module for JSON and human-readable output
//!
//! Provides structured output that can be formatted as either human-readable
//! text or JSON for machine consumption and scripting.

use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicBool, Ordering};

/// Global flag for JSON output mode
static JSON_MODE: AtomicBool = AtomicBool::new(false);

/// Enable JSON output mode globally
pub fn set_json_mode(enabled: bool) {
    JSON_MODE.store(enabled, Ordering::SeqCst);
}

/// Check if JSON mode is enabled
pub fn is_json_mode() -> bool {
    JSON_MODE.load(Ordering::SeqCst)
}

/// Output message types for structured output
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum OutputMessage {
    /// Welcome/startup message
    Welcome(WelcomeInfo),
    /// Provider connection info
    Provider(ProviderInfo),
    /// User prompt echo (in noop mode)
    UserInput { content: String },
    /// Assistant response
    Response { content: String, tokens: Option<u32> },
    /// Tool call notification
    ToolCall { name: String, status: String },
    /// Tool result
    ToolResult { name: String, success: bool, output: Option<String> },
    /// Session info
    Session(SessionInfo),
    /// Session list
    SessionList { sessions: Vec<SessionInfo> },
    /// Memory/fact info
    Memory(MemoryInfo),
    /// Memory list
    MemoryList { facts: Vec<FactInfo> },
    /// Token statistics
    TokenStats(TokenStatsInfo),
    /// Queue status
    QueueStatus(QueueStatusInfo),
    /// Command result
    CommandResult { command: String, success: bool, message: String },
    /// Warning message
    Warning { message: String },
    /// Error message
    Error { code: Option<String>, message: String, hint: Option<String> },
    /// Info message
    Info { message: String },
    /// Context info
    Context { messages: Vec<ContextMessage> },
    /// Tools list
    Tools { tools: Vec<ToolInfo> },
    /// Help output
    Help { sections: Vec<HelpSection> },
    /// Stats output
    Stats(StatsInfo),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WelcomeInfo {
    pub version: String,
    pub mode: String, // "normal", "noop", "self_modification"
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderInfo {
    pub name: String,
    pub model: String,
    pub connected: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionInfo {
    pub name: String,
    pub model: String,
    pub message_count: usize,
    pub updated_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryInfo {
    pub action: String, // "added", "removed", "cleared"
    pub key: Option<String>,
    pub value: Option<String>,
    pub category: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FactInfo {
    pub key: String,
    pub value: String,
    pub category: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenStatsInfo {
    pub total: usize,
    pub max: usize,
    pub percentage: f64,
    pub user_tokens: usize,
    pub user_messages: usize,
    pub assistant_tokens: usize,
    pub assistant_messages: usize,
    pub function_tokens: usize,
    pub function_messages: usize,
    pub system_tokens: usize,
    pub remaining: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueueStatusInfo {
    pub pending_count: usize,
    pub requests: Vec<QueueRequestInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueueRequestInfo {
    pub id: String,
    pub request_type: String,
    pub filename: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextMessage {
    pub role: String,
    pub content: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolInfo {
    pub name: String,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HelpSection {
    pub title: String,
    pub items: Vec<HelpItem>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HelpItem {
    pub command: String,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatsInfo {
    pub session_name: Option<String>,
    pub message_count: usize,
    pub user_messages: usize,
    pub assistant_messages: usize,
    pub total_chars: usize,
}

impl OutputMessage {
    /// Output the message in the appropriate format
    pub fn print(&self) {
        if is_json_mode() {
            self.print_json();
        } else {
            self.print_human();
        }
    }

    /// Output as JSON (one line)
    pub fn print_json(&self) {
        if let Ok(json) = serde_json::to_string(self) {
            println!("{}", json);
        }
    }

    /// Output as human-readable text
    pub fn print_human(&self) {
        match self {
            OutputMessage::Welcome(info) => {
                println!(
                    "Gemini REPL v{} - Type /help for commands, /exit to quit",
                    info.version
                );
                if info.mode == "noop" {
                    println!("Running in NOOP mode (no API calls will be made)");
                } else if info.mode == "self_modification" {
                    println!("Self-modification features: ENABLED");
                }
            }
            OutputMessage::Provider(info) => {
                if info.connected {
                    println!(
                        "Connected to {} (model: {})",
                        info.name.to_uppercase(),
                        info.model
                    );
                } else {
                    println!("Note: No provider available.");
                    println!("  - For Ollama: Start ollama serve");
                    println!("  - For Gemini: Set GEMINI_API_KEY or use --api-key");
                }
            }
            OutputMessage::UserInput { content } => {
                println!("You said: {}", content);
                println!("(Running in noop mode - no API calls made)");
            }
            OutputMessage::Response { content, .. } => {
                println!("{}", content);
            }
            OutputMessage::ToolCall { name, status } => {
                println!("[{}: {}]", name, status);
            }
            OutputMessage::ToolResult { name, success, output } => {
                if *success {
                    if let Some(out) = output {
                        println!("[{} completed]: {}", name, out);
                    }
                } else {
                    println!("[{} failed]", name);
                }
            }
            OutputMessage::Session(info) => {
                println!(
                    "Session '{}' ({} messages)",
                    info.name, info.message_count
                );
            }
            OutputMessage::SessionList { sessions } => {
                if sessions.is_empty() {
                    println!("No saved sessions found");
                } else {
                    println!("Saved sessions:");
                    for s in sessions {
                        let updated = s.updated_at.as_deref().unwrap_or("unknown");
                        println!("  {} - {} messages ({})", s.name, s.message_count, updated);
                    }
                }
            }
            OutputMessage::Memory(info) => {
                match info.action.as_str() {
                    "added" => {
                        println!(
                            "Remembered: {} = {} [{}]",
                            info.key.as_deref().unwrap_or(""),
                            info.value.as_deref().unwrap_or(""),
                            info.category.as_deref().unwrap_or("general")
                        );
                    }
                    "removed" => {
                        println!("Forgot: {}", info.key.as_deref().unwrap_or(""));
                    }
                    "cleared" => {
                        println!("Memory cleared");
                    }
                    _ => {}
                }
            }
            OutputMessage::MemoryList { facts } => {
                if facts.is_empty() {
                    println!("No remembered facts. Use /remember to add facts.");
                } else {
                    println!("Remembered facts ({}):", facts.len());
                    for fact in facts {
                        println!("  [{}] {}: {}", fact.category, fact.key, fact.value);
                    }
                }
            }
            OutputMessage::TokenStats(stats) => {
                println!("Token Statistics:");
                println!(
                    "  {} / {} tokens ({:.1}%)",
                    stats.total, stats.max, stats.percentage
                );
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
                println!("  Remaining capacity: {} tokens", stats.remaining);
            }
            OutputMessage::QueueStatus(status) => {
                if status.pending_count == 0 {
                    println!("No pending requests in queue");
                } else {
                    println!("Pending queue requests ({}):", status.pending_count);
                    for req in &status.requests {
                        println!("  {} - {}: {}", req.id, req.request_type, req.filename);
                    }
                }
            }
            OutputMessage::CommandResult { command, success, message } => {
                if *success {
                    println!("{}", message);
                } else {
                    eprintln!("Error in {}: {}", command, message);
                }
            }
            OutputMessage::Warning { message } => {
                println!("Warning: {}", message);
            }
            OutputMessage::Error { code, message, hint } => {
                if let Some(c) = code {
                    eprintln!("Error [{}]: {}", c, message);
                } else {
                    eprintln!("Error: {}", message);
                }
                if let Some(h) = hint {
                    eprintln!("  Hint: {}", h);
                }
            }
            OutputMessage::Info { message } => {
                println!("{}", message);
            }
            OutputMessage::Context { messages } => {
                if messages.is_empty() {
                    println!("No conversation history yet");
                } else {
                    println!("Conversation history ({} messages):", messages.len());
                    for msg in messages {
                        println!("{}: {}", msg.role, msg.content);
                    }
                }
            }
            OutputMessage::Tools { tools } => {
                println!("Available tools:");
                for tool in tools {
                    println!("  - {}: {}", tool.name, tool.description);
                }
            }
            OutputMessage::Help { sections } => {
                for section in sections {
                    if !section.title.is_empty() {
                        println!("\n{}:", section.title);
                    }
                    for item in &section.items {
                        println!("  {}  - {}", item.command, item.description);
                    }
                }
            }
            OutputMessage::Stats(stats) => {
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
        }
    }
}

/// Convenience functions for common output patterns
pub mod emit {
    use super::*;

    pub fn welcome(version: &str, mode: &str) {
        OutputMessage::Welcome(WelcomeInfo {
            version: version.to_string(),
            mode: mode.to_string(),
        })
        .print();
    }

    pub fn provider(name: &str, model: &str, connected: bool) {
        OutputMessage::Provider(ProviderInfo {
            name: name.to_string(),
            model: model.to_string(),
            connected,
        })
        .print();
    }

    pub fn response(content: &str, tokens: Option<u32>) {
        OutputMessage::Response {
            content: content.to_string(),
            tokens,
        }
        .print();
    }

    pub fn tool_call(name: &str, status: &str) {
        OutputMessage::ToolCall {
            name: name.to_string(),
            status: status.to_string(),
        }
        .print();
    }

    pub fn error(message: &str) {
        OutputMessage::Error {
            code: None,
            message: message.to_string(),
            hint: None,
        }
        .print();
    }

    pub fn error_with_code(code: &str, message: &str, hint: Option<&str>) {
        OutputMessage::Error {
            code: Some(code.to_string()),
            message: message.to_string(),
            hint: hint.map(|s| s.to_string()),
        }
        .print();
    }

    pub fn warning(message: &str) {
        OutputMessage::Warning {
            message: message.to_string(),
        }
        .print();
    }

    pub fn info(message: &str) {
        OutputMessage::Info {
            message: message.to_string(),
        }
        .print();
    }

    pub fn command_result(command: &str, success: bool, message: &str) {
        OutputMessage::CommandResult {
            command: command.to_string(),
            success,
            message: message.to_string(),
        }
        .print();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_json_serialization() {
        let msg = OutputMessage::Welcome(WelcomeInfo {
            version: "0.9.0".to_string(),
            mode: "normal".to_string(),
        });

        let json = serde_json::to_string(&msg).unwrap();
        assert!(json.contains("\"type\":\"welcome\""));
        assert!(json.contains("\"version\":\"0.9.0\""));
    }

    #[test]
    fn test_error_with_code() {
        let msg = OutputMessage::Error {
            code: Some("E1001".to_string()),
            message: "Connection failed".to_string(),
            hint: Some("Check if server is running".to_string()),
        };

        let json = serde_json::to_string(&msg).unwrap();
        assert!(json.contains("\"code\":\"E1001\""));
        assert!(json.contains("\"hint\""));
    }

    #[test]
    fn test_json_mode_toggle() {
        assert!(!is_json_mode());
        set_json_mode(true);
        assert!(is_json_mode());
        set_json_mode(false);
        assert!(!is_json_mode());
    }

    #[test]
    fn test_token_stats_serialization() {
        let stats = OutputMessage::TokenStats(TokenStatsInfo {
            total: 1000,
            max: 128000,
            percentage: 0.78,
            user_tokens: 400,
            user_messages: 5,
            assistant_tokens: 600,
            assistant_messages: 5,
            function_tokens: 0,
            function_messages: 0,
            system_tokens: 0,
            remaining: 127000,
        });

        let json = serde_json::to_string(&stats).unwrap();
        assert!(json.contains("\"total\":1000"));
        assert!(json.contains("\"remaining\":127000"));
    }
}
