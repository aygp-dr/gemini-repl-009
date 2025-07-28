# Command Parser Experiment

.PHONY: run build clean test help

help:
	@echo "Command Parser Experiment:"
	@echo "  make run   - Run interactive test"
	@echo "  make test  - Run automated tests only"
	@echo "  make build - Build the experiment"
	@echo "  make clean - Clean build artifacts"

build:
	@echo "Building command parser experiment..."
	cargo build

test: build
	@echo "Running parser tests..."
	@echo "/exit" | cargo run | head -30

run: build
	@echo "Starting command parser test..."
	cargo run

clean:
	@echo "Cleaning..."
	cargo clean