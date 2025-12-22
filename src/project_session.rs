//! Project-based session management
//!
//! Provides per-directory conversation history, enabling continuation
//! of conversations specific to each project directory.
//!
//! Directory structure:
//! ```text
//! ~/.gemini-repl/projects/
//! +-- project-name-a1b2c3d4/
//! |   +-- conversation.jsonl    # Current conversation
//! |   +-- history/              # Previous conversations
//! |       +-- 2025-01-15-abc123.jsonl
//! ```

use crate::api::Content;
use crate::config::AppDirs;
use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fs::{self, File, OpenOptions};
use std::io::{BufRead, BufReader, Write};
use std::path::PathBuf;

/// A single conversation turn stored in JSONL format
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConversationEntry {
    /// Timestamp of this entry
    pub timestamp: DateTime<Utc>,
    /// The message content
    pub content: Content,
}

/// Project session manager
pub struct ProjectSessionManager {
    /// App directories
    dirs: AppDirs,
    /// Current working directory
    cwd: PathBuf,
    /// Project-specific directory
    project_dir: PathBuf,
}

impl ProjectSessionManager {
    /// Create a new project session manager for the current directory
    pub fn new(dirs: AppDirs) -> Result<Self> {
        let cwd = std::env::current_dir()?;
        let project_dir = dirs.project_dir(&cwd);

        // Ensure project directory exists
        fs::create_dir_all(&project_dir)?;
        fs::create_dir_all(project_dir.join("history"))?;

        Ok(Self {
            dirs,
            cwd,
            project_dir,
        })
    }

    /// Create with a specific working directory (for testing)
    pub fn with_cwd(dirs: AppDirs, cwd: PathBuf) -> Result<Self> {
        let project_dir = dirs.project_dir(&cwd);
        fs::create_dir_all(&project_dir)?;
        fs::create_dir_all(project_dir.join("history"))?;

        Ok(Self {
            dirs,
            cwd,
            project_dir,
        })
    }

    /// Get path to the current conversation file
    pub fn conversation_path(&self) -> PathBuf {
        self.project_dir.join("conversation.jsonl")
    }

    /// Check if there's an existing conversation to continue
    pub fn has_conversation(&self) -> bool {
        let path = self.conversation_path();
        path.exists() && path.metadata().map(|m| m.len() > 0).unwrap_or(false)
    }

    /// Load the current conversation
    pub fn load_conversation(&self) -> Result<Vec<Content>> {
        let path = self.conversation_path();
        if !path.exists() {
            return Ok(Vec::new());
        }

        let file = File::open(&path).with_context(|| format!("Failed to open {:?}", path))?;
        let reader = BufReader::new(file);
        let mut conversation = Vec::new();

        for line in reader.lines() {
            let line = line?;
            if line.trim().is_empty() {
                continue;
            }
            let entry: ConversationEntry = serde_json::from_str(&line)
                .with_context(|| "Failed to parse conversation entry")?;
            conversation.push(entry.content);
        }

        Ok(conversation)
    }

    /// Append a message to the current conversation
    pub fn append_message(&self, content: &Content) -> Result<()> {
        let path = self.conversation_path();
        let entry = ConversationEntry {
            timestamp: Utc::now(),
            content: content.clone(),
        };

        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&path)
            .with_context(|| format!("Failed to open {:?} for append", path))?;

        let json = serde_json::to_string(&entry)?;
        writeln!(file, "{}", json)?;

        Ok(())
    }

    /// Save the entire conversation (overwrites existing)
    pub fn save_conversation(&self, conversation: &[Content]) -> Result<()> {
        let path = self.conversation_path();
        let mut file = File::create(&path)?;

        for content in conversation {
            let entry = ConversationEntry {
                timestamp: Utc::now(),
                content: content.clone(),
            };
            let json = serde_json::to_string(&entry)?;
            writeln!(file, "{}", json)?;
        }

        Ok(())
    }

    /// Archive the current conversation and start fresh
    pub fn archive_conversation(&self) -> Result<Option<PathBuf>> {
        let current_path = self.conversation_path();
        if !current_path.exists() {
            return Ok(None);
        }

        // Generate archive filename with timestamp
        let timestamp = Utc::now().format("%Y-%m-%d-%H%M%S");
        let hash: u32 = rand::random();
        let archive_name = format!("{}-{:08x}.jsonl", timestamp, hash);
        let archive_path = self.project_dir.join("history").join(&archive_name);

        // Move current to archive
        fs::rename(&current_path, &archive_path)?;

        Ok(Some(archive_path))
    }

    /// Clear the current conversation
    pub fn clear_conversation(&self) -> Result<()> {
        let path = self.conversation_path();
        if path.exists() {
            fs::remove_file(&path)?;
        }
        Ok(())
    }

    /// Get conversation metadata
    pub fn get_metadata(&self) -> Result<ConversationMetadata> {
        let path = self.conversation_path();
        let message_count = if path.exists() {
            let file = File::open(&path)?;
            BufReader::new(file).lines().count()
        } else {
            0
        };

        let modified = if path.exists() {
            path.metadata()?.modified().ok().map(|t| t.into())
        } else {
            None
        };

        Ok(ConversationMetadata {
            project_dir: self.project_dir.clone(),
            cwd: self.cwd.clone(),
            message_count,
            last_modified: modified,
        })
    }

    /// List archived conversations
    pub fn list_archives(&self) -> Result<Vec<ArchiveInfo>> {
        let history_dir = self.project_dir.join("history");
        if !history_dir.exists() {
            return Ok(Vec::new());
        }

        let mut archives = Vec::new();
        for entry in fs::read_dir(&history_dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.extension().map(|e| e == "jsonl").unwrap_or(false) {
                let name = path
                    .file_stem()
                    .and_then(|n| n.to_str())
                    .unwrap_or("unknown")
                    .to_string();

                let message_count = {
                    let file = File::open(&path)?;
                    BufReader::new(file).lines().count()
                };

                let modified = path.metadata()?.modified().ok().map(|t| t.into());

                archives.push(ArchiveInfo {
                    name,
                    path,
                    message_count,
                    modified,
                });
            }
        }

        // Sort by modified time (newest first)
        archives.sort_by(|a, b| b.modified.cmp(&a.modified));

        Ok(archives)
    }
}

/// Metadata about the current conversation
#[derive(Debug, Clone)]
pub struct ConversationMetadata {
    pub project_dir: PathBuf,
    pub cwd: PathBuf,
    pub message_count: usize,
    pub last_modified: Option<DateTime<Utc>>,
}

/// Information about an archived conversation
#[derive(Debug, Clone)]
pub struct ArchiveInfo {
    pub name: String,
    pub path: PathBuf,
    pub message_count: usize,
    pub modified: Option<DateTime<Utc>>,
}

// Use a simple random implementation to avoid adding rand crate
mod rand {
    pub fn random<T: Default>() -> T
    where
        T: From<u32>,
    {
        use std::time::{SystemTime, UNIX_EPOCH};
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.subsec_nanos())
            .unwrap_or(0);
        T::from(nanos)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::api::Part;
    use tempfile::tempdir;

    fn make_message(role: &str, text: &str) -> Content {
        Content {
            role: role.to_string(),
            parts: vec![Part {
                text: Some(text.to_string()),
                function_call: None,
                function_response: None,
            }],
        }
    }

    #[test]
    fn test_project_session_creation() {
        let temp = tempdir().unwrap();
        let dirs = AppDirs::with_root(temp.path().to_path_buf()).unwrap();
        let cwd = temp.path().join("test-project");
        fs::create_dir_all(&cwd).unwrap();

        let manager = ProjectSessionManager::with_cwd(dirs, cwd).unwrap();
        assert!(!manager.has_conversation());
    }

    #[test]
    fn test_append_and_load() {
        let temp = tempdir().unwrap();
        let dirs = AppDirs::with_root(temp.path().to_path_buf()).unwrap();
        let cwd = temp.path().join("test-project");
        fs::create_dir_all(&cwd).unwrap();

        let manager = ProjectSessionManager::with_cwd(dirs, cwd).unwrap();

        // Append messages
        manager
            .append_message(&make_message("user", "Hello"))
            .unwrap();
        manager
            .append_message(&make_message("model", "Hi there!"))
            .unwrap();

        assert!(manager.has_conversation());

        // Load and verify
        let conversation = manager.load_conversation().unwrap();
        assert_eq!(conversation.len(), 2);
        assert_eq!(conversation[0].role, "user");
        assert_eq!(conversation[1].role, "model");
    }

    #[test]
    fn test_save_conversation() {
        let temp = tempdir().unwrap();
        let dirs = AppDirs::with_root(temp.path().to_path_buf()).unwrap();
        let cwd = temp.path().join("test-project");
        fs::create_dir_all(&cwd).unwrap();

        let manager = ProjectSessionManager::with_cwd(dirs, cwd).unwrap();

        let conversation = vec![
            make_message("user", "Question 1"),
            make_message("model", "Answer 1"),
            make_message("user", "Question 2"),
            make_message("model", "Answer 2"),
        ];

        manager.save_conversation(&conversation).unwrap();

        let loaded = manager.load_conversation().unwrap();
        assert_eq!(loaded.len(), 4);
    }

    #[test]
    fn test_archive_conversation() {
        let temp = tempdir().unwrap();
        let dirs = AppDirs::with_root(temp.path().to_path_buf()).unwrap();
        let cwd = temp.path().join("test-project");
        fs::create_dir_all(&cwd).unwrap();

        let manager = ProjectSessionManager::with_cwd(dirs, cwd).unwrap();

        // Create a conversation
        manager
            .append_message(&make_message("user", "Test"))
            .unwrap();
        assert!(manager.has_conversation());

        // Archive it
        let archive_path = manager.archive_conversation().unwrap();
        assert!(archive_path.is_some());
        assert!(!manager.has_conversation());

        // Check archives list
        let archives = manager.list_archives().unwrap();
        assert_eq!(archives.len(), 1);
    }

    #[test]
    fn test_metadata() {
        let temp = tempdir().unwrap();
        let dirs = AppDirs::with_root(temp.path().to_path_buf()).unwrap();
        let cwd = temp.path().join("test-project");
        fs::create_dir_all(&cwd).unwrap();

        let manager = ProjectSessionManager::with_cwd(dirs, cwd.clone()).unwrap();

        manager
            .append_message(&make_message("user", "Test"))
            .unwrap();
        manager
            .append_message(&make_message("model", "Response"))
            .unwrap();

        let metadata = manager.get_metadata().unwrap();
        assert_eq!(metadata.message_count, 2);
        assert_eq!(metadata.cwd, cwd);
    }
}
