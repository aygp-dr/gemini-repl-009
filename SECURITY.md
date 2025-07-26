# Security Policy

## Supported Versions

| Version | Supported          |
| ------- | ------------------ |
| 0.1.x   | :white_check_mark: |

## Reporting a Vulnerability

We take security seriously. If you discover a security vulnerability, please follow these steps:

1. **Do NOT** open a public issue
2. Email security details to: security@example.com
3. Include:
   - Description of the vulnerability
   - Steps to reproduce
   - Potential impact
   - Suggested fix (if any)

## Security Measures

### API Key Protection
- Never commit API keys to the repository
- Use environment variables or secure config files
- API keys are masked in logs and error messages

### File System Security
- All file operations are sandboxed to workspace directory
- Path traversal attacks are prevented
- Symlinks are not followed
- File size limits are enforced

### Command Execution
- Commands are executed in a sandboxed environment
- Only whitelisted commands are allowed
- Input sanitization is performed

### Dependencies
- Regular dependency updates
- Security audit with `cargo audit`
- Minimal dependency footprint

## Best Practices for Users

1. **API Keys**:
   - Store in `.env` file (git-ignored)
   - Use environment variables
   - Rotate keys regularly

2. **File Access**:
   - Be cautious with tool permissions
   - Review file operations
   - Use read-only mode when possible

3. **Updates**:
   - Keep the tool updated
   - Monitor security advisories
   - Update dependencies regularly