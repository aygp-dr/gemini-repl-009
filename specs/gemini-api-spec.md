# Gemini API Specification

## Message Format

### Single Request (No Conversation History)
```json
{
  "contents": [{
    "parts": [{
      "text": "Your prompt here"
    }]
  }]
}
```

### Multi-turn Conversation
```json
{
  "contents": [
    {
      "role": "user",
      "parts": [{
        "text": "First user message"
      }]
    },
    {
      "role": "model",
      "parts": [{
        "text": "First model response"
      }]
    },
    {
      "role": "user",
      "parts": [{
        "text": "Second user message"
      }]
    }
  ]
}
```

## Key Requirements

1. **Role Field**: Required for multi-turn conversations
   - Valid values: `"user"`, `"model"` 
   - NOT `"assistant"` (common mistake from OpenAI API)

2. **Content Structure**: Each content must have:
   - `role`: String ("user" or "model")
   - `parts`: Array of part objects
   - Each part has `text`: String

3. **API Endpoint**: 
   ```
   https://generativelanguage.googleapis.com/v1beta/models/{MODEL}:generateContent?key={API_KEY}
   ```

## Common Errors

- **"Please use a valid role: user, model."**: Missing or invalid role field
- **400 Bad Request**: Malformed JSON or missing required fields
- **429 Resource Exhausted**: Rate limit exceeded

## Implementation Notes

- System messages are not directly supported; prepend to first user message
- Conversation history should alternate between user and model roles
- Empty messages are not allowed