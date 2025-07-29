# Alloy Demo: Formal Specification of Gemini REPL

This experiment demonstrates using Alloy to formally specify the conversation model for our Gemini REPL.

## What is Alloy?

Alloy is a declarative specification language for expressing complex structural constraints and behavior. It's particularly useful for:
- Modeling system architectures
- Verifying invariants
- Finding counterexamples to assumptions

## The Demo Specification

Our `demo.als` models:
1. **Conversation Structure**: Messages with roles (User/Model/System)
2. **Context Windows**: Bounded message history
3. **Tool Calling**: Function calls and responses
4. **Invariants**: Rules like message alternation

## Running the Demo

```bash
# Download Alloy (only needed once)
make download

# Run Alloy GUI
make run

# In the GUI:
# 1. Press Ctrl+E to execute
# 2. Use arrow keys to browse instances
# 3. Try Theme -> Magic Layout for better graphs
```

## Key Insights

The formal model helps us:
- Verify that conversations alternate between user and model
- Ensure tool responses always reference a call
- Visualize edge cases like context overflow
- Prove properties about our conversation model

## Example Visualizations

Alloy generates instance diagrams showing:
- Valid conversation sequences
- Tool calling patterns  
- Context window boundaries

## Integration with REPL

This specification can guide our implementation:
```rust
// Matches Alloy's Message sig
struct Message {
    role: Role,
    content: String,
    timestamp: i64,
}

// Matches conversation rules
fn validate_conversation(messages: &[Message]) -> bool {
    // Check alternation, timestamps, etc.
}
```