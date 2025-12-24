//! Memory system for persistent facts across sessions
//!
//! Stores user-defined facts and preferences that persist across REPL sessions,
//! similar to Claude Code's CLAUDE.md functionality.

#![allow(dead_code)]

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

/// A fact stored in memory
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Fact {
    /// The key/identifier for this fact
    pub key: String,
    /// The fact content
    pub content: String,
    /// Category for organization
    #[serde(default)]
    pub category: FactCategory,
    /// When the fact was added
    #[serde(with = "chrono::serde::ts_seconds")]
    pub created_at: chrono::DateTime<chrono::Utc>,
}

/// Categories for organizing facts
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum FactCategory {
    /// General information
    #[default]
    General,
    /// User preferences
    Preference,
    /// Project-specific information
    Project,
    /// Technical constraints or requirements
    Technical,
}

impl std::fmt::Display for FactCategory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FactCategory::General => write!(f, "general"),
            FactCategory::Preference => write!(f, "preference"),
            FactCategory::Project => write!(f, "project"),
            FactCategory::Technical => write!(f, "technical"),
        }
    }
}

impl std::str::FromStr for FactCategory {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self> {
        match s.to_lowercase().as_str() {
            "general" | "gen" => Ok(FactCategory::General),
            "preference" | "pref" => Ok(FactCategory::Preference),
            "project" | "proj" => Ok(FactCategory::Project),
            "technical" | "tech" => Ok(FactCategory::Technical),
            _ => anyhow::bail!(
                "Unknown category: {}. Use: general, preference, project, or technical",
                s
            ),
        }
    }
}

/// Memory store for persistent facts
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Memory {
    /// All stored facts
    facts: HashMap<String, Fact>,
    /// Version for future migrations
    #[serde(default = "default_version")]
    version: u32,
}

fn default_version() -> u32 {
    1
}

impl Memory {
    /// Create a new empty memory store
    pub fn new() -> Self {
        Self {
            facts: HashMap::new(),
            version: 1,
        }
    }

    /// Add or update a fact
    pub fn add_fact(&mut self, key: &str, content: &str, category: FactCategory) {
        let fact = Fact {
            key: key.to_string(),
            content: content.to_string(),
            category,
            created_at: chrono::Utc::now(),
        };
        self.facts.insert(key.to_string(), fact);
    }

    /// Remove a fact by key
    pub fn remove_fact(&mut self, key: &str) -> Option<Fact> {
        self.facts.remove(key)
    }

    /// Get a fact by key
    pub fn get_fact(&self, key: &str) -> Option<&Fact> {
        self.facts.get(key)
    }

    /// List all facts
    pub fn list_facts(&self) -> Vec<&Fact> {
        let mut facts: Vec<_> = self.facts.values().collect();
        facts.sort_by(|a, b| a.key.cmp(&b.key));
        facts
    }

    /// List facts by category
    pub fn list_facts_by_category(&self, category: FactCategory) -> Vec<&Fact> {
        let mut facts: Vec<_> = self
            .facts
            .values()
            .filter(|f| f.category == category)
            .collect();
        facts.sort_by(|a, b| a.key.cmp(&b.key));
        facts
    }

    /// Check if memory is empty
    pub fn is_empty(&self) -> bool {
        self.facts.is_empty()
    }

    /// Get count of facts
    pub fn len(&self) -> usize {
        self.facts.len()
    }

    /// Generate a system prompt from stored facts
    pub fn to_system_prompt(&self) -> String {
        if self.facts.is_empty() {
            return String::new();
        }

        let mut prompt = String::from("## Remembered Information\n\n");

        // Group by category
        for category in [
            FactCategory::Technical,
            FactCategory::Project,
            FactCategory::Preference,
            FactCategory::General,
        ] {
            let facts = self.list_facts_by_category(category);
            if !facts.is_empty() {
                prompt.push_str(&format!(
                    "### {}\n",
                    match category {
                        FactCategory::Technical => "Technical Constraints",
                        FactCategory::Project => "Project Information",
                        FactCategory::Preference => "User Preferences",
                        FactCategory::General => "General Information",
                    }
                ));
                for fact in facts {
                    prompt.push_str(&format!("- {}: {}\n", fact.key, fact.content));
                }
                prompt.push('\n');
            }
        }

        prompt
    }

    /// Search facts by content (case-insensitive)
    pub fn search(&self, query: &str) -> Vec<&Fact> {
        let query_lower = query.to_lowercase();
        let mut matches: Vec<_> = self
            .facts
            .values()
            .filter(|f| {
                f.key.to_lowercase().contains(&query_lower)
                    || f.content.to_lowercase().contains(&query_lower)
            })
            .collect();
        matches.sort_by(|a, b| a.key.cmp(&b.key));
        matches
    }
}

/// Manager for persisting memory to disk
pub struct MemoryManager {
    /// Directory where memory is stored
    memory_dir: PathBuf,
    /// Path to the memory file
    memory_file: PathBuf,
}

impl MemoryManager {
    /// Create a new memory manager
    pub fn new() -> Result<Self> {
        let data_dir = dirs::data_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("gemini-repl");

        let memory_dir = data_dir.join("memory");
        let memory_file = memory_dir.join("facts.json");

        // Ensure directory exists
        fs::create_dir_all(&memory_dir)
            .with_context(|| format!("Failed to create memory directory: {:?}", memory_dir))?;

        Ok(Self {
            memory_dir,
            memory_file,
        })
    }

    /// Create with a custom directory (for testing)
    pub fn with_dir(dir: PathBuf) -> Result<Self> {
        fs::create_dir_all(&dir)?;
        let memory_file = dir.join("facts.json");
        Ok(Self {
            memory_dir: dir,
            memory_file,
        })
    }

    /// Get the memory directory path
    pub fn memory_dir(&self) -> &PathBuf {
        &self.memory_dir
    }

    /// Load memory from disk
    pub fn load(&self) -> Result<Memory> {
        if !self.memory_file.exists() {
            return Ok(Memory::new());
        }

        let content = fs::read_to_string(&self.memory_file)
            .with_context(|| format!("Failed to read memory file: {:?}", self.memory_file))?;

        let memory: Memory =
            serde_json::from_str(&content).with_context(|| "Failed to parse memory file")?;

        Ok(memory)
    }

    /// Save memory to disk
    pub fn save(&self, memory: &Memory) -> Result<()> {
        let content = serde_json::to_string_pretty(memory)?;
        fs::write(&self.memory_file, content)
            .with_context(|| format!("Failed to write memory file: {:?}", self.memory_file))?;
        Ok(())
    }

    /// Clear all memory
    pub fn clear(&self) -> Result<()> {
        if self.memory_file.exists() {
            fs::remove_file(&self.memory_file)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_memory_add_and_get() {
        let mut memory = Memory::new();
        memory.add_fact("name", "Test User", FactCategory::General);

        let fact = memory.get_fact("name");
        assert!(fact.is_some());
        assert_eq!(fact.unwrap().content, "Test User");
    }

    #[test]
    fn test_memory_remove() {
        let mut memory = Memory::new();
        memory.add_fact("temp", "Temporary", FactCategory::General);
        assert!(memory.get_fact("temp").is_some());

        memory.remove_fact("temp");
        assert!(memory.get_fact("temp").is_none());
    }

    #[test]
    fn test_memory_list_by_category() {
        let mut memory = Memory::new();
        memory.add_fact("pref1", "Dark mode", FactCategory::Preference);
        memory.add_fact("pref2", "Vim keys", FactCategory::Preference);
        memory.add_fact("proj1", "Rust project", FactCategory::Project);

        let prefs = memory.list_facts_by_category(FactCategory::Preference);
        assert_eq!(prefs.len(), 2);

        let projs = memory.list_facts_by_category(FactCategory::Project);
        assert_eq!(projs.len(), 1);
    }

    #[test]
    fn test_memory_to_system_prompt() {
        let mut memory = Memory::new();
        memory.add_fact("editor", "Use vim", FactCategory::Preference);
        memory.add_fact("language", "Rust", FactCategory::Project);

        let prompt = memory.to_system_prompt();
        assert!(prompt.contains("## Remembered Information"));
        assert!(prompt.contains("editor: Use vim"));
        assert!(prompt.contains("language: Rust"));
    }

    #[test]
    fn test_memory_search() {
        let mut memory = Memory::new();
        memory.add_fact("editor", "Use vim bindings", FactCategory::Preference);
        memory.add_fact("theme", "Dark mode", FactCategory::Preference);
        memory.add_fact("project", "Vim plugin", FactCategory::Project);

        let results = memory.search("vim");
        assert_eq!(results.len(), 2);
    }

    #[test]
    fn test_memory_manager_save_load() {
        let dir = tempdir().unwrap();
        let manager = MemoryManager::with_dir(dir.path().to_path_buf()).unwrap();

        let mut memory = Memory::new();
        memory.add_fact("test", "value", FactCategory::General);

        manager.save(&memory).unwrap();
        let loaded = manager.load().unwrap();

        assert_eq!(loaded.len(), 1);
        assert_eq!(loaded.get_fact("test").unwrap().content, "value");
    }

    #[test]
    fn test_memory_manager_clear() {
        let dir = tempdir().unwrap();
        let manager = MemoryManager::with_dir(dir.path().to_path_buf()).unwrap();

        let mut memory = Memory::new();
        memory.add_fact("test", "value", FactCategory::General);
        manager.save(&memory).unwrap();

        manager.clear().unwrap();
        let loaded = manager.load().unwrap();
        assert!(loaded.is_empty());
    }

    #[test]
    fn test_fact_category_parsing() {
        assert_eq!(
            "general".parse::<FactCategory>().unwrap(),
            FactCategory::General
        );
        assert_eq!(
            "pref".parse::<FactCategory>().unwrap(),
            FactCategory::Preference
        );
        assert_eq!(
            "TECHNICAL".parse::<FactCategory>().unwrap(),
            FactCategory::Technical
        );
        assert!("invalid".parse::<FactCategory>().is_err());
    }
}
