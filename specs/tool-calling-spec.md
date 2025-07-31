# Tool Calling Specification

## Overview
Implement Gemini API tool calling (function calling) to enable the model to interact with the filesystem.

## Phase 1: Core Tool Infrastructure

### 1.1 Tool Registry
```rust
// src/tools/mod.rs
pub trait Tool {
    fn name(&self) -> &str;
    fn description(&self) -> &str;
    fn parameters_schema(&self) -> serde_json::Value;
    async fn execute(&self, params: serde_json::Value) -> Result<String>;
}

pub struct ToolRegistry {
    tools: HashMap<String, Box<dyn Tool>>,
}
```

### 1.2 Tool Declaration Format
```rust
#[derive(Serialize, Deserialize)]
pub struct ToolDeclaration {
    name: String,
    description: String,
    parameters: ParameterSchema,
}

#[derive(Serialize, Deserialize)]
pub struct ParameterSchema {
    #[serde(rename = "type")]
    param_type: String,
    properties: HashMap<String, Property>,
    required: Vec<String>,
}
```

## Phase 2: File System Tools

### 2.1 List Files Tool
- **Name**: `list_files`
- **Parameters**: 
  - `path`: Optional directory path (default: ".")
  - `pattern`: Optional glob pattern (e.g., "*.rs")
- **Returns**: JSON array of file paths
- **Security**: Sandbox to project directory

### 2.2 Read File Tool  
- **Name**: `read_file`
- **Parameters**:
  - `path`: Required file path
- **Returns**: File contents as string
- **Security**: 
  - Sandbox to project directory
  - Size limit (1MB default)
  - Binary file detection

### 2.3 Write File Tool
- **Name**: `write_file`
- **Parameters**:
  - `path`: Required file path
  - `content`: Required content
  - `mode`: Optional ("overwrite" | "append")
- **Returns**: Success confirmation
- **Security**:
  - Sandbox to project directory
  - Backup before overwrite
  - Atomic writes

## Phase 3: API Integration

### 3.1 Request Format
```json
{
  "contents": [
    {
      "role": "user",
      "parts": [{"text": "List all Rust files"}]
    }
  ],
  "tools": [{
    "function_declarations": [
      /* tool declarations */
    ]
  }]
}
```

### 3.2 Response Format
```json
{
  "candidates": [{
    "content": {
      "parts": [{
        "functionCall": {
          "name": "list_files",
          "args": {
            "pattern": "*.rs"
          }
        }
      }]
    }
  }]
}
```

### 3.3 Function Response
```json
{
  "contents": [
    /* previous messages */
    {
      "role": "model",
      "parts": [{
        "functionCall": {
          "name": "list_files",
          "args": {"pattern": "*.rs"}
        }
      }]
    },
    {
      "role": "function",
      "parts": [{
        "functionResponse": {
          "name": "list_files",
          "response": {
            "files": ["src/main.rs", "src/api.rs", "src/lib.rs"]
          }
        }
      }]
    }
  ]
}
```

## Phase 4: Implementation Plan

### Week 1: Core Infrastructure
- [ ] Create tools module structure
- [ ] Implement Tool trait and ToolRegistry
- [ ] Add tool declaration serialization
- [ ] Unit tests for tool registry

### Week 2: File System Tools
- [ ] Implement list_files tool
- [ ] Implement read_file tool with safety checks
- [ ] Implement write_file tool with backups
- [ ] Integration tests for each tool

### Week 3: API Integration
- [ ] Update API client to include tool declarations
- [ ] Parse function calls from API responses
- [ ] Execute tools and send responses
- [ ] Handle tool calling conversation flow

### Week 4: Polish and Testing
- [ ] Error handling and recovery
- [ ] Performance optimization
- [ ] Security audit
- [ ] End-to-end tests

## Security Considerations

1. **Sandboxing**: All file operations restricted to project directory
2. **Path Traversal**: Reject paths with ".." or absolute paths
3. **Size Limits**: Enforce reasonable file size limits
4. **Rate Limiting**: Prevent excessive file operations
5. **Audit Logging**: Log all tool executions
6. **Confirmation**: Optional user confirmation for writes

## Testing Strategy

1. **Unit Tests**: Each tool in isolation
2. **Integration Tests**: Tool registry and execution
3. **API Tests**: Mock Gemini API responses
4. **E2E Tests**: Full conversation flows
5. **Security Tests**: Path traversal, size limits
6. **Performance Tests**: Large file handling

## Success Metrics

- All baseline tool calling scenarios pass
- No security vulnerabilities in file access
- Tool execution < 100ms for typical operations
- Clear error messages for all failure modes
- 90%+ test coverage for tools module