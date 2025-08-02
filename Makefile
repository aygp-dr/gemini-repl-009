# Makefile for Gemini REPL
.PHONY: help build test clippy fmt check clean run dev install bench docs

# Default target
help: ## Show this help message
	@echo "Available targets:"
	@grep -E '^[a-zA-Z_-]+:.*?## .*$$' $(MAKEFILE_LIST) | sort | awk 'BEGIN {FS = ":.*?## "}; {printf "  %-15s %s\n", $$1, $$2}'

# Development targets
build: ## Build the project
	cargo build

build-release: ## Build the project in release mode
	cargo build --release

test: ## Run tests
	cargo test

test-verbose: ## Run tests with verbose output
	cargo test -- --nocapture

clippy: ## Run clippy linter
	cargo clippy --all-targets -- -D warnings

clippy-fix: ## Run clippy with automatic fixes
	cargo clippy --fix --allow-dirty --allow-staged

fmt: ## Format the code
	cargo fmt

fmt-check: ## Check if code is formatted
	cargo fmt -- --check

check: ## Check the code without building
	cargo check

check-all: ## Check all targets and features
	cargo check --all-targets --all-features

# Quality assurance
audit: ## Audit dependencies for security issues
	cargo audit

outdated: ## Check for outdated dependencies
	cargo outdated

# Cleaning
clean: ## Clean build artifacts
	cargo clean

# Running
run: ## Run the application with default features
	cargo run

run-noop: ## Run in noop mode (no API calls)
	NOOP_MODE=true cargo run

run-debug: ## Run with debug logging
	RUST_LOG=debug cargo run

run-self-mod: ## Run with self-modification enabled
	cargo run -- --enable-self-modification

dev: ## Run in development mode with debug logging and self-modification
	RUST_LOG=debug cargo run -- --enable-self-modification --debug

# Installation
install: ## Install the binary
	cargo install --path .

install-force: ## Force reinstall the binary
	cargo install --path . --force

# Benchmarking (when benchmarks are added)
bench: ## Run benchmarks
	cargo bench

# Documentation
docs: ## Generate documentation
	cargo doc --no-deps --open

docs-all: ## Generate documentation for all dependencies
	cargo doc --open

# Feature-specific builds
build-minimal: ## Build with minimal features
	cargo build --no-default-features

build-full: ## Build with all features
	cargo build --all-features

test-minimal: ## Test with minimal features
	cargo test --no-default-features

test-full: ## Test with all features
	cargo test --all-features

# Release preparation
check-release: build-release test clippy fmt-check ## Full pre-release check
	@echo "Release checks passed!"

pre-commit: fmt clippy test ## Run pre-commit checks
	@echo "Pre-commit checks passed!"

# Tool installation helpers
install-tools: ## Install development tools
	cargo install cargo-audit
	cargo install cargo-outdated
	rustup component add clippy rustfmt

# CI/CD simulation
ci: fmt-check clippy test build-release ## Simulate CI pipeline
	@echo "CI pipeline completed successfully!"

# Docker targets (if Dockerfile exists)
docker-build: ## Build Docker image
	docker build -t gemini-repl .

docker-run: ## Run Docker container
	docker run --rm -it gemini-repl

# Git hooks
setup-hooks: ## Set up git hooks
	@echo "Setting up git hooks..."
	@echo "#!/bin/sh" > .git/hooks/pre-commit
	@echo "make pre-commit" >> .git/hooks/pre-commit
	@chmod +x .git/hooks/pre-commit
	@echo "Git hooks set up successfully!"

# Performance profiling
profile: ## Run with profiling
	cargo build --release
	perf record -g target/release/gemini-repl --help || echo "perf not available"

# Size analysis
bloat: ## Analyze binary size
	cargo bloat --release --crates

# Coverage (requires cargo-tarpaulin)
coverage: ## Generate test coverage report
	cargo tarpaulin --out Html

# Environment info
env-info: ## Show environment information
	@echo "Rust version:"
	@rustc --version
	@echo "Cargo version:"
	@cargo --version
	@echo "Clippy version:"
	@cargo clippy --version
	@echo "Rustfmt version:"
	@cargo fmt --version

# All-in-one quality check
qa: fmt-check clippy test audit ## Run all quality assurance checks
	@echo "All QA checks passed!"