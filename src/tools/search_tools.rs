//! Search tools for codebase exploration
//!
//! Provides ripgrep-based code search and glob-based file finding
//! for the AI agent to explore and understand codebases.

use anyhow::{bail, Result};
use async_trait::async_trait;
use serde_json::{json, Value};
use std::path::PathBuf;
use std::process::Command;

use super::Tool;

/// Check if ripgrep is available
fn has_ripgrep() -> bool {
    Command::new("rg")
        .arg("--version")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

/// Execute ripgrep and return output
fn run_ripgrep(args: &[&str], workspace: &PathBuf) -> Result<String> {
    let output = Command::new("rg")
        .args(args)
        .current_dir(workspace)
        .output()?;

    // ripgrep returns exit code 1 when no matches found (not an error)
    if !output.status.success() && output.status.code() != Some(1) {
        let stderr = String::from_utf8_lossy(&output.stderr);
        bail!("rg {} failed: {}", args.join(" "), stderr);
    }

    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}

// ============================================================================
// CodeSearchTool (ripgrep)
// ============================================================================

/// Tool to search code using ripgrep
pub struct CodeSearchTool {
    workspace: PathBuf,
}

impl CodeSearchTool {
    pub fn new(workspace: PathBuf) -> Self {
        Self { workspace }
    }
}

#[async_trait]
impl Tool for CodeSearchTool {
    fn name(&self) -> &str {
        "code_search"
    }

    fn description(&self) -> &str {
        "Search code using ripgrep. Supports regex patterns, file type filtering, and context lines."
    }

    fn parameters_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "pattern": {
                    "type": "string",
                    "description": "Search pattern (supports regex)"
                },
                "path": {
                    "type": "string",
                    "description": "Path to search in (default: current directory)"
                },
                "file_type": {
                    "type": "string",
                    "description": "File type to search (e.g., 'rust', 'py', 'js', 'ts', 'go')"
                },
                "glob": {
                    "type": "string",
                    "description": "Glob pattern for files (e.g., '*.rs', 'src/**/*.ts')"
                },
                "ignore_case": {
                    "type": "boolean",
                    "description": "Case insensitive search (default: false)"
                },
                "word": {
                    "type": "boolean",
                    "description": "Match whole words only (default: false)"
                },
                "context": {
                    "type": "integer",
                    "description": "Lines of context around matches (default: 0)"
                },
                "max_count": {
                    "type": "integer",
                    "description": "Maximum matches per file (default: unlimited)"
                },
                "files_only": {
                    "type": "boolean",
                    "description": "Only show filenames, not matching lines (default: false)"
                },
                "count": {
                    "type": "boolean",
                    "description": "Only show count of matches per file (default: false)"
                },
                "hidden": {
                    "type": "boolean",
                    "description": "Search hidden files and directories (default: false)"
                },
                "no_ignore": {
                    "type": "boolean",
                    "description": "Don't respect .gitignore files (default: false)"
                }
            },
            "required": ["pattern"]
        })
    }

    async fn execute(&self, params: Value) -> Result<Value> {
        if !has_ripgrep() {
            return Ok(json!({
                "error": "ripgrep (rg) is not installed",
                "hint": "Install with: cargo install ripgrep"
            }));
        }

        let pattern = params.get("pattern")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("pattern parameter is required"))?;

        let path = params.get("path").and_then(|v| v.as_str());
        let file_type = params.get("file_type").and_then(|v| v.as_str());
        let glob = params.get("glob").and_then(|v| v.as_str());
        let ignore_case = params.get("ignore_case").and_then(|v| v.as_bool()).unwrap_or(false);
        let word = params.get("word").and_then(|v| v.as_bool()).unwrap_or(false);
        let context = params.get("context").and_then(|v| v.as_i64());
        let max_count = params.get("max_count").and_then(|v| v.as_i64());
        let files_only = params.get("files_only").and_then(|v| v.as_bool()).unwrap_or(false);
        let count = params.get("count").and_then(|v| v.as_bool()).unwrap_or(false);
        let hidden = params.get("hidden").and_then(|v| v.as_bool()).unwrap_or(false);
        let no_ignore = params.get("no_ignore").and_then(|v| v.as_bool()).unwrap_or(false);

        let mut args: Vec<String> = vec![
            "--color=never".to_string(),
            "--line-number".to_string(),
        ];

        if ignore_case {
            args.push("-i".to_string());
        }

        if word {
            args.push("-w".to_string());
        }

        if let Some(c) = context {
            args.push(format!("-C{}", c));
        }

        if let Some(m) = max_count {
            args.push(format!("-m{}", m));
        }

        if files_only {
            args.push("-l".to_string());
        }

        if count {
            args.push("-c".to_string());
        }

        if hidden {
            args.push("--hidden".to_string());
        }

        if no_ignore {
            args.push("--no-ignore".to_string());
        }

        if let Some(t) = file_type {
            args.push(format!("-t{}", t));
        }

        if let Some(g) = glob {
            args.push(format!("--glob={}", g));
        }

        args.push(pattern.to_string());

        if let Some(p) = path {
            args.push(p.to_string());
        }

        let args_ref: Vec<&str> = args.iter().map(|s| s.as_str()).collect();
        let output = run_ripgrep(&args_ref, &self.workspace)?;

        // Parse results
        let lines: Vec<&str> = output.lines().collect();
        let match_count = lines.len();

        // Extract unique files
        let files: Vec<String> = if files_only || count {
            lines.iter().map(|l| l.to_string()).collect()
        } else {
            lines
                .iter()
                .filter_map(|l| l.split(':').next())
                .map(|s| s.to_string())
                .collect::<std::collections::HashSet<_>>()
                .into_iter()
                .collect()
        };

        Ok(json!({
            "output": output.trim(),
            "match_count": match_count,
            "file_count": files.len(),
            "files": files,
            "pattern": pattern,
            "is_empty": output.trim().is_empty()
        }))
    }
}

// ============================================================================
// GlobFilesTool
// ============================================================================

/// Tool to find files using glob patterns
pub struct GlobFilesTool {
    workspace: PathBuf,
}

impl GlobFilesTool {
    pub fn new(workspace: PathBuf) -> Self {
        Self { workspace }
    }
}

#[async_trait]
impl Tool for GlobFilesTool {
    fn name(&self) -> &str {
        "glob_files"
    }

    fn description(&self) -> &str {
        "Find files matching glob patterns. Uses ripgrep's --files with glob filtering."
    }

    fn parameters_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "pattern": {
                    "type": "string",
                    "description": "Glob pattern (e.g., '**/*.rs', 'src/**/*.ts', '*.md')"
                },
                "path": {
                    "type": "string",
                    "description": "Directory to search in (default: current directory)"
                },
                "file_type": {
                    "type": "string",
                    "description": "File type filter (e.g., 'rust', 'py', 'js')"
                },
                "hidden": {
                    "type": "boolean",
                    "description": "Include hidden files (default: false)"
                },
                "max_depth": {
                    "type": "integer",
                    "description": "Maximum directory depth to search"
                }
            },
            "required": ["pattern"]
        })
    }

    async fn execute(&self, params: Value) -> Result<Value> {
        if !has_ripgrep() {
            return Ok(json!({
                "error": "ripgrep (rg) is not installed"
            }));
        }

        let pattern = params.get("pattern")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("pattern parameter is required"))?;

        let path = params.get("path").and_then(|v| v.as_str());
        let file_type = params.get("file_type").and_then(|v| v.as_str());
        let hidden = params.get("hidden").and_then(|v| v.as_bool()).unwrap_or(false);
        let max_depth = params.get("max_depth").and_then(|v| v.as_i64());

        let mut args: Vec<String> = vec![
            "--files".to_string(),
            "--color=never".to_string(),
            format!("--glob={}", pattern),
        ];

        if hidden {
            args.push("--hidden".to_string());
        }

        if let Some(d) = max_depth {
            args.push(format!("--max-depth={}", d));
        }

        if let Some(t) = file_type {
            args.push(format!("-t{}", t));
        }

        if let Some(p) = path {
            args.push(p.to_string());
        }

        let args_ref: Vec<&str> = args.iter().map(|s| s.as_str()).collect();
        let output = run_ripgrep(&args_ref, &self.workspace)?;

        let files: Vec<&str> = output.lines().filter(|l| !l.is_empty()).collect();

        Ok(json!({
            "files": files,
            "count": files.len(),
            "pattern": pattern,
            "is_empty": files.is_empty()
        }))
    }
}

// ============================================================================
// SearchAndReplaceTool
// ============================================================================

/// Tool to search and preview replacements (doesn't actually replace)
pub struct SearchPreviewTool {
    workspace: PathBuf,
}

impl SearchPreviewTool {
    pub fn new(workspace: PathBuf) -> Self {
        Self { workspace }
    }
}

#[async_trait]
impl Tool for SearchPreviewTool {
    fn name(&self) -> &str {
        "search_preview"
    }

    fn description(&self) -> &str {
        "Preview what a search-and-replace would change (read-only, doesn't modify files)"
    }

    fn parameters_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "pattern": {
                    "type": "string",
                    "description": "Search pattern (regex)"
                },
                "replacement": {
                    "type": "string",
                    "description": "Replacement string"
                },
                "path": {
                    "type": "string",
                    "description": "Path to search in"
                },
                "file_type": {
                    "type": "string",
                    "description": "File type filter"
                },
                "glob": {
                    "type": "string",
                    "description": "Glob pattern for files"
                }
            },
            "required": ["pattern", "replacement"]
        })
    }

    async fn execute(&self, params: Value) -> Result<Value> {
        if !has_ripgrep() {
            return Ok(json!({
                "error": "ripgrep (rg) is not installed"
            }));
        }

        let pattern = params.get("pattern")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("pattern parameter is required"))?;

        let replacement = params.get("replacement")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("replacement parameter is required"))?;

        let path = params.get("path").and_then(|v| v.as_str());
        let file_type = params.get("file_type").and_then(|v| v.as_str());
        let glob = params.get("glob").and_then(|v| v.as_str());

        let mut args: Vec<String> = vec![
            "--color=never".to_string(),
            "--line-number".to_string(),
            format!("--replace={}", replacement),
        ];

        if let Some(t) = file_type {
            args.push(format!("-t{}", t));
        }

        if let Some(g) = glob {
            args.push(format!("--glob={}", g));
        }

        args.push(pattern.to_string());

        if let Some(p) = path {
            args.push(p.to_string());
        }

        let args_ref: Vec<&str> = args.iter().map(|s| s.as_str()).collect();
        let output = run_ripgrep(&args_ref, &self.workspace)?;

        let lines: Vec<&str> = output.lines().collect();

        Ok(json!({
            "preview": output.trim(),
            "match_count": lines.len(),
            "pattern": pattern,
            "replacement": replacement,
            "note": "This is a preview only. No files were modified."
        }))
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    fn setup_test_dir() -> tempfile::TempDir {
        let dir = tempdir().unwrap();

        // Create some test files
        fs::write(dir.path().join("main.rs"), "fn main() {\n    println!(\"hello\");\n}\n").unwrap();
        fs::write(dir.path().join("lib.rs"), "pub fn add(a: i32, b: i32) -> i32 {\n    a + b\n}\n").unwrap();
        fs::write(dir.path().join("test.py"), "def hello():\n    print('hello')\n").unwrap();
        fs::create_dir(dir.path().join("src")).unwrap();
        fs::write(dir.path().join("src/utils.rs"), "pub fn util_fn() {}\n").unwrap();

        dir
    }

    #[tokio::test]
    async fn test_code_search_basic() {
        if !has_ripgrep() {
            return; // Skip if rg not installed
        }

        let dir = setup_test_dir();
        let tool = CodeSearchTool::new(dir.path().to_path_buf());

        let result = tool.execute(json!({
            "pattern": "fn"
        })).await.unwrap();

        assert!(!result["is_empty"].as_bool().unwrap());
        assert!(result["match_count"].as_i64().unwrap() > 0);
    }

    #[tokio::test]
    async fn test_code_search_file_type() {
        if !has_ripgrep() {
            return;
        }

        let dir = setup_test_dir();
        let tool = CodeSearchTool::new(dir.path().to_path_buf());

        let result = tool.execute(json!({
            "pattern": "def",
            "file_type": "py"
        })).await.unwrap();

        assert!(!result["is_empty"].as_bool().unwrap());
    }

    #[tokio::test]
    async fn test_code_search_no_matches() {
        if !has_ripgrep() {
            return;
        }

        let dir = setup_test_dir();
        let tool = CodeSearchTool::new(dir.path().to_path_buf());

        let result = tool.execute(json!({
            "pattern": "xyznonexistent123"
        })).await.unwrap();

        assert!(result["is_empty"].as_bool().unwrap());
    }

    #[tokio::test]
    async fn test_code_search_files_only() {
        if !has_ripgrep() {
            return;
        }

        let dir = setup_test_dir();
        let tool = CodeSearchTool::new(dir.path().to_path_buf());

        let result = tool.execute(json!({
            "pattern": "fn",
            "files_only": true
        })).await.unwrap();

        assert!(result["file_count"].as_i64().unwrap() > 0);
    }

    #[tokio::test]
    async fn test_glob_files_basic() {
        if !has_ripgrep() {
            return;
        }

        let dir = setup_test_dir();
        let tool = GlobFilesTool::new(dir.path().to_path_buf());

        let result = tool.execute(json!({
            "pattern": "*.rs"
        })).await.unwrap();

        assert!(!result["is_empty"].as_bool().unwrap());
        assert!(result["count"].as_i64().unwrap() >= 2); // main.rs, lib.rs
    }

    #[tokio::test]
    async fn test_glob_files_recursive() {
        if !has_ripgrep() {
            return;
        }

        let dir = setup_test_dir();
        let tool = GlobFilesTool::new(dir.path().to_path_buf());

        let result = tool.execute(json!({
            "pattern": "**/*.rs"
        })).await.unwrap();

        assert!(result["count"].as_i64().unwrap() >= 3); // main.rs, lib.rs, src/utils.rs
    }

    #[tokio::test]
    async fn test_search_preview() {
        if !has_ripgrep() {
            return;
        }

        let dir = setup_test_dir();
        let tool = SearchPreviewTool::new(dir.path().to_path_buf());

        let result = tool.execute(json!({
            "pattern": "hello",
            "replacement": "world"
        })).await.unwrap();

        assert!(result["preview"].as_str().unwrap().contains("world"));
        assert!(result["note"].as_str().unwrap().contains("preview"));
    }
}
