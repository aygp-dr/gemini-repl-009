#!/bin/bash
# Evaluation: Model responses WITH tool calling enabled
# This shows expected behavior when tools are available

set -e

GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

echo -e "${BLUE}=== After Tools Evaluation ===${NC}"
echo "Testing model responses when tools ARE available"
echo

# Load environment
. ../../.env

# Tool declarations
TOOLS=$(cat <<'EOF'
{
  "function_declarations": [
    {
      "name": "list_files",
      "description": "List files in a directory with optional pattern matching",
      "parameters": {
        "type": "object",
        "properties": {
          "path": {
            "type": "string",
            "description": "Directory path (default: current directory)"
          },
          "pattern": {
            "type": "string",
            "description": "Glob pattern to filter files (e.g., '*.rs')"
          }
        }
      }
    },
    {
      "name": "read_file",
      "description": "Read the contents of a file",
      "parameters": {
        "type": "object",
        "properties": {
          "path": {
            "type": "string",
            "description": "Path to the file to read"
          }
        },
        "required": ["path"]
      }
    },
    {
      "name": "write_file",
      "description": "Write content to a file",
      "parameters": {
        "type": "object",
        "properties": {
          "path": {
            "type": "string",
            "description": "Path to the file to write"
          },
          "content": {
            "type": "string",
            "description": "Content to write to the file"
          }
        },
        "required": ["path", "content"]
      }
    }
  ]
}
EOF
)

# Test 1: File listing with tools
echo -e "${YELLOW}Test 1: Request to list files (with tools)${NC}"
REQUEST=$(cat <<EOF
{
  "contents": [{
    "role": "user",
    "parts": [{"text": "List all the Rust files in the src directory"}]
  }],
  "tools": [$TOOLS]
}
EOF
)

echo "Request: 'List all the Rust files in the src directory'"
echo "Expected: Model calls list_files tool with path='src' and pattern='*.rs'"

response=$(curl -s -X POST "https://generativelanguage.googleapis.com/v1beta/models/gemini-1.5-flash:generateContent?key=${GEMINI_API_KEY}" \
    -H "Content-Type: application/json" \
    -d "$REQUEST")

if echo "$response" | jq -e '.candidates[0].content.parts[0]' > /dev/null 2>&1; then
    part=$(echo "$response" | jq -r '.candidates[0].content.parts[0]')
    
    # Check for function call
    if echo "$part" | jq -e '.functionCall' > /dev/null 2>&1; then
        func_name=$(echo "$part" | jq -r '.functionCall.name')
        func_args=$(echo "$part" | jq -r '.functionCall.args')
        echo -e "${GREEN}✓ Model called function: $func_name${NC}"
        echo "Arguments: $func_args"
        
        # Simulate function response
        FUNCTION_RESPONSE=$(cat <<EOF
{
  "contents": [
    {
      "role": "user",
      "parts": [{"text": "List all the Rust files in the src directory"}]
    },
    {
      "role": "model",
      "parts": [{
        "functionCall": {
          "name": "list_files",
          "args": {"path": "src", "pattern": "*.rs"}
        }
      }]
    },
    {
      "role": "function",
      "parts": [{
        "functionResponse": {
          "name": "list_files",
          "response": {
            "files": ["src/main.rs", "src/api.rs", "src/lib.rs", "src/tools.rs"]
          }
        }
      }]
    }
  ],
  "tools": [$TOOLS]
}
EOF
)
        
        # Get final response
        echo -e "\n${YELLOW}Sending function response back...${NC}"
        final_response=$(curl -s -X POST "https://generativelanguage.googleapis.com/v1beta/models/gemini-1.5-flash:generateContent?key=${GEMINI_API_KEY}" \
            -H "Content-Type: application/json" \
            -d "$FUNCTION_RESPONSE")
            
        if echo "$final_response" | jq -e '.candidates[0].content.parts[0].text' > /dev/null 2>&1; then
            answer=$(echo "$final_response" | jq -r '.candidates[0].content.parts[0].text')
            echo -e "${GREEN}Final response:${NC}"
            echo "$answer" | head -10
        fi
    else
        # Model responded with text instead of function call
        text=$(echo "$part" | jq -r '.text // ""')
        echo -e "${YELLOW}⚠ Model responded with text instead of function call:${NC}"
        echo "$text" | head -5
    fi
fi
echo

# Test 2: Multi-step tool usage
echo -e "${YELLOW}Test 2: Multi-step tool usage${NC}"
REQUEST=$(cat <<EOF
{
  "contents": [{
    "role": "user",
    "parts": [{"text": "Read the Cargo.toml file and create a summary of dependencies in a new file called DEPENDENCIES.md"}]
  }],
  "tools": [$TOOLS]
}
EOF
)

echo "Request: 'Read Cargo.toml and create DEPENDENCIES.md summary'"
echo "Expected: Model calls read_file then write_file"

response=$(curl -s -X POST "https://generativelanguage.googleapis.com/v1beta/models/gemini-1.5-flash:generateContent?key=${GEMINI_API_KEY}" \
    -H "Content-Type: application/json" \
    -d "$REQUEST")

if echo "$response" | jq -e '.candidates[0].content.parts[0].functionCall' > /dev/null 2>&1; then
    func_name=$(echo "$response" | jq -r '.candidates[0].content.parts[0].functionCall.name')
    echo -e "${GREEN}✓ First function call: $func_name${NC}"
fi

# Save results
mkdir -p results
echo "$response" | jq . > results/after_tools_responses.json

echo -e "\n${BLUE}=== Tool Calling Behavior Summary ===${NC}"
echo "1. With tools: Model attempts to call appropriate functions"
echo "2. Without tools: Model explains limitations or provides instructions"
echo "3. Function responses must be sent back for final answer"
echo
echo "Results saved to results/after_tools_responses.json"