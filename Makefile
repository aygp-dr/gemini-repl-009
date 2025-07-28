# Gemini REPL Makefile
# Quick install/run targets for development

.PHONY: help install run test clean build

# Default target
help:
	@echo "Gemini REPL 009 - Available targets:"
	@echo "  make install  - Build and install the REPL"
	@echo "  make run      - Run the REPL"
	@echo "  make test     - Run tests"
	@echo "  make clean    - Clean build artifacts"
	@echo "  make build    - Build debug version"

# Install the REPL (build release and put in path)
install: build
	@echo "Installing Gemini REPL..."
	cargo build --release
	@echo "Binary available at: target/release/gemini-repl"
	@echo "Run with: ./target/release/gemini-repl"

# Run the REPL
run:
	@echo "Running Gemini REPL..."
	cargo run --bin gemini-repl

# Run tests
test:
	@echo "Running tests..."
	cargo test

# Clean build artifacts
clean:
	@echo "Cleaning..."
	cargo clean

# Build debug version
build:
	@echo "Building..."
	cargo build