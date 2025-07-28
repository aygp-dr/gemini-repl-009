#!/bin/bash
# Test Ollama API endpoints to understand request/response format
# Run with: bash test_ollama_api.sh

set -e

GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

OLLAMA_URL="http://localhost:11434"
MODEL="llama2" # Change to your installed model

echo -e "${BLUE}=== Ollama API Test Suite ===${NC}"
echo "Testing Ollama API endpoints for compatibility"
echo "Using model: $MODEL"
echo

# Check if Ollama is running
echo -e "${YELLOW}0. Checking Ollama status...${NC}"
if curl -s "$OLLAMA_URL/api/tags" > /dev/null 2>&1; then
    echo -e "${GREEN}✓ Ollama is running${NC}"
    echo "Available models:"
    curl -s "$OLLAMA_URL/api/tags" | jq -r '.models[].name' 2>/dev/null || echo "(unable to list models)"
else
    echo -e "${RED}✗ Ollama is not running. Start with: ollama serve${NC}"
    exit 1
fi
echo

# Test 1: Basic generation
echo -e "${YELLOW}1. Basic generation test${NC}"
echo "Request:"
REQUEST='{"model": "'$MODEL'", "prompt": "Hello, how are you?", "stream": false}'
echo "$REQUEST" | jq .

echo -e "\nResponse:"
response=$(curl -s -X POST "$OLLAMA_URL/api/generate" \
    -H "Content-Type: application/json" \
    -d "$REQUEST")

if [ -n "$response" ]; then
    echo "$response" | jq . | head -20
    echo -e "${GREEN}✓ Basic generation works${NC}"
else
    echo -e "${RED}✗ No response${NC}"
fi
echo

# Test 2: Chat format (conversation)
echo -e "${YELLOW}2. Chat format test (conversation)${NC}"
echo "Request:"
CHAT_REQUEST='{
  "model": "'$MODEL'",
  "messages": [
    {"role": "user", "content": "What is 2+2?"},
    {"role": "assistant", "content": "2+2 equals 4."},
    {"role": "user", "content": "Double it"}
  ],
  "stream": false
}'
echo "$CHAT_REQUEST" | jq .

echo -e "\nResponse:"
response=$(curl -s -X POST "$OLLAMA_URL/api/chat" \
    -H "Content-Type: application/json" \
    -d "$CHAT_REQUEST")

if [ -n "$response" ]; then
    echo "$response" | jq . | head -20
    echo -e "${GREEN}✓ Chat format works${NC}"
else
    echo -e "${RED}✗ Chat endpoint may not be available${NC}"
fi
echo

# Test 3: Tool calling (if supported)
echo -e "${YELLOW}3. Tool calling test${NC}"
echo "Request:"
TOOL_REQUEST='{
  "model": "'$MODEL'",
  "messages": [
    {"role": "user", "content": "What files are in the current directory?"}
  ],
  "tools": [{
    "type": "function",
    "function": {
      "name": "list_files",
      "description": "List files in a directory",
      "parameters": {
        "type": "object",
        "properties": {
          "path": {
            "type": "string",
            "description": "Directory path"
          }
        }
      }
    }
  }],
  "stream": false
}'
echo "$TOOL_REQUEST" | jq .

echo -e "\nResponse:"
response=$(curl -s -X POST "$OLLAMA_URL/api/chat" \
    -H "Content-Type: application/json" \
    -d "$TOOL_REQUEST")

if [ -n "$response" ]; then
    echo "$response" | jq . | head -30
    
    # Check if response contains tool call
    if echo "$response" | jq -e '.message.tool_calls' > /dev/null 2>&1; then
        echo -e "${GREEN}✓ Tool calling is supported${NC}"
    else
        echo -e "${YELLOW}⚠ Tool calling may not be supported by this model${NC}"
    fi
else
    echo -e "${RED}✗ No response${NC}"
fi
echo

# Test 4: Streaming response
echo -e "${YELLOW}4. Streaming test${NC}"
echo "Testing streaming response (first 5 chunks)..."
STREAM_REQUEST='{"model": "'$MODEL'", "prompt": "Count from 1 to 5", "stream": true}'

echo -e "\nStreaming chunks:"
count=0
curl -s -X POST "$OLLAMA_URL/api/generate" \
    -H "Content-Type: application/json" \
    -d "$STREAM_REQUEST" | while IFS= read -r line && [ $count -lt 5 ]; do
    if [ -n "$line" ]; then
        echo "Chunk $((count+1)): $line" | head -1
        ((count++))
    fi
done
echo -e "${GREEN}✓ Streaming works${NC}"
echo

# Test 5: Model information
echo -e "${YELLOW}5. Model information${NC}"
echo "Request: GET /api/show"
SHOW_REQUEST='{"name": "'$MODEL'"}'
response=$(curl -s -X POST "$OLLAMA_URL/api/show" \
    -H "Content-Type: application/json" \
    -d "$SHOW_REQUEST")

if [ -n "$response" ]; then
    echo "$response" | jq -r '.modelfile' | head -20
    echo -e "${GREEN}✓ Model info retrieved${NC}"
fi
echo

# Summary
echo -e "${BLUE}=== API Comparison Summary ===${NC}"
echo "Ollama API characteristics:"
echo "- Uses /api/generate for simple prompts"
echo "- Uses /api/chat for conversations"
echo "- Role names: user, assistant, system"
echo "- Supports streaming with stream:true"
echo "- Tool calling support varies by model"
echo
echo "Key differences from Gemini:"
echo "- No API key required (local)"
echo "- Different endpoint structure"
echo "- 'assistant' instead of 'model' role"
echo "- 'content' instead of 'parts' for messages"
echo "- Simpler response format"