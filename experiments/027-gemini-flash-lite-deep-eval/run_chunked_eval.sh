#!/bin/bash
# Run evaluation in chunks to avoid rate limiting

set -e

# Source .env from project root
PROJECT_ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
if [ -f "$PROJECT_ROOT/.env" ]; then
    export $(grep -v '^#' "$PROJECT_ROOT/.env" | xargs)
fi

MODEL="${MODEL:-gemini-2.0-flash-lite}"
DELAY="${DELAY:-3}"  # 3 seconds between requests
CHUNK_SIZE="${CHUNK_SIZE:-5}"  # Process 5 batches at a time
CHUNK_DELAY="${CHUNK_DELAY:-60}"  # 60 seconds between chunks

echo "Running chunked evaluation"
echo "Model: $MODEL"
echo "Delay between requests: ${DELAY}s"
echo "Chunk size: $CHUNK_SIZE batches"
echo "Delay between chunks: ${CHUNK_DELAY}s"
echo ""

# Check for API key
if [ -z "$GEMINI_API_KEY" ]; then
    echo "Error: GEMINI_API_KEY environment variable not set"
    exit 1
fi

# Create results directory
mkdir -p results

# Process batches in chunks
for ((start=1; start<=40; start+=CHUNK_SIZE)); do
    end=$((start + CHUNK_SIZE - 1))
    if [ $end -gt 40 ]; then
        end=40
    fi
    
    echo "Processing batches $start-$end..."
    
    for ((batch=start; batch<=end; batch++)); do
        echo "Running batch $batch..."
        cargo run --bin run-eval -- --start-batch $batch --model $MODEL --delay $DELAY
        
        # Small delay between batches within a chunk
        if [ $batch -lt $end ]; then
            sleep 5
        fi
    done
    
    if [ $end -lt 40 ]; then
        echo "Chunk complete. Waiting ${CHUNK_DELAY}s before next chunk..."
        sleep $CHUNK_DELAY
    fi
done

echo ""
echo "Evaluation complete! Run 'make analyze' to see results."