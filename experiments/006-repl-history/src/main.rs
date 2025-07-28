//! REPL History Experiment

use anyhow::Result;
use rustyline::DefaultEditor;
use rustyline::error::ReadlineError;
use std::path::Path;

fn main() -> Result<()> {
    println!("=== REPL History Test ===");
    
    let history_file = "test_history.txt";
    
    // Initialize editor
    let mut rl = DefaultEditor::new()?;
    
    // Try to load existing history
    if Path::new(history_file).exists() {
        if rl.load_history(history_file).is_ok() {
            println!("✓ Loaded existing history from {}", history_file);
        } else {
            println!("✗ Failed to load history");
        }
    } else {
        println!("• No existing history file found");
    }
    
    println!("Enter some commands (type 'exit' to quit):");
    println!("Try using arrow keys to navigate history");
    
    let mut command_count = 0;
    
    loop {
        match rl.readline("history-test> ") {
            Ok(line) => {
                command_count += 1;
                
                // Add to history
                let _ = rl.add_history_entry(line.as_str());
                
                let trimmed = line.trim();
                
                if trimmed == "exit" {
                    break;
                } else if trimmed == "history" {
                    // Show history
                    println!("Command history:");
                    for (i, entry) in rl.history().iter().enumerate() {
                        println!("  {}: {}", i + 1, entry);
                    }
                } else if trimmed.is_empty() {
                    continue;
                } else {
                    println!("Entered: {}", trimmed);
                }
            }
            Err(ReadlineError::Interrupted) => {
                println!("^C - Use 'exit' to quit");
                continue;
            }
            Err(ReadlineError::Eof) => {
                println!("^D");
                break;
            }
            Err(err) => {
                println!("Error: {:?}", err);
                break;
            }
        }
    }
    
    // Save history
    if rl.save_history(history_file).is_ok() {
        println!("✓ History saved to {}", history_file);
        println!("✓ Total commands entered: {}", command_count);
    } else {
        println!("✗ Failed to save history");
    }
    
    // Test history file size
    if let Ok(metadata) = std::fs::metadata(history_file) {
        println!("✓ History file size: {} bytes", metadata.len());
    }
    
    println!("\n=== History Test Complete ===");
    Ok(())
}