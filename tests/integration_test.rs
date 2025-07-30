//! Integration tests for the Gemini REPL

use std::process::{Command, Stdio};
use std::io::Write;

#[test]
fn test_repl_starts_and_exits() {
    let mut child = Command::new("cargo")
        .args(["run", "--bin", "gemini-repl"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("Failed to spawn REPL");
    
    // Send exit command
    let stdin = child.stdin.as_mut().unwrap();
    writeln!(stdin, "/exit").unwrap();
    
    // Wait for process to exit
    let output = child.wait_with_output().unwrap();
    
    // Check it exited successfully
    assert!(output.status.success());
    
    // Check output contains welcome message
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Gemini REPL"));
    assert!(stdout.contains("Type /help for commands"));
}

#[test]
fn test_help_command() {
    let mut child = Command::new("cargo")
        .args(["run", "--bin", "gemini-repl"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("Failed to spawn REPL");
    
    let stdin = child.stdin.as_mut().unwrap();
    
    // Send help command
    writeln!(stdin, "/help").unwrap();
    writeln!(stdin, "/exit").unwrap();
    
    let output = child.wait_with_output().unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);
    
    // Check help output
    assert!(stdout.contains("Available commands:"));
    assert!(stdout.contains("/help"));
    assert!(stdout.contains("/exit"));
    assert!(stdout.contains("/model"));
    assert!(stdout.contains("/clear"));
    assert!(stdout.contains("/context"));
}

#[test]
fn test_model_command() {
    let mut child = Command::new("cargo")
        .args(["run", "--bin", "gemini-repl"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("Failed to spawn REPL");
    
    let stdin = child.stdin.as_mut().unwrap();
    
    // Send model command
    writeln!(stdin, "/model").unwrap();
    writeln!(stdin, "/exit").unwrap();
    
    let output = child.wait_with_output().unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);
    
    // Check model output
    assert!(stdout.contains("Current model: gemini-2.0-flash-exp"));
}

// Note: Testing actual API calls would require mocking or a test API key
// These tests focus on the REPL functionality itself