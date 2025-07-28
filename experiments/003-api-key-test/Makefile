# Makefile for API Key Test

.PHONY: run test clean help

help:
	@echo "API Key Test Commands:"
	@echo "  make run   - Run API tests with curl"
	@echo "  make test  - Same as run"
	@echo "  make clean - Remove test artifacts"

run: test

test:
	@chmod +x test_api.sh
	@echo "Running API key test..."
	@bash test_api.sh

clean:
	@rm -f response.json
	@echo "Cleaned test artifacts"

# Save full response for debugging
debug:
	@chmod +x test_api.sh
	@./test_api.sh 2>&1 | tee debug.log
	@echo "Debug output saved to debug.log"