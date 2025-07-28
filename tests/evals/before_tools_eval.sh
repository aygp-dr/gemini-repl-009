#!/bin/bash
# Evaluation: Model responses WITHOUT tool calling enabled
# This establishes baseline behavior when model can't use tools

set -e

GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

echo -e "${BLUE}=== Before Tools Evaluation ===${NC}"
echo "Testing model responses when tools are NOT available"
echo

# Load environment
. ../../.env

# Test 1: File listing request
echo -e "${YELLOW}Test 1: Request to list files${NC}"
REQUEST=$(cat <<'EOF'
{
  "contents": [{
    "role": "user",
    "parts": [{"text": "List all the Rust files in the src directory"}]
  }]
}
EOF
)

echo "Request: 'List all the Rust files in the src directory'"
echo "Expected: Model explains it cannot access filesystem"

response=$(curl -s -X POST "https://generativelanguage.googleapis.com/v1beta/models/gemini-1.5-flash:generateContent?key=${GEMINI_API_KEY}" \
    -H "Content-Type: application/json" \
    -d "$REQUEST")

if echo "$response" | jq -e '.candidates[0].content.parts[0].text' > /dev/null 2>&1; then
    answer=$(echo "$response" | jq -r '.candidates[0].content.parts[0].text')
    echo -e "${GREEN}Response received:${NC}"
    echo "$answer" | head -5
    echo "..."
    
    # Check if model indicates it can't access files
    if echo "$answer" | grep -iE "(cannot|can't|unable|don't have access|need to|you.*(need|have|should))" > /dev/null; then
        echo -e "${GREEN}✓ Model correctly indicates limitation${NC}"
    else
        echo -e "${YELLOW}⚠ Model may be hallucinating file access${NC}"
    fi
fi
echo

# Test 2: File reading request
echo -e "${YELLOW}Test 2: Request to read a file${NC}"
REQUEST=$(cat <<'EOF'
{
  "contents": [{
    "role": "user",
    "parts": [{"text": "Read the Cargo.toml file and tell me what dependencies this project uses"}]
  }]
}
EOF
)

echo "Request: 'Read the Cargo.toml file and tell me what dependencies this project uses'"
echo "Expected: Model explains it cannot read files"

response=$(curl -s -X POST "https://generativelanguage.googleapis.com/v1beta/models/gemini-1.5-flash:generateContent?key=${GEMINI_API_KEY}" \
    -H "Content-Type: application/json" \
    -d "$REQUEST")

if echo "$response" | jq -e '.candidates[0].content.parts[0].text' > /dev/null 2>&1; then
    answer=$(echo "$response" | jq -r '.candidates[0].content.parts[0].text')
    echo -e "${GREEN}Response received:${NC}"
    echo "$answer" | head -5
    echo "..."
fi
echo

# Test 3: File writing request
echo -e "${YELLOW}Test 3: Request to create a file${NC}"
REQUEST=$(cat <<'EOF'
{
  "contents": [{
    "role": "user",
    "parts": [{"text": "Create a new file called test.rs with a hello world function"}]
  }]
}
EOF
)

echo "Request: 'Create a new file called test.rs with a hello world function'"
echo "Expected: Model provides code but cannot actually create the file"

response=$(curl -s -X POST "https://generativelanguage.googleapis.com/v1beta/models/gemini-1.5-flash:generateContent?key=${GEMINI_API_KEY}" \
    -H "Content-Type: application/json" \
    -d "$REQUEST")

if echo "$response" | jq -e '.candidates[0].content.parts[0].text' > /dev/null 2>&1; then
    answer=$(echo "$response" | jq -r '.candidates[0].content.parts[0].text')
    echo -e "${GREEN}Response received:${NC}"
    echo "$answer" | head -10
    echo "..."
    
    # Check if model provides code without claiming to create file
    if echo "$answer" | grep -i "fn.*hello" > /dev/null; then
        echo -e "${GREEN}✓ Model provides code example${NC}"
    fi
    if echo "$answer" | grep -iE "(created|wrote|saved).*file" > /dev/null; then
        echo -e "${RED}✗ Model incorrectly claims to have created file${NC}"
    fi
fi
echo

# Save baseline responses
mkdir -p results
echo "$response" | jq . > results/before_tools_responses.json

echo -e "${BLUE}=== Evaluation Complete ===${NC}"
echo "Baseline responses saved to results/before_tools_responses.json"