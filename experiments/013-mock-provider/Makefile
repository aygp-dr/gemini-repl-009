# Mock Provider Experiment

.PHONY: all run clean

all: build

build:
	@cargo build

run: build
	@echo "Testing mock provider with recorded data..."
	@cargo run

clean:
	@cargo clean
	@rm -rf mock_data/