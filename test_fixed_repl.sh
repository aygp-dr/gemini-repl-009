#!/bin/bash
# Test the fixed REPL with context

echo -e "\033[32m🟢 Testing Fixed Gemini REPL\033[0m"
echo "This will test:"
echo "1. Basic math question"
echo "2. Follow-up using context (show in elisp)"
echo "3. Another follow-up (show in bc)"
echo

# Load environment
. .env

# Create test input
cat > test_input.txt << 'EOF'
what is 2 + 2
show that in elisp
show how to do that using bc
/context
/exit
EOF

echo "Running REPL with live API..."
cargo run < test_input.txt

# Cleanup
rm -f test_input.txt

echo -e "\n\033[32m✅ Test complete\033[0m"