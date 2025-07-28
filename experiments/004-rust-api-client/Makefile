# Makefile for Rust API Client Test

.PHONY: run test build clean help

help:
	@echo "Rust API Client Test Commands:"
	@echo "  make run    - Run the client"
	@echo "  make test   - Same as run"
	@echo "  make build  - Build the client"
	@echo "  make clean  - Clean build artifacts"

build:
	@echo "Building Rust API client..."
	cargo build

run: build
	@echo "Running Rust API client..."
	@. ../../.env && cargo run -- --proxy

test: run

clean:
	@echo "Cleaning build artifacts..."
	cargo clean