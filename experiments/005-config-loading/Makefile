# Config Loading Experiment

.PHONY: run build clean help

help:
	@echo "Config Loading Experiment:"
	@echo "  make run   - Test configuration loading"
	@echo "  make build - Build the experiment"
	@echo "  make clean - Clean build artifacts"

build:
	@echo "Building config loading experiment..."
	cargo build

run: build
	@echo "Testing configuration loading..."
	cargo run

clean:
	@echo "Cleaning..."
	cargo clean