# Error Handling Experiment

.PHONY: run build clean help

help:
	@echo "Error Handling Experiment:"
	@echo "  make run   - Test error handling"
	@echo "  make build - Build the experiment"
	@echo "  make clean - Clean build artifacts"

build:
	@echo "Building error handling experiment..."
	cargo build

run: build
	@echo "Testing error handling..."
	cargo run

clean:
	@echo "Cleaning..."
	cargo clean