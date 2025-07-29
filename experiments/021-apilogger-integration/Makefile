.PHONY: run test clean build

run:
	@echo "Testing ApiLogger integration..."
	cargo run --bin apilogger-test

test:
	@echo "Running ApiLogger integration tests..."
	cargo test

build:
	@echo "Building ApiLogger integration experiment..."
	cargo build

clean:
	@echo "Cleaning ApiLogger integration artifacts..."
	cargo clean
	rm -rf logs/
	rm -f *.log