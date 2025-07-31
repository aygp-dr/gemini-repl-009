#!/bin/bash
# Test Gemini API with curl

# Load environment variables
if [ -f "../../.env" ]; then
    set -a
    source ../../.env
    set +a
fi

# Check for API key
if [ -z "$GEMINI_API_KEY" ]; then
    echo "Error: GEMINI_API_KEY not set"
    echo "Please set it in .env or export GEMINI_API_KEY=your-key"
    exit 1
fi

echo "=== Gemini API Test ==="
echo "Using API Key: ${GEMINI_API_KEY:0:10}..."
echo

# Test 1: List models
echo "1. Testing models endpoint..."
MODELS_URL="https://generativelanguage.googleapis.com/v1beta/models?key=${GEMINI_API_KEY}"

# Use proxy if configured
CURL_OPTS=""
if [ -n "$HTTPS_PROXY" ]; then
    CURL_OPTS="-x $HTTPS_PROXY"
    echo "Using proxy: $HTTPS_PROXY"
fi

response=$(curl -s $CURL_OPTS "$MODELS_URL")
if echo "$response" | jq -e '.models' > /dev/null 2>&1; then
    echo "✓ Models endpoint works"
    echo "Available models:"
    echo "$response" | jq -r '.models[].name' | head -5
else
    echo "✗ Models endpoint failed"
    echo "Response: $response"
fi
echo

# Test 2: Generate content
echo "2. Testing generate content endpoint..."
MODEL="gemini-1.5-flash"
GENERATE_URL="https://generativelanguage.googleapis.com/v1beta/models/${MODEL}:generateContent?key=${GEMINI_API_KEY}"

# Create request payload
REQUEST_BODY=$(cat <<EOF
{
  "contents": [{
    "parts": [{
      "text": "What is 2 + 40? Just give the number."
    }]
  }]
}
EOF
)

# Make request
response=$(curl -s $CURL_OPTS -X POST "$GENERATE_URL" \
    -H "Content-Type: application/json" \
    -d "$REQUEST_BODY")

if echo "$response" | jq -e '.candidates[0].content.parts[0].text' > /dev/null 2>&1; then
    echo "✓ Generate content endpoint works"
    answer=$(echo "$response" | jq -r '.candidates[0].content.parts[0].text' | tr -d '\n')
    echo "API Response: $answer"
    
    # Check if answer contains 42
    if echo "$answer" | grep -q "42"; then
        echo "✓ Correct answer received!"
    fi
else
    echo "✗ Generate content failed"
    echo "Response: $response"
fi

echo
echo "=== Test Complete ==="