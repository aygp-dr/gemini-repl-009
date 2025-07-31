# Function Calling Experiment

.PHONY: run test clean build help

help:
	@echo "Function Calling Experiment:"
	@echo "  make run   - Run function calling demo"
	@echo "  make test  - Run automated tests"
	@echo "  make build - Build the experiment"
	@echo "  make clean - Clean build artifacts"

build:
	@echo "Building function calling experiment..."
	cargo build

run: build
	@echo "Running function calling demo..."
	@. ../../.env && cargo run

makefile-test: build
	@echo "Running Makefile dependency test..."
	@. ../../.env && cargo run --bin makefile-test

test: build
	@echo "Running function calling tests..."
	cargo test

clean:
	@echo "Cleaning..."
	cargo clean
	rm -f *.log