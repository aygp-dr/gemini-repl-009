//! Self-awareness tools for understanding project structure and capabilities

use super::Tool;
use anyhow::Result;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::fs;
use std::path::{Path, PathBuf};

/// Tool for mapping project structure
pub struct ProjectMapTool {
    workspace: PathBuf,
}

impl ProjectMapTool {
    pub fn new(workspace: PathBuf) -> Self {
        Self { workspace }
    }
}

#[async_trait]
impl Tool for ProjectMapTool {
    fn name(&self) -> &str {
        "project_map"
    }
    
    fn description(&self) -> &str {
        "Generate a comprehensive map of the project structure and key files"
    }
    
    fn parameters_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "include_hidden": {
                    "type": "boolean",
                    "description": "Include hidden files and directories",
                    "default": false
                },
                "max_depth": {
                    "type": "integer",
                    "description": "Maximum directory depth to traverse",
                    "default": 3
                }
            }
        })
    }
    
    async fn execute(&self, params: Value) -> Result<Value> {
        #[derive(Deserialize)]
        struct Params {
            #[serde(default)]
            include_hidden: bool,
            #[serde(default = "default_depth")]
            max_depth: usize,
        }
        
        fn default_depth() -> usize { 3 }
        
        let params: Params = serde_json::from_value(params)?;
        
        // Analyze Cargo.toml
        let cargo_info = analyze_cargo_toml(&self.workspace)?;
        
        // Map source structure
        let src_structure = map_directory(&self.workspace.join("src"), params.max_depth, params.include_hidden)?;
        
        // Identify key files
        let key_files = identify_key_files(&self.workspace)?;
        
        // Get dependencies
        let dependencies = cargo_info.get("dependencies").cloned().unwrap_or_default();
        
        Ok(json!({
            "success": true,
            "project": {
                "name": cargo_info.get("name").unwrap_or(&Value::String("unknown".to_string())),
                "version": cargo_info.get("version").unwrap_or(&Value::String("unknown".to_string())),
                "description": cargo_info.get("description"),
            },
            "structure": {
                "src": src_structure,
                "key_files": key_files,
            },
            "dependencies": dependencies,
            "capabilities": {
                "self_modification": true,
                "code_analysis": true,
                "file_operations": true,
                "rust_tooling": true,
            }
        }))
    }
}

/// Tool for getting current capabilities
pub struct GetCurrentCapabilitiesTool;

impl GetCurrentCapabilitiesTool {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl Tool for GetCurrentCapabilitiesTool {
    fn name(&self) -> &str {
        "get_current_capabilities"
    }
    
    fn description(&self) -> &str {
        "List all current capabilities and available tools"
    }
    
    fn parameters_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {}
        })
    }
    
    async fn execute(&self, _params: Value) -> Result<Value> {
        let capabilities = json!({
            "file_operations": {
                "read_file": "Read file contents with security validation",
                "write_file": "Write content to files in workspace",
                "edit_file": "Edit files with various operations",
                "list_files": "List directory contents"
            },
            "code_analysis": {
                "analyze_rust_code": "Parse and analyze Rust code structure",
                "find_function": "Find function definitions in codebase",
                "find_struct": "Find struct definitions in codebase"
            },
            "rust_tooling": {
                "cargo_build": "Build the project with cargo",
                "cargo_test": "Run tests with cargo",
                "cargo_check": "Check code without building",
                "clippy": "Run clippy linter",
                "rustfmt": "Format Rust code"
            },
            "self_awareness": {
                "project_map": "Map project structure and dependencies",
                "get_current_capabilities": "List available capabilities",
                "explain_architecture": "Explain system architecture"
            },
            "security": {
                "workspace_sandboxing": "All operations restricted to workspace",
                "path_validation": "Prevents directory traversal attacks",
                "safe_execution": "Sandboxed command execution"
            }
        });
        
        Ok(json!({
            "success": true,
            "capabilities": capabilities,
            "summary": {
                "total_categories": 5,
                "self_modification_enabled": true,
                "security_features": true,
                "rust_native": true,
            }
        }))
    }
}

/// Tool for explaining architecture
pub struct ExplainArchitectureTool {
    workspace: PathBuf,
}

impl ExplainArchitectureTool {
    pub fn new(workspace: PathBuf) -> Self {
        Self { workspace }
    }
}

#[async_trait]
impl Tool for ExplainArchitectureTool {
    fn name(&self) -> &str {
        "explain_architecture"
    }
    
    fn description(&self) -> &str {
        "Explain the system architecture and design patterns"
    }
    
    fn parameters_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "detail_level": {
                    "type": "string",
                    "enum": ["high", "medium", "low"],
                    "description": "Level of detail in explanation",
                    "default": "medium"
                }
            }
        })
    }
    
    async fn execute(&self, params: Value) -> Result<Value> {
        #[derive(Deserialize)]
        struct Params {
            #[serde(default = "default_medium")]
            detail_level: String,
        }
        
        fn default_medium() -> String { "medium".to_string() }
        
        let params: Params = serde_json::from_value(params)?;
        
        let architecture = json!({
            "overview": {
                "type": "Self-Modifying AI REPL",
                "language": "Rust",
                "paradigm": "Async, Event-Driven",
                "security_model": "Workspace Sandboxing"
            },
            "core_components": {
                "main": "Entry point and REPL loop",
                "api": "Gemini API client and communication",
                "tools": "Tool system with security sandboxing",
                "models": "Model service abstraction layer",
                "self_modification": "Safe code modification capabilities"
            },
            "design_patterns": {
                "tool_pattern": "Strategy pattern for extensible tools",
                "security_pattern": "Defense in depth with validation layers",
                "async_pattern": "Tokio-based async execution",
                "error_pattern": "Result-based error handling"
            },
            "security_architecture": {
                "workspace_isolation": "All operations restricted to workspace directory",
                "path_validation": "Canonicalization and traversal prevention",
                "command_sandboxing": "Whitelisted command execution",
                "audit_logging": "All security-relevant operations logged"
            },
            "self_modification_flow": {
                "1_analysis": "Code analysis tools understand current state",
                "2_planning": "AI plans modifications based on analysis",
                "3_validation": "Security and syntax validation",
                "4_application": "Safe application with rollback capability",
                "5_testing": "Automated testing of modifications"
            }
        });
        
        let detail = match params.detail_level.as_str() {
            "high" => json!({
                "architecture": architecture,
                "implementation_details": get_implementation_details(&self.workspace)?,
                "dependency_graph": analyze_dependencies(&self.workspace)?,
            }),
            "low" => json!({
                "architecture": {
                    "overview": architecture["overview"],
                    "core_components": architecture["core_components"]
                }
            }),
            _ => architecture,
        };
        
        Ok(json!({
            "success": true,
            "explanation": detail
        }))
    }
}

fn analyze_cargo_toml(workspace: &Path) -> Result<Value> {
    let cargo_path = workspace.join("Cargo.toml");
    if cargo_path.exists() {
        let content = fs::read_to_string(cargo_path)?;
        let parsed: toml::Value = toml::from_str(&content)?;
        Ok(serde_json::to_value(parsed)?)
    } else {
        Ok(json!({}))
    }
}

fn map_directory(path: &Path, max_depth: usize, include_hidden: bool) -> Result<Value> {
    if !path.exists() {
        return Ok(json!(null));
    }
    
    let mut files = Vec::new();
    let mut dirs = Vec::new();
    
    for entry in fs::read_dir(path)? {
        let entry = entry?;
        let name = entry.file_name().to_string_lossy().to_string();
        
        if !include_hidden && name.starts_with('.') {
            continue;
        }
        
        let metadata = entry.metadata()?;
        if metadata.is_file() {
            files.push(json!({
                "name": name,
                "size": metadata.len(),
                "extension": Path::new(&name).extension()
                    .and_then(|s| s.to_str())
                    .unwrap_or("")
            }));
        } else if metadata.is_dir() && max_depth > 0 {
            let subdir = map_directory(&entry.path(), max_depth - 1, include_hidden)?;
            dirs.push(json!({
                "name": name,
                "contents": subdir
            }));
        }
    }
    
    Ok(json!({
        "files": files,
        "directories": dirs
    }))
}

fn identify_key_files(workspace: &Path) -> Result<Vec<Value>> {
    let mut key_files = Vec::new();
    
    let important_files = [
        ("Cargo.toml", "Project configuration"),
        ("README.md", "Project documentation"),
        ("README.org", "Project documentation (Org mode)"),
        ("src/main.rs", "Main entry point"),
        ("src/lib.rs", "Library root"),
        (".gitignore", "Git ignore rules"),
        ("LICENSE", "License information"),
    ];
    
    for (file, description) in important_files {
        let path = workspace.join(file);
        if path.exists() {
            let metadata = fs::metadata(&path)?;
            key_files.push(json!({
                "file": file,
                "description": description,
                "size": metadata.len(),
                "exists": true
            }));
        }
    }
    
    Ok(key_files)
}

fn get_implementation_details(workspace: &Path) -> Result<Value> {
    // This would analyze the actual implementation details
    // For now, return a placeholder
    Ok(json!({
        "async_runtime": "Tokio",
        "http_client": "reqwest",
        "cli_framework": "clap",
        "repl_library": "rustyline",
        "serialization": "serde + serde_json",
        "code_parsing": "syn + quote"
    }))
}

fn analyze_dependencies(workspace: &Path) -> Result<Value> {
    let cargo_toml = analyze_cargo_toml(workspace)?;
    let deps = cargo_toml.get("dependencies").cloned().unwrap_or_default();
    
    Ok(json!({
        "production": deps,
        "development": cargo_toml.get("dev-dependencies").cloned().unwrap_or_default(),
        "count": if let Value::Object(obj) = &deps { obj.len() } else { 0 }
    }))
}