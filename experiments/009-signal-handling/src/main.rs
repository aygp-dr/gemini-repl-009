//! Signal Handling Experiment

use anyhow::Result;
use tokio::signal;
use crossterm::{
    event::{self, Event, KeyCode, KeyModifiers},
    terminal::{self, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};
use std::io::{self, stdout};

#[tokio::main]
async fn main() -> Result<()> {
    println!("=== Signal Handling Test ===");
    println!("Press Ctrl+C to test interrupt handling");
    println!("Press 'q' to quit");
    
    // Enable raw mode for better control
    terminal::enable_raw_mode()?;
    stdout().execute(EnterAlternateScreen)?;
    
    let result = run_signal_test().await;
    
    // Cleanup
    stdout().execute(LeaveAlternateScreen)?;
    terminal::disable_raw_mode()?;
    
    result
}

async fn run_signal_test() -> Result<()> {
    let mut interrupt_count = 0;
    
    loop {
        tokio::select! {
            // Handle Ctrl+C
            _ = signal::ctrl_c() => {
                interrupt_count += 1;
                println!("\r\n✓ Ctrl+C received ({} times)", interrupt_count);
                
                if interrupt_count >= 3 {
                    println!("Three interrupts received, exiting...");
                    break;
                }
                
                println!("Press Ctrl+C again or 'q' to quit");
            }
            
            // Handle keyboard events
            Ok(event) = event::read_async() => {
                match event {
                    Event::Key(key_event) => {
                        match key_event.code {
                            KeyCode::Char('q') => {
                                println!("\r\n✓ Quit key pressed");
                                break;
                            }
                            KeyCode::Char('c') if key_event.modifiers.contains(KeyModifiers::CONTROL) => {
                                // This is handled by the signal handler above
                            }
                            KeyCode::Char(c) => {
                                println!("\r\nKey pressed: '{}'", c);
                            }
                            _ => {}
                        }
                    }
                    _ => {}
                }
            }
        }
    }
    
    println!("✓ Signal handling test complete");
    Ok(())
}