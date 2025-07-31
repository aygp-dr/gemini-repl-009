# Actual API Test Results

## First Real API Test - gemini-2.0-flash-lite
- **Date**: 2025-07-31
- **Model**: gemini-2.0-flash-lite
- **Batch**: 001 (25 questions)
- **Results**: 12/25 successful (48.0%)
- **API**: Real Gemini API calls with function calling

## Failure Analysis

### Pattern of Failures
- Questions expecting `list_files`: Often no function call made
- Questions expecting `read_file`: Often no function call made  
- Questions expecting `search_code`: Mixed results
- Non-tool questions: Generally successful (as expected)
- `write_file` questions: Better success rate

### Examples of Failures
- q011: "Read the README.org file..." - Expected `read_file`, got no function call
- q012: "List all Cargo.toml files..." - Expected `list_files`, got no function call
- q020: "List all markdown files..." - Expected `list_files`, got no function call

## Conclusion
The 48% success rate is much lower than the expected ~100% from experiment 023. This suggests:

1. **Different evaluation criteria** - Our test may be stricter
2. **Model behavior change** - The model might have been updated
3. **Prompt engineering needed** - Questions might need clearer tool-calling hints

## Next Steps
- Test with `gemini-2.0-flash-exp` to compare
- Review experiment 023's exact methodology
- Consider prompt adjustments to improve tool-calling rates

## UPDATE: gemini-2.0-flash-exp Test Results
- **Model**: gemini-2.0-flash-exp
- **Results**: 19/25 successful (76.0%) before rate limiting
- **Rate limit**: 10 requests per minute (hit after q10)

### Surprising Finding
**gemini-2.0-flash-exp (76%) outperformed gemini-2.0-flash-lite (48%)!**

This contradicts Issue #24 which reported:
- flash-lite: 100% success
- flash-exp: 15% success

### Possible Explanations
1. **Model updates** - Both models may have been updated since the original tests
2. **Different test methodology** - Our evaluation may differ from experiment 023
3. **Rate limiting impact** - flash-exp has stricter limits (10/min vs higher for flash-lite)

### Rate Limit Differences
- `flash-lite`: Completed 25 questions without rate limiting
- `flash-exp`: Hit 429 errors after 10 questions (10 req/min limit)