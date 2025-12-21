# Agent Instructions

This project uses **bd** (beads) for issue tracking and follows **Experiment-Driven Development (EDD)** with **Progressive Commit Protocol (PCP)**.

## Quick Reference

```bash
# Issue tracking (bd/beads)
bd ready              # Find available work
bd show <id>          # View issue details
bd update <id> --status in_progress  # Claim work
bd close <id>         # Complete work
bd sync               # Sync with git
bd create "title" -p 1 -t feature  # Create issue

# Repository management (ghq)
ghq list | grep gemini-repl     # Find repo
cd $(ghq root)/github.com/aygp-dr/gemini-repl-009

# GitHub CLI (gh)
gh pr create          # Create pull request
gh issue list         # List GitHub issues
gh repo view --web    # Open in browser
```

## Experiment-Driven Development (EDD)

This project uses EDD to validate ideas before committing to full implementation.

### Experiment Structure

```
experiments/
├── 000-test-dependencies/     # Completed experiments
│   └── README.org
├── 001-streaming-api/         # New experiment
│   ├── README.org             # Purpose, hypothesis, results
│   ├── Cargo.toml             # Minimal dependencies
│   └── src/main.rs            # Isolated test code
```

### EDD Workflow

1. **Identify** - Create a bd issue for the experiment
   ```bash
   bd create "Experiment: streaming API responses" -t experiment -p 2
   ```

2. **Isolate** - Create experiment directory
   ```bash
   mkdir -p experiments/NNN-descriptive-name
   ```

3. **Document** - Write README.org with:
   - Purpose and hypothesis
   - Success criteria
   - Dependencies to test
   - Expected vs actual results

4. **Implement** - Write minimal code to test hypothesis

5. **Record** - Use git notes for experiment metadata
   ```bash
   git notes add -m "experiment: streaming-api, status: success, findings: ..."
   ```

6. **Integrate or Archive** - Based on results:
   - Success: Create implementation issue, reference experiment
   - Failure: Document learnings, close experiment issue

### Git Notes for Experiments

```bash
# Add experiment note to current commit
git notes add -m "experiment: NNN-name
status: success|failure|inconclusive
findings: Brief summary of what was learned
next-steps: What to do with this knowledge"

# View notes
git notes show HEAD
git log --show-notes
```

## Progressive Commit Protocol (PCP)

PCP ensures incremental, reviewable progress with clear commit boundaries.

### Commit Rhythm

1. **Atomic Commits** - Each commit does ONE thing
   - Single feature addition
   - Single bug fix
   - Single refactor
   - Never mix concerns

2. **Commit Frequently** - After each logical step:
   ```bash
   # Good: Many small commits
   git commit -m "feat(api): add streaming endpoint struct"
   git commit -m "feat(api): implement stream parsing"
   git commit -m "test(api): add streaming unit tests"

   # Bad: One giant commit
   git commit -m "add streaming support"  # Too vague, too large
   ```

3. **Use Conventional Commits**
   ```
   type(scope): description

   Types: feat, fix, docs, style, refactor, test, chore, experiment
   Scope: api, repl, tools, config, etc.
   ```

4. **Trailer Attribution**
   ```bash
   git commit -m "feat(api): add response streaming" \
     --trailer "Co-Authored-By: Claude Opus 4.5 <noreply@anthropic.com>"
   ```

### PCP Checkpoints

After each significant change:

```bash
# 1. Verify build
gmake build

# 2. Run tests
gmake test

# 3. Check lints
gmake clippy

# 4. Commit with clear message
git add -p  # Stage interactively
git commit  # Use conventional commit format

# 5. Update bd issue status
bd update <id> --status in_progress
bd comment <id> "Completed: streaming struct, Next: parsing"
```

### Integration with bd

Track PCP progress in issues:

```bash
# Create sub-tasks for complex work
bd create "Implement streaming struct" -p 1
bd create "Add stream parsing" -p 1
bd create "Write streaming tests" -p 1

# Link dependencies
bd dep add <parsing-id> <struct-id>
bd dep add <tests-id> <parsing-id>

# Close as you complete
bd close <struct-id> --reason "Implemented in abc1234"
```

## Landing the Plane (Session Completion)

**When ending a work session**, complete ALL steps. Work is NOT complete until `git push` succeeds.

### Mandatory Workflow

1. **File issues for remaining work**
   ```bash
   bd create "TODO: remaining item" -p 2 -t chore
   ```

2. **Run quality gates** (if code changed)
   ```bash
   gmake pre-commit  # fmt, clippy, test
   ```

3. **Update issue status**
   ```bash
   bd close <completed-id>
   bd update <partial-id> --status in_progress
   bd comment <partial-id> "Session end: completed X, remaining Y"
   ```

4. **Sync and push** (MANDATORY)
   ```bash
   git pull --rebase
   bd sync
   git push
   git status  # MUST show "up to date with origin"
   ```

5. **Verify**
   ```bash
   bd list --status open  # Review open work
   git log -3 --oneline   # Confirm commits pushed
   ```

6. **Hand off** - Provide session summary:
   - What was completed
   - What's in progress
   - Blockers or questions
   - Recommended next steps

### Critical Rules

- Work is NOT complete until `git push` succeeds
- NEVER stop before pushing - that leaves work stranded locally
- NEVER say "ready to push when you are" - YOU must push
- If push fails, resolve and retry until it succeeds
- Always leave the repo in a clean, buildable state

## Project-Specific Commands

```bash
# Build (use gmake on FreeBSD)
gmake build          # Debug build
gmake build-release  # Release build
gmake test           # Run tests
gmake clippy         # Linter
gmake fmt            # Format code
gmake pre-commit     # All checks

# Run
gmake run            # Run with defaults
gmake run-noop       # No API calls
gmake run-debug      # Debug logging

# Development
gmake dev            # Debug + self-modification
```

## Repository Navigation (ghq)

```bash
# Find this repo
ghq list | grep gemini-repl-009
cd $(ghq root)/github.com/aygp-dr/gemini-repl-009

# Clone related repos
ghq get github.com/steveyegge/beads

# Create worktrees for parallel work
git worktree add ../gemini-repl-009-feature-x -b feature-x
```
