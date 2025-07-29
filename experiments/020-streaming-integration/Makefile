.PHONY: run test clean build

run:
	@echo "Testing streaming response integration..."
	cargo run --bin streaming-test

test:
	@echo "Running streaming integration tests..."
	cargo test

build:
	@echo "Building streaming integration experiment..."
	cargo build

clean:
	@echo "Cleaning streaming integration artifacts..."
	cargo clean
	rm -f *.log