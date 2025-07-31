#!/bin/bash
# Test script for function calling

echo "summarize Makefile" | cargo run --bin gemini-repl 2>&1 | tee function_call_test.log