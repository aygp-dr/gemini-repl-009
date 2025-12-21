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

    #[test]
    fn test_sanitize_filename() {
        assert_eq!(sanitize_filename("test session"), "test_session");
        assert_eq!(sanitize_filename("test/file"), "test_file");
        assert_eq!(sanitize_filename("normal_name"), "normal_name");
    }

    #[test]
    fn test_session_new() {
        let session = Session::new("test", "gemini-2.0-flash-exp");
        assert_eq!(session.name, "test");
        assert_eq!(session.model, "gemini-2.0-flash-exp");
        assert_eq!(session.message_count, 0);
        assert!(session.conversation.is_empty());
    }
}
