#!/usr/bin/env bash

# Integration test for function calling - simplified version

echo "=== GEMINI REPL FUNCTION CALLING INTEGRATION TEST ==="
echo "Version: 0.1.2"
echo "Model: gemini-2.0-flash-exp"
echo "Date: $(date)"
echo "=================================================="
echo

cd ../..

# Test categories
declare -A test_results
test_results["read_file"]=0
test_results["list_files"]=0
test_results["search_code"]=0
test_results["write_file"]=0
test_results["total"]=0
test_results["success"]=0

# Function to run a test
run_test() {
    local prompt="$1"
    local expected_function="$2"
    local category="$3"
    
    echo -n "."
    
    result=$(echo "$prompt" | env RUST_LOG=error timeout 5 cargo run --quiet 2>/dev/null | grep "FUNCTION_CALL" || echo "FAIL")
    
    test_results["total"]=$((test_results["total"] + 1))
    
    if [[ "$result" == *"FUNCTION_CALL: $expected_function"* ]]; then
        test_results["success"]=$((test_results["success"] + 1))
        test_results["$category"]=$((test_results["$category"] + 1))
        return 0
    else
        return 1
    fi
}

# Save detailed results
results_file="experiments/025-function-calling-proof/integration_results_$(date +%Y%m%d_%H%M%S).txt"
echo "Test results will be saved to: $results_file"
echo

# Test suites
echo "Running read_file tests..."
for file in "Makefile" "README.md" "Cargo.toml" "src/main.rs" "src/api.rs"; do
    run_test "read $file" "read_file" "read_file"
    run_test "show me $file" "read_file" "read_file"
    run_test "what's in $file" "read_file" "read_file"
    run_test "display $file" "read_file" "read_file"
done
echo " Done!"

echo "Running list_files tests..."
for dir in "src" "tests" "experiments" "."; do
    run_test "list files in $dir" "list_files" "list_files"
    run_test "show files in $dir" "list_files" "list_files"
    run_test "what files are in $dir" "list_files" "list_files"
done
echo " Done!"

echo "Running search_code tests..."
for term in "TODO" "function" "async" "impl" "struct"; do
    run_test "search for $term" "search_code" "search_code"
    run_test "find $term" "search_code" "search_code"
    run_test "look for $term" "search_code" "search_code"
done
echo " Done!"

echo "Running write_file tests..."
run_test "create test.txt with hello" "write_file" "write_file"
run_test "write hello to test.txt" "write_file" "write_file"
echo " Done!"

# Calculate percentages
success_rate=$(echo "scale=2; ${test_results["success"]} * 100 / ${test_results["total"]}" | bc)
read_rate=$(echo "scale=2; ${test_results["read_file"]} * 100 / 20" | bc)
list_rate=$(echo "scale=2; ${test_results["list_files"]} * 100 / 12" | bc)
search_rate=$(echo "scale=2; ${test_results["search_code"]} * 100 / 15" | bc)
write_rate=$(echo "scale=2; ${test_results["write_file"]} * 100 / 2" | bc)

# Generate report
{
    echo "=== INTEGRATION TEST REPORT ==="
    echo "Generated: $(date)"
    echo "Total Tests: ${test_results["total"]}"
    echo "Successful: ${test_results["success"]}"
    echo "Success Rate: $success_rate%"
    echo
    echo "By Function Type:"
    echo "- read_file: ${test_results["read_file"]}/20 ($read_rate%)"
    echo "- list_files: ${test_results["list_files"]}/12 ($list_rate%)"
    echo "- search_code: ${test_results["search_code"]}/15 ($search_rate%)"
    echo "- write_file: ${test_results["write_file"]}/2 ($write_rate%)"
} | tee "$results_file"

echo
echo "=================================================="
echo "SUMMARY:"
if (( $(echo "$success_rate > 80" | bc -l) )); then
    echo "✅ INTEGRATION TEST PASSED! Success rate: $success_rate%"
    echo "Function calling is working reliably!"
else
    echo "⚠️  Success rate below 80%: $success_rate%"
    echo "Some function calls may need improvement."
fi
echo "=================================================="