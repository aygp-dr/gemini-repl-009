# Lessons Learned: API Specification Gap

## The Issue
When implementing the Gemini API client, we encountered the error:
```
Error: API Error (400): Please use a valid role: user, model.
```

## Root Cause
We failed to check the API specification before implementation. The Gemini API requires:
- Each content in a conversation must have a `role` field
- Valid roles are `"user"` and `"model"` (not `"assistant"` like OpenAI)

## Why We Missed It
1. **No formal spec review**: Jumped directly to implementation
2. **Assumptions from other APIs**: Used OpenAI patterns without verification
3. **Single message tests worked**: The API doesn't require roles for single messages

## What We Should Have Done
1. **Check existing implementations**: Other gemini-repl versions had TLA+/Alloy specs
2. **Review API documentation**: Official docs clearly show the role requirement
3. **Test multi-turn conversations early**: Would have caught the error sooner

## Improvements Made
1. Created `specs/gemini-api-spec.md` documenting the API format
2. Added validation tests before implementation
3. Fixed the Content struct to include role field

## Future Prevention
- Always create/review specs before implementing external APIs
- Test edge cases (multi-turn conversations) not just happy path
- Use formal verification tools when available (TLA+, Alloy)
- Check sibling projects for existing specifications