#!/bin/bash
# Baseline tests for current conversation capabilities

set -e

# Colors for output
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Load environment
. ../../.env

echo -e "${GREEN}=== Gemini REPL Baseline Tests ===${NC}"
echo "Testing current conversation capabilities before tool calling"
echo

# Test 1: Multi-step reasoning
echo -e "${YELLOW}Test 1: Multi-step reasoning${NC}"
cat > test1_input.txt << 'EOF'
I have 5 apples. I eat 2 and buy 3 more. How many do I have?
Now if I give half to my friend, how many do I have left?
What if my friend gives me back 1 apple?
/context
/exit
EOF

echo "Expected: Context preservation across mathematical reasoning"
timeout 30 cargo run --quiet < test1_input.txt > test1_output.txt 2>&1
if grep -q "4" test1_output.txt && grep -q "3" test1_output.txt; then
    echo -e "${GREEN}✓ Multi-step reasoning test passed${NC}"
else
    echo -e "${RED}✗ Multi-step reasoning test failed${NC}"
    cat test1_output.txt
fi
echo

# Test 2: Code generation and explanation
echo -e "${YELLOW}Test 2: Code generation across languages${NC}"
cat > test2_input.txt << 'EOF'
Write a function to calculate fibonacci numbers
Now show me the same function in Python
Can you optimize the Python version?
Explain the time complexity
/exit
EOF

echo "Expected: Code generation with context about previous implementations"
timeout 30 cargo run --quiet < test2_input.txt > test2_output.txt 2>&1
if grep -q "def" test2_output.txt && grep -q "fibonacci" test2_output.txt; then
    echo -e "${GREEN}✓ Code generation test passed${NC}"
else
    echo -e "${RED}✗ Code generation test failed${NC}"
fi
echo

# Test 3: Context limits
echo -e "${YELLOW}Test 3: Context window handling${NC}"
cat > test3_input.txt << 'EOF'
Tell me about the solar system
What did I just ask you about?
Describe Mars in detail
What was my first question?
/context
/exit
EOF

echo "Expected: Maintains context of previous questions"
timeout 30 cargo run --quiet < test3_input.txt > test3_output.txt 2>&1
if grep -q "solar system" test3_output.txt && grep -q "first question" test3_output.txt; then
    echo -e "${GREEN}✓ Context window test passed${NC}"
else
    echo -e "${RED}✗ Context window test failed${NC}"
fi
echo

# Test 4: Error recovery
echo -e "${YELLOW}Test 4: Error handling and recovery${NC}"
cat > test4_input.txt << 'EOF'
/unknown-command
Hello after error
/help
/exit
EOF

echo "Expected: Graceful error handling and continued operation"
timeout 30 cargo run --quiet < test4_input.txt > test4_output.txt 2>&1
if grep -q "Unknown command" test4_output.txt && grep -q "Available commands" test4_output.txt; then
    echo -e "${GREEN}✓ Error recovery test passed${NC}"
else
    echo -e "${RED}✗ Error recovery test failed${NC}"
fi
echo

# Cleanup
rm -f test*_input.txt test*_output.txt

echo -e "${GREEN}=== Baseline Tests Complete ===${NC}"