.PHONY: all test test-batch test-small test-full generate-evals clean

MODEL ?= gemini-2.0-flash-lite
BATCH_SIZE ?= 25
RATE_LIMIT_DELAY ?= 2

all: generate-evals

generate-evals:
	@echo "Generating evaluation questions..."
	cargo run --bin generate-evals

test-small:
	@echo "Running small baseline test (first batch, 25 questions)..."
	cargo run --bin run-eval -- --start-batch 1 --model $(MODEL)

test-batch:
	@echo "Running batch test with $(BATCH_SIZE) questions..."
	cargo run --bin run-eval -- --batch-size $(BATCH_SIZE) --model $(MODEL)

test-full:
	@echo "Running full evaluation with rate limiting..."
	cargo run --bin run-eval -- --full --delay $(RATE_LIMIT_DELAY) --model $(MODEL)

test-chunked:
	@echo "Running evaluation in chunks to avoid rate limiting..."
	./run_chunked_eval.sh

analyze:
	@echo "Analyzing results..."
	python3 src/analyze_results.py

clean:
	rm -rf results/*.json data/*.json
	cargo clean

help:
	@echo "Usage:"
	@echo "  make generate-evals  - Generate 1000 evaluation questions"
	@echo "  make test-batch     - Test a batch of $(BATCH_SIZE) questions"
	@echo "  make test-full      - Run full evaluation with rate limiting"
	@echo "  make analyze        - Analyze test results"