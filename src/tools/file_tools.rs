//! File operation tools with security sandboxing

use super::{Tool, security};
use anyhow::{bail, Result};
use async_trait::async_trait;
use serde::Deserialize;
use serde_json::{json, Value};
use std::fs;
use std::path::{Path, PathBuf};

/// Tool for reading files
pub struct ReadFileTool {
    workspace: PathBuf,
}

impl ReadFileTool {
    pub fn new(workspace: PathBuf) -> Self {
        Self { workspace }
    }
}

#[async_trait]
impl Tool for ReadFileTool {
    fn name(&self) -> &str {
        "read_file"
    }
    
    fn description(&self) -> &str {
        "Read the contents of a file within the workspace"
    }
    
    fn parameters_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "path": {
                    "type": "string",
                    "description": "Path to the file to read (relative to workspace)"
                }
            },
            "required": ["path"]
        })
    }
    
    async fn execute(&self, params: Value) -> Result<Value> {
        #[derive(Deserialize)]
        struct Params {
            path: String,
        }
        
        let params: Params = serde_json::from_value(params)?;
        let path = Path::new(&params.path);
        
        // Security validation
        if !security::is_path_safe(path) {
            bail!("Access denied: unsafe path");
        }
        
        let full_path = self.workspace.join(path);
        let validated_path = security::validate_path(&full_path, &self.workspace)?;
        
        // Read file
        let content = fs::read_to_string(&validated_path)?;
        
        Ok(json!({
            "success": true,
            "path": params.path,
            "content": content,
            "size": content.len(),
        }))
    }
}

/// Tool for writing files
pub struct WriteFileTool {
    workspace: PathBuf,
}

impl WriteFileTool {
    pub fn new(workspace: PathBuf) -> Self {
        Self { workspace }
    }
}

#[async_trait]
impl Tool for WriteFileTool {
    fn name(&self) -> &str {
        "write_file"
    }
    
    fn description(&self) -> &str {
        "Write content to a file within the workspace"
    }
    
    fn parameters_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "path": {
                    "type": "string",
                    "description": "Path to the file to write (relative to workspace)"
                },
                "content": {
                    "type": "string",
                    "description": "Content to write to the file"
                },
                "create_dirs": {
                    "type": "boolean",
                    "description": "Create parent directories if they don't exist",
                    "default": true
                }
            },
            "required": ["path", "content"]
        })
    }
    
    async fn execute(&self, params: Value) -> Result<Value> {
        #[derive(Deserialize)]
        struct Params {
            path: String,
            content: String,
            #[serde(default = "default_true")]
            create_dirs: bool,
        }
        
        fn default_true() -> bool { true }
        
        let params: Params = serde_json::from_value(params)?;
        let path = Path::new(&params.path);
        
        // Security validation
        if !security::is_path_safe(path) {
            bail!("Access denied: unsafe path");
        }
        
        let full_path = self.workspace.join(path);
        let validated_path = security::validate_path(&full_path, &self.workspace)?;
        
        // Create parent directories if requested
        if params.create_dirs {
            if let Some(parent) = validated_path.parent() {
                fs::create_dir_all(parent)?;
            }
        }
        
        // Write file
        fs::write(&validated_path, &params.content)?;
        
        Ok(json!({
            "success": true,
            "path": params.path,
            "bytes_written": params.content.len(),
        }))
    }
}

/// Tool for editing files
pub struct EditFileTool {
    workspace: PathBuf,
}

impl EditFileTool {
    pub fn new(workspace: PathBuf) -> Self {
        Self { workspace }
    }
}

#[derive(Deserialize)]
struct EditOperation {
    operation: String,  // "replace", "insert", "delete"
    search: Option<String>,
    replace: Option<String>,
    line: Option<usize>,
    content: Option<String>,
    start_line: Option<usize>,
    end_line: Option<usize>,
}

#[async_trait]
impl Tool for EditFileTool {
    fn name(&self) -> &str {
        "edit_file"
    }
    
    fn description(&self) -> &str {
        "Edit a file with various operations (replace, insert, delete)"
    }
    
    fn parameters_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "path": {
                    "type": "string",
                    "description": "Path to the file to edit (relative to workspace)"
                },
                "operations": {
                    "type": "array",
                    "description": "List of edit operations to perform",
                    "items": {
                        "type": "object",
                        "properties": {
                            "operation": {
                                "type": "string",
                                "enum": ["replace", "insert", "delete"],
                                "description": "Type of operation"
                            },
                            "search": {
                                "type": "string",
                                "description": "Text to search for (for replace operation)"
                            },
                            "replace": {
                                "type": "string",
                                "description": "Text to replace with (for replace operation)"
                            },
                            "line": {
                                "type": "integer",
                                "description": "Line number (1-indexed) for insert operation"
                            },
                            "content": {
                                "type": "string",
                                "description": "Content to insert"
                            },
                            "start_line": {
                                "type": "integer",
                                "description": "Start line for delete operation (1-indexed)"
                            },
                            "end_line": {
                                "type": "integer",
                                "description": "End line for delete operation (1-indexed, inclusive)"
                            }
                        }
                    }
                }
            },
            "required": ["path", "operations"]
        })
    }
    
    async fn execute(&self, params: Value) -> Result<Value> {
        #[derive(Deserialize)]
        struct Params {
            path: String,
            operations: Vec<EditOperation>,
        }
        
        let params: Params = serde_json::from_value(params)?;
        let path = Path::new(&params.path);
        
        // Security validation
        if !security::is_path_safe(path) {
            bail!("Access denied: unsafe path");
        }
        
        let full_path = self.workspace.join(path);
        let validated_path = security::validate_path(&full_path, &self.workspace)?;
        
        // Read file
        let content = fs::read_to_string(&validated_path)?;
        let mut lines: Vec<String> = content.lines().map(|l| l.to_string()).collect();
        
        // Apply operations
        let mut changes = 0;
        for op in params.operations {
            match op.operation.as_str() {
                "replace" => {
                    let search = op.search.ok_or_else(|| anyhow::anyhow!("search required for replace"))?;
                    let replace = op.replace.ok_or_else(|| anyhow::anyhow!("replace required for replace"))?;
                    
                    // More efficient string replacement without intermediate allocation
                    let mut modified = false;
                    for line in &mut lines {
                        if line.contains(&search) {
                            *line = line.replace(&search, &replace);
                            modified = true;
                        }
                    }
                    if modified {
                        changes += 1;
                    }
                }
                "insert" => {
                    let line = op.line.ok_or_else(|| anyhow::anyhow!("line required for insert"))?;
                    let content = op.content.ok_or_else(|| anyhow::anyhow!("content required for insert"))?;
                    
                    if line > 0 && line <= lines.len() + 1 {
                        lines.insert(line - 1, content);
                        changes += 1;
                    } else {
                        bail!("Line {} out of range", line);
                    }
                }
                "delete" => {
                    let start = op.start_line.ok_or_else(|| anyhow::anyhow!("start_line required for delete"))?;
                    let end = op.end_line.ok_or_else(|| anyhow::anyhow!("end_line required for delete"))?;
                    
                    if start > 0 && end >= start && end <= lines.len() {
                        lines.drain((start - 1)..end);
                        changes += 1;
                    } else {
                        bail!("Invalid line range {}-{}", start, end);
                    }
                }
                _ => bail!("Unknown operation: {}", op.operation),
            }
        }
        
        // Write back
        let new_content = lines.join("\n");
        fs::write(&validated_path, &new_content)?;
        
        Ok(json!({
            "success": true,
            "path": params.path,
            "changes_applied": changes,
            "new_size": new_content.len(),
        }))
    }
}

/// Tool for listing files
pub struct ListFilesTool {
    workspace: PathBuf,
}

impl ListFilesTool {
    pub fn new(workspace: PathBuf) -> Self {
        Self { workspace }
    }
}

#[async_trait]
impl Tool for ListFilesTool {
    fn name(&self) -> &str {
        "list_files"
    }
    
    fn description(&self) -> &str {
        "List files and directories in a given path"
    }
    
    fn parameters_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "path": {
                    "type": "string",
                    "description": "Path to list (relative to workspace)",
                    "default": "."
                },
                "pattern": {
                    "type": "string",
                    "description": "Optional glob pattern to filter files"
                },
                "recursive": {
                    "type": "boolean",
                    "description": "List recursively",
                    "default": false
                }
            }
        })
    }
    
    async fn execute(&self, params: Value) -> Result<Value> {
        #[derive(Deserialize)]
        struct Params {
            #[serde(default = "default_dot")]
            path: String,
            pattern: Option<String>,
            #[serde(default)]
            recursive: bool,
        }
        
        fn default_dot() -> String { ".".to_string() }
        
        let params: Params = serde_json::from_value(params)?;
        let path = Path::new(&params.path);
        
        // Security validation
        if !security::is_path_safe(path) {
            bail!("Access denied: unsafe path");
        }
        
        let full_path = self.workspace.join(path);
        let validated_path = security::validate_path(&full_path, &self.workspace)?;
        
        let mut files = Vec::new();
        
        if params.recursive {
            walk_directory(&validated_path, &self.workspace, &mut files, params.pattern.as_deref())?;
        } else {
            list_directory(&validated_path, &self.workspace, &mut files, params.pattern.as_deref())?;
        }
        
        Ok(json!({
            "success": true,
            "path": params.path,
            "files": files,
            "count": files.len(),
        }))
    }
}

fn list_directory(dir: &Path, workspace: &Path, files: &mut Vec<Value>, pattern: Option<&str>) -> Result<()> {
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        
        let relative = path.strip_prefix(workspace).unwrap_or(&path);
        let name = relative.to_string_lossy().to_string();
        
        if let Some(pattern) = pattern {
            if !glob::Pattern::new(pattern)?.matches(&name) {
                continue;
            }
        }
        
        let metadata = entry.metadata()?;
        files.push(json!({
            "name": name,
            "type": if metadata.is_dir() { "directory" } else { "file" },
            "size": metadata.len(),
        }));
    }
    Ok(())
}

fn walk_directory(dir: &Path, workspace: &Path, files: &mut Vec<Value>, pattern: Option<&str>) -> Result<()> {
    for entry in walkdir::WalkDir::new(dir) {
        let entry = entry?;
        let path = entry.path();
        
        // Skip hidden directories
        if path.file_name()
            .and_then(|n| n.to_str())
            .map(|n| n.starts_with('.'))
            .unwrap_or(false) {
            continue;
        }
        
        let relative = path.strip_prefix(workspace).unwrap_or(path);
        let name = relative.to_string_lossy().to_string();
        
        if let Some(pattern) = pattern {
            if !glob::Pattern::new(pattern)?.matches(&name) {
                continue;
            }
        }
        
        let metadata = entry.metadata()?;
        files.push(json!({
            "name": name,
            "type": if metadata.is_dir() { "directory" } else { "file" },
            "size": metadata.len(),
        }));
    }
    Ok(())
}