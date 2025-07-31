# Gemini 2.0 Flash Lite Deep Evaluation Experiment

## Overview

This experiment conducts a comprehensive evaluation of `gemini-2.0-flash-lite`'s function calling capabilities using 1000 diverse questions across multiple programming domains.

## Motivation

Issue #24 revealed a critical regression where `gemini-2.0-flash-exp` showed only 15% success rate in function calling, compared to 100% success with `gemini-2.0-flash-lite` (per experiment 023). This deep evaluation validates the performance of the recommended model.

## Evaluation Design

### Question Distribution
- **Total Questions**: 1000 (40 batches × 25 questions)
- **Non-tool Questions**: 20% (theoretical/conceptual)
- **Tool-calling Questions**: 80%
  - `list_files`: Directory exploration
  - `read_file`: Content examination
  - `search_code`: Pattern finding
  - `write_file`: File creation/modification

### Coverage Areas
- **Languages**: Rust, Go, Python, Ruby, Clojure, Scheme, JavaScript/TypeScript, C/C++
- **Domains**: AI/ML, Systems Programming, Web Development, Formal Methods, DevOps
- **Project Types**: CLI tools, Web servers, ML experiments, System utilities

## Usage

```bash
# Generate evaluation questions (already completed)
make generate-evals

# Test a single batch
make test-batch BATCH_SIZE=25

# Run full evaluation with rate limiting
make test-full RATE_LIMIT_DELAY=2

# Analyze results
make analyze
```

## Rate Limiting

To avoid API rate limits, the evaluation runner includes:
- Configurable delay between API calls (default: 2 seconds)
- Batch processing with progress tracking
- Automatic result persistence

## Results Storage

Results are saved in `results/` with timestamps:
- Individual batch results: `results_batch_XXX_TIMESTAMP.json`
- Aggregated analysis available via `make analyze`

## Next Steps

1. Implement actual Gemini API integration in `run_eval.rs`
2. Run full evaluation with proper API credentials
3. Compare results against experiment 025 baseline
4. Generate statistical analysis report