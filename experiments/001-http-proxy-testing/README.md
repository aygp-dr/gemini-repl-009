# Experiment 001: HTTP Proxy Testing

## Purpose
Test HTTP proxy functionality on port 3129 for potential use with Gemini API requests.

## Background
User has an HTTP proxy running on port 3129. This experiment tests:
1. Port availability and proxy responsiveness
2. Proxy configuration for API requests
3. Integration with Rust's reqwest library

## Hypothesis
The local HTTP proxy on port 3129 can be used to:
- Route Gemini API requests through a proxy
- Enable request/response inspection for debugging
- Potentially cache API responses for development

## Tests Performed

### 1. Port Availability Check
```bash
netstat -an | grep -E "[:.]3129"
```
Result: Port 3129 is LISTENING (proxy is running)

### 2. Basic Proxy Test
```bash
curl -x http://localhost:3129 -I http://example.com
```
Result: (Testing in progress)

### 3. HTTPS Proxy Test
```bash
curl -x http://localhost:3129 -I https://generativelanguage.googleapis.com
```
Result: (To be tested)

## Implementation Notes

For Rust integration, update reqwest client configuration:
```rust
let client = reqwest::Client::builder()
    .proxy(reqwest::Proxy::http("http://localhost:3129")?)
    .proxy(reqwest::Proxy::https("http://localhost:3129")?)
    .timeout(Duration::from_secs(30))
    .build()?;
```

## Environment Variables
Add to `.env` for proxy support:
```bash
HTTP_PROXY=http://localhost:3129
HTTPS_PROXY=http://localhost:3129
```

## Next Steps
1. Test HTTPS tunneling through proxy
2. Verify Gemini API works through proxy
3. Add proxy configuration to main application
4. Consider proxy authentication if needed