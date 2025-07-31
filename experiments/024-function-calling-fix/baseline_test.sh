#!/bin/bash

# Function calling baseline test
# Tests the before and after states of function calling implementation

EXPERIMENT_DIR="experiments/024-function-calling-fix"
TIMESTAMP=$(date +%Y%m%d_%H%M%S)

echo "=== Function Calling Baseline Test ==="
echo "Timestamp: $TIMESTAMP"
echo

# Test prompts that should trigger function calls
TEST_PROMPTS=(
    "summarize the Makefile in this repo"
    "what does the Makefile contain?"
    "read the Cargo.toml file"
    "show me what's in src/main.rs"
    "list all files in the src directory"
)

# Function to run a single test
run_test() {
    local prompt="$1"
    local output_file="$2"
    
    echo "Testing: $prompt"
    echo "$prompt" | cargo run --bin gemini-repl 2>&1 | tee "$output_file"
    echo "---"
    echo
}

# Create before directory
mkdir -p "$EXPERIMENT_DIR/before"

echo "=== BEFORE STATE ==="
echo "Running tests with current implementation..."
echo

for i in "${!TEST_PROMPTS[@]}"; do
    run_test "${TEST_PROMPTS[$i]}" "$EXPERIMENT_DIR/before/test_${i}_${TIMESTAMP}.log"
done

echo "=== Analyzing Results ==="
echo

# Check for function call indicators
for log in "$EXPERIMENT_DIR/before"/*.log; do
    echo "Checking $log:"
    if grep -i "function_call\|tool_call\|read_file\|list_files" "$log" > /dev/null; then
        echo "  ✓ Function call detected"
    else
        echo "  ✗ No function call detected"
    fi
    
    # Check for common failure patterns
    if grep -i "cannot access\|need the content\|provide.*file" "$log" > /dev/null; then
        echo "  ⚠ Model claims no file access"
    fi
done

echo
echo "Baseline test complete. Results saved in $EXPERIMENT_DIR/before/"