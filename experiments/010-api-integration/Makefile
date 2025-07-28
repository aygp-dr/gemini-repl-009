# API Integration Experiment

.PHONY: run build clean help

help:
	@echo "API Integration Experiment:"
	@echo "  make run   - Test full API integration"
	@echo "  make build - Build the experiment"
	@echo "  make clean - Clean build artifacts"

build:
	@echo "Building API integration experiment..."
	cargo build

run: build
	@echo "Testing API integration..."
	cargo run

clean:
	@echo "Cleaning..."
	cargo clean