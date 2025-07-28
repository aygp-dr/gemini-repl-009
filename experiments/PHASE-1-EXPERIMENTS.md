# Phase 1 Experiments Plan

## Completed Experiments
- [x] **001-http-proxy-testing** - Verify proxy on port 3129 works
- [x] **002-github-actions** - Track CI/CD workflow status
- [x] **003-api-key-test** - Test GEMINI_API_KEY with curl
- [x] **004-rust-api-client** - Basic Rust API client (in progress)

## Remaining Phase 1 Experiments

### **005-config-loading**
**Purpose**: Test configuration loading from .env and CLI args
- Load all environment variables
- Validate configuration structure
- Test defaults and overrides
- **Command**: `gmake -C experiments/005-config-loading run`

### **006-repl-history**
**Purpose**: Implement persistent REPL history
- History file management
- Session persistence
- History search/navigation
- **Command**: `gmake -C experiments/006-repl-history run`

### **007-command-parser**
**Purpose**: Enhanced command parsing and validation
- Command prefix detection
- Argument parsing
- Help system
- **Command**: `gmake -C experiments/007-command-parser run`

### **008-error-handling**
**Purpose**: Comprehensive error handling and logging
- Structured error types
- User-friendly error messages
- Debug logging
- **Command**: `gmake -C experiments/008-error-handling run`

### **009-signal-handling**
**Purpose**: Advanced signal handling (Ctrl+C, Ctrl+D, etc.)
- Graceful shutdown
- Interrupt handling
- Terminal state management
- **Command**: `gmake -C experiments/009-signal-handling run`

### **010-api-integration**
**Purpose**: Full API integration with streaming
- Real API calls
- Streaming responses
- Rate limiting
- **Command**: `gmake -C experiments/010-api-integration run`

## Daily Progress Target
- Complete 2-3 experiments per day
- Each experiment should:
  1. Have clear purpose and success criteria
  2. Include Makefile with `run` target
  3. Document findings in README.md
  4. Commit incrementally with detailed git notes

## Success Metrics
- All experiments pass `gmake -C experiments/NNN run`
- API key validation works
- Basic REPL functionality complete
- Foundation ready for Phase 2 expansion