#!/bin/bash
# HTTP Proxy Testing Script

echo "=== HTTP Proxy Test on Port 3129 ==="
echo

# Check if port is listening
echo "1. Checking port 3129 status..."
if netstat -an | grep -E "[:.]3129.*LISTEN" > /dev/null; then
    echo "✓ Port 3129 is listening"
else
    echo "✗ Port 3129 is not listening"
    exit 1
fi
echo

# Test HTTP through proxy
echo "2. Testing HTTP request through proxy..."
if timeout 5 curl -s -x http://localhost:3129 -I http://example.com | grep "HTTP" > /dev/null; then
    echo "✓ HTTP proxy works"
    timeout 5 curl -s -x http://localhost:3129 -I http://example.com | head -3
else
    echo "✗ HTTP proxy failed"
fi
echo

# Test HTTPS through proxy
echo "3. Testing HTTPS request through proxy..."
if timeout 5 curl -s -x http://localhost:3129 -I https://www.google.com | grep "HTTP" > /dev/null; then
    echo "✓ HTTPS proxy works"
    timeout 5 curl -s -x http://localhost:3129 -I https://www.google.com | head -3
else
    echo "✗ HTTPS proxy failed or not supported"
fi
echo

# Test Gemini API endpoint through proxy
echo "4. Testing Gemini API endpoint through proxy..."
GEMINI_ENDPOINT="https://generativelanguage.googleapis.com/v1beta/models"
if timeout 10 curl -s -x http://localhost:3129 -I "$GEMINI_ENDPOINT" | grep "HTTP" > /dev/null; then
    echo "✓ Can reach Gemini API through proxy"
    timeout 10 curl -s -x http://localhost:3129 -I "$GEMINI_ENDPOINT" | head -3
else
    echo "✗ Cannot reach Gemini API through proxy"
fi
echo

# Show proxy environment variables
echo "5. Current proxy environment variables:"
env | grep -i proxy || echo "No proxy variables set"
echo

echo "=== Test Complete ==="