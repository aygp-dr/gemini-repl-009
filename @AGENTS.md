# Agent Quick Reference

See [AGENTS.md](./AGENTS.md) for full documentation.

## Essential Commands

```bash
# Find work
bd ready

# Claim and work
bd update <id> --status in_progress
# ... do work ...
bd close <id>

# Session end (MANDATORY)
gmake pre-commit
git pull --rebase && bd sync && git push
```

## Key Principles

1. **bd tracks all work** - Every task gets an issue
2. **PCP commits** - Small, atomic, conventional commits
3. **EDD experiments** - Test ideas in `experiments/` first
4. **Push before stopping** - Work isn't done until pushed
