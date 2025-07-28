#!/bin/bash
# Run all baseline tests and generate report

set -e

# Colors
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

echo -e "${BLUE}=== Gemini REPL Baseline Test Suite ===${NC}"
echo "Running comprehensive tests before tool calling implementation"
echo "Date: $(date)"
echo

# Create test directories
mkdir -p tests/baseline
mkdir -p test-results

# Make scripts executable
chmod +x tests/baseline/*.sh

# Run conversation tests
echo -e "${YELLOW}Running conversation tests...${NC}"
if bash tests/baseline/conversation_tests.sh > test-results/conversation_tests.log 2>&1; then
    echo -e "${GREEN}✓ Conversation tests passed${NC}"
    CONV_PASS=1
else
    echo -e "${RED}✗ Conversation tests failed${NC}"
    CONV_PASS=0
fi

# Document tool calling scenarios
echo -e "${YELLOW}Documenting tool calling scenarios...${NC}"
if bash tests/baseline/tool_calling_scenarios.sh > test-results/tool_scenarios.log 2>&1; then
    echo -e "${GREEN}✓ Tool scenarios documented${NC}"
    TOOL_DOC=1
else
    echo -e "${RED}✗ Tool scenario documentation failed${NC}"
    TOOL_DOC=0
fi

# Run existing integration tests
echo -e "${YELLOW}Running integration tests...${NC}"
if gmake -C tests/integration test > test-results/integration_tests.log 2>&1; then
    echo -e "${GREEN}✓ Integration tests passed${NC}"
    INT_PASS=1
else
    echo -e "${RED}✗ Integration tests failed${NC}"
    INT_PASS=0
fi

# Test API with real calls
echo -e "${YELLOW}Testing live API calls...${NC}"
if bash test_fixed_repl.sh > test-results/api_test.log 2>&1; then
    echo -e "${GREEN}✓ API tests passed${NC}"
    API_PASS=1
else
    echo -e "${RED}✗ API tests failed${NC}"
    API_PASS=0
fi

# Generate test report
echo -e "\n${BLUE}=== Test Report ===${NC}"
cat > test-results/baseline_report.md << EOF
# Baseline Test Report

Generated: $(date)

## Test Results

| Test Suite | Status | Details |
|------------|--------|---------|
| Conversation Tests | $([ $CONV_PASS -eq 1 ] && echo '✓ Pass' || echo '✗ Fail') | Multi-step reasoning, code generation, context preservation |
| Tool Scenarios | $([ $TOOL_DOC -eq 1 ] && echo '✓ Documented' || echo '✗ Failed') | Future tool calling scenarios documented |
| Integration Tests | $([ $INT_PASS -eq 1 ] && echo '✓ Pass' || echo '✗ Fail') | Basic REPL functionality with expect |
| API Tests | $([ $API_PASS -eq 1 ] && echo '✓ Pass' || echo '✗ Fail') | Live Gemini API interaction |

## Current Capabilities

- ✓ Multi-turn conversations with context
- ✓ Role-based message formatting (user/model)
- ✓ NOOP mode for testing
- ✓ Basic REPL commands (/help, /exit, /context)
- ✓ Error handling and recovery
- ✓ Proxy support

## Planned Tool Calling Features

- ❌ list_files: List directory contents with patterns
- ❌ read_file: Read file contents safely
- ❌ write_file: Create/update files with backups
- ❌ Tool declaration in API requests
- ❌ Function call parsing and execution
- ❌ Sandboxed file operations

## Next Steps

1. Implement core tool infrastructure (Tool trait, ToolRegistry)
2. Create file system tools with security measures
3. Integrate tool declarations with Gemini API
4. Add tool execution to conversation loop
5. Comprehensive security testing

See [tool-calling-spec.md](../specs/tool-calling-spec.md) for detailed implementation plan.
EOF

echo -e "\n${GREEN}Test report generated: test-results/baseline_report.md${NC}"

# Display summary
echo -e "\n${BLUE}=== Summary ===${NC}"
TOTAL_PASS=$((CONV_PASS + TOOL_DOC + INT_PASS + API_PASS))
echo "Tests passed: $TOTAL_PASS/4"

if [ $TOTAL_PASS -eq 4 ]; then
    echo -e "${GREEN}All baseline tests passed! Ready for tool calling implementation.${NC}"
    exit 0
else
    echo -e "${YELLOW}Some tests failed. Check test-results/ for details.${NC}"
    exit 1
fi