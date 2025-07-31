#!/bin/bash
# Run the function calling test harness multiple times and collect results

EXPERIMENT_DIR="/home/jwalsh/projects/aygp-dr/gemini-repl-009/experiments/023-function-calling"
RESULTS_DIR="$EXPERIMENT_DIR/test_results"
TIMESTAMP=$(date +%Y%m%d_%H%M%S)
RESULTS_FILE="$RESULTS_DIR/test_run_${TIMESTAMP}.log"
SUMMARY_FILE="$RESULTS_DIR/summary_${TIMESTAMP}.json"

# Create results directory
mkdir -p "$RESULTS_DIR"

# Load environment
source /home/jwalsh/projects/aygp-dr/gemini-repl-009/.env

echo "=== Function Calling Test Harness ===" | tee "$RESULTS_FILE"
echo "Starting 20 test iterations at $(date)" | tee -a "$RESULTS_FILE"
echo "Model: ${GEMINI_MODEL:-gemini-1.5-flash}" | tee -a "$RESULTS_FILE"
echo "" | tee -a "$RESULTS_FILE"

# Run tests
cd "$EXPERIMENT_DIR"

# First, run the test runner to check local function detection
echo "=== Running Local Test Suite ===" | tee -a "$RESULTS_FILE"
cargo run --bin test-runner 2>&1 | tee -a "$RESULTS_FILE"

echo -e "\n=== Running Makefile Test 20 Times ===" | tee -a "$RESULTS_FILE"

SUCCESS_COUNT=0
FAIL_COUNT=0
NO_FUNCTION_COUNT=0
WRONG_FUNCTION_COUNT=0

for i in {1..20}; do
    echo -e "\n--- Test Run $i/20 ---" | tee -a "$RESULTS_FILE"
    
    # Run the makefile test and capture output
    OUTPUT=$(cargo run --bin makefile-test 2>&1)
    echo "$OUTPUT" | tee -a "$RESULTS_FILE"
    
    # Check results
    if echo "$OUTPUT" | grep -q "✅ CORRECT: Would read Makefile!"; then
        echo "Result: SUCCESS - Correct function call detected" | tee -a "$RESULTS_FILE"
        ((SUCCESS_COUNT++))
    elif echo "$OUTPUT" | grep -q "✅ Function call detected!"; then
        echo "Result: WRONG FUNCTION - Function called but not read_file for Makefile" | tee -a "$RESULTS_FILE"
        ((WRONG_FUNCTION_COUNT++))
    elif echo "$OUTPUT" | grep -q "I don't have.*access" || echo "$OUTPUT" | grep -q "cannot.*read.*files"; then
        echo "Result: NO FUNCTION - Model claimed no filesystem access" | tee -a "$RESULTS_FILE"
        ((NO_FUNCTION_COUNT++))
    else
        echo "Result: FAIL - Unknown response" | tee -a "$RESULTS_FILE"
        ((FAIL_COUNT++))
    fi
    
    # Small delay between API calls
    sleep 2
done

# Generate summary
echo -e "\n=== TEST SUMMARY ===" | tee -a "$RESULTS_FILE"
echo "Total runs: 20" | tee -a "$RESULTS_FILE"
echo "Success (correct function): $SUCCESS_COUNT ($(( SUCCESS_COUNT * 100 / 20 ))%)" | tee -a "$RESULTS_FILE"
echo "Wrong function called: $WRONG_FUNCTION_COUNT ($(( WRONG_FUNCTION_COUNT * 100 / 20 ))%)" | tee -a "$RESULTS_FILE"
echo "No function (claimed no access): $NO_FUNCTION_COUNT ($(( NO_FUNCTION_COUNT * 100 / 20 ))%)" | tee -a "$RESULTS_FILE"
echo "Failed/Unknown: $FAIL_COUNT ($(( FAIL_COUNT * 100 / 20 ))%)" | tee -a "$RESULTS_FILE"

# Create JSON summary
cat > "$SUMMARY_FILE" << EOF
{
  "timestamp": "$TIMESTAMP",
  "model": "${GEMINI_MODEL:-gemini-1.5-flash}",
  "total_runs": 20,
  "results": {
    "success": $SUCCESS_COUNT,
    "wrong_function": $WRONG_FUNCTION_COUNT,
    "no_function": $NO_FUNCTION_COUNT,
    "failed": $FAIL_COUNT
  },
  "success_rate": $(( SUCCESS_COUNT * 100 / 20 )),
  "function_call_rate": $(( (SUCCESS_COUNT + WRONG_FUNCTION_COUNT) * 100 / 20 ))
}
EOF

echo -e "\nResults saved to:" | tee -a "$RESULTS_FILE"
echo "  Log: $RESULTS_FILE"
echo "  Summary: $SUMMARY_FILE"