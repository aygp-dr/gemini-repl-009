#!/bin/bash
# Test live API with context

echo -e "\033[32m🟢 Testing Gemini REPL with live API\033[0m"
echo "This will demonstrate:"
echo "1. Basic math question"
echo "2. Follow-up using context"
echo "3. Context command to show history"
echo

# Load environment variables
. .env

# Create test input
cat > test_api_input.txt << EOF
What is 2 + 40?
Can you show me how to calculate that in Python?
/context
/exit
EOF

echo "Running REPL with API..."
cargo run < test_api_input.txt

# Cleanup
rm -f test_api_input.txt

echo -e "\n\033[32m✅ API test complete\033[0m"