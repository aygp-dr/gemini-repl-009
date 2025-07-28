//! Gemini REPL library

pub mod api;
pub mod logging;

// Re-export public API
pub use api::{GeminiClient, Content, Part};

pub fn add(a: i32, b: i32) -> i32 {
    a + b
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add() {
        assert_eq!(add(2, 40), 42);
    }
    
    #[test]
    fn test_add_zero() {
        assert_eq!(add(42, 0), 42);
    }
}