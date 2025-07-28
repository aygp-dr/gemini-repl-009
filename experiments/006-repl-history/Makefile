# REPL History Experiment

.PHONY: run build clean test-auto help

help:
	@echo "REPL History Experiment:"
	@echo "  make run      - Interactive history test"
	@echo "  make test-auto - Automated history test"
	@echo "  make build    - Build the experiment"
	@echo "  make clean    - Clean build artifacts"

build:
	@echo "Building REPL history experiment..."
	cargo build

run: build
	@echo "Starting interactive history test..."
	@echo "Enter some commands, use arrow keys to test history navigation"
	cargo run

test-auto: build
	@echo "Running automated history test..."
	@echo -e "test command 1\ntest command 2\nhistory\nexit" | cargo run
	@echo "✓ Automated test complete"

clean:
	@echo "Cleaning..."
	cargo clean
	@rm -f test_history.txt
	@echo "✓ Cleaned history file"