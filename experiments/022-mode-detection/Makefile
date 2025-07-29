.PHONY: run test clean build

run:
	@echo "Testing mode detection (noop/mock/production)..."
	cargo run --bin mode-test

test:
	@echo "Running mode detection tests..."
	cargo test

build: 
	@echo "Building mode detection experiment..."
	cargo build

clean:
	@echo "Cleaning mode detection artifacts..."
	cargo clean
	rm -f *.log