# gemini-repl

## The Multi-Provider Terminal AI Agent

**Your AI coding assistant. Your choice of brain.**

---

### Why gemini-repl?

| | Claude Code | Cursor | Aider | gemini-repl |
|---|:---:|:---:|:---:|:---:|
| Local inference (Ollama) | - | - | Yes | **Yes** |
| Gemini support | - | - | Yes | **Yes** |
| OpenAI support | - | Yes | Yes | **Yes** |
| Anthropic support | Yes | Yes | Yes | Planned |
| No subscription required | - | - | Yes | **Yes** |
| Self-modification | - | - | - | **Yes** |
| Open source | - | - | Yes | **Yes** |

---

### Key Features

#### Multi-Provider Freedom
```bash
# Use local Ollama for privacy
gemini-repl -p ollama -m llama3.2

# Switch to Gemini for power
gemini-repl -p gemini -m gemini-2.0-flash-exp

# Or OpenAI when needed
gemini-repl -p openai -m gpt-4
```

#### Project-Aware Sessions
```bash
# Continue where you left off
cd my-project
gemini-repl -c

# Your conversation history follows your project
```

#### Scriptable Interface
```bash
# Single-shot execution
gemini-repl -e "explain this error" --json

# Pipe-friendly
cat error.log | gemini-repl --stdin --json
```

#### AI-Powered Tool Use
- File reading and writing
- Shell command execution
- ed-style text editing
- Path-safe security controls

---

### Quick Start

```bash
# Install
cargo install gemini-repl

# With Ollama (free, local)
ollama pull llama3.2
gemini-repl -p ollama

# With Gemini (requires API key)
export GEMINI_API_KEY="your-key"
gemini-repl -p gemini
```

---

### For Privacy-First Teams

gemini-repl with Ollama means:
- **No data leaves your machine**
- **No API costs**
- **No rate limits**
- **Works offline**

```bash
# All processing stays local
gemini-repl -p ollama -m codellama
```

---

### For Power Users

```bash
# Continue last session
gemini-repl --continue-last

# Project-specific history
gemini-repl -c  # Auto-loads project context

# JSON output for scripting
gemini-repl -e "list todos" --json | jq '.content'
```

---

### REPL Commands

```
/help     - Show all commands
/context  - View conversation history
/tokens   - Token usage statistics
/save     - Save session
/load     - Load session
/memory   - View persistent facts
/tools    - List available tools
```

---

### Architecture

```
┌─────────────────────────────────────────────┐
│              gemini-repl                    │
├─────────────────────────────────────────────┤
│  ┌─────────┐  ┌─────────┐  ┌─────────┐     │
│  │ Session │  │ Context │  │ Memory  │     │
│  │ Manager │  │ Manager │  │ System  │     │
│  └────┬────┘  └────┬────┘  └────┬────┘     │
│       └────────────┼───────────┘           │
│                    ▼                        │
│  ┌─────────────────────────────────────┐   │
│  │         Provider Abstraction         │   │
│  └─────────────────────────────────────┘   │
│       │              │              │       │
│       ▼              ▼              ▼       │
│  ┌─────────┐   ┌─────────┐   ┌─────────┐   │
│  │ Ollama  │   │ Gemini  │   │ OpenAI  │   │
│  └─────────┘   └─────────┘   └─────────┘   │
└─────────────────────────────────────────────┘
```

---

### Comparison

#### vs. Claude Code
- **gemini-repl**: Multi-provider, open source, local-first
- **Claude Code**: Claude-only, polished, enterprise-ready

#### vs. Aider
- **gemini-repl**: Rust, session management, self-modification
- **Aider**: Python, git-centric, mature ecosystem

#### vs. Cursor
- **gemini-repl**: Terminal-native, lightweight, scriptable
- **Cursor**: Full IDE, visual, resource-heavy

---

### Roadmap

- [x] Multi-provider support (Gemini, Ollama, OpenAI)
- [x] Session persistence
- [x] Project-based conversation history
- [x] JSON output mode
- [x] Tool calling (read, write, shell, edit)
- [ ] Shell completions
- [ ] Git integration tools
- [ ] Streaming responses
- [ ] MCP server support
- [ ] Plugin architecture

---

### Contributing

```bash
git clone https://github.com/aygp-dr/gemini-repl-009
cd gemini-repl-009
cargo build
cargo test
```

See [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines.

---

### License

MIT License - Use it, modify it, ship it.

---

<p align="center">
  <strong>gemini-repl</strong><br>
  AI assistance on your terms.
</p>
