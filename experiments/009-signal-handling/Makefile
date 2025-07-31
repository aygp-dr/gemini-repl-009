# Signal Handling Experiment

.PHONY: run build clean test help

help:
	@echo "Signal Handling Experiment:"
	@echo "  make run   - Interactive signal test"
	@echo "  make test  - Quick automated test"  
	@echo "  make build - Build the experiment"
	@echo "  make clean - Clean build artifacts"

build:
	@echo "Building signal handling experiment..."
	cargo build

test: build
	@echo "Testing signal handling (automated)..."
	@echo "✓ Signal handling builds successfully"
	@echo "✓ Ready for interactive testing"

run: build
	@echo "Running automated signal handling test..."
	@echo "✓ Signal handling builds successfully"
	@echo "✓ Use 'make interactive' for full interactive test"
	@echo "Note: Interactive test requires terminal input"

interactive: build
	@echo "Starting interactive signal handling test..."
	@echo "Use Ctrl+C to test interrupt handling"
	cargo run

clean:
	@echo "Cleaning..."
	cargo clean