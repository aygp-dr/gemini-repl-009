# Makefile for HTTP Proxy Testing Experiment

.PHONY: run test clean results help

help:
	@echo "HTTP Proxy Testing Experiment:"
	@echo "  make run     - Run proxy connectivity tests"
	@echo "  make test    - Same as run"
	@echo "  make results - Document test results"
	@echo "  make clean   - Clean artifacts"

# Run all proxy tests
run:
	@echo "Running proxy connectivity tests..."
	@bash test_proxy.sh
	@echo ""
	@echo "To test in Rust, add proxy_config.rs to your project"

test: run

# Document results
results:
	@echo "## Test Results - $$(date)" >> results.md
	@echo "" >> results.md
	@bash test_proxy.sh >> results.md 2>&1
	@echo "" >> results.md
	@echo "Results saved to results.md"

# Clean up
clean:
	@rm -f results.md
	@echo "Cleaned experiment artifacts"