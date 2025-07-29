# Experiment 023: Function Calling

## Purpose
Explore and implement function calling capabilities for the Gemini REPL, allowing the LLM to invoke predefined functions and use their results.

## Status
✅ **Basic Implementation** - Function registry and local testing complete

## Implementation Details

### Function Registry
- Dynamic function registration with type-safe handlers
- JSON Schema parameter validation support
- Error handling and result formatting

### Example Functions
1. **calculator** - Basic arithmetic operations (add, subtract, multiply, divide)
2. **get_weather** - Mock weather data retrieval
3. **get_current_time** - Current time and date

### Architecture
```
User Query → Gemini API → Function Call → Execute → Result → Gemini API → Final Answer
```

## Running the Experiment

```bash
# From project root
gmake -C experiments/023-function-calling run

# Or directly
cd experiments/023-function-calling
cargo run
```

## Integration Plan

### Phase 1: API Integration
1. Update GenerateRequest to include function declarations
2. Parse function_call from API responses
3. Execute functions and return results
4. Handle continuation prompts

### Phase 2: REPL Integration
1. Add /functions command to list available functions
2. Allow dynamic function registration
3. Add function result caching
4. Implement function call history

### Phase 3: Advanced Features
1. Multi-step function calling
2. Parallel function execution
3. Function composition
4. Custom function definitions via config

## Example Usage (Future)

```
gemini> What's the weather like in San Francisco and what's 25 * 4?

[Function Call: get_weather({"location": "San Francisco"})]
[Function Result: {"temperature": 72, "conditions": "Partly cloudy", ...}]

[Function Call: calculator({"a": 25, "b": 4, "operation": "multiply"})]
[Function Result: {"result": 100}]

The weather in San Francisco is currently 72°F with partly cloudy conditions. 
And 25 multiplied by 4 equals 100.
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