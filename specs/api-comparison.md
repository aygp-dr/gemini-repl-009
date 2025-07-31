# API Comparison: Gemini vs Ollama

## Overview
Gemini and Ollama have different API specifications for content generation and tool calling.

## 1. Basic Generation

### Gemini API
```bash
# Endpoint: https://generativelanguage.googleapis.com/v1beta/models/{model}:generateContent
# Method: POST
# Auth: API key in URL

curl -X POST "https://generativelanguage.googleapis.com/v1beta/models/gemini-1.5-flash:generateContent?key=${API_KEY}" \
  -H "Content-Type: application/json" \
  -d '{
    "contents": [{
      "role": "user",
      "parts": [{"text": "Hello"}]
    }]
  }'
```

### Ollama API
```bash
# Endpoint: http://localhost:11434/api/generate
# Method: POST
# Auth: None (local)

curl -X POST "http://localhost:11434/api/generate" \
  -H "Content-Type: application/json" \
  -d '{
    "model": "llama2",
    "prompt": "Hello",
    "stream": false
  }'
```

## 2. Chat/Conversation Format

### Gemini API
```json
{
  "contents": [
    {"role": "user", "parts": [{"text": "What is 2+2?"}]},
    {"role": "model", "parts": [{"text": "4"}]},
    {"role": "user", "parts": [{"text": "Double it"}]}
  ]
}
```

### Ollama API (Chat endpoint)
```json
{
  "model": "llama2",
  "messages": [
    {"role": "user", "content": "What is 2+2?"},
    {"role": "assistant", "content": "4"},
    {"role": "user", "content": "Double it"}
  ],
  "stream": false
}
```

## 3. Tool/Function Calling

### Gemini API
```json
{
  "contents": [{"role": "user", "parts": [{"text": "List files"}]}],
  "tools": [{
    "function_declarations": [{
      "name": "list_files",
      "description": "List files in a directory",
      "parameters": {
        "type": "object",
        "properties": {
          "path": {"type": "string"}
        }
      }
    }]
  }]
}
```

### Ollama API (Tool calling - experimental)
```json
{
  "model": "mistral",
  "messages": [
    {"role": "user", "content": "List files"}
  ],
  "tools": [{
    "type": "function",
    "function": {
      "name": "list_files",
      "description": "List files in a directory",
      "parameters": {
        "type": "object",
        "properties": {
          "path": {"type": "string"}
        }
      }
    }
  }],
  "stream": false
}
```

## 4. Response Formats

### Gemini Response
```json
{
  "candidates": [{
    "content": {
      "parts": [{
        "text": "Response text"
      }],
      "role": "model"
    },
    "finishReason": "STOP"
  }],
  "usageMetadata": {
    "promptTokenCount": 10,
    "candidatesTokenCount": 20,
    "totalTokenCount": 30
  }
}
```

### Ollama Response
```json
{
  "model": "llama2",
  "created_at": "2024-01-01T00:00:00Z",
  "response": "Response text",
  "done": true,
  "context": [1, 2, 3],
  "total_duration": 1000000000,
  "prompt_eval_count": 10,
  "eval_count": 20
}
```

## 5. Key Differences

| Feature | Gemini | Ollama |
|---------|--------|---------|
| Endpoint structure | `/models/{model}:generateContent` | `/api/generate` or `/api/chat` |
| Authentication | API key in URL | None (local) |
| Message format | `contents` with `parts` | `messages` with `content` |
| Role names | `user`, `model` | `user`, `assistant`, `system` |
| Tool format | `tools.function_declarations` | `tools[].function` |
| Streaming | Via `streamGenerateContent` endpoint | `stream: true` parameter |
| Response format | `candidates[].content.parts` | `response` or `message.content` |

## 6. Abstraction Layer Design

To support both APIs, we need:

```rust
trait LLMProvider {
    async fn generate(&self, messages: Vec<Message>) -> Result<String>;
    async fn generate_with_tools(&self, messages: Vec<Message>, tools: Vec<Tool>) -> Result<Response>;
}

struct GeminiProvider { /* ... */ }
struct OllamaProvider { /* ... */ }
```