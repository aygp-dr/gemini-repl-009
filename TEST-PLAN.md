# Comprehensive Test Plan

## Current State Baseline (Phase 1)

### Completed
- ✅ Basic REPL functionality with Gemini API
- ✅ Context preservation across messages
- ✅ NOOP mode for testing
- ✅ API specification documentation
- ✅ Integration tests with expect

### Test Suites Available
1. **Unit Tests**: `cargo test`
2. **Integration Tests**: `gmake -C tests/integration test`
3. **Baseline Tests**: `bash run_baseline_tests.sh`
4. **API Tests**: `bash test_fixed_repl.sh`

## Tool Calling Implementation (Phase 2)

### Pre-Implementation Tests
1. **API Behavior Documentation**
   - `bash tests/evals/before_tools_eval.sh` - Baseline without tools
   - `bash tests/evals/after_tools_eval.sh` - Expected with tools
   - `bash tests/baseline/tool_calling_scenarios.sh` - Use cases

2. **Provider Compatibility**
   - `bash tests/ollama/test_ollama_api.sh` - Ollama API format
   - Compare request/response formats
   - Document abstraction requirements

### Implementation Milestones

#### Week 1: Core Infrastructure
- [ ] Implement Tool trait
- [ ] Create ToolRegistry
- [ ] Add tool serialization
- [ ] Unit tests for registry

#### Week 2: File System Tools
- [ ] list_files with sandboxing
- [ ] read_file with size limits
- [ ] write_file with backups
- [ ] Security tests

#### Week 3: API Integration
- [ ] Update Gemini client for tools
- [ ] Parse function calls
- [ ] Execute and respond
- [ ] Error handling

#### Week 4: Production Ready
- [ ] Rate limiting
- [ ] Circuit breaker
- [ ] Ollama fallback
- [ ] Performance tests

## Test Scenarios

### Scenario 1: Basic Tool Usage
```
User: List all Rust files
Expected: Model calls list_files(pattern="*.rs")
Response: Shows actual files
```

### Scenario 2: Multi-Step Workflow
```
User: Read main.rs and add a new function
Expected: 
1. Model calls read_file("src/main.rs")
2. Model calls write_file with updated content
```

### Scenario 3: Error Handling
```
User: Read /etc/passwd
Expected: Security error - outside sandbox
```

### Scenario 4: Rate Limiting
```
Rapid requests to trigger limits
Expected: Graceful backoff and retry
```

## Throttling Test Cases

### Rate Limit Scenarios
1. **Burst Protection**
   - Send 100 requests in 10 seconds
   - Verify queuing and delays
   - No dropped requests

2. **Token Limit**
   - Send large prompts
   - Track token usage
   - Warn at 80% capacity

3. **Daily Budget**
   - Simulate day's usage
   - Test cutoff behavior
   - Reset verification

### Error Recovery
1. **Network Failures**
   - Disconnect during request
   - Verify retry with backoff
   - Test max retry limit

2. **API Errors**
   - 429 Rate Limit
   - 503 Service Unavailable
   - 400 Bad Request

3. **Ollama Failover**
   - Gemini unavailable
   - Auto-switch to Ollama
   - Maintain conversation

## Performance Benchmarks

### Target Metrics
- Tool execution: < 100ms
- API response: < 2s (p95)
- Rate limit check: < 10ms
- File operations: < 50ms

### Load Tests
1. Concurrent users: 10
2. Requests per minute: 50
3. Token throughput: 500k/min
4. File operations: 1000/min

## Security Tests

### Sandboxing
- Path traversal attempts
- Absolute path rejection
- Symlink following
- Size limit enforcement

### Input Validation
- Malformed tool calls
- Injection attempts
- Binary file handling
- Unicode edge cases

## Offline Testing Strategy

### Ollama Setup
```bash
# Install Ollama
curl -fsSL https://ollama.ai/install.sh | sh

# Pull model
ollama pull llama2

# Start server
ollama serve

# Test REPL
PROVIDER=ollama cargo run
```

### Benefits
- No API limits
- Fast iteration
- Airplane mode
- Cost-free testing

## Continuous Testing

### Pre-commit
- cargo fmt
- cargo clippy
- cargo test

### CI Pipeline
- All unit tests
- Integration tests
- Security scans
- Performance benchmarks

### Nightly
- Full tool scenarios
- Rate limit tests
- Provider compatibility
- Load testing