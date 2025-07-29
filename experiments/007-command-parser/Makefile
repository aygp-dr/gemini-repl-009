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

run: build
	@echo "Running parser tests..."
	@echo -e "/help\n/exit" | timeout 10 cargo run

test: build
	@echo "Running automated parser tests..."
	@echo -e "/help\n/version\n/exit" | timeout 5 cargo run | head -20

interactive: build
	@echo "Starting interactive command parser test..."
	cargo run

clean:
	@echo "Cleaning..."
	cargo clean