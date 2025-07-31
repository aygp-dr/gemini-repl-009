# Evaluation Progress Log

## Strategy
Given the rate limits discovered:
- flash-lite: Higher quota, but lower success rate (48%)
- flash-exp: 10 req/min limit, but higher success rate (76%)

We'll proceed cautiously with chunked evaluation.

## Progress Tracking

### 2025-07-31 Initial Tests
- ✅ Batch 001: Completed multiple runs
  - Run 1: flash-lite: 12/25 (48%)
  - Run 2: flash-lite: 10/25 (40%) - encountered MALFORMED_FUNCTION_CALL
  - Run 3: flash-lite: 12/25 (48%) - Fixed MALFORMED_FUNCTION_CALL with system instruction
  - flash-exp: 19/25 (76%) - hit rate limit after q10
- ⏳ Batch 002: Partial (timed out after 17 questions with 5s delays)
  - Estimated ~8/17 successful based on output

### Key Learning: System Instruction Fixed Format Issue
✅ Adding system instruction solved MALFORMED_FUNCTION_CALL errors
❌ But success rate remains at 48% - model still not calling functions when expected

### Next Steps
1. **Small chunks**: Process 5 batches at a time (125 questions)
2. **Extended delays**: 5s between requests, 60s between chunks
3. **Monitor closely**: Stop if we hit persistent 429 errors
4. **Adaptive strategy**: Adjust based on rate limit responses

### Rate Limit Management
- If 429 errors persist: Back off for longer periods
- Consider time-of-day effects on quotas
- May need to spread evaluation over multiple days

### Backup Plan
If rate limits become prohibitive:
1. Focus on high-value batches (diverse question types)
2. Sample evaluation instead of full 1000
3. Document patterns from partial results

## Observations So Far

### Performance Variability
- flash-lite shows inconsistent results (40-48% success)
- New error type: MALFORMED_FUNCTION_CALL
- Performance seems to degrade with repeated calls

### Timing Challenges
- 5s delays × 25 questions = 2+ minutes per batch
- Process timeouts becoming an issue
- May need to reduce batch sizes further

### Strategic Adjustments Needed
1. Consider smaller batches (10-15 questions)
2. Implement retry logic for malformed calls
3. Add timeout handling in the evaluation runner
4. Focus on getting representative samples rather than full 1000

## Notes
- This is a marathon, not a sprint
- Each batch provides valuable data
- Progress > Perfection
- **Learning**: The evaluation itself is revealing system constraints