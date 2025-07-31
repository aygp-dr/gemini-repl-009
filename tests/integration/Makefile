# Integration Tests Makefile

.PHONY: test test-basic test-advanced help

help:
	@echo "Integration Test Commands:"
	@echo "  make test          - Run all integration tests"
	@echo "  make test-basic    - Run basic REPL test"
	@echo "  make test-advanced - Run advanced REPL test"

test: test-basic test-advanced

test-basic:
	@echo "Running basic integration test..."
	@NOOP_MODE=true expect test_basic.exp

test-advanced:
	@echo "Running advanced integration test..."
	@NOOP_MODE=true expect test_advanced.exp