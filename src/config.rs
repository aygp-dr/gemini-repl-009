//! Configuration and data directory management
//!
//! Provides a centralized configuration system with a unified directory structure
//! at ~/.gemini-repl/, inspired by Efrit and Continue CLI.
//!
//! Directory structure:
//! ```text
//! ~/.gemini-repl/
//! +-- config.yaml       # Main configuration
//! +-- sessions/         # Saved conversation sessions
//! +-- memory/           # Persistent facts
//! +-- queues/           # File-based I/O for inter-agent communication
//! |   +-- input/        # Incoming requests
//! |   +-- output/       # Outgoing responses
//! |   +-- archive/      # Processed requests
//! +-- cache/            # Temporary cache
//! +-- logs/             # Debug logs
//! +-- permissions.yaml  # Tool permission policies
//! ```

#![allow(dead_code)]

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

/// Application data directory manager
#[derive(Debug, Clone)]
pub struct AppDirs {
    /// Root directory (~/.gemini-repl)
    root: PathBuf,
}

impl AppDirs {
    /// Create or get the application directories
    pub fn new() -> Result<Self> {
        let root = Self::get_root_dir()?;
        let dirs = Self { root };
        dirs.ensure_directories()?;
        Ok(dirs)
    }

    /// Create with a custom root (for testing)
    pub fn with_root(root: PathBuf) -> Result<Self> {
        let dirs = Self { root };
        dirs.ensure_directories()?;
        Ok(dirs)
    }

    /// Get the root directory path
    fn get_root_dir() -> Result<PathBuf> {
        // Check for override environment variable
        if let Ok(override_dir) = std::env::var("GEMINI_REPL_HOME") {
            return Ok(PathBuf::from(override_dir));
        }

        // Default to ~/.gemini-repl
        let home = dirs::home_dir()
            .ok_or_else(|| anyhow::anyhow!("Could not determine home directory"))?;

        Ok(home.join(".gemini-repl"))
    }

    /// Ensure all directories exist
    fn ensure_directories(&self) -> Result<()> {
        let dirs = [
            self.root.clone(),
            self.sessions_dir(),
            self.memory_dir(),
            self.queues_dir(),
            self.queue_input_dir(),
            self.queue_output_dir(),
            self.queue_archive_dir(),
            self.cache_dir(),
            self.logs_dir(),
            self.projects_dir(),
        ];

        for dir in dirs {
            fs::create_dir_all(&dir)
                .with_context(|| format!("Failed to create directory: {:?}", dir))?;
        }

        Ok(())
    }

    /// Root directory
    pub fn root(&self) -> &PathBuf {
        &self.root
    }

    /// Sessions directory
    pub fn sessions_dir(&self) -> PathBuf {
        self.root.join("sessions")
    }

    /// Memory directory
    pub fn memory_dir(&self) -> PathBuf {
        self.root.join("memory")
    }

    /// Queues directory (for file-based I/O)
    pub fn queues_dir(&self) -> PathBuf {
        self.root.join("queues")
    }

    /// Queue input directory
    pub fn queue_input_dir(&self) -> PathBuf {
        self.root.join("queues").join("input")
    }

    /// Queue output directory
    pub fn queue_output_dir(&self) -> PathBuf {
        self.root.join("queues").join("output")
    }

    /// Queue archive directory
    pub fn queue_archive_dir(&self) -> PathBuf {
        self.root.join("queues").join("archive")
    }

    /// Cache directory
    pub fn cache_dir(&self) -> PathBuf {
        self.root.join("cache")
    }

    /// Logs directory
    pub fn logs_dir(&self) -> PathBuf {
        self.root.join("logs")
    }

    /// Projects directory (per-directory conversation history)
    pub fn projects_dir(&self) -> PathBuf {
        self.root.join("projects")
    }

    /// Get project-specific directory for current working directory
    pub fn project_dir(&self, cwd: &std::path::Path) -> PathBuf {
        // Encode the path as a safe directory name
        let encoded = Self::encode_path(cwd);
        self.projects_dir().join(encoded)
    }

    /// Encode a path as a safe directory name
    fn encode_path(path: &std::path::Path) -> String {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        // Create a hash-based short name with readable prefix
        let path_str = path.to_string_lossy();

        // Get the last component for readability
        let name = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown");

        // Hash the full path for uniqueness
        let mut hasher = DefaultHasher::new();
        path_str.hash(&mut hasher);
        let hash = hasher.finish();

        // Format: name-hash (e.g., "gemini-repl-009-a1b2c3d4")
        format!("{}-{:08x}", name.chars().take(32).collect::<String>(), hash as u32)
    }

    /// Main config file path
    pub fn config_file(&self) -> PathBuf {
        self.root.join("config.yaml")
    }

    /// Permissions file path
    pub fn permissions_file(&self) -> PathBuf {
        self.root.join("permissions.yaml")
    }

    /// Memory facts file
    pub fn memory_file(&self) -> PathBuf {
        self.memory_dir().join("facts.json")
    }

    /// Get a session file path
    pub fn session_file(&self, name: &str) -> PathBuf {
        self.sessions_dir().join(format!("{}.json", name))
    }

    /// List all session files
    pub fn list_session_files(&self) -> Result<Vec<PathBuf>> {
        let mut sessions = Vec::new();
        for entry in fs::read_dir(self.sessions_dir())? {
            let entry = entry?;
            let path = entry.path();
            if path.extension().map(|e| e == "json").unwrap_or(false) {
                sessions.push(path);
            }
        }
        Ok(sessions)
    }
}

impl Default for AppDirs {
    fn default() -> Self {
        Self::new().expect("Failed to initialize app directories")
    }
}

/// Main application configuration
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AppConfig {
    /// Default provider (ollama, gemini, openai)
    #[serde(default)]
    pub default_provider: Option<String>,

    /// Default model
    #[serde(default)]
    pub default_model: Option<String>,

    /// Ollama URL
    #[serde(default)]
    pub ollama_url: Option<String>,

    /// Enable self-modification by default
    #[serde(default)]
    pub enable_self_modification: bool,

    /// Auto-save sessions
    #[serde(default = "default_auto_save")]
    pub auto_save_sessions: bool,

    /// Session auto-save interval in minutes
    #[serde(default = "default_auto_save_interval")]
    pub auto_save_interval_mins: u32,

    /// Enable file queue watching
    #[serde(default)]
    pub enable_queue_watching: bool,

    /// Queue poll interval in seconds
    #[serde(default = "default_queue_poll_interval")]
    pub queue_poll_interval_secs: u32,
}

fn default_auto_save() -> bool {
    true
}

fn default_auto_save_interval() -> u32 {
    5
}

fn default_queue_poll_interval() -> u32 {
    1
}

impl AppConfig {
    /// Load configuration from file
    pub fn load(dirs: &AppDirs) -> Result<Self> {
        let config_path = dirs.config_file();
        if !config_path.exists() {
            return Ok(Self::default());
        }

        let content = fs::read_to_string(&config_path)
            .with_context(|| format!("Failed to read config: {:?}", config_path))?;

        let config: AppConfig = serde_yaml::from_str(&content)
            .with_context(|| "Failed to parse config.yaml")?;

        Ok(config)
    }

    /// Save configuration to file
    pub fn save(&self, dirs: &AppDirs) -> Result<()> {
        let config_path = dirs.config_file();
        let content = serde_yaml::to_string(self)?;
        fs::write(&config_path, content)
            .with_context(|| format!("Failed to write config: {:?}", config_path))?;
        Ok(())
    }
}

/// Tool permission policy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolPermission {
    /// Tool name or pattern (e.g., "Bash", "Bash(git*)")
    pub pattern: String,
    /// Permission action
    pub action: PermissionAction,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PermissionAction {
    /// Always allow
    Allow,
    /// Always ask
    Ask,
    /// Never allow
    Deny,
}

/// Permission configuration
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Permissions {
    /// Tool-specific permissions
    #[serde(default)]
    pub tools: Vec<ToolPermission>,
}

impl Permissions {
    /// Load permissions from file
    pub fn load(dirs: &AppDirs) -> Result<Self> {
        let path = dirs.permissions_file();
        if !path.exists() {
            return Ok(Self::default());
        }

        let content = fs::read_to_string(&path)?;
        let perms: Permissions = serde_yaml::from_str(&content)?;
        Ok(perms)
    }

    /// Save permissions to file
    pub fn save(&self, dirs: &AppDirs) -> Result<()> {
        let path = dirs.permissions_file();
        let content = serde_yaml::to_string(self)?;
        fs::write(&path, content)?;
        Ok(())
    }

    /// Check permission for a tool
    pub fn check(&self, tool_name: &str, command: Option<&str>) -> PermissionAction {
        for perm in &self.tools {
            if Self::matches_pattern(&perm.pattern, tool_name, command) {
                return perm.action;
            }
        }
        // Default: ask for confirmation
        PermissionAction::Ask
    }

    /// Add or update a permission
    pub fn set(&mut self, pattern: &str, action: PermissionAction) {
        // Remove existing matching pattern
        self.tools.retain(|p| p.pattern != pattern);
        self.tools.push(ToolPermission {
            pattern: pattern.to_string(),
            action,
        });
    }

    /// Check if a pattern matches a tool call
    fn matches_pattern(pattern: &str, tool_name: &str, command: Option<&str>) -> bool {
        // Pattern formats:
        // "Bash" - matches any Bash call
        // "Bash(git*)" - matches Bash calls starting with "git"
        // "Bash(git commit*)" - matches specific git commands

        if let Some(start) = pattern.find('(') {
            if let Some(end) = pattern.find(')') {
                let name = &pattern[..start];
                let cmd_pattern = &pattern[start + 1..end];

                if name != tool_name {
                    return false;
                }

                if let Some(cmd) = command {
                    return Self::glob_match(cmd_pattern, cmd);
                }
                return false;
            }
        }

        // Simple tool name match
        pattern == tool_name
    }

    /// Simple glob matching (supports * at end)
    fn glob_match(pattern: &str, text: &str) -> bool {
        if let Some(prefix) = pattern.strip_suffix('*') {
            text.starts_with(prefix)
        } else {
            pattern == text
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_app_dirs_creation() {
        let dir = tempdir().unwrap();
        let app_dirs = AppDirs::with_root(dir.path().to_path_buf()).unwrap();

        assert!(app_dirs.sessions_dir().exists());
        assert!(app_dirs.memory_dir().exists());
        assert!(app_dirs.queues_dir().exists());
        assert!(app_dirs.queue_input_dir().exists());
        assert!(app_dirs.queue_output_dir().exists());
        assert!(app_dirs.cache_dir().exists());
        assert!(app_dirs.logs_dir().exists());
    }

    #[test]
    fn test_config_save_load() {
        let dir = tempdir().unwrap();
        let app_dirs = AppDirs::with_root(dir.path().to_path_buf()).unwrap();

        let config = AppConfig {
            default_provider: Some("ollama".to_string()),
            default_model: Some("llama3.2".to_string()),
            ..Default::default()
        };

        config.save(&app_dirs).unwrap();
        let loaded = AppConfig::load(&app_dirs).unwrap();

        assert_eq!(loaded.default_provider, Some("ollama".to_string()));
        assert_eq!(loaded.default_model, Some("llama3.2".to_string()));
    }

    #[test]
    fn test_permissions_pattern_matching() {
        // Simple match
        assert!(Permissions::matches_pattern("Bash", "Bash", None));
        assert!(!Permissions::matches_pattern("Bash", "Read", None));

        // Pattern with command
        assert!(Permissions::matches_pattern(
            "Bash(git*)",
            "Bash",
            Some("git status")
        ));
        assert!(Permissions::matches_pattern(
            "Bash(git commit*)",
            "Bash",
            Some("git commit -m test")
        ));
        assert!(!Permissions::matches_pattern(
            "Bash(git*)",
            "Bash",
            Some("rm -rf /")
        ));
    }

    #[test]
    fn test_permissions_save_load() {
        let dir = tempdir().unwrap();
        let app_dirs = AppDirs::with_root(dir.path().to_path_buf()).unwrap();

        let mut perms = Permissions::default();
        perms.set("Bash(git*)", PermissionAction::Allow);
        perms.set("Bash(rm*)", PermissionAction::Deny);

        perms.save(&app_dirs).unwrap();
        let loaded = Permissions::load(&app_dirs).unwrap();

        assert_eq!(loaded.check("Bash", Some("git status")), PermissionAction::Allow);
        assert_eq!(loaded.check("Bash", Some("rm -rf /")), PermissionAction::Deny);
        assert_eq!(loaded.check("Bash", Some("echo hello")), PermissionAction::Ask);
    }
}
