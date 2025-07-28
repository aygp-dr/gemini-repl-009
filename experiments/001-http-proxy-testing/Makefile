# Makefile for HTTP Proxy Testing Experiment

.PHONY: test clean results

# Run all proxy tests
test:
	@echo "Running proxy connectivity tests..."
	@bash test_proxy.sh
	@echo ""
	@echo "To test in Rust, add proxy_config.rs to your project"

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