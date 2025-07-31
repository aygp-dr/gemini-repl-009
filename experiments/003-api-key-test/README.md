# Experiment 003: API Key Validation

## Purpose
Validate that the GEMINI_API_KEY works with direct HTTP/curl calls to the Gemini API.

## Background
- API key added to GitHub secrets
- Need to verify key works before building Rust client
- Test basic API functionality

## API Endpoints
- Models list: `https://generativelanguage.googleapis.com/v1beta/models`
- Generate content: `https://generativelanguage.googleapis.com/v1beta/models/{model}:generateContent`

## Tests
1. List available models (GET request)
2. Simple prompt test "What is 2 + 40?"
3. Verify response structure

## Expected Results
- HTTP 200 status
- Valid JSON response
- Correct answer (42) in response