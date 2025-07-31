//! Utility functions and helpers

use std::path::{Path, PathBuf};

/// Get the project workspace root
pub fn get_workspace_root() -> PathBuf {
    std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."))
}

/// Check if a path is within the workspace
pub fn is_within_workspace(path: &Path, workspace: &Path) -> bool {
    if let (Ok(canonical_path), Ok(canonical_workspace)) = (path.canonicalize(), workspace.canonicalize()) {
        canonical_path.starts_with(canonical_workspace)
    } else {
        false
    }
}

/// Sanitize a file name for safe usage
pub fn sanitize_filename(name: &str) -> String {
    name.chars()
        .map(|c| match c {
            '/' | '\\' | ':' | '*' | '?' | '"' | '<' | '>' | '|' => '_',
            c => c,
        })
        .collect()
}