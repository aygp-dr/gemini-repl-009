# Gemini REPL 009 - Claude Code Instructions

## Project Overview

Rust implementation of Gemini REPL (version 009), focusing on security, performance, and self-modification capabilities.

## Workflow Requirements

### Issue Tracking with bd (beads)

ALL work must be tracked in bd:

```bash
bd ready                    # Find available work
bd update <id> --status in_progress  # Start work
bd close <id>               # Complete work
bd create "title" -t type   # New issue
```

### Progressive Commit Protocol (PCP)

- Make atomic commits (one logical change each)
- Use conventional commits: `type(scope): description`
- Commit frequently, not in large batches
- Always use trailer for attribution:
  ```bash
  --trailer "Co-Authored-By: Claude Opus 4.5 <noreply@anthropic.com>"
  ```

### Experiment-Driven Development (EDD)

Before implementing complex features:
1. Create experiment in `experiments/NNN-name/`
2. Document hypothesis in README.org
3. Test minimal implementation
4. Record results with git notes
5. Then implement in main codebase

## Build Commands

Use `gmake` (not `make`) on FreeBSD:

```bash
gmake build       # Build
gmake test        # Test
gmake clippy      # Lint
gmake pre-commit  # All checks
```

## Commit Guidelines

- Use conventional commits
- Use `--trailer` for co-author attribution
- Do NOT use 'generated with' in commit messages
- Repositories and gists should be private by default

## Session Completion

Before ending any session:

1. Create bd issues for remaining work
2. Run `gmake pre-commit`
3. Update bd issue statuses
4. **MUST push**: `git pull --rebase && bd sync && git push`
5. Verify with `git status`

Work is NOT complete until pushed to remote.

## Project Structure

```
src/
├── main.rs           # CLI entry, REPL loop
├── api.rs            # Gemini API client
├── tools/            # Tool implementations
├── errors.rs         # Error types
└── logging.rs        # Structured logging

experiments/          # EDD experiment directories
.beads/              # bd issue database
```

## Current Phase

Project is in Phase 1-2 transition. See `bd ready` for available work.
