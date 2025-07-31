#!/bin/bash

# Simple test to verify function calling is working

echo "Testing basic function calling..."

# Test 1: read_file
echo -n "Test read_file: "
result=$(echo "read Makefile" | env RUST_LOG=error cargo run --quiet 2>/dev/null)
if echo "$result" | grep -q "FUNCTION_CALL: read_file"; then
    echo "✅ PASS"
else
    echo "❌ FAIL"
    echo "Output was: $result"
fi

# Test 2: list_files  
echo -n "Test list_files: "
result=$(echo "list files in src" | env RUST_LOG=error cargo run --quiet 2>/dev/null)
if echo "$result" | grep -q "FUNCTION_CALL: list_files"; then
    echo "✅ PASS"
else
    echo "❌ FAIL"
fi

# Test 3: search_code
echo -n "Test search_code: "
result=$(echo "search for TODO" | env RUST_LOG=error cargo run --quiet 2>/dev/null)
if echo "$result" | grep -q "FUNCTION_CALL: search_code"; then
    echo "✅ PASS"
else
    echo "❌ FAIL"
fi

# Test 4: write_file
echo -n "Test write_file: "
result=$(echo "write hello to test.txt" | env RUST_LOG=error cargo run --quiet 2>/dev/null)
if echo "$result" | grep -q "FUNCTION_CALL: write_file"; then
    echo "✅ PASS"
else
    echo "❌ FAIL"
fi