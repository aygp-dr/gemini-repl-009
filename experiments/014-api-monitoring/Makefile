# API Monitoring Experiment

.PHONY: all run clean test-local

all: build

build:
	@cargo build

run: build
	@echo "Testing Gemini API monitoring and validation..."
	@if [ -z "$$GEMINI_API_KEY" ]; then \
		echo "Error: GEMINI_API_KEY not set"; \
		exit 1; \
	fi
	@. ../../.env && cargo run

test-local:
	@echo "Run with local .env file"
	@. ../../.env && DEBUG=true cargo run

show-logs:
	@echo "Latest observations:"
	@tail -3 logs/gemini/*/observations.jsonl 2>/dev/null | jq -c '{timestamp, status: .response.status, valid: .validation.is_valid, tokens: .metrics.total_tokens}' || echo "No logs found"

clean:
	@cargo clean
	@rm -rf logs/