//! Git tools for version control operations
//!
//! Provides read-only git operations for the AI agent to understand
//! repository state, changes, and history.

use anyhow::{bail, Result};
use async_trait::async_trait;
use serde_json::{json, Value};
use std::path::PathBuf;
use std::process::Command;

use super::Tool;

/// Execute a git command and return output
fn run_git_command(args: &[&str], workspace: &PathBuf) -> Result<String> {
    let output = Command::new("git")
        .args(args)
        .current_dir(workspace)
        .output()?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        bail!("git {} failed: {}", args.join(" "), stderr);
    }

    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}

/// Check if we're in a git repository
fn is_git_repo(workspace: &PathBuf) -> bool {
    Command::new("git")
        .args(["rev-parse", "--git-dir"])
        .current_dir(workspace)
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

// ============================================================================
// GitStatusTool
// ============================================================================

/// Tool to get git repository status
pub struct GitStatusTool {
    workspace: PathBuf,
}

impl GitStatusTool {
    pub fn new(workspace: PathBuf) -> Self {
        Self { workspace }
    }
}

#[async_trait]
impl Tool for GitStatusTool {
    fn name(&self) -> &str {
        "git_status"
    }

    fn description(&self) -> &str {
        "Get the current git repository status including staged, unstaged, and untracked files"
    }

    fn parameters_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "short": {
                    "type": "boolean",
                    "description": "Use short format output (default: false)"
                },
                "branch": {
                    "type": "boolean",
                    "description": "Show branch information (default: true)"
                }
            },
            "required": []
        })
    }

    async fn execute(&self, params: Value) -> Result<Value> {
        if !is_git_repo(&self.workspace) {
            return Ok(json!({
                "error": "Not a git repository",
                "workspace": self.workspace.display().to_string()
            }));
        }

        let short = params
            .get("short")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);
        let branch = params
            .get("branch")
            .and_then(|v| v.as_bool())
            .unwrap_or(true);

        let mut args = vec!["status"];
        if short {
            args.push("--short");
        }
        if branch {
            args.push("--branch");
        }

        let output = run_git_command(&args, &self.workspace)?;

        // Also get porcelain output for structured data
        let porcelain = run_git_command(&["status", "--porcelain"], &self.workspace)?;

        let staged: Vec<&str> = porcelain
            .lines()
            .filter(|l| {
                l.starts_with("M ")
                    || l.starts_with("A ")
                    || l.starts_with("D ")
                    || l.starts_with("R ")
            })
            .collect();

        let unstaged: Vec<&str> = porcelain
            .lines()
            .filter(|l| l.starts_with(" M") || l.starts_with(" D"))
            .collect();

        let untracked: Vec<&str> = porcelain.lines().filter(|l| l.starts_with("??")).collect();

        Ok(json!({
            "output": output.trim(),
            "summary": {
                "staged_count": staged.len(),
                "unstaged_count": unstaged.len(),
                "untracked_count": untracked.len()
            },
            "staged": staged,
            "unstaged": unstaged,
            "untracked": untracked
        }))
    }
}

// ============================================================================
// GitDiffTool
// ============================================================================

/// Tool to show git diff
pub struct GitDiffTool {
    workspace: PathBuf,
}

impl GitDiffTool {
    pub fn new(workspace: PathBuf) -> Self {
        Self { workspace }
    }
}

#[async_trait]
impl Tool for GitDiffTool {
    fn name(&self) -> &str {
        "git_diff"
    }

    fn description(&self) -> &str {
        "Show changes between commits, commit and working tree, etc. By default shows unstaged changes."
    }

    fn parameters_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "staged": {
                    "type": "boolean",
                    "description": "Show staged changes (--cached). Default: false (shows unstaged)"
                },
                "file": {
                    "type": "string",
                    "description": "Limit diff to a specific file path"
                },
                "commit": {
                    "type": "string",
                    "description": "Compare against a specific commit (e.g., HEAD~1, main, abc123)"
                },
                "stat": {
                    "type": "boolean",
                    "description": "Show diffstat instead of full diff"
                },
                "name_only": {
                    "type": "boolean",
                    "description": "Show only names of changed files"
                }
            },
            "required": []
        })
    }

    async fn execute(&self, params: Value) -> Result<Value> {
        if !is_git_repo(&self.workspace) {
            return Ok(json!({
                "error": "Not a git repository"
            }));
        }

        let staged = params
            .get("staged")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);
        let file = params.get("file").and_then(|v| v.as_str());
        let commit = params.get("commit").and_then(|v| v.as_str());
        let stat = params
            .get("stat")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);
        let name_only = params
            .get("name_only")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);

        let mut args = vec!["diff"];

        if staged {
            args.push("--cached");
        }

        if stat {
            args.push("--stat");
        }

        if name_only {
            args.push("--name-only");
        }

        if let Some(c) = commit {
            args.push(c);
        }

        if let Some(f) = file {
            args.push("--");
            args.push(f);
        }

        let output = run_git_command(&args, &self.workspace)?;

        // Get summary stats
        let stat_output = if !stat {
            run_git_command(&[&args[..], &["--stat"]].concat(), &self.workspace).ok()
        } else {
            None
        };

        Ok(json!({
            "diff": output.trim(),
            "stat": stat_output.map(|s| s.trim().to_string()),
            "is_empty": output.trim().is_empty()
        }))
    }
}

// ============================================================================
// GitLogTool
// ============================================================================

/// Tool to show git commit history
pub struct GitLogTool {
    workspace: PathBuf,
}

impl GitLogTool {
    pub fn new(workspace: PathBuf) -> Self {
        Self { workspace }
    }
}

#[async_trait]
impl Tool for GitLogTool {
    fn name(&self) -> &str {
        "git_log"
    }

    fn description(&self) -> &str {
        "Show commit history with various formatting options"
    }

    fn parameters_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "count": {
                    "type": "integer",
                    "description": "Number of commits to show (default: 10)"
                },
                "oneline": {
                    "type": "boolean",
                    "description": "Use one-line format (default: true)"
                },
                "file": {
                    "type": "string",
                    "description": "Show commits affecting a specific file"
                },
                "author": {
                    "type": "string",
                    "description": "Filter by author name or email"
                },
                "since": {
                    "type": "string",
                    "description": "Show commits since date (e.g., '1 week ago', '2024-01-01')"
                },
                "grep": {
                    "type": "string",
                    "description": "Filter commits by message pattern"
                }
            },
            "required": []
        })
    }

    async fn execute(&self, params: Value) -> Result<Value> {
        if !is_git_repo(&self.workspace) {
            return Ok(json!({
                "error": "Not a git repository"
            }));
        }

        let count = params.get("count").and_then(|v| v.as_i64()).unwrap_or(10);
        let oneline = params
            .get("oneline")
            .and_then(|v| v.as_bool())
            .unwrap_or(true);
        let file = params.get("file").and_then(|v| v.as_str());
        let author = params.get("author").and_then(|v| v.as_str());
        let since = params.get("since").and_then(|v| v.as_str());
        let grep = params.get("grep").and_then(|v| v.as_str());

        let count_str = format!("-{}", count);
        let mut args = vec!["log", &count_str];

        if oneline {
            args.push("--oneline");
        }

        let author_arg;
        if let Some(a) = author {
            author_arg = format!("--author={}", a);
            args.push(&author_arg);
        }

        let since_arg;
        if let Some(s) = since {
            since_arg = format!("--since={}", s);
            args.push(&since_arg);
        }

        let grep_arg;
        if let Some(g) = grep {
            grep_arg = format!("--grep={}", g);
            args.push(&grep_arg);
        }

        if let Some(f) = file {
            args.push("--");
            args.push(f);
        }

        let output = run_git_command(&args, &self.workspace)?;

        // Parse commits for structured output
        let commits: Vec<&str> = output.lines().collect();

        Ok(json!({
            "output": output.trim(),
            "commit_count": commits.len(),
            "commits": commits
        }))
    }
}

// ============================================================================
// GitBlameTool
// ============================================================================

/// Tool to show line-by-line authorship of a file
pub struct GitBlameTool {
    workspace: PathBuf,
}

impl GitBlameTool {
    pub fn new(workspace: PathBuf) -> Self {
        Self { workspace }
    }
}

#[async_trait]
impl Tool for GitBlameTool {
    fn name(&self) -> &str {
        "git_blame"
    }

    fn description(&self) -> &str {
        "Show what revision and author last modified each line of a file"
    }

    fn parameters_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "file": {
                    "type": "string",
                    "description": "Path to the file to blame"
                },
                "line_start": {
                    "type": "integer",
                    "description": "Starting line number"
                },
                "line_end": {
                    "type": "integer",
                    "description": "Ending line number"
                }
            },
            "required": ["file"]
        })
    }

    async fn execute(&self, params: Value) -> Result<Value> {
        if !is_git_repo(&self.workspace) {
            return Ok(json!({
                "error": "Not a git repository"
            }));
        }

        let file = params
            .get("file")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("file parameter is required"))?;

        let line_start = params.get("line_start").and_then(|v| v.as_i64());
        let line_end = params.get("line_end").and_then(|v| v.as_i64());

        let mut args = vec!["blame"];

        let line_range;
        if let (Some(start), Some(end)) = (line_start, line_end) {
            line_range = format!("-L{},{}", start, end);
            args.push(&line_range);
        }

        args.push(file);

        let output = run_git_command(&args, &self.workspace)?;

        Ok(json!({
            "output": output.trim(),
            "file": file,
            "line_count": output.lines().count()
        }))
    }
}

// ============================================================================
// GitBranchTool
// ============================================================================

/// Tool to list and get information about branches
pub struct GitBranchTool {
    workspace: PathBuf,
}

impl GitBranchTool {
    pub fn new(workspace: PathBuf) -> Self {
        Self { workspace }
    }
}

#[async_trait]
impl Tool for GitBranchTool {
    fn name(&self) -> &str {
        "git_branch"
    }

    fn description(&self) -> &str {
        "List branches and show current branch information"
    }

    fn parameters_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "all": {
                    "type": "boolean",
                    "description": "Show remote branches too (default: false)"
                },
                "verbose": {
                    "type": "boolean",
                    "description": "Show last commit for each branch"
                }
            },
            "required": []
        })
    }

    async fn execute(&self, params: Value) -> Result<Value> {
        if !is_git_repo(&self.workspace) {
            return Ok(json!({
                "error": "Not a git repository"
            }));
        }

        let all = params.get("all").and_then(|v| v.as_bool()).unwrap_or(false);
        let verbose = params
            .get("verbose")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);

        let mut args = vec!["branch"];
        if all {
            args.push("-a");
        }
        if verbose {
            args.push("-v");
        }

        let output = run_git_command(&args, &self.workspace)?;

        // Get current branch
        let current = run_git_command(&["branch", "--show-current"], &self.workspace)?;

        // Parse branches
        let branches: Vec<String> = output
            .lines()
            .map(|l| l.trim_start_matches("* ").trim().to_string())
            .filter(|l| !l.is_empty())
            .collect();

        Ok(json!({
            "output": output.trim(),
            "current_branch": current.trim(),
            "branches": branches,
            "branch_count": branches.len()
        }))
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    fn setup_git_repo() -> tempfile::TempDir {
        let dir = tempdir().unwrap();
        Command::new("git")
            .args(["init"])
            .current_dir(dir.path())
            .output()
            .unwrap();
        Command::new("git")
            .args(["config", "user.email", "test@test.com"])
            .current_dir(dir.path())
            .output()
            .unwrap();
        Command::new("git")
            .args(["config", "user.name", "Test"])
            .current_dir(dir.path())
            .output()
            .unwrap();
        dir
    }

    #[tokio::test]
    async fn test_git_status_not_repo() {
        let dir = tempdir().unwrap();
        let tool = GitStatusTool::new(dir.path().to_path_buf());
        let result = tool.execute(json!({})).await.unwrap();
        assert!(result.get("error").is_some());
    }

    #[tokio::test]
    async fn test_git_status_empty_repo() {
        let dir = setup_git_repo();
        let tool = GitStatusTool::new(dir.path().to_path_buf());
        let result = tool.execute(json!({})).await.unwrap();
        assert!(result.get("output").is_some());
        assert_eq!(result["summary"]["staged_count"], 0);
    }

    #[tokio::test]
    async fn test_git_status_with_changes() {
        let dir = setup_git_repo();
        std::fs::write(dir.path().join("test.txt"), "hello").unwrap();

        let tool = GitStatusTool::new(dir.path().to_path_buf());
        let result = tool.execute(json!({})).await.unwrap();
        assert_eq!(result["summary"]["untracked_count"], 1);
    }

    #[tokio::test]
    async fn test_git_diff_empty() {
        let dir = setup_git_repo();
        let tool = GitDiffTool::new(dir.path().to_path_buf());
        let result = tool.execute(json!({})).await.unwrap();
        assert_eq!(result["is_empty"], true);
    }

    #[tokio::test]
    async fn test_git_log_with_commit() {
        let dir = setup_git_repo();
        // Create a file and commit it
        std::fs::write(dir.path().join("test.txt"), "hello").unwrap();
        Command::new("git")
            .args(["add", "test.txt"])
            .current_dir(dir.path())
            .output()
            .unwrap();
        Command::new("git")
            .args(["commit", "-m", "Initial commit"])
            .current_dir(dir.path())
            .output()
            .unwrap();

        let tool = GitLogTool::new(dir.path().to_path_buf());
        let result = tool.execute(json!({})).await.unwrap();
        assert!(result["commit_count"].as_i64().unwrap() >= 1);
    }

    #[tokio::test]
    async fn test_git_branch_new_repo() {
        let dir = setup_git_repo();
        let tool = GitBranchTool::new(dir.path().to_path_buf());
        let result = tool.execute(json!({})).await.unwrap();
        // New repo might not have a branch until first commit
        assert!(result.get("output").is_some());
    }
}
