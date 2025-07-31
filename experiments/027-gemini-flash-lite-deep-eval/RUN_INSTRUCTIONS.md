# Running the Gemini Flash Lite Evaluation

## Prerequisites
1. Set your Gemini API key:
   ```bash
   export GEMINI_API_KEY="your-api-key-here"
   ```

2. Build the evaluation tool:
   ```bash
   cd experiments/027-gemini-flash-lite-deep-eval
   cargo build --release --bin run-eval
   ```

## Running the Evaluation

### Option 1: Test Single Batch (Recommended First)
```bash
make test-small
```
This runs batch 001 (25 questions) with 2-second delays between requests.

### Option 2: Run in Chunks (Recommended for Full Test)
```bash
make test-chunked
```
This runs all 40 batches in chunks of 5, with:
- 3-second delays between requests
- 5-second delays between batches
- 60-second delays between chunks

### Option 3: Custom Parameters
```bash
# Run specific batches with custom delays
./run_chunked_eval.sh
# Or with environment variables:
CHUNK_SIZE=3 CHUNK_DELAY=120 DELAY=5 ./run_chunked_eval.sh
```

### Option 4: Test Different Models
```bash
# Test gemini-2.0-flash-lite (expected ~100% success)
make test-small MODEL=gemini-2.0-flash-lite

# Test gemini-2.0-flash-exp (expected ~15% success based on issue #24)
make test-small MODEL=gemini-2.0-flash-exp
```

## Analyzing Results
```bash
make analyze
```

## Expected Outcomes
- `gemini-2.0-flash-lite`: Near 100% success rate for function calling
- `gemini-2.0-flash-exp`: ~15% success rate (confirming regression)

## Rate Limiting Strategy
The chunked approach processes 125 questions per chunk (5 batches × 25 questions) with a 1-minute cooldown between chunks. This should stay well within API rate limits while completing the full 1000-question evaluation in approximately 40 minutes.