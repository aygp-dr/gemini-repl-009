//! Tool system with enhanced capabilities for self-modification
//!
//! Tools are registered and their definitions are passed to the Gemini API.
//! The execute path will be connected when function_call handling is implemented.

#![allow(dead_code)]

use anyhow::{bail, Result};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::path::PathBuf;

pub mod code_analysis;
pub mod ed_tools;
pub mod file_tools;
pub mod rust_tools;
pub mod self_awareness;

use code_analysis::{AnalyzeRustCodeTool, FindFunctionTool, FindStructTool};
use ed_tools::EdTool;
use file_tools::{EditFileTool, ListFilesTool, ReadFileTool, WriteFileTool};
use rust_tools::{CargoBuildTool, CargoCheckTool, CargoTestTool, ClippyTool, RustfmtTool};
use self_awareness::{ExplainArchitectureTool, GetCurrentCapabilitiesTool, ProjectMapTool};

/// Tool trait that all tools must implement
#[async_trait]
pub trait Tool: Send + Sync {
    /// Name of the tool
    fn name(&self) -> &str;

    /// Description of what the tool does
    fn description(&self) -> &str;

    /// JSON schema for the tool's parameters
    fn parameters_schema(&self) -> Value;

    /// Execute the tool with given parameters
    async fn execute(&self, params: Value) -> Result<Value>;

    /// Validate parameters before execution
    fn validate_params(&self, _params: &Value) -> Result<()> {
        // Default implementation - tools can override
        Ok(())
    }
}

/// Tool information for listing
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct ToolInfo {
    pub name: String,
    pub description: String,
    pub category: String,
    pub self_modification: bool,
}

impl ToolInfo {
    pub fn new(
        name: impl Into<String>,
        description: impl Into<String>,
        category: impl Into<String>,
        self_modification: bool,
    ) -> Self {
        Self {
            name: name.into(),
            description: description.into(),
            category: category.into(),
            self_modification,
        }
    }
}

/// Registry for managing available tools
pub struct ToolRegistry {
    tools: HashMap<String, Box<dyn Tool>>,
    workspace: PathBuf,
}

impl ToolRegistry {
    /// Create a new tool registry
    pub fn new() -> Self {
        let workspace = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
        Self {
            tools: HashMap::new(),
            workspace,
        }
    }

    /// Initialize default tools
    pub fn initialize_default_tools(&mut self) -> Result<()> {
        // File operation tools
        self.register_tool(Box::new(ReadFileTool::new(self.workspace.clone())))?;
        self.register_tool(Box::new(WriteFileTool::new(self.workspace.clone())))?;
        self.register_tool(Box::new(ListFilesTool::new(self.workspace.clone())))?;

        Ok(())
    }

    /// Initialize self-modification tools
    pub fn initialize_self_modification_tools(&mut self) -> Result<()> {
        // Enhanced file operations
        self.register_tool(Box::new(EditFileTool::new(self.workspace.clone())))?;

        // Code analysis tools
        self.register_tool(Box::new(AnalyzeRustCodeTool::new()))?;
        self.register_tool(Box::new(FindFunctionTool::new(self.workspace.clone())))?;
        self.register_tool(Box::new(FindStructTool::new(self.workspace.clone())))?;

        // Rust-specific tools
        self.register_tool(Box::new(CargoBuildTool::new(self.workspace.clone())))?;
        self.register_tool(Box::new(CargoTestTool::new(self.workspace.clone())))?;
        self.register_tool(Box::new(CargoCheckTool::new(self.workspace.clone())))?;
        self.register_tool(Box::new(ClippyTool::new(self.workspace.clone())))?;
        self.register_tool(Box::new(RustfmtTool::new()))?;

        // Self-awareness tools
        self.register_tool(Box::new(ProjectMapTool::new(self.workspace.clone())))?;
        self.register_tool(Box::new(GetCurrentCapabilitiesTool::new()))?;
        self.register_tool(Box::new(ExplainArchitectureTool::new(
            self.workspace.clone(),
        )))?;

        // Ed-based editing tools
        self.register_tool(Box::new(EdTool::new(self.workspace.clone())))?;

        Ok(())
    }

    /// Register a new tool
    pub fn register_tool(&mut self, tool: Box<dyn Tool>) -> Result<()> {
        let name = tool.name().to_string();
        if self.tools.contains_key(&name) {
            bail!("Tool '{}' is already registered", name);
        }
        self.tools.insert(name, tool);
        Ok(())
    }

    /// Get a tool by name
    pub fn get_tool(&self, name: &str) -> Option<&dyn Tool> {
        self.tools.get(name).map(|b| b.as_ref())
    }

    /// List all available tools
    pub fn list_tools(&self) -> Vec<ToolInfo> {
        self.tools
            .iter()
            .map(|(name, tool)| {
                let category = match name.as_str() {
                    "read_file" | "write_file" | "edit_file" | "list_files" => "file_operations",
                    "analyze_rust_code" | "find_function" | "find_struct" => "code_analysis",
                    "cargo_build" | "cargo_test" | "cargo_check" | "clippy" | "rustfmt" => {
                        "rust_tools"
                    }
                    "project_map" | "get_current_capabilities" | "explain_architecture" => {
                        "self_awareness"
                    }
                    _ => "other",
                };

                let self_modification = matches!(
                    name.as_str(),
                    "edit_file"
                        | "analyze_rust_code"
                        | "find_function"
                        | "find_struct"
                        | "cargo_build"
                        | "cargo_test"
                        | "cargo_check"
                        | "clippy"
                        | "rustfmt"
                        | "project_map"
                        | "get_current_capabilities"
                        | "explain_architecture"
                        | "ed_editor"
                );

                ToolInfo {
                    name: name.clone(),
                    description: tool.description().to_string(),
                    category: category.to_string(),
                    self_modification,
                }
            })
            .collect()
    }

    /// Get tool definitions for API
    pub fn get_tool_definitions(&self) -> Vec<Value> {
        self.tools
            .values()
            .map(|tool| {
                serde_json::json!({
                    "name": tool.name(),
                    "description": tool.description(),
                    "parameters": tool.parameters_schema(),
                })
            })
            .collect()
    }

    /// Execute a tool by name
    pub async fn execute_tool(&self, name: &str, params: Value) -> Result<Value> {
        let tool = self
            .tools
            .get(name)
            .ok_or_else(|| anyhow::anyhow!("Tool '{}' not found", name))?;

        // Validate parameters
        tool.validate_params(&params)?;

        // Execute tool
        tool.execute(params).await
    }
}

/// Security utilities for path validation
pub mod security {
    use anyhow::{bail, Result};
    use std::path::{Component, Path, PathBuf};

    /// Validate that a path is within the workspace
    pub fn validate_path(path: &Path, workspace: &Path) -> Result<PathBuf> {
        let canonical = path.canonicalize().or_else(|_| {
            // If file doesn't exist yet, canonicalize parent and append filename
            if let Some(parent) = path.parent() {
                if let Ok(canonical_parent) = parent.canonicalize() {
                    if let Some(file_name) = path.file_name() {
                        return Ok(canonical_parent.join(file_name));
                    }
                }
            }
            Err(anyhow::anyhow!("Invalid path: {}", path.display()))
        })?;

        let workspace_canonical = workspace.canonicalize()?;

        if !canonical.starts_with(&workspace_canonical) {
            bail!("Path escapes workspace: {}", path.display());
        }

        Ok(canonical)
    }

    /// Check if a path is safe to read/write
    /// This provides early rejection of obviously malicious paths before
    /// the more expensive canonicalization in validate_path
    pub fn is_path_safe(path: &Path) -> bool {
        let path_str = path.to_string_lossy();

        // Reject absolute paths - must be relative to workspace
        if path.is_absolute() {
            return false;
        }

        // Reject paths starting with ~ (home directory expansion)
        if path_str.starts_with('~') {
            return false;
        }

        // Check for path traversal attempts using .. components
        for component in path.components() {
            match component {
                Component::ParentDir => return false,
                Component::RootDir => return false, // Redundant with is_absolute, but explicit
                _ => {}
            }
        }

        // Check for null bytes (shouldn't happen in Rust, but defense in depth)
        if path_str.contains('\0') {
            return false;
        }

        // Check for sensitive files
        if let Some(file_name) = path.file_name() {
            let name = file_name.to_string_lossy();
            let name_lower = name.to_lowercase();
            if name_lower.starts_with(".env")
                || name_lower == ".git"
                || name_lower.contains("secret")
                || name_lower.contains("password")
                || name_lower.contains("credential")
                || name_lower == ".ssh"
                || name_lower == ".gnupg"
                || name_lower == ".aws"
            {
                return false;
            }
        }

        true
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn test_rejects_absolute_paths() {
            assert!(!is_path_safe(Path::new("/etc/passwd")));
            assert!(!is_path_safe(Path::new("/home/user/file.txt")));
        }

        #[test]
        fn test_rejects_parent_traversal() {
            assert!(!is_path_safe(Path::new("../secret.txt")));
            assert!(!is_path_safe(Path::new("foo/../../../etc/passwd")));
            assert!(!is_path_safe(Path::new("a/b/c/../../..")));
        }

        #[test]
        fn test_rejects_home_expansion() {
            assert!(!is_path_safe(Path::new("~/secret.txt")));
            assert!(!is_path_safe(Path::new("~root/.ssh/id_rsa")));
        }

        #[test]
        fn test_rejects_sensitive_files() {
            assert!(!is_path_safe(Path::new(".env")));
            assert!(!is_path_safe(Path::new(".env.local")));
            assert!(!is_path_safe(Path::new("config/.git")));
            assert!(!is_path_safe(Path::new("db_password.txt")));
            assert!(!is_path_safe(Path::new("secret_key.pem")));
            assert!(!is_path_safe(Path::new(".ssh")));
            assert!(!is_path_safe(Path::new(".aws")));
            assert!(!is_path_safe(Path::new("credentials.json")));
        }

        #[test]
        fn test_allows_safe_paths() {
            assert!(is_path_safe(Path::new("src/main.rs")));
            assert!(is_path_safe(Path::new("Cargo.toml")));
            assert!(is_path_safe(Path::new("tests/integration/test_api.rs")));
            assert!(is_path_safe(Path::new("docs/README.md")));
        }

        #[test]
        fn test_case_insensitive_sensitive_check() {
            assert!(!is_path_safe(Path::new(".ENV")));
            assert!(!is_path_safe(Path::new("SECRET.txt")));
            assert!(!is_path_safe(Path::new("Password.json")));
        }
    }
}
