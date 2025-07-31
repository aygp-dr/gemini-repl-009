#!/bin/bash
# Test conversation API with proper role format

# Load environment
. .env

if [ -z "$GEMINI_API_KEY" ]; then
    echo "Error: GEMINI_API_KEY not set"
    exit 1
fi

echo "Testing conversation API with roles..."

MODEL="gemini-1.5-flash"
URL="https://generativelanguage.googleapis.com/v1beta/models/${MODEL}:generateContent?key=${GEMINI_API_KEY}"

# Test with proper role format
REQUEST=$(cat <<'EOF'
{
  "contents": [
    {
      "role": "user",
      "parts": [{"text": "What is 2 + 2?"}]
    },
    {
      "role": "model",
      "parts": [{"text": "2 + 2 = 4"}]
    },
    {
      "role": "user",
      "parts": [{"text": "Now show that in elisp"}]
    }
  ]
}
EOF
)

echo "Request JSON:"
echo "$REQUEST" | jq .

CURL_OPTS=""
if [ -n "$HTTPS_PROXY" ]; then
    CURL_OPTS="-x $HTTPS_PROXY"
fi

echo -e "\nSending request..."
response=$(curl -s $CURL_OPTS -X POST "$URL" \
    -H "Content-Type: application/json" \
    -d "$REQUEST")

echo -e "\nResponse:"
if echo "$response" | jq . > /dev/null 2>&1; then
    echo "$response" | jq .
    
    # Extract the answer
    if echo "$response" | jq -e '.candidates[0].content.parts[0].text' > /dev/null 2>&1; then
        answer=$(echo "$response" | jq -r '.candidates[0].content.parts[0].text')
        echo -e "\n✅ Success! Model response:"
        echo "$answer"
    fi
else
    echo "Raw response: $response"
fi