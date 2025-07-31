# Experiment 004: Rust API Client

## Purpose
Test Gemini API integration using Rust with reqwest client.

## Features
- Structured API requests/responses with serde
- Proxy support via HTTPS_PROXY
- Command-line interface with clap
- Proper error handling

## Usage
```bash
# With real API key
gmake -C experiments/004-rust-api-client run

# Or directly
cd experiments/004-rust-api-client
cargo run -- --proxy --api-key YOUR_KEY
```

## Test Case
- Sends "What is 2 + 40? Just give the number."
- Expects response containing "42"
- Uses proxy if configured

## Next Steps
1. Validate with real API key
2. Add streaming support
3. Integrate into main application