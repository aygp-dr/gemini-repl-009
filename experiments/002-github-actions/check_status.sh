#!/bin/bash
# Quick status check script for GitHub Actions

echo "=== GitHub Actions Status Check ==="
echo "Repository: aygp-dr/gemini-repl-009"
echo "Time: $(date)"
echo

# Check if gh CLI is available
if ! command -v gh &> /dev/null; then
    echo "Error: GitHub CLI (gh) not found"
    exit 1
fi

# Check authentication
echo "Checking GitHub authentication..."
if gh auth status &> /dev/null; then
    echo "✓ Authenticated"
else
    echo "✗ Not authenticated. Run: gh auth login"
    exit 1
fi
echo

# Quick status checks
echo "Repository Secrets:"
gh secret list -R aygp-dr/gemini-repl-009 2>/dev/null | grep -E "GEMINI|CARGO" || echo "No relevant secrets found"
echo

echo "Latest Workflow Runs:"
gh run list -R aygp-dr/gemini-repl-009 --limit 3 2>/dev/null || echo "No runs yet"
echo

echo "Branch Protection:"
gh api repos/aygp-dr/gemini-repl-009/branches/main/protection 2>/dev/null | jq -r '.url' || echo "No branch protection set"