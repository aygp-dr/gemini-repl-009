//! Tool system with enhanced capabilities for self-modification

use anyhow::{bail, Result};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

pub mod file_tools;
pub mod code_analysis;
pub mod rust_tools;
pub mod self_awareness;

use file_tools::{ReadFileTool, WriteFileTool, EditFileTool, ListFilesTool};
use code_analysis::{AnalyzeRustCodeTool, FindFunctionTool, FindStructTool};
use rust_tools::{CargoBuildTool, CargoTestTool, RustfmtTool, ClippyTool, CargoCheckTool};
use self_awareness::{ProjectMapTool, GetCurrentCapabilitiesTool, ExplainArchitectureTool};

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
    fn validate_params(&self, params: &Value) -> Result<()> {
        // Default implementation - tools can override
        Ok(())
    }
}

/// Tool information for listing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolInfo {
    pub name: String,
    pub description: String,
    pub category: String,
    pub self_modification: bool,
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
        self.register_tool(Box::new(ExplainArchitectureTool::new(self.workspace.clone())))?;
        
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
    pub fn get_tool(&self, name: &str) -> Option<&Box<dyn Tool>> {
        self.tools.get(name)
    }
    
    /// List all available tools
    pub fn list_tools(&self) -> Vec<ToolInfo> {
        self.tools
            .iter()
            .map(|(name, tool)| {
                let category = match name.as_str() {
                    "read_file" | "write_file" | "edit_file" | "list_files" => "file_operations",
                    "analyze_rust_code" | "find_function" | "find_struct" => "code_analysis",
                    "cargo_build" | "cargo_test" | "cargo_check" | "clippy" | "rustfmt" => "rust_tools",
                    "project_map" | "get_current_capabilities" | "explain_architecture" => "self_awareness",
                    _ => "other",
                };
                
                let self_modification = matches!(
                    name.as_str(),
                    "edit_file" | "analyze_rust_code" | "find_function" | "find_struct" |
                    "cargo_build" | "cargo_test" | "cargo_check" | "clippy" | "rustfmt" | 
                    "project_map" | "get_current_capabilities" | "explain_architecture"
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
        let tool = self.tools
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
    use std::path::{Path, PathBuf};
    use anyhow::{bail, Result};
    
    /// Validate that a path is within the workspace
    pub fn validate_path(path: &Path, workspace: &Path) -> Result<PathBuf> {
        let canonical = path.canonicalize()
            .or_else(|_| {
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
    pub fn is_path_safe(path: &Path) -> bool {
        // Check for path traversal attempts
        for component in path.components() {
            if let std::path::Component::ParentDir = component {
                return false;
            }
        }
        
        // Check for sensitive files
        if let Some(file_name) = path.file_name() {
            let name = file_name.to_string_lossy();
            if name.starts_with('.env') || 
               name == ".git" || 
               name.contains("secret") ||
               name.contains("password") {
                return false;
            }
        }
        
        true
    }
}