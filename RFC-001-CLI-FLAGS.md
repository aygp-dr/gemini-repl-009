# RFC-001: CLI Flags for Provider Modes

**Status**: Draft  
**Created**: 2025-07-28  
**Author**: jwalsh  

## Summary

Introduce CLI flags to control the REPL's backend provider mode, enabling different operational contexts from development to production.

## Motivation

We need to support multiple operational modes:
1. **noop**: For integration testing without external dependencies
2. **mock**: For replaying captured requests/responses
3. **prd**: For production use with live APIs
4. **ollama**: For offline development (future)

## Design

### CLI Interface

```bash
# Default: production mode
gemini-repl

# Explicit modes
gemini-repl --mode noop      # No external calls
gemini-repl --mode mock      # Use recorded responses
gemini-repl --mode prd       # Production (default)
gemini-repl --mode ollama    # Local Ollama server

# Short form
gemini-repl -m noop

# With other flags
gemini-repl --mode mock --debug --api-key xxx
```

### Environment Variables

```bash
# .env or .env.local
REPL_MODE=noop|mock|prd|ollama
DEBUG=true|false
LOG_REQUESTS=true|false
LOG_DIR=logs/

# Mode-specific configs
MOCK_DATA_DIR=tests/fixtures/
OLLAMA_URL=http://localhost:11434
```

### Mode Behaviors

#### 1. NOOP Mode
```rust
// Returns canned responses
struct NoopProvider {
    responses: HashMap<String, String>,
}

impl LLMProvider for NoopProvider {
    async fn generate(&self, messages: Vec<Message>) -> Result<ProviderResponse> {
        Ok(ProviderResponse {
            text: Some(format!("[NOOP] Received: {}", messages.last()?.content)),
            function_call: None,
            usage: Some(Usage { prompt_tokens: 10, completion_tokens: 20, total_tokens: 30 }),
        })
    }
}
```

#### 2. MOCK Mode
```rust
// Replays recorded interactions
struct MockProvider {
    recordings: Vec<ApiLogEntry>,
    current_index: AtomicUsize,
}

impl MockProvider {
    fn load_from_jsonl(path: &Path) -> Result<Self> {
        // Load reqs.jsonl and resps.jsonl
    }
}
```

#### 3. PRD Mode
```rust
// Live API calls with full logging
struct ProductionProvider {
    client: GeminiClient,
    logger: Option<ApiLogger>,
}
```

### Implementation Plan

#### Phase 1: CLI Argument Parsing
```rust
#[derive(Parser, Debug)]
struct Args {
    /// Operation mode
    #[arg(short, long, value_enum, default_value = "prd")]
    mode: OperationMode,
    
    /// Enable debug logging
    #[arg(short, long, env = "DEBUG")]
    debug: bool,
    
    /// Log API requests/responses
    #[arg(long, env = "LOG_REQUESTS")]
    log_requests: bool,
}

#[derive(Debug, Clone, ValueEnum)]
enum OperationMode {
    Noop,
    Mock,
    Prd,
    Ollama,
}
```

#### Phase 2: Provider Factory
```rust
pub fn create_provider(args: &Args) -> Result<Box<dyn LLMProvider>> {
    let logger = if args.log_requests || args.debug {
        Some(ApiLogger::new("logs", true)?)
    } else {
        None
    };
    
    match args.mode {
        OperationMode::Noop => Ok(Box::new(NoopProvider::new())),
        OperationMode::Mock => Ok(Box::new(MockProvider::load()?)),
        OperationMode::Prd => Ok(Box::new(ProductionProvider::new(logger)?)),
        OperationMode::Ollama => Ok(Box::new(OllamaProvider::new()?)),
    }
}
```

### Proxy Support

```rust
// Auto-detect proxy
fn detect_proxy() -> Option<String> {
    // Check if port 3129 is listening
    if std::net::TcpStream::connect("127.0.0.1:3129").is_ok() {
        Some("http://localhost:3129".to_string())
    } else {
        std::env::var("HTTPS_PROXY").ok()
    }
}
```

## Testing Strategy

### Unit Tests
```rust
#[cfg(test)]
mod tests {
    #[test]
    fn test_mode_parsing() {
        let args = Args::parse_from(&["gemini-repl", "--mode", "noop"]);
        assert_eq!(args.mode, OperationMode::Noop);
    }
}
```

### Integration Tests
```bash
# Test each mode
gemini-repl --mode noop < test_input.txt
gemini-repl --mode mock --mock-data tests/fixtures
gemini-repl --mode prd --api-key $TEST_KEY
```

## Security Considerations

1. **API Keys**: Never log in debug mode
2. **PII**: Sanitize logged requests/responses
3. **File Paths**: Validate mock data paths
4. **Proxy**: Verify proxy certificates

## Future Extensions

1. **Multiple Providers**: `--provider gemini,ollama` for fallback
2. **Recording Mode**: `--record` to capture new interactions
3. **Replay Speed**: `--mock-delay 100ms` for realistic timing
4. **Provider Weights**: `--weights gemini:0.8,ollama:0.2`

## Migration Path

1. Current: `NOOP_MODE=true` environment variable
2. Phase 1: Support both old and new methods
3. Phase 2: Deprecate `NOOP_MODE` in favor of `--mode`
4. Phase 3: Remove old environment variable

## Alternatives Considered

1. **Separate Binaries**: `gemini-repl-mock`, `gemini-repl-noop`
   - Rejected: Too many binaries to maintain

2. **Config Files**: `gemini-repl --config mock.toml`
   - Rejected: CLI flags are more explicit

3. **Subcommands**: `gemini-repl mock`, `gemini-repl test`
   - Rejected: Breaks existing usage patterns