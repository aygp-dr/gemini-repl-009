//! Session persistence for conversation management
//!
//! Provides save/load functionality for conversation sessions.

#![allow(dead_code)]

use anyhow::{bail, Context, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

use crate::api::Content;

/// Session metadata and conversation data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Session {
    /// Session name/identifier
    pub name: String,
    /// Model used for the session
    pub model: String,
    /// When the session was created
    pub created_at: DateTime<Utc>,
    /// When the session was last modified
    pub updated_at: DateTime<Utc>,
    /// Conversation history
    pub conversation: Vec<Content>,
    /// Number of messages in the conversation
    pub message_count: usize,
}

impl Session {
    /// Create a new empty session
    pub fn new(name: impl Into<String>, model: impl Into<String>) -> Self {
        let now = Utc::now();
        Self {
            name: name.into(),
            model: model.into(),
            created_at: now,
            updated_at: now,
            conversation: Vec::new(),
            message_count: 0,
        }
    }

    /// Create a session from existing conversation
    pub fn from_conversation(
        name: impl Into<String>,
        model: impl Into<String>,
        conversation: Vec<Content>,
    ) -> Self {
        let now = Utc::now();
        let message_count = conversation.len();
        Self {
            name: name.into(),
            model: model.into(),
            created_at: now,
            updated_at: now,
            conversation,
            message_count,
        }
    }

    /// Update the session with new conversation data
    pub fn update(&mut self, conversation: Vec<Content>) {
        self.message_count = conversation.len();
        self.conversation = conversation;
        self.updated_at = Utc::now();
    }
}

/// Session manager for handling persistence
pub struct SessionManager {
    /// Base directory for sessions
    sessions_dir: PathBuf,
}

impl SessionManager {
    /// Create a new session manager
    pub fn new() -> Result<Self> {
        let sessions_dir = get_sessions_dir()?;

        // Ensure the sessions directory exists
        if !sessions_dir.exists() {
            fs::create_dir_all(&sessions_dir).context("Failed to create sessions directory")?;
        }

        Ok(Self { sessions_dir })
    }

    /// Save a session to disk
    pub fn save(&self, session: &Session) -> Result<PathBuf> {
        let filename = sanitize_filename(&session.name);
        let path = self.sessions_dir.join(format!("{}.json", filename));

        let json = serde_json::to_string_pretty(session).context("Failed to serialize session")?;

        fs::write(&path, json).context("Failed to write session file")?;

        Ok(path)
    }

    /// Load a session from disk
    pub fn load(&self, name: &str) -> Result<Session> {
        let filename = sanitize_filename(name);
        let path = self.sessions_dir.join(format!("{}.json", filename));

        if !path.exists() {
            bail!("Session '{}' not found", name);
        }

        let content = fs::read_to_string(&path).context("Failed to read session file")?;

        let session: Session =
            serde_json::from_str(&content).context("Failed to parse session file")?;

        Ok(session)
    }

    /// List all available sessions
    pub fn list(&self) -> Result<Vec<SessionInfo>> {
        let mut sessions = Vec::new();

        for entry in fs::read_dir(&self.sessions_dir)? {
            let entry = entry?;
            let path = entry.path();

            if path.extension().and_then(|s| s.to_str()) == Some("json") {
                if let Ok(session) = self.load_info(&path) {
                    sessions.push(session);
                }
            }
        }

        // Sort by updated_at descending (most recent first)
        sessions.sort_by(|a, b| b.updated_at.cmp(&a.updated_at));

        Ok(sessions)
    }

    /// Delete a session
    pub fn delete(&self, name: &str) -> Result<()> {
        let filename = sanitize_filename(name);
        let path = self.sessions_dir.join(format!("{}.json", filename));

        if !path.exists() {
            bail!("Session '{}' not found", name);
        }

        fs::remove_file(&path).context("Failed to delete session file")?;

        Ok(())
    }

    /// Load just the session info without the full conversation
    fn load_info(&self, path: &Path) -> Result<SessionInfo> {
        let content = fs::read_to_string(path)?;
        let session: Session = serde_json::from_str(&content)?;

        Ok(SessionInfo {
            name: session.name,
            model: session.model,
            created_at: session.created_at,
            updated_at: session.updated_at,
            message_count: session.message_count,
        })
    }

    /// Generate a unique session name based on timestamp
    pub fn generate_name(&self) -> String {
        Utc::now().format("session_%Y%m%d_%H%M%S").to_string()
    }

    /// Get the sessions directory path
    pub fn sessions_dir(&self) -> &Path {
        &self.sessions_dir
    }

    /// Get the most recently updated session
    pub fn get_last_session(&self) -> Result<Option<Session>> {
        let sessions = self.list()?;
        if sessions.is_empty() {
            return Ok(None);
        }
        // Sessions are sorted by updated_at descending, so first is most recent
        let last_info = &sessions[0];
        let session = self.load(&last_info.name)?;
        Ok(Some(session))
    }

    /// Check if a session exists
    pub fn exists(&self, name: &str) -> bool {
        let filename = sanitize_filename(name);
        let path = self.sessions_dir.join(format!("{}.json", filename));
        path.exists()
    }
}

/// Summary info for a session (without full conversation)
#[derive(Debug, Clone)]
pub struct SessionInfo {
    pub name: String,
    pub model: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub message_count: usize,
}

/// Get the sessions directory path
fn get_sessions_dir() -> Result<PathBuf> {
    let base_dir = dirs::data_local_dir()
        .or_else(dirs::home_dir)
        .context("Could not determine home directory")?;

    Ok(base_dir.join(".gemini_repl").join("sessions"))
}

/// Sanitize a filename to be safe for the filesystem
fn sanitize_filename(name: &str) -> String {
    name.chars()
        .map(|c| match c {
            '/' | '\\' | ':' | '*' | '?' | '"' | '<' | '>' | '|' | ' ' => '_',
            c => c,
        })
        .collect()
}

/// Export format for sessions
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExportFormat {
    Json,
    Markdown,
}

impl std::str::FromStr for ExportFormat {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self> {
        match s.to_lowercase().as_str() {
            "json" => Ok(ExportFormat::Json),
            "md" | "markdown" => Ok(ExportFormat::Markdown),
            _ => bail!("Unknown export format: {}. Use 'json' or 'markdown'", s),
        }
    }
}

impl Session {
    /// Export session to a string in the specified format
    pub fn export(&self, format: ExportFormat) -> Result<String> {
        match format {
            ExportFormat::Json => {
                serde_json::to_string_pretty(self).context("Failed to serialize session")
            }
            ExportFormat::Markdown => Ok(self.to_markdown()),
        }
    }

    /// Convert session to markdown format
    fn to_markdown(&self) -> String {
        let mut md = String::new();

        // Header
        md.push_str(&format!("# Session: {}\n\n", self.name));
        md.push_str(&format!("- **Model**: {}\n", self.model));
        md.push_str(&format!(
            "- **Created**: {}\n",
            self.created_at.format("%Y-%m-%d %H:%M:%S UTC")
        ));
        md.push_str(&format!(
            "- **Updated**: {}\n",
            self.updated_at.format("%Y-%m-%d %H:%M:%S UTC")
        ));
        md.push_str(&format!("- **Messages**: {}\n\n", self.message_count));
        md.push_str("---\n\n");

        // Conversation
        for content in &self.conversation {
            let role = match content.role.as_str() {
                "user" => "**User**",
                "model" => "**Assistant**",
                "function" => "**Function**",
                _ => &content.role,
            };

            md.push_str(&format!("### {}\n\n", role));

            for part in &content.parts {
                if let Some(text) = &part.text {
                    md.push_str(text);
                    md.push_str("\n\n");
                }
                if let Some(fc) = &part.function_call {
                    md.push_str(&format!(
                        "```json\n// Function call: {}\n{}\n```\n\n",
                        fc.name,
                        serde_json::to_string_pretty(&fc.args).unwrap_or_default()
                    ));
                }
                if let Some(fr) = &part.function_response {
                    md.push_str(&format!(
                        "```json\n// Function response: {}\n{}\n```\n\n",
                        fr.name,
                        serde_json::to_string_pretty(&fr.response).unwrap_or_default()
                    ));
                }
            }
        }

        md
    }
}

/// Session statistics
#[derive(Debug)]
pub struct SessionStats {
    pub message_count: usize,
    pub user_messages: usize,
    pub assistant_messages: usize,
    pub total_chars: usize,
    pub session_name: Option<String>,
}

impl SessionStats {
    pub fn from_conversation(conversation: &[Content], session_name: Option<String>) -> Self {
        let mut user_messages = 0;
        let mut assistant_messages = 0;
        let mut total_chars = 0;

        for content in conversation {
            match content.role.as_str() {
                "user" => user_messages += 1,
                "model" => assistant_messages += 1,
                _ => {}
            }

            for part in &content.parts {
                if let Some(text) = &part.text {
                    total_chars += text.len();
                }
            }
        }

        Self {
            message_count: conversation.len(),
            user_messages,
            assistant_messages,
            total_chars,
            session_name,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::api::Part;

    #[test]
    fn test_sanitize_filename() {
        assert_eq!(sanitize_filename("test session"), "test_session");
        assert_eq!(sanitize_filename("test/file"), "test_file");
        assert_eq!(sanitize_filename("normal_name"), "normal_name");
    }

    #[test]
    fn test_sanitize_filename_special_chars() {
        assert_eq!(sanitize_filename("file:name"), "file_name");
        assert_eq!(sanitize_filename("path\\to\\file"), "path_to_file");
        assert_eq!(
            sanitize_filename("name*with?special<>chars"),
            "name_with_special__chars"
        );
        assert_eq!(sanitize_filename("quote\"test"), "quote_test");
        assert_eq!(sanitize_filename("pipe|test"), "pipe_test");
    }

    #[test]
    fn test_session_new() {
        let session = Session::new("test", "gemini-2.0-flash-exp");
        assert_eq!(session.name, "test");
        assert_eq!(session.model, "gemini-2.0-flash-exp");
        assert_eq!(session.message_count, 0);
        assert!(session.conversation.is_empty());
    }

    #[test]
    fn test_session_from_conversation() {
        let conversation = vec![
            Content {
                role: "user".to_string(),
                parts: vec![Part {
                    text: Some("Hello".to_string()),
                    function_call: None,
                    function_response: None,
                }],
            },
            Content {
                role: "model".to_string(),
                parts: vec![Part {
                    text: Some("Hi there!".to_string()),
                    function_call: None,
                    function_response: None,
                }],
            },
        ];

        let session =
            Session::from_conversation("test_session", "gemini-pro", conversation.clone());

        assert_eq!(session.name, "test_session");
        assert_eq!(session.model, "gemini-pro");
        assert_eq!(session.message_count, 2);
        assert_eq!(session.conversation.len(), 2);
    }

    #[test]
    fn test_session_update() {
        let mut session = Session::new("test", "gemini-pro");
        let original_updated = session.updated_at;

        let conversation = vec![Content {
            role: "user".to_string(),
            parts: vec![Part {
                text: Some("New message".to_string()),
                function_call: None,
                function_response: None,
            }],
        }];

        // Small delay to ensure different timestamps
        std::thread::sleep(std::time::Duration::from_millis(10));
        session.update(conversation);

        assert_eq!(session.message_count, 1);
        assert_eq!(session.conversation.len(), 1);
        assert!(session.updated_at >= original_updated);
    }

    #[test]
    fn test_session_stats_empty() {
        let conversation: Vec<Content> = vec![];
        let stats = SessionStats::from_conversation(&conversation, None);

        assert_eq!(stats.message_count, 0);
        assert_eq!(stats.user_messages, 0);
        assert_eq!(stats.assistant_messages, 0);
        assert_eq!(stats.total_chars, 0);
        assert!(stats.session_name.is_none());
    }

    #[test]
    fn test_session_stats_with_messages() {
        let conversation = vec![
            Content {
                role: "user".to_string(),
                parts: vec![Part {
                    text: Some("Hello".to_string()), // 5 chars
                    function_call: None,
                    function_response: None,
                }],
            },
            Content {
                role: "model".to_string(),
                parts: vec![Part {
                    text: Some("Hi there!".to_string()), // 9 chars
                    function_call: None,
                    function_response: None,
                }],
            },
            Content {
                role: "user".to_string(),
                parts: vec![Part {
                    text: Some("Goodbye".to_string()), // 7 chars
                    function_call: None,
                    function_response: None,
                }],
            },
        ];

        let stats =
            SessionStats::from_conversation(&conversation, Some("test_session".to_string()));

        assert_eq!(stats.message_count, 3);
        assert_eq!(stats.user_messages, 2);
        assert_eq!(stats.assistant_messages, 1);
        assert_eq!(stats.total_chars, 21); // 5 + 9 + 7
        assert_eq!(stats.session_name, Some("test_session".to_string()));
    }

    #[test]
    fn test_session_stats_function_role() {
        let conversation = vec![Content {
            role: "function".to_string(),
            parts: vec![Part {
                text: Some("result".to_string()),
                function_call: None,
                function_response: None,
            }],
        }];

        let stats = SessionStats::from_conversation(&conversation, None);

        assert_eq!(stats.message_count, 1);
        assert_eq!(stats.user_messages, 0);
        assert_eq!(stats.assistant_messages, 0);
        assert_eq!(stats.total_chars, 6);
    }

    #[test]
    fn test_session_serialization() {
        let session = Session::new("serialize_test", "gemini-pro");
        let json = serde_json::to_string(&session).unwrap();

        assert!(json.contains("\"name\":\"serialize_test\""));
        assert!(json.contains("\"model\":\"gemini-pro\""));

        let deserialized: Session = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.name, "serialize_test");
        assert_eq!(deserialized.model, "gemini-pro");
    }

    #[test]
    fn test_session_manager_generate_name() {
        // Skip if we can't create a session manager (no home dir)
        if let Ok(manager) = SessionManager::new() {
            let name = manager.generate_name();
            assert!(name.starts_with("session_"));
            assert!(name.len() > 8); // session_ + date/time
        }
    }

    #[test]
    fn test_session_manager_exists() {
        if let Ok(manager) = SessionManager::new() {
            // Non-existent session should return false
            assert!(!manager.exists("nonexistent_session_12345"));
        }
    }

    #[test]
    fn test_session_manager_get_last_session_empty() {
        if let Ok(manager) = SessionManager::new() {
            // This test depends on whether there are existing sessions
            // Just verify it doesn't crash
            let _ = manager.get_last_session();
        }
    }

    #[test]
    fn test_export_format_from_str() {
        assert_eq!("json".parse::<ExportFormat>().unwrap(), ExportFormat::Json);
        assert_eq!(
            "markdown".parse::<ExportFormat>().unwrap(),
            ExportFormat::Markdown
        );
        assert_eq!(
            "md".parse::<ExportFormat>().unwrap(),
            ExportFormat::Markdown
        );
        assert!("invalid".parse::<ExportFormat>().is_err());
    }

    #[test]
    fn test_session_export_json() {
        let session = Session::new("test_export", "gemini-pro");
        let json = session.export(ExportFormat::Json).unwrap();

        assert!(json.contains("\"name\": \"test_export\""));
        assert!(json.contains("\"model\": \"gemini-pro\""));
    }

    #[test]
    fn test_session_export_markdown() {
        let conversation = vec![
            Content {
                role: "user".to_string(),
                parts: vec![Part {
                    text: Some("Hello".to_string()),
                    function_call: None,
                    function_response: None,
                }],
            },
            Content {
                role: "model".to_string(),
                parts: vec![Part {
                    text: Some("Hi there!".to_string()),
                    function_call: None,
                    function_response: None,
                }],
            },
        ];

        let session = Session::from_conversation("md_test", "gemini-pro", conversation);
        let md = session.export(ExportFormat::Markdown).unwrap();

        assert!(md.contains("# Session: md_test"));
        assert!(md.contains("**Model**: gemini-pro"));
        assert!(md.contains("### **User**"));
        assert!(md.contains("Hello"));
        assert!(md.contains("### **Assistant**"));
        assert!(md.contains("Hi there!"));
    }
}
