//! Command Parser Experiment

use anyhow::Result;

#[derive(Debug, PartialEq)]
pub enum Command {
    Help,
    Exit,
    Model,
    Clear,
    History,
    Set { key: String, value: String },
    Chat { message: String },
}

pub fn parse_command(input: &str) -> Result<Command> {
    let trimmed = input.trim();
    
    if trimmed.is_empty() {
        return Err(anyhow::anyhow!("Empty input"));
    }
    
    if !trimmed.starts_with('/') {
        // Regular chat message
        return Ok(Command::Chat {
            message: trimmed.to_string(),
        });
    }
    
    // Parse command (starts with /)
    let parts: Vec<&str> = trimmed[1..].split_whitespace().collect();
    
    if parts.is_empty() {
        return Err(anyhow::anyhow!("Empty command"));
    }
    
    match parts[0] {
        "help" | "h" => Ok(Command::Help),
        "exit" | "quit" | "q" => Ok(Command::Exit),
        "model" | "m" => Ok(Command::Model),
        "clear" | "cls" => Ok(Command::Clear),
        "history" | "hist" => Ok(Command::History),
        "set" => {
            if parts.len() != 3 {
                return Err(anyhow::anyhow!("Usage: /set <key> <value>"));
            }
            Ok(Command::Set {
                key: parts[1].to_string(),
                value: parts[2].to_string(),
            })
        }
        cmd => Err(anyhow::anyhow!("Unknown command: /{}", cmd)),
    }
}

fn test_commands() -> Result<()> {
    let test_cases = vec![
        // Chat messages
        ("Hello, how are you?", Command::Chat { message: "Hello, how are you?".to_string() }),
        ("What is 2 + 40?", Command::Chat { message: "What is 2 + 40?".to_string() }),
        
        // Commands
        ("/help", Command::Help),
        ("/h", Command::Help),
        ("/exit", Command::Exit),
        ("/quit", Command::Exit),
        ("/q", Command::Exit),
        ("/model", Command::Model),
        ("/m", Command::Model),
        ("/clear", Command::Clear),
        ("/cls", Command::Clear),
        ("/history", Command::History),
        ("/hist", Command::History),
        ("/set debug true", Command::Set { 
            key: "debug".to_string(), 
            value: "true".to_string() 
        }),
    ];
    
    println!("=== Command Parser Tests ===");
    
    let mut passed = 0;
    let mut failed = 0;
    
    for (input, expected) in test_cases {
        match parse_command(input) {
            Ok(parsed) => {
                if parsed == expected {
                    println!("✓ '{}' -> {:?}", input, parsed);
                    passed += 1;
                } else {
                    println!("✗ '{}' -> {:?}, expected {:?}", input, parsed, expected);
                    failed += 1;
                }
            }
            Err(e) => {
                println!("✗ '{}' -> Error: {}", input, e);
                failed += 1;
            }
        }
    }
    
    // Test error cases
    println!("\n--- Error Cases ---");
    let error_cases = vec![
        ("", "Empty input"),
        ("/", "Empty command"),
        ("/unknown", "Unknown command"),
        ("/set", "Usage: /set <key> <value>"),
        ("/set key", "Usage: /set <key> <value>"),
    ];
    
    for (input, _expected_error) in error_cases {
        match parse_command(input) {
            Ok(parsed) => {
                println!("✗ '{}' should have failed but got: {:?}", input, parsed);
                failed += 1;
            }
            Err(_) => {
                println!("✓ '{}' correctly failed", input);
                passed += 1;
            }
        }
    }
    
    println!("\n--- Test Results ---");
    println!("Passed: {}", passed);
    println!("Failed: {}", failed);
    
    if failed == 0 {
        println!("✓ All tests passed!");
    } else {
        println!("✗ {} tests failed", failed);
    }
    
    Ok(())
}

fn main() -> Result<()> {
    test_commands()?;
    
    println!("\n=== Interactive Test ===");
    println!("Enter commands to test the parser (type '/exit' to quit):");
    
    loop {
        use std::io::{self, Write};
        
        print!("parser-test> ");
        io::stdout().flush()?;
        
        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        
        match parse_command(&input) {
            Ok(Command::Exit) => {
                println!("Goodbye!");
                break;
            }
            Ok(cmd) => {
                println!("Parsed: {:?}", cmd);
            }
            Err(e) => {
                println!("Error: {}", e);
            }
        }
    }
    
    println!("\n=== Command Parser Test Complete ===");
    Ok(())
}