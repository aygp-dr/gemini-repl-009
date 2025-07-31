# GitHub Secrets Setup

## Adding Secrets via GitHub CLI

```bash
# Set GEMINI_API_KEY secret for the repository
gh secret set GEMINI_API_KEY -b "your-actual-api-key-here" -R aygp-dr/gemini-repl-009

# Or read from file (more secure)
echo "your-actual-api-key-here" > /tmp/gemini-key.txt
gh secret set GEMINI_API_KEY < /tmp/gemini-key.txt -R aygp-dr/gemini-repl-009
rm /tmp/gemini-key.txt

# List secrets to verify
gh secret list -R aygp-dr/gemini-repl-009

# Set additional secrets if needed
gh secret set GEMINI_MODEL -b "gemini-1.5-flash" -R aygp-dr/gemini-repl-009
```

## Using Secrets in GitHub Actions

```yaml
env:
  GEMINI_API_KEY: ${{ secrets.GEMINI_API_KEY }}
  GEMINI_MODEL: ${{ secrets.GEMINI_MODEL || 'gemini-1.5-flash' }}
```

## Local Development

1. Copy `.env.example` to `.env`
2. Add your API key to `.env`
3. Use `direnv allow` to load environment

## Security Notes

- Never commit API keys to the repository
- Use `gh secret` command for GitHub Actions
- Keep `.env` files local only
- Rotate keys regularly