# Experiment 023: Function Calling

## Purpose
Implement Gemini API function calling capabilities for the REPL, enabling tool use for file operations and code analysis.

## Status
🚧 **Work in Progress** - API format implemented, testing function call triggering

## Implementation Details

### Core File Tools (Phase 1)
Based on gemini-repl-007 CODEBASE_TOOL_DECLARATIONS:
1. **read_file** - Read file contents from filesystem
2. **write_file** - Write content to files  
3. **list_files** - List files matching glob patterns
4. **search_code** - Search for patterns in codebase

### Architecture
```
User Query → Gemini API → Function Call Detection → Tool Execution → Response
                ↑                                            ↓
                └────────── Function Results ←───────────────┘
```

## Running the Experiment

```bash
# Run the main demo
make run

# Run Makefile dependency test
make makefile-test

# Run full test suite
make test
```

## Test Suite
- **40+ test cases** across 6 categories
- Validates function call triggering vs text generation
- Key test: "What are the target dependencies of Makefile?" → read_file("Makefile")

## Current Challenge
Gemini API responds with "I don't have filesystem access" instead of using provided tools. Need to investigate:
1. API version compatibility
2. Model capabilities (gemini-1.5-flash vs gemini-1.5-pro)
3. Request format variations

## Example Usage

```
gemini> Show me the contents of README.md

[Function Call: read_file({"file_path": "README.md"})]
[Function Result: "# Gemini REPL 009\n\nA Rust implementation..."]

Here are the contents of README.md:
# Gemini REPL 009
A Rust implementation...
```

## API Format (Gemini)

```json
{
  "contents": [...],
  "tools": [{
    "function_declarations": [
      {
        "name": "calculator",
        "description": "Perform basic arithmetic operations",
        "parameters": {
          "type": "object",
          "properties": {
            "a": {"type": "number"},
            "b": {"type": "number"},
            "operation": {"type": "string", "enum": ["add", "subtract", "multiply", "divide"]}
          },
          "required": ["a", "b", "operation"]
        }
      }
    ]
  }]
}
```

## Next Steps
1. Research Gemini API function calling format
2. Implement API request/response handling
3. Create function execution pipeline
4. Add to main REPL with feature flag