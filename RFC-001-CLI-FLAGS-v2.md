# RFC-001 (Revised): LLM Backend Configuration

**Status**: Draft v2  
**Created**: 2025-07-28  
**Author**: jwalsh  

## Summary

Hybrid approach using CLI flags for quick overrides and config files for complex setups, with sensible defaults for common use cases.

## Motivation

After deeper analysis of our phased approach:
- Phase 1: Basic REPL with single provider (current)
- Phase 2: Tool calling implementation
- Phase 3: Multi-provider support
- Phase 4: Advanced features (caching, routing)

We need a configuration system that grows with these phases.

## Proposed Design

### CLI Interface

```bash
# Default: uses .gemini-repl.toml or environment
gemini-repl

# Quick override with --backend flag
gemini-repl --backend gemini    # Production Gemini API
gemini-repl --backend ollama    # Local Ollama
gemini-repl --backend mock      # Mock from recordings
gemini-repl --backend noop      # No-op for testing

# Short form
gemini-repl -b ollama

# With config file
gemini-repl --config dev.toml
gemini-repl -c prod.toml

# Override specific settings
gemini-repl --backend ollama --model llama2 --debug
```

### Configuration Files

#### Default: `.gemini-repl.toml`
```toml
# Default configuration
[repl]
default_backend = "gemini"
prompt = "> "
history_file = ".gemini_history"

[backends.gemini]
api_key_env = "GEMINI_API_KEY"
model = "gemini-1.5-flash"
api_url = "https://generativelanguage.googleapis.com/v1beta"
timeout_ms = 30000
max_retries = 3

[backends.ollama]
url = "http://localhost:11434"
model = "llama2"
timeout_ms = 60000

[backends.mock]
data_dir = "tests/fixtures"
delay_ms = 100

[backends.noop]
response_template = "[NOOP] Received: {input}"

[logging]
debug = false
log_requests = false
log_dir = "logs"
log_format = "json"  # or "pretty"

[proxy]
auto_detect = true
http_proxy = "${HTTP_PROXY}"
https_proxy = "${HTTPS_PROXY}"
no_proxy = "localhost,127.0.0.1"
```

#### Development: `dev.toml`
```toml
[repl]
default_backend = "ollama"  # Use local model for development

[backends.ollama]
model = "codellama"  # Better for code tasks

[logging]
debug = true
log_requests = true  # Capture all requests for mock generation

[tools]
enabled = true
sandbox_dir = "./sandbox"
confirm_writes = true  # Ask before file writes
```

#### Testing: `test.toml`
```toml
[repl]
default_backend = "mock"

[backends.mock]
data_dir = "tests/recordings"
strict_mode = true  # Fail if no recording found

[logging]
debug = false  # Keep test output clean
```

#### Production: `prod.toml`
```toml
[repl]
default_backend = "gemini"

[backends.gemini]
model = "gemini-2.0-flash-exp"
max_retries = 5

[rate_limiting]
requests_per_minute = 60
tokens_per_minute = 1000000

[logging]
debug = false
log_requests = false  # Privacy
```

### Environment Variables

```bash
# .env or .env.local (git-ignored)
GEMINI_API_KEY=xxx
DEBUG=true
LOG_REQUESTS=true

# Override config file
GEMINI_REPL_CONFIG=dev.toml

# Override specific settings
GEMINI_REPL_BACKEND=ollama
GEMINI_REPL_MODEL=mixtral
```

### Priority Order

1. CLI flags (highest priority)
2. Environment variables
3. Config file (--config or .gemini-repl.toml)
4. Built-in defaults

### Implementation Phases

#### Phase 1: Basic Backend Selection (Current Sprint)
```rust
#[derive(Parser)]
struct Args {
    /// LLM backend to use
    #[arg(short, long, env = "GEMINI_REPL_BACKEND")]
    backend: Option<Backend>,
    
    /// Configuration file
    #[arg(short, long, env = "GEMINI_REPL_CONFIG")]
    config: Option<PathBuf>,
    
    /// Enable debug mode
    #[arg(short, long, env = "DEBUG")]
    debug: bool,
}

#[derive(Debug, Clone, ValueEnum)]
enum Backend {
    Gemini,
    Ollama,
    Mock,
    Noop,
}
```

#### Phase 2: Config File Support
```rust
#[derive(Debug, Deserialize)]
struct Config {
    repl: ReplConfig,
    backends: HashMap<String, BackendConfig>,
    logging: LoggingConfig,
    proxy: Option<ProxyConfig>,
}

impl Config {
    fn load(path: Option<&Path>) -> Result<Self> {
        let path = path
            .or_else(|| Path::new(".gemini-repl.toml").exists().then(|| Path::new(".gemini-repl.toml")))
            .ok_or_else(|| anyhow!("No config file found"))?;
        
        let contents = fs::read_to_string(path)?;
        let mut config: Config = toml::from_str(&contents)?;
        
        // Expand environment variables
        config.expand_env_vars();
        
        Ok(config)
    }
}
```

#### Phase 3: Request/Response Logging
```rust
if config.logging.log_requests {
    let entry = RequestLog {
        timestamp: Utc::now(),
        backend: backend_name,
        request: serde_json::to_value(&request)?,
        // ...
    };
    
    // Write to logs/{backend}/{date}/requests.jsonl
    logger.log_request(entry)?;
}
```

## Config Examples

### 1. Airplane Mode Development
```toml
# airplane.toml
[repl]
default_backend = "ollama"

[backends.ollama]
url = "http://localhost:11434"
model = "llama2"
offline_mode = true  # Don't check for model updates
```

### 2. Cost-Conscious Testing
```toml
# budget.toml
[repl]
default_backend = "mock"
fallback_backend = "ollama"  # If no mock available

[backends.mock]
data_dir = "tests/fixtures"
record_missing = true  # Record new interactions to Ollama
```

### 3. High-Performance Production
```toml
# performance.toml
[repl]
default_backend = "gemini"

[backends.gemini]
model = "gemini-2.0-flash-exp"
concurrent_requests = 5

[caching]
enabled = true
ttl_seconds = 300
max_entries = 1000
```

### 4. Multi-Provider with Routing
```toml
# advanced.toml (Phase 4)
[routing]
strategy = "cost_optimized"  # or "latency_optimized", "round_robin"

[[routing.rules]]
pattern = "code.*"
backend = "ollama"  # Use CodeLlama for code questions

[[routing.rules]]
pattern = ".*tool_call.*"
backend = "gemini"  # Gemini for tool calling

[backends.gemini]
weight = 0.7
model = "gemini-1.5-flash"

[backends.ollama]
weight = 0.3
model = "mixtral"
```

## Migration Path

1. **v0.1**: Support `--backend` flag only
2. **v0.2**: Add `.gemini-repl.toml` support
3. **v0.3**: Full config file with examples
4. **v0.4**: Multi-backend routing

## Testing

```bash
# Test different configs
gemini-repl --config dev.toml --dry-run
gemini-repl --config test.toml --validate

# Generate default config
gemini-repl --generate-config > .gemini-repl.toml

# Show effective config (after all merging)
gemini-repl --show-config
```