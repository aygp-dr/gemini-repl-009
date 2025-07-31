# Evaluation Progress Log

## Strategy
Given the rate limits discovered:
- flash-lite: Higher quota, but lower success rate (48%)
- flash-exp: 10 req/min limit, but higher success rate (76%)

We'll proceed cautiously with chunked evaluation.

## Progress Tracking

### 2025-07-31 Initial Tests
- ✅ Batch 001: Completed
  - flash-lite: 12/25 (48%)
  - flash-exp: 19/25 (76%) - hit rate limit after q10

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

## Notes
- This is a marathon, not a sprint
- Each batch provides valuable data
- Progress > Perfection