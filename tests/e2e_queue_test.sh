#!/usr/bin/env bash
# E2E test for file-based queue communication
#
# This test simulates how another agent (like Claude Code) would
# communicate with the gemini-repl via the queue system.

set -e

QUEUE_DIR="${GEMINI_REPL_HOME:-$HOME/.gemini-repl}/queues"
INPUT_DIR="$QUEUE_DIR/input"
OUTPUT_DIR="$QUEUE_DIR/output"
ARCHIVE_DIR="$QUEUE_DIR/archive"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

echo "=== Gemini REPL Queue E2E Test ==="
echo ""

# Ensure directories exist
mkdir -p "$INPUT_DIR" "$OUTPUT_DIR" "$ARCHIVE_DIR"

# Clean up any old test files
rm -f "$INPUT_DIR"/test-*.json
rm -f "$OUTPUT_DIR"/test-*.json

# Test 1: Submit a prompt request
echo -e "${YELLOW}Test 1: Submit prompt request${NC}"
REQUEST_ID="test-prompt-$(date +%s)"
cat > "$INPUT_DIR/$REQUEST_ID.json" << EOF
{
  "id": "$REQUEST_ID",
  "type": "prompt",
  "content": "What is 2 + 2?",
  "context": null,
  "created_at": "$(date -u +%Y-%m-%dT%H:%M:%SZ)"
}
EOF

if [ -f "$INPUT_DIR/$REQUEST_ID.json" ]; then
    echo -e "${GREEN}✓ Prompt request created: $REQUEST_ID${NC}"
else
    echo -e "${RED}✗ Failed to create prompt request${NC}"
    exit 1
fi

# Test 2: Submit a command request
echo -e "${YELLOW}Test 2: Submit command request${NC}"
CMD_REQUEST_ID="test-cmd-$(date +%s)"
cat > "$INPUT_DIR/$CMD_REQUEST_ID.json" << EOF
{
  "id": "$CMD_REQUEST_ID",
  "type": "command",
  "content": "/help",
  "context": null,
  "created_at": "$(date -u +%Y-%m-%dT%H:%M:%SZ)"
}
EOF

if [ -f "$INPUT_DIR/$CMD_REQUEST_ID.json" ]; then
    echo -e "${GREEN}✓ Command request created: $CMD_REQUEST_ID${NC}"
else
    echo -e "${RED}✗ Failed to create command request${NC}"
    exit 1
fi

# Test 3: Submit a ping request
echo -e "${YELLOW}Test 3: Submit ping request${NC}"
PING_REQUEST_ID="test-ping-$(date +%s)"
cat > "$INPUT_DIR/$PING_REQUEST_ID.json" << EOF
{
  "id": "$PING_REQUEST_ID",
  "type": "ping",
  "content": "",
  "context": null,
  "created_at": "$(date -u +%Y-%m-%dT%H:%M:%SZ)"
}
EOF

if [ -f "$INPUT_DIR/$PING_REQUEST_ID.json" ]; then
    echo -e "${GREEN}✓ Ping request created: $PING_REQUEST_ID${NC}"
else
    echo -e "${RED}✗ Failed to create ping request${NC}"
    exit 1
fi

# Test 4: Submit request with context
echo -e "${YELLOW}Test 4: Submit request with context${NC}"
CTX_REQUEST_ID="test-ctx-$(date +%s)"
cat > "$INPUT_DIR/$CTX_REQUEST_ID.json" << EOF
{
  "id": "$CTX_REQUEST_ID",
  "type": "prompt",
  "content": "Explain this code",
  "context": {
    "file": "src/main.rs",
    "language": "rust",
    "selection": "fn main() { println!(\"Hello\"); }"
  },
  "created_at": "$(date -u +%Y-%m-%dT%H:%M:%SZ)"
}
EOF

if [ -f "$INPUT_DIR/$CTX_REQUEST_ID.json" ]; then
    echo -e "${GREEN}✓ Context request created: $CTX_REQUEST_ID${NC}"
else
    echo -e "${RED}✗ Failed to create context request${NC}"
    exit 1
fi

# Verify all requests in queue
echo ""
echo -e "${YELLOW}Verifying queue state...${NC}"
QUEUE_COUNT=$(ls -1 "$INPUT_DIR"/*.json 2>/dev/null | wc -l)
echo "Requests in queue: $QUEUE_COUNT"

if [ "$QUEUE_COUNT" -eq 4 ]; then
    echo -e "${GREEN}✓ All 4 test requests in queue${NC}"
else
    echo -e "${RED}✗ Expected 4 requests, found $QUEUE_COUNT${NC}"
    exit 1
fi

# List queue contents
echo ""
echo "Queue contents:"
for f in "$INPUT_DIR"/*.json; do
    if [ -f "$f" ]; then
        id=$(jq -r '.id' "$f" 2>/dev/null || echo "parse-error")
        type=$(jq -r '.type' "$f" 2>/dev/null || echo "parse-error")
        echo "  - $id ($type)"
    fi
done

# Test 5: Simulate response writing
echo ""
echo -e "${YELLOW}Test 5: Simulate response writing${NC}"
cat > "$OUTPUT_DIR/$REQUEST_ID.json" << EOF
{
  "id": "$REQUEST_ID",
  "status": "success",
  "content": "2 + 2 = 4",
  "error": null,
  "processed_at": "$(date -u +%Y-%m-%dT%H:%M:%SZ)"
}
EOF

if [ -f "$OUTPUT_DIR/$REQUEST_ID.json" ]; then
    echo -e "${GREEN}✓ Response written successfully${NC}"
    echo "Response content:"
    jq '.' "$OUTPUT_DIR/$REQUEST_ID.json"
else
    echo -e "${RED}✗ Failed to write response${NC}"
    exit 1
fi

# Test 6: Simulate archiving
echo ""
echo -e "${YELLOW}Test 6: Simulate request archiving${NC}"
mv "$INPUT_DIR/$REQUEST_ID.json" "$ARCHIVE_DIR/"

if [ -f "$ARCHIVE_DIR/$REQUEST_ID.json" ] && [ ! -f "$INPUT_DIR/$REQUEST_ID.json" ]; then
    echo -e "${GREEN}✓ Request archived successfully${NC}"
else
    echo -e "${RED}✗ Archiving failed${NC}"
    exit 1
fi

# Cleanup
echo ""
echo -e "${YELLOW}Cleaning up test files...${NC}"
rm -f "$INPUT_DIR"/test-*.json
rm -f "$OUTPUT_DIR"/test-*.json
rm -f "$ARCHIVE_DIR"/test-*.json
echo -e "${GREEN}✓ Cleanup complete${NC}"

echo ""
echo -e "${GREEN}=== All E2E Queue Tests Passed ===${NC}"
echo ""
echo "Queue directory: $QUEUE_DIR"
echo "  input/:   Drop request JSON files here"
echo "  output/:  Responses appear here"
echo "  archive/: Processed requests are moved here"
