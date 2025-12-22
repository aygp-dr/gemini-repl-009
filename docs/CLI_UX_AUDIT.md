# CLI UX Audit Report

**Bead:** gemini-repl-2mx
**Date:** 2025-12-22
**Status:** Complete

---

## Executive Summary

| Category | Score | Issues | Priority Fixes |
|----------|-------|--------|----------------|
| Command Structure | 7/10 | 5 | Subcommand grouping, aliases |
| Flag Conventions | 8/10 | 3 | Short flags, consistency |
| Output Formatting | 6/10 | 7 | Color, structure, machine-readable |
| Error Messages | 7/10 | 4 | Actionable hints, recovery |
| Help System | 6/10 | 6 | Examples, man page, completions |
| **Overall** | **6.8/10** | **25** | See P0 recommendations |

---

## 1. Command Structure Analysis

### Current State

```
gemini-repl [OPTIONS]

REPL Commands (/-prefixed):
  /help, /exit, /quit, /model, /clear, /context, /tools
  /reset, /stats, /tokens, /capabilities
  /save, /load, /delete, /continue, /export, /sessions
  /memory, /remember, /forget, /memory-clear
  /queue, /queue-submit
```

### Issues Identified

| ID | Issue | Severity | Recommendation |
|----|-------|----------|----------------|
| CS-1 | No command grouping/namespacing | Medium | Group as `/session save`, `/memory add` |
| CS-2 | Inconsistent verb usage | Low | Standardize: add/remove vs remember/forget |
| CS-3 | `/quit` and `/exit` duplicates | Low | Keep both as aliases (good for UX) |
| CS-4 | No command abbreviations | Medium | Add `/s` for `/save`, `/l` for `/load` |
| CS-5 | No tab completion for commands | High | Implement readline completions |

### Recommendations

**P0: Tab Completion**
```rust
// Add to rustyline config
let completer = CommandCompleter::new(vec![
    "/help", "/exit", "/save", "/load", ...
]);
rl.set_helper(Some(completer));
```

**P1: Command Aliases**
```
/s  -> /save
/l  -> /load
/q  -> /quit
/h  -> /help
/m  -> /memory
/?  -> /help
```

**P2: Subcommand Grouping (Future)**
```
/session save <name>
/session load <name>
/session list
/session delete <name>

/memory add <key> <value>
/memory list
/memory remove <key>
/memory clear
```

---

## 2. Flag Conventions Review

### Current Flags

| Flag | Short | Long | Env Var | Status |
|------|-------|------|---------|--------|
| API Key | `-a` | `--api-key` | `GEMINI_API_KEY` | OK |
| Model | `-m` | `--model` | - | Missing env |
| Provider | `-p` | `--provider` | `LLM_PROVIDER` | OK |
| Ollama URL | - | `--ollama-url` | `OLLAMA_HOST` | Missing short |
| Debug | `-d` | `--debug` | - | Missing env |
| Self-mod | - | `--enable-self-modification` | - | Too long |
| Continue | `-C` | `--continue` | - | OK |
| Continue Last | - | `--continue-last` | - | OK |

### Issues Identified

| ID | Issue | Severity | Recommendation |
|----|-------|----------|----------------|
| FC-1 | `--enable-self-modification` too verbose | Low | Add `-S` short flag |
| FC-2 | No `MODEL` env var support | Medium | Add `GEMINI_MODEL` env |
| FC-3 | No `--ollama-url` short flag | Low | Add `-o` short flag |
| FC-4 | Inconsistent env var naming | Low | Standardize to `GEMINI_*` prefix |

### Recommendations

**Updated Flag Structure:**
```
-a, --api-key          GEMINI_API_KEY
-m, --model            GEMINI_MODEL
-p, --provider         GEMINI_PROVIDER (or LLM_PROVIDER)
-o, --ollama-url       OLLAMA_HOST
-d, --debug            GEMINI_DEBUG
-S, --self-modify      GEMINI_SELF_MODIFY
-C, --continue         -
    --continue-last    -
-v, --verbose          GEMINI_VERBOSE (new)
-q, --quiet            - (new, suppress banners)
```

---

## 3. Output Formatting Review

### Current Output Patterns

```
# Welcome banner
Gemini REPL v0.9.0 - Type /help for commands, /exit to quit
Connected to OLLAMA (model: llama3.2:3b)

# Help output (unstructured plain text)
Available commands:
  /help          - Show this help message
  /exit          - Exit the REPL (/quit also works)
  ...

# Token stats (indented, no color)
Token Statistics:
  1,234 / 128,000 tokens (0.96%)

  By role:
    User:      456 tokens (5 messages)
    Assistant: 778 tokens (5 messages)

# Error output
Error: Tool execution error: invalid path
```

### Issues Identified

| ID | Issue | Severity | Recommendation |
|----|-------|----------|----------------|
| OF-1 | No color coding | Medium | Add ANSI colors for roles, errors |
| OF-2 | No `--json` output mode | High | Add for scripting/piping |
| OF-3 | Inconsistent indentation | Low | Standardize to 2-space |
| OF-4 | No progress indicators | Medium | Add spinners for API calls |
| OF-5 | No `--quiet` mode | Medium | Suppress banners for scripts |
| OF-6 | Tool output unformatted | Low | Syntax highlight code blocks |
| OF-7 | No timestamp on messages | Low | Optional `--timestamps` flag |

### Recommendations

**P0: JSON Output Mode**
```bash
gemini-repl --json  # All output as JSON lines

# Example output:
{"type":"welcome","version":"0.9.0","provider":"ollama","model":"llama3.2:3b"}
{"type":"response","content":"Hello!","tokens":12}
{"type":"error","message":"Rate limit exceeded","code":"RATE_LIMIT"}
```

**P1: Color Scheme**
```
User prompt:     White/Default
Assistant:       Cyan
System messages: Yellow
Errors:          Red
Tool calls:      Magenta
Success:         Green
Warnings:        Yellow
```

**P1: Progress Indicators**
```
Thinking... ⠋⠙⠹⠸⠼⠴⠦⠧⠇⠏
[Calling tool: read_file] ...
[Context: 45% full]
```

---

## 4. Error Messages Review

### Current Error Patterns

```rust
// Generic errors
eprintln!("Error: {e}");
eprintln!("Error loading session '{}': {}", session_name, e);

// Tool errors
eprintln!("Tool execution error: {e}");

// API errors
eprintln!("Invalid provider: {}", e);
eprintln!("Failed to create provider: {}", e);
```

### Issues Identified

| ID | Issue | Severity | Recommendation |
|----|-------|----------|----------------|
| EM-1 | No error codes | Medium | Add unique error codes |
| EM-2 | Missing recovery hints | High | Tell user how to fix |
| EM-3 | No distinction warning vs error | Medium | Add warning level |
| EM-4 | Stack traces hidden | Low | Add `--debug` to show |

### Recommendations

**P0: Actionable Error Messages**
```
# Before
Error: Failed to create provider: Connection refused

# After
Error [E1001]: Cannot connect to Ollama
  Provider: ollama
  URL: http://localhost:11434

  To fix:
  - Start Ollama: ollama serve
  - Or use Gemini: export GEMINI_API_KEY=...
  - Or specify URL: --ollama-url http://host:port

  Run with --debug for more details.
```

**Error Code Categories:**
```
E1xxx - Provider/Connection errors
E2xxx - Authentication errors
E3xxx - Tool execution errors
E4xxx - Session/Memory errors
E5xxx - Configuration errors
E9xxx - Internal errors
```

---

## 5. Help System Review

### Current Help

```
# --help (clap auto-generated)
A secure, performant REPL for AI conversations with self-modification capabilities

Usage: gemini-repl [OPTIONS]

Options:
  -a, --api-key <API_KEY>  ...

# /help (manual, in-REPL)
Available commands:
  /help          - Show this help message
  ...
```

### Issues Identified

| ID | Issue | Severity | Recommendation |
|----|-------|----------|----------------|
| HS-1 | No examples in --help | High | Add usage examples |
| HS-2 | No man page | High | Generate gemini-repl.1 |
| HS-3 | No per-command help | Medium | `/help save` for details |
| HS-4 | No shell completions | High | bash/zsh/fish scripts |
| HS-5 | No config file docs | Medium | Document config.yaml |
| HS-6 | /help too long | Medium | Paginate or categorize |

### Recommendations

**P0: --help Examples**
```
EXAMPLES:
    gemini-repl                    # Auto-detect provider
    gemini-repl -p ollama -m llama3.2
    gemini-repl -p gemini --api-key KEY
    gemini-repl --continue-last    # Resume last session
    gemini-repl -C my-session      # Resume specific session

ENVIRONMENT:
    GEMINI_API_KEY    API key for Gemini provider
    LLM_PROVIDER      Default provider (ollama, gemini, openai)
    OLLAMA_HOST       Ollama server URL

FILES:
    ~/.gemini-repl/config.yaml     Configuration file
    ~/.gemini-repl/sessions/       Saved sessions
    ~/.gemini-repl/memory/         Persistent memory
```

**P0: Per-Command Help**
```
> /help save
Usage: /save [name]

Save the current conversation to a session file.

Arguments:
  name    Optional session name (auto-generated if not provided)

Examples:
  /save                 # Auto-generate name like "session-2025-01-15"
  /save my-project      # Save as "my-project"

Related: /load, /sessions, /delete, /export
```

---

## 6. Comparison with Industry Standards

### Claude Code CLI

| Feature | Claude Code | gemini-repl | Gap |
|---------|------------|-------------|-----|
| JSON output | Yes | No | Missing |
| Color output | Yes | No | Missing |
| Shell completions | Yes | No | Missing |
| Man page | Yes | No | Missing |
| --quiet mode | Yes | No | Missing |
| Per-command help | Yes | No | Missing |
| Examples in help | Yes | No | Missing |

### Aider

| Feature | Aider | gemini-repl | Gap |
|---------|-------|-------------|-----|
| /help categories | Yes | No | Missing |
| --no-auto-commit | Yes | N/A | Different scope |
| Config file | Yes | Yes | OK |
| Voice input | Yes | No | Out of scope |

### Gemini CLI

| Feature | Gemini CLI | gemini-repl | Gap |
|---------|-----------|-------------|-----|
| Extensions | Yes | No | Future |
| Sandbox mode | Yes | Partial | Enhance |
| MCP support | Yes | No | Future |

---

## 7. Priority Recommendations

### P0 - Critical (Before v1.0)

1. **Shell Completions** (gemini-repl-rfk)
   - Implement bash, zsh, fish completion scripts
   - Use clap_complete crate

2. **JSON Output Mode**
   - Add `--json` flag for machine-readable output
   - Essential for scripting and integration

3. **Man Page** (gemini-repl-npv)
   - Generate using clap_mangen
   - Install to /usr/local/share/man/man1/

4. **Actionable Error Messages**
   - Add error codes and recovery hints
   - Include "how to fix" suggestions

### P1 - Important (v1.1)

5. **Color Output**
   - Use termcolor or colored crate
   - Respect NO_COLOR environment variable

6. **Per-Command Help**
   - `/help <command>` for detailed info
   - Include examples

7. **Command Aliases**
   - Common shortcuts like `/s`, `/l`, `/h`

8. **Progress Indicators**
   - Spinner during API calls
   - Show context usage warnings

### P2 - Nice to Have (v1.2+)

9. **Tab Completion in REPL**
   - Command names
   - Session names for /load
   - File paths for tools

10. **--quiet Mode**
    - Suppress welcome banner
    - For use in scripts

11. **Help Pagination**
    - Use pager for long output
    - Or categorize help sections

---

## 8. Implementation Checklist

```markdown
## Shell Completions
- [ ] Add clap_complete to Cargo.toml
- [ ] Generate bash completion script
- [ ] Generate zsh completion script
- [ ] Generate fish completion script
- [ ] Add install instructions to README

## Man Page
- [ ] Add clap_mangen to build.rs
- [ ] Generate gemini-repl.1
- [ ] Add to Makefile install target

## JSON Output
- [ ] Add --json flag to Args
- [ ] Create OutputFormat enum
- [ ] Wrap all println! in output helper
- [ ] Define JSON schema for messages

## Error Improvements
- [ ] Create error code enum
- [ ] Add context to error messages
- [ ] Add recovery suggestions
- [ ] Create error documentation

## Color Support
- [ ] Add termcolor/colored dependency
- [ ] Define color scheme constants
- [ ] Add --no-color flag
- [ ] Respect NO_COLOR env var
```

---

## Appendix: Current Command Reference

| Command | Arguments | Description |
|---------|-----------|-------------|
| `/help` | - | Show help |
| `/exit`, `/quit` | - | Exit REPL |
| `/model` | - | Show current model |
| `/clear` | - | Clear screen |
| `/context` | - | Show conversation |
| `/tools` | - | List tools |
| `/reset` | - | Clear conversation |
| `/stats` | - | Session statistics |
| `/tokens` | - | Token usage |
| `/save` | `[name]` | Save session |
| `/load` | `<name>` | Load session |
| `/delete` | `<name>` | Delete session |
| `/continue` | - | Load last session |
| `/sessions` | - | List sessions |
| `/export` | `[fmt] [file]` | Export session |
| `/memory` | - | List facts |
| `/remember` | `[--category] <key> <value>` | Add fact |
| `/forget` | `<key>` | Remove fact |
| `/memory-clear` | - | Clear all facts |
| `/queue` | - | Show queue status |
| `/queue-submit` | `<prompt>` | Submit request |
| `/capabilities` | - | Show self-mod features |

---

## Appendix: CLI Flag Reference

| Short | Long | Env Var | Description |
|-------|------|---------|-------------|
| `-a` | `--api-key` | `GEMINI_API_KEY` | API key |
| `-m` | `--model` | - | Model name |
| `-p` | `--provider` | `LLM_PROVIDER` | Provider |
| - | `--ollama-url` | `OLLAMA_HOST` | Ollama URL |
| `-d` | `--debug` | - | Debug mode |
| - | `--enable-self-modification` | - | Self-mod |
| `-C` | `--continue` | - | Continue session |
| - | `--continue-last` | - | Continue last |
| `-h` | `--help` | - | Help |
| `-V` | `--version` | - | Version |
