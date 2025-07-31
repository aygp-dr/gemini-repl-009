# Function Calling Hypothesis

## Problem Statement
The Gemini model is not invoking function calls despite being provided with tool definitions. Instead, it responds with text saying it cannot access files.

## Hypothesis

### Primary Hypothesis
The response parsing in `send_message_with_tools` is not correctly handling function call responses from the API. The model might be returning function calls in a different format than expected.

### Supporting Evidence
1. From experiment 023 results: 0% function call success rate with gemini-1.5-flash
2. Model responds "I cannot access local files" despite having tools available
3. The API response handling only extracts text from the first part (`parts[0].text`)

### Possible Root Causes
1. **Response Format Issue**: The API returns function calls in a different structure (e.g., `parts[0].functionCall` instead of `parts[0].text`)
2. **Model Selection**: gemini-2.0-flash-lite might not support function calling properly
3. **Missing Configuration**: Additional parameters might be needed in the request (e.g., `toolConfig`)
4. **System Prompt**: The model might need explicit instructions about available tools

## Experiment Design

### Test 1: Response Structure Analysis
Log the full API response to understand the actual structure when tools are provided.

### Test 2: Model Comparison
Test with different models:
- gemini-2.0-flash-exp
- gemini-1.5-pro
- gemini-2.0-flash-lite (current)

### Test 3: Enhanced Tool Description
Add more explicit instructions in tool descriptions about when to use them.

### Test 4: Tool Config
Add `toolConfig` parameter to force function calling mode.