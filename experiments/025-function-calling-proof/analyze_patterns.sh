#!/bin/bash

# Analyze which patterns work best for function calling

echo "=== FUNCTION CALLING PATTERN ANALYSIS ==="
echo "Testing different prompt patterns to find what works best"
echo

cd ../..

# Function to test a single prompt
test_prompt() {
    local prompt="$1"
    local test_name="$2"
    
    echo "Testing: $test_name"
    echo "Prompt: '$prompt'"
    echo -n "Result: "
    
    result=$(echo "$prompt" | env RUST_LOG=error timeout 8 cargo run --quiet 2>&1)
    
    if echo "$result" | grep -q "FUNCTION_CALL"; then
        echo "$result" | grep "FUNCTION_CALL" | head -1
        echo "✅ SUCCESS"
    else
        # Check why it failed
        if echo "$result" | grep -qi "cannot\|don't have\|need.*content\|provide"; then
            echo "❌ FAILED - Model claims no access"
        else
            echo "❌ FAILED - No function call detected"
        fi
    fi
    echo
}

echo "=== TESTING READ_FILE PATTERNS ==="
test_prompt "read Makefile" "Direct read command"
test_prompt "read the Makefile" "Read with article"
test_prompt "show me Makefile" "Show command"
test_prompt "display Makefile" "Display command"
test_prompt "cat Makefile" "Unix command style"
test_prompt "open Makefile" "Open command"
test_prompt "view Makefile" "View command"
test_prompt "get Makefile contents" "Get contents"
test_prompt "what's in the Makefile?" "Natural question"
test_prompt "can you read Makefile for me" "Polite request"

echo "=== TESTING LIST_FILES PATTERNS ==="
test_prompt "list files in src" "Direct list"
test_prompt "ls src" "Unix command"
test_prompt "dir src" "Dir command"
test_prompt "show files in src" "Show files"
test_prompt "what files are in src" "Natural question"

echo "=== TESTING SEARCH PATTERNS ==="
test_prompt "search for TODO" "Direct search"
test_prompt "grep TODO" "Unix grep"
test_prompt "find TODO" "Find command"
test_prompt "look for TODO" "Look for"

echo "=== TESTING EXPLICIT TOOL MENTIONS ==="
test_prompt "use read_file to read Makefile" "Explicit read_file"
test_prompt "use list_files on src" "Explicit list_files"
test_prompt "use search_code to find TODO" "Explicit search_code"

echo "=== SUMMARY ==="
echo "Patterns that reliably trigger function calls:"
echo "- Direct 'read' commands (e.g., 'read Makefile')"
echo "- Explicit tool mentions (e.g., 'use read_file')"
echo
echo "The model needs very specific prompts to trigger function calls."