# Development Guidelines

## Testing Requirements

### 🔴 IMPORTANT: Run Tests Before Every Commit

After making changes to source code, **always** run:

```bash
# Unit tests
cargo test

# Integration tests (NOOP mode)
gmake test-integration

# Or run all tests
gmake test-all
```

### Quick Test Commands

```bash
# Before committing
gmake pre-commit

# This runs:
# 1. cargo fmt --check
# 2. cargo clippy -- -D warnings  
# 3. cargo test
# 4. gmake -C tests/integration test
```

## Development Workflow

### 1. Feature Development

```bash
# Create feature branch
git checkout -b feature/your-feature

# Make changes
vim src/main.rs

# Run tests immediately
cargo test
gmake test-integration

# If tests pass, commit
git add -p
git commit -m "feat: your feature"
```

### 2. Bug Fixes

```bash
# Reproduce the bug with a test
echo 'Write a failing test first!'

# Fix the bug
vim src/api.rs

# Verify fix
cargo test
gmake test-integration

# Commit with test
git add -p
git commit -m "fix: description

Added test to prevent regression"
```

### 3. Refactoring

```bash
# Ensure tests pass before refactoring
gmake test-all

# Refactor
vim src/module.rs

# Verify nothing broke
gmake test-all

# Commit
git commit -m "refactor: description"
```

## Test Types

### Unit Tests
- Located in `src/*.rs` as `#[cfg(test)]` modules
- Test individual functions and modules
- Run with: `cargo test`
- Should be fast (<1s total)

### Integration Tests  
- Located in `tests/integration/`
- Use expect scripts for REPL interaction
- Run in NOOP mode by default
- Run with: `gmake -C tests/integration test`

### API Tests
- Located in `tests/evals/`
- Test actual API behavior (requires API key)
- Run with: `bash tests/evals/before_tools_eval.sh`

### Experiments
- Located in `experiments/`
- Isolated proof-of-concepts
- Run individually: `gmake -C experiments/011-debug-logging run`

## Continuous Integration

### Pre-commit Hook

Install the git hook:

```bash
cat > .git/hooks/pre-commit << 'EOF'
#!/bin/sh
echo "Running pre-commit tests..."
gmake pre-commit || {
    echo "🔴 Tests failed! Commit aborted."
    echo "Fix the issues or use --no-verify to skip (not recommended)"
    exit 1
}
EOF

chmod +x .git/hooks/pre-commit
```

### GitHub Actions (Future)

```yaml
name: CI
on: [push, pull_request]

jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - run: cargo test
      - run: make test-integration
```

## Debug Mode Testing

When debugging issues:

```bash
# Enable debug logs
DEBUG=true cargo run

# Check logs
tail -f logs/gemini-repl.log

# Run with request logging
LOG_REQUESTS=true cargo run

# Check API logs
find logs -name "*.jsonl" -mtime 0 | xargs tail -f
```

## Performance Testing

```bash
# Run benchmarks
cargo bench

# Profile with flamegraph
cargo flamegraph

# Check binary size
cargo bloat
```

## Common Issues

### Tests Failing on FreeBSD

```bash
# Use gmake instead of make
gmake test-all

# Ensure bash is available for scripts
pkg install bash
```

### Slow Tests

```bash
# Run only unit tests (fast)
cargo test --lib

# Skip integration tests
cargo test --bins
```

### Flaky Tests

```bash
# Run test multiple times
for i in {1..10}; do cargo test test_name || break; done

# Increase timeouts for expect scripts
export EXPECT_TIMEOUT=10
```

## Test Coverage

```bash
# Install tarpaulin
cargo install cargo-tarpaulin

# Generate coverage report
cargo tarpaulin --out Html

# Open report
open tarpaulin-report.html
```

## Remember

1. **No commit without tests** - Every bug fix needs a test
2. **Test first** - Write failing test before implementing
3. **Keep tests fast** - Mock external dependencies
4. **Test the edges** - Error cases, empty inputs, large inputs
5. **Document why** - Comments in tests explain the scenario

🎆 Happy testing!