#!/bin/bash

echo "=== Single Function Call Test ==="
echo "Testing with prompt: 'read the Makefile'"
echo

cd ../..
echo "read the Makefile" | gmake run 2>&1 | grep -A2 -B2 "FUNCTION_CALL"