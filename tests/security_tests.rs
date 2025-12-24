//! Security-critical tests for file operations and path validation
//!
//! These tests ensure that the security boundaries are properly enforced.

use gemini_repl::tools::security::{is_path_safe, validate_path};
use std::path::Path;
use tempfile::TempDir;

// ============================================================================
// Path Traversal Attack Prevention Tests
// ============================================================================

#[test]
fn test_rejects_basic_path_traversal() {
    assert!(!is_path_safe(Path::new("../file.txt")));
    assert!(!is_path_safe(Path::new("../../etc/passwd")));
    assert!(!is_path_safe(Path::new("foo/../../../etc/shadow")));
}

#[test]
fn test_deep_nested_traversal() {
    // Deep nesting with path traversal should still be caught
    assert!(!is_path_safe(Path::new("a/b/c/d/e/../../../../..")));
    assert!(!is_path_safe(Path::new(
        "very/deep/path/../../../../../../../etc"
    )));
    // Note: On Unix, backslashes are regular filename characters, not separators
    // So "..\\..\\windows" is just a weird filename, not traversal
}

#[test]
fn test_rejects_absolute_paths() {
    assert!(!is_path_safe(Path::new("/etc/passwd")));
    assert!(!is_path_safe(Path::new("/home/user/.ssh/id_rsa")));
    assert!(!is_path_safe(Path::new("/var/log/auth.log")));
}

#[test]
fn test_rejects_home_directory_expansion() {
    assert!(!is_path_safe(Path::new("~/.ssh/id_rsa")));
    assert!(!is_path_safe(Path::new("~/Desktop/secrets.txt")));
    assert!(!is_path_safe(Path::new("~root/.bashrc")));
}

// ============================================================================
// Sensitive File Protection Tests
// ============================================================================

#[test]
fn test_rejects_env_files() {
    assert!(!is_path_safe(Path::new(".env")));
    assert!(!is_path_safe(Path::new(".env.local")));
    assert!(!is_path_safe(Path::new(".env.production")));
    assert!(!is_path_safe(Path::new("config/.env.development")));
}

#[test]
fn test_rejects_git_directory() {
    // The .git directory and any paths containing .git are blocked
    assert!(!is_path_safe(Path::new(".git")));
    assert!(!is_path_safe(Path::new(".git/config")));
    assert!(!is_path_safe(Path::new(".git/hooks/pre-commit")));
    assert!(!is_path_safe(Path::new("submodule/.git")));
}

#[test]
fn test_rejects_credential_files() {
    // Sensitive directories
    assert!(!is_path_safe(Path::new(".ssh")));
    assert!(!is_path_safe(Path::new(".aws")));
    assert!(!is_path_safe(Path::new(".gnupg")));
    assert!(!is_path_safe(Path::new(".kube")));
    assert!(!is_path_safe(Path::new(".docker")));

    // Files inside sensitive directories
    assert!(!is_path_safe(Path::new(".ssh/id_rsa")));
    assert!(!is_path_safe(Path::new(".aws/credentials")));
    assert!(!is_path_safe(Path::new(".kube/config")));

    // Credential files by pattern
    assert!(!is_path_safe(Path::new("credentials.json")));
    assert!(!is_path_safe(Path::new("db_password.txt")));
    assert!(!is_path_safe(Path::new("api_secret.key")));
    assert!(!is_path_safe(Path::new(".npmrc")));
    assert!(!is_path_safe(Path::new(".netrc")));

    // Private key files
    assert!(!is_path_safe(Path::new("server.pem")));
    assert!(!is_path_safe(Path::new("private.key")));
    assert!(!is_path_safe(Path::new("id_rsa")));
    assert!(!is_path_safe(Path::new("id_ed25519")));
}

#[test]
fn test_case_insensitive_sensitive_files() {
    // Ensure case-insensitive matching for sensitive patterns
    assert!(!is_path_safe(Path::new(".ENV")));
    assert!(!is_path_safe(Path::new("SECRET.txt")));
    assert!(!is_path_safe(Path::new("Password.json")));
    assert!(!is_path_safe(Path::new("CREDENTIALS.yaml")));
}

// ============================================================================
// Safe Path Acceptance Tests
// ============================================================================

#[test]
fn test_allows_normal_source_files() {
    assert!(is_path_safe(Path::new("src/main.rs")));
    assert!(is_path_safe(Path::new("src/lib.rs")));
    assert!(is_path_safe(Path::new("tests/integration_test.rs")));
}

#[test]
fn test_allows_project_config_files() {
    assert!(is_path_safe(Path::new("Cargo.toml")));
    assert!(is_path_safe(Path::new("Cargo.lock")));
    assert!(is_path_safe(Path::new("README.md")));
    assert!(is_path_safe(Path::new("Makefile")));
}

#[test]
fn test_allows_nested_paths() {
    assert!(is_path_safe(Path::new("src/tools/file_tools.rs")));
    assert!(is_path_safe(Path::new("docs/api/reference.md")));
    assert!(is_path_safe(Path::new("tests/fixtures/sample.json")));
}

// ============================================================================
// Workspace Boundary Validation Tests
// ============================================================================

#[test]
fn test_validate_path_rejects_escape() {
    let workspace = TempDir::new().unwrap();
    let workspace_path = workspace.path();

    // Create a path that escapes the workspace
    let escaped = workspace_path.join("../../etc/passwd");
    let result = validate_path(&escaped, workspace_path);

    assert!(result.is_err());
    let err_msg = result.unwrap_err().to_string();
    assert!(err_msg.contains("escape") || err_msg.contains("Invalid"));
}

#[test]
fn test_validate_path_allows_safe_paths() {
    let workspace = TempDir::new().unwrap();
    let workspace_path = workspace.path();

    // Create a test file within workspace
    let test_file = workspace_path.join("test.txt");
    std::fs::write(&test_file, "test content").unwrap();

    let result = validate_path(&test_file, workspace_path);
    assert!(result.is_ok());
}

// ============================================================================
// Edge Cases
// ============================================================================

#[test]
fn test_rejects_null_bytes() {
    // Null bytes could potentially truncate paths in C-based systems
    assert!(!is_path_safe(Path::new("file\0.txt")));
}

#[test]
fn test_handles_empty_path() {
    // Empty path should be safe (current directory)
    let result = is_path_safe(Path::new(""));
    // Empty path is technically safe since it's just current directory
    assert!(result);
}

#[test]
fn test_handles_dot_path() {
    // Single dot is current directory, should be safe
    assert!(is_path_safe(Path::new(".")));
}
