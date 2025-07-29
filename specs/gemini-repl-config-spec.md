# Gemini REPL Configuration Specification

## Overview

This document defines the formal specification for `.gemini-repl/config.toml` files. It can be used to validate configuration files programmatically or manually audit them for correctness.

## File Location

- **Primary**: `~/.gemini-repl/config.toml`
- **Alternative**: Specified via `--config` flag
- **Format**: TOML v1.0.0 (https://toml.io/en/v1.0.0)

## Schema Definition

### Top-Level Tables

```toml
[api]        # OPTIONAL - API configuration
[repl]       # OPTIONAL - REPL interface settings
[logging]    # OPTIONAL - Logging configuration
[tools]      # OPTIONAL - Tool system configuration
[session]    # OPTIONAL - Session management
[ui]         # OPTIONAL - User interface customization
[response]   # OPTIONAL - Response configuration
[network]    # OPTIONAL - Network configuration
[security]   # OPTIONAL - Privacy and security
[debug]      # OPTIONAL - Development/debug settings
[aliases]    # OPTIONAL - Command aliases
[models.*]   # OPTIONAL - Model-specific configurations
[prompts]    # OPTIONAL - Custom prompts
[features]   # OPTIONAL - Feature flags
```

### Detailed Field Specifications

#### `[api]` Table

| Field | Type | Required | Default | Valid Values | Description |
|-------|------|----------|---------|--------------|-------------|
| `api_key` | string | No | env:GEMINI_API_KEY | Any string | API key (NOT RECOMMENDED in file) |
| `model` | string | No | "gemini-1.5-flash" | See model list below | Default model |
| `base_url` | string | No | "https://generativelanguage.googleapis.com" | Valid URL | API endpoint |
| `timeout` | integer | No | 30 | 1-300 | Request timeout in seconds |
| `max_retries` | integer | No | 3 | 0-10 | Number of retry attempts |
| `retry_delay` | float | No | 1.0 | 0.1-60.0 | Initial retry delay in seconds |

**Valid Models:**
- `"gemini-1.5-flash"`
- `"gemini-1.5-pro"`
- `"gemini-2.0-flash-exp"`
- `"gemini-pro"`
- `"gemini-pro-vision"`

#### `[repl]` Table

| Field | Type | Required | Default | Valid Values | Description |
|-------|------|----------|---------|--------------|-------------|
| `prompt` | string | No | "> " | Any string | Input prompt |
| `colored_prompt` | boolean | No | true | true/false | Use ANSI colors |
| `history_file` | string | No | "~/.gemini-repl/history" | Valid path | History file location |
| `history_size` | integer | No | 1000 | 0-100000 | Max history entries |
| `welcome_banner` | boolean | No | true | true/false | Show startup banner |
| `auto_save_interval` | integer | No | 300 | 0-3600 | Auto-save interval (0=disabled) |
| `vi_mode` | boolean | No | false | true/false | Vi key bindings |
| `multiline_mode` | boolean | No | true | true/false | Support multi-line input |

#### `[logging]` Table

| Field | Type | Required | Default | Valid Values | Description |
|-------|------|----------|---------|--------------|-------------|
| `level` | string | No | "info" | "error", "warn", "info", "debug", "trace" | Log level |
| `file` | string | No | "~/.gemini-repl/logs/gemini.log" | Valid path | Log file path |
| `format` | string | No | "json" | "json", "pretty", "compact" | Log format |
| `max_file_size` | string | No | "10MB" | Size with unit (KB/MB/GB) | Max log file size |
| `max_files` | integer | No | 5 | 1-100 | Number of rotated files |
| `log_requests` | boolean | No | false | true/false | Log API requests |

#### `[tools]` Table

| Field | Type | Required | Default | Valid Values | Description |
|-------|------|----------|---------|--------------|-------------|
| `enabled` | boolean | No | true | true/false | Enable tools |
| `sandbox_dir` | string | No | "~/gemini-repl-workspace" | Valid path | Tool workspace |
| `confirm_writes` | boolean | No | false | true/false | Confirm file writes |
| `confirm_deletes` | boolean | No | true | true/false | Confirm file deletes |
| `max_file_size` | string | No | "1MB" | Size with unit | Max file size |
| `allowed_extensions` | array | No | See default list | Array of strings | Allowed file types |

##### `[tools.commands]` Sub-table

| Field | Type | Required | Default | Valid Values | Description |
|-------|------|----------|---------|--------------|-------------|
| `enabled` | boolean | No | false | true/false | Enable command execution |
| `allowed_commands` | array | No | [] | Array of strings | Whitelisted commands |
| `timeout` | integer | No | 10 | 1-300 | Command timeout in seconds |

## Validation Rules

### 1. Type Validation
```rust
// Example validation function
fn validate_config(config: &Config) -> Result<(), Vec<ValidationError>> {
    let mut errors = Vec::new();
    
    // Validate timeout is positive
    if config.api.timeout < 1 || config.api.timeout > 300 {
        errors.push(ValidationError::OutOfRange("api.timeout", 1, 300));
    }
    
    // Validate URL format
    if !is_valid_url(&config.api.base_url) {
        errors.push(ValidationError::InvalidFormat("api.base_url", "URL"));
    }
    
    // Validate file size format
    if !is_valid_size(&config.tools.max_file_size) {
        errors.push(ValidationError::InvalidFormat("tools.max_file_size", "size"));
    }
    
    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}
```

### 2. Path Validation
- Paths starting with `~` are expanded to home directory
- Relative paths are resolved from config file location
- Parent directories must be writable for log/session files

### 3. Size Format Validation
Valid size formats:
- `"100"` - bytes
- `"10KB"` or `"10K"` - kilobytes
- `"5MB"` or `"5M"` - megabytes  
- `"1GB"` or `"1G"` - gigabytes

### 4. Cross-Field Validation
- If `logging.log_requests` is true, `logging.file` must be set
- If `tools.enabled` is false, `tools.commands` settings are ignored
- If `session.auto_save` is true, `session.default_dir` must exist or be creatable

### 5. Security Validation
- `api_key` should NOT be present (use environment variable)
- `sandbox_dir` must not be a system directory
- `allowed_commands` must not contain shell operators (`;`, `|`, `>`, etc.)

## Audit Checklist

### Manual Audit Steps

1. **Structure Audit**
   - [ ] File is valid TOML syntax
   - [ ] No unknown top-level tables
   - [ ] No unknown fields in tables

2. **Type Audit**
   - [ ] All boolean fields are `true` or `false`
   - [ ] All integer fields are whole numbers
   - [ ] All arrays use consistent types
   - [ ] Strings are properly quoted

3. **Value Range Audit**
   - [ ] Timeouts are between 1-300 seconds
   - [ ] Port numbers (if any) are 1-65535
   - [ ] Percentages are 0-100
   - [ ] File sizes are reasonable

4. **Security Audit**
   - [ ] No API keys in config file
   - [ ] Sandbox directory is safe
   - [ ] No dangerous commands whitelisted
   - [ ] File permissions are restrictive

5. **Path Audit**
   - [ ] All paths are valid
   - [ ] Required directories exist or can be created
   - [ ] No path traversal attempts
   - [ ] Appropriate permissions

## Programmatic Validation

### Using JSON Schema (converted from TOML)

```json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "type": "object",
  "properties": {
    "api": {
      "type": "object",
      "properties": {
        "model": {
          "type": "string",
          "enum": ["gemini-1.5-flash", "gemini-1.5-pro", "gemini-2.0-flash-exp", "gemini-pro", "gemini-pro-vision"]
        },
        "timeout": {
          "type": "integer",
          "minimum": 1,
          "maximum": 300
        }
      }
    }
  }
}
```

### Using Rust Validation

```rust
use validator::{Validate, ValidationError};

#[derive(Debug, Validate, Deserialize)]
struct ApiConfig {
    #[validate(url)]
    base_url: String,
    
    #[validate(range(min = 1, max = 300))]
    timeout: u32,
    
    #[validate(range(min = 0, max = 10))]
    max_retries: u32,
}
```

## Example Valid Configuration

```toml
# Minimal valid configuration
[api]
model = "gemini-1.5-flash"

[repl]
prompt = "🤖 > "

[tools]
enabled = true
sandbox_dir = "~/workspace"
```

## Common Validation Errors

1. **Invalid TOML Syntax**
   ```toml
   # ERROR: Missing quotes
   prompt = > 
   ```

2. **Type Mismatch**
   ```toml
   # ERROR: timeout should be integer
   timeout = "30 seconds"
   ```

3. **Out of Range**
   ```toml
   # ERROR: timeout too high
   timeout = 500
   ```

4. **Invalid Enum Value**
   ```toml
   # ERROR: Unknown model
   model = "gpt-4"
   ```

5. **Security Violation**
   ```toml
   # ERROR: Dangerous command
   allowed_commands = ["rm -rf"]
   ```

## Version Compatibility

- **v0.1.x**: Initial specification
- **v0.2.x**: Added `[features]` table
- **v0.3.x**: Added `[models.*]` configurations

Configuration files are forward-compatible: older versions ignore unknown fields.
