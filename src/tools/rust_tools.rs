//! Rust-specific tools for building, testing, and formatting code

use super::Tool;
use anyhow::Result;
use async_trait::async_trait;
use serde::Deserialize;
use serde_json::{json, Value};
use std::path::PathBuf;
use tokio::process::Command as AsyncCommand;

/// Tool for running cargo build
pub struct CargoBuildTool {
    workspace: PathBuf,
}

impl CargoBuildTool {
    pub fn new(workspace: PathBuf) -> Self {
        Self { workspace }
    }
}

#[async_trait]
impl Tool for CargoBuildTool {
    fn name(&self) -> &str {
        "cargo_build"
    }
    
    fn description(&self) -> &str {
        "Build the Rust project using cargo"
    }
    
    fn parameters_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "release": {
                    "type": "boolean",
                    "description": "Build in release mode",
                    "default": false
                },
                "features": {
                    "type": "array",
                    "items": {"type": "string"},
                    "description": "Features to enable"
                },
                "target": {
                    "type": "string",
                    "description": "Target architecture to build for"
                }
            }
        })
    }
    
    async fn execute(&self, params: Value) -> Result<Value> {
        #[derive(Deserialize)]
        struct Params {
            #[serde(default)]
            release: bool,
            features: Option<Vec<String>>,
            target: Option<String>,
        }
        
        let params: Params = serde_json::from_value(params)?;
        
        let mut cmd = AsyncCommand::new("cargo");
        cmd.arg("build");
        cmd.current_dir(&self.workspace);
        
        if params.release {
            cmd.arg("--release");
        }
        
        if let Some(features) = params.features {
            if !features.is_empty() {
                cmd.arg("--features");
                cmd.arg(features.join(","));
            }
        }
        
        if let Some(target) = params.target {
            cmd.arg("--target");
            cmd.arg(target);
        }
        
        let output = tokio::time::timeout(
            std::time::Duration::from_secs(300), // 5 minutes timeout
            cmd.output()
        ).await
        .map_err(|_| anyhow::anyhow!("Command timed out after 5 minutes"))??;
        
        Ok(json!({
            "success": output.status.success(),
            "exit_code": output.status.code(),
            "stdout": String::from_utf8_lossy(&output.stdout),
            "stderr": String::from_utf8_lossy(&output.stderr),
        }))
    }
}

/// Tool for running cargo test
pub struct CargoTestTool {
    workspace: PathBuf,
}

impl CargoTestTool {
    pub fn new(workspace: PathBuf) -> Self {
        Self { workspace }
    }
}

#[async_trait]
impl Tool for CargoTestTool {
    fn name(&self) -> &str {
        "cargo_test"
    }
    
    fn description(&self) -> &str {
        "Run tests using cargo test"
    }
    
    fn parameters_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "test_name": {
                    "type": "string",
                    "description": "Specific test to run (optional)"
                },
                "release": {
                    "type": "boolean",
                    "description": "Run tests in release mode",
                    "default": false
                },
                "features": {
                    "type": "array",
                    "items": {"type": "string"},
                    "description": "Features to enable"
                },
                "verbose": {
                    "type": "boolean",
                    "description": "Verbose output",
                    "default": false
                }
            }
        })
    }
    
    async fn execute(&self, params: Value) -> Result<Value> {
        #[derive(Deserialize)]
        struct Params {
            test_name: Option<String>,
            #[serde(default)]
            release: bool,
            features: Option<Vec<String>>,
            #[serde(default)]
            verbose: bool,
        }
        
        let params: Params = serde_json::from_value(params)?;
        
        let mut cmd = AsyncCommand::new("cargo");
        cmd.arg("test");
        cmd.current_dir(&self.workspace);
        
        if params.release {
            cmd.arg("--release");
        }
        
        if let Some(features) = params.features {
            if !features.is_empty() {
                cmd.arg("--features");
                cmd.arg(features.join(","));
            }
        }
        
        if params.verbose {
            cmd.arg("--verbose");
        }
        
        if let Some(test_name) = params.test_name {
            cmd.arg(test_name);
        }
        
        let output = tokio::time::timeout(
            std::time::Duration::from_secs(300), // 5 minutes timeout
            cmd.output()
        ).await
        .map_err(|_| anyhow::anyhow!("Command timed out after 5 minutes"))??;
        
        Ok(json!({
            "success": output.status.success(),
            "exit_code": output.status.code(),
            "stdout": String::from_utf8_lossy(&output.stdout),
            "stderr": String::from_utf8_lossy(&output.stderr),
        }))
    }
}

/// Tool for running rustfmt
pub struct RustfmtTool;

impl RustfmtTool {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl Tool for RustfmtTool {
    fn name(&self) -> &str {
        "rustfmt"
    }
    
    fn description(&self) -> &str {
        "Format Rust code using rustfmt"
    }
    
    fn parameters_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "code": {
                    "type": "string",
                    "description": "Rust code to format"
                },
                "check": {
                    "type": "boolean",
                    "description": "Check if code is formatted without making changes",
                    "default": false
                }
            },
            "required": ["code"]
        })
    }
    
    async fn execute(&self, params: Value) -> Result<Value> {
        #[derive(Deserialize)]
        struct Params {
            code: String,
            #[serde(default)]
            check: bool,
        }
        
        let params: Params = serde_json::from_value(params)?;
        
        // Create a temporary file for the code
        let temp_file = tempfile::NamedTempFile::new()?;
        std::fs::write(temp_file.path(), &params.code)?;
        
        let mut cmd = AsyncCommand::new("rustfmt");
        if params.check {
            cmd.arg("--check");
        }
        cmd.arg(temp_file.path());
        
        let output = tokio::time::timeout(
            std::time::Duration::from_secs(300), // 5 minutes timeout
            cmd.output()
        ).await
        .map_err(|_| anyhow::anyhow!("Command timed out after 5 minutes"))??;
        
        let formatted_code = if !params.check && output.status.success() {
            std::fs::read_to_string(temp_file.path())?
        } else {
            params.code.clone()
        };
        
        Ok(json!({
            "success": output.status.success(),
            "exit_code": output.status.code(),
            "formatted_code": formatted_code,
            "changes_needed": !output.status.success() && params.check,
            "stderr": String::from_utf8_lossy(&output.stderr),
        }))
    }
}

/// Tool for running clippy
pub struct ClippyTool {
    workspace: PathBuf,
}

impl ClippyTool {
    pub fn new(workspace: PathBuf) -> Self {
        Self { workspace }
    }
}

#[async_trait]
impl Tool for ClippyTool {
    fn name(&self) -> &str {
        "clippy"
    }
    
    fn description(&self) -> &str {
        "Run clippy linter on the Rust code"
    }
    
    fn parameters_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "fix": {
                    "type": "boolean",
                    "description": "Automatically fix issues where possible",
                    "default": false
                },
                "all_targets": {
                    "type": "boolean",
                    "description": "Check all targets (lib, bin, tests, etc.)",
                    "default": true
                }
            }
        })
    }
    
    async fn execute(&self, params: Value) -> Result<Value> {
        #[derive(Deserialize)]
        struct Params {
            #[serde(default)]
            fix: bool,
            #[serde(default = "default_true")]
            all_targets: bool,
        }
        
        fn default_true() -> bool { true }
        
        let params: Params = serde_json::from_value(params)?;
        
        let mut cmd = AsyncCommand::new("cargo");
        cmd.arg("clippy");
        cmd.current_dir(&self.workspace);
        
        if params.all_targets {
            cmd.arg("--all-targets");
        }
        
        if params.fix {
            cmd.arg("--fix");
            cmd.arg("--allow-dirty");
        }
        
        cmd.arg("--");
        cmd.arg("-D");
        cmd.arg("warnings");
        
        let output = tokio::time::timeout(
            std::time::Duration::from_secs(300), // 5 minutes timeout
            cmd.output()
        ).await
        .map_err(|_| anyhow::anyhow!("Command timed out after 5 minutes"))??;
        
        Ok(json!({
            "success": output.status.success(),
            "exit_code": output.status.code(),
            "stdout": String::from_utf8_lossy(&output.stdout),
            "stderr": String::from_utf8_lossy(&output.stderr),
        }))
    }
}

/// Tool for running cargo check
pub struct CargoCheckTool {
    workspace: PathBuf,
}

impl CargoCheckTool {
    pub fn new(workspace: PathBuf) -> Self {
        Self { workspace }
    }
}

#[async_trait]
impl Tool for CargoCheckTool {
    fn name(&self) -> &str {
        "cargo_check"
    }
    
    fn description(&self) -> &str {
        "Check the Rust project for errors without building"
    }
    
    fn parameters_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "all_targets": {
                    "type": "boolean",
                    "description": "Check all targets (lib, bin, tests, etc.)",
                    "default": true
                },
                "features": {
                    "type": "array",
                    "items": {"type": "string"},
                    "description": "Features to enable"
                }
            }
        })
    }
    
    async fn execute(&self, params: Value) -> Result<Value> {
        #[derive(Deserialize)]
        struct Params {
            #[serde(default = "default_true")]
            all_targets: bool,
            features: Option<Vec<String>>,
        }
        
        fn default_true() -> bool { true }
        
        let params: Params = serde_json::from_value(params)?;
        
        let mut cmd = AsyncCommand::new("cargo");
        cmd.arg("check");
        cmd.current_dir(&self.workspace);
        
        if params.all_targets {
            cmd.arg("--all-targets");
        }
        
        if let Some(features) = params.features {
            if !features.is_empty() {
                cmd.arg("--features");
                cmd.arg(features.join(","));
            }
        }
        
        let output = tokio::time::timeout(
            std::time::Duration::from_secs(300), // 5 minutes timeout
            cmd.output()
        ).await
        .map_err(|_| anyhow::anyhow!("Command timed out after 5 minutes"))??;
        
        Ok(json!({
            "success": output.status.success(),
            "exit_code": output.status.code(),
            "stdout": String::from_utf8_lossy(&output.stdout),
            "stderr": String::from_utf8_lossy(&output.stderr),
        }))
    }
}