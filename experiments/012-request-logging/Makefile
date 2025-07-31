# Request Logging Experiment

.PHONY: all run clean

all: build

build:
	@cargo build

run: build
	@echo "Testing request/response logging to JSONL files..."
	@cargo run
	@echo "\nLog structure created:"
	@find logs -type f -name "*.jsonl" | sort

show-logs:
	@echo "Sample request log:"
	@head -1 logs/*/v1_posts/reqs.jsonl 2>/dev/null | jq . || echo "No logs found"
	@echo "\nSample response log:"
	@head -1 logs/*/v1_posts/resps.jsonl 2>/dev/null | jq . || echo "No logs found"

clean:
	@cargo clean
	@rm -rf logs/