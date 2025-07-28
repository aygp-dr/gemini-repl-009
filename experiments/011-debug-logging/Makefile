# Debug Logging Experiment

.PHONY: all run test clean

all: build

build:
	@echo "Building debug logging experiment..."
	@cargo build

run: build
	@echo "\n=== Running without debug mode ==="
	@cargo run
	@echo "\n=== Running with DEBUG=true ==="
	@DEBUG=true cargo run
	@echo "\n=== Check logs directory ==="
	@ls -la logs/ 2>/dev/null || echo "No logs directory created"

test-env:
	@echo "Testing .env loading..."
	@echo "DEBUG=true" > .env
	@echo "LOG_LEVEL=trace" >> .env
	@cargo run
	@rm -f .env

clean:
	@cargo clean
	@rm -rf logs/
	@rm -f .env