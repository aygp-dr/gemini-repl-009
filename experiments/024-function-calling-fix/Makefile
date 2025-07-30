# Makefile for function calling fix experiment

.PHONY: all run test before after compare clean

# Default target
all: before

# Run baseline test (before state)
before:
	@echo "=== Running BEFORE state baseline test ==="
	@./baseline_test.sh

# Run the fix (to be implemented)
fix:
	@echo "=== Applying function calling fix ==="
	@echo "Fix will be implemented here..."

# Run after state test
after: fix
	@echo "=== Running AFTER state test ==="
	@./after_test.sh

# Compare before and after
compare:
	@echo "=== Comparing results ==="
	@python3 compare_results.py

# Run a single test prompt
test:
	@echo "=== Quick function call test ==="
	@echo "summarize the Makefile in this repo" | cargo run --bin gemini-repl 2>&1 | grep -C5 -i "function\|tool\|read_file\|cannot access"

# Run with debug logging
debug:
	@echo "=== Running with debug logging ==="
	@RUST_LOG=debug DEBUG=true echo "summarize the Makefile" | cargo run --bin gemini-repl 2>&1

# Clean up test results
clean:
	@echo "=== Cleaning test results ==="
	@rm -rf before/ after/ *.log

# Help
help:
	@echo "Available targets:"
	@echo "  make before  - Run baseline test (current state)"
	@echo "  make fix     - Apply function calling fix"
	@echo "  make after   - Run test after fix"
	@echo "  make compare - Compare before/after results"
	@echo "  make test    - Quick single test"
	@echo "  make debug   - Run with debug logging"
	@echo "  make clean   - Clean test results"

# Default run target
run: test