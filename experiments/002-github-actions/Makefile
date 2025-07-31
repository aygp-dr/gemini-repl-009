# Makefile for GitHub Actions Experiment

.PHONY: run status workflows secrets help

# Default target
help:
	@echo "GitHub Actions Experiment Commands:"
	@echo "  make run       - Check all GitHub Actions status"
	@echo "  make status    - Show workflow runs"
	@echo "  make workflows - List available workflows"
	@echo "  make secrets   - List repository secrets"

# Main run command - check everything
run: status workflows secrets
	@echo "=== GitHub Actions Status Complete ==="

# Check workflow run status
status:
	@echo "=== Recent Workflow Runs ==="
	@gh run list -R aygp-dr/gemini-repl-009 --limit 5 2>/dev/null || echo "No workflow runs yet"
	@echo

# List workflows
workflows:
	@echo "=== Available Workflows ==="
	@gh workflow list -R aygp-dr/gemini-repl-009 2>/dev/null || echo "No workflows defined yet"
	@echo

# List secrets (names only, not values)
secrets:
	@echo "=== Repository Secrets ==="
	@gh secret list -R aygp-dr/gemini-repl-009 2>/dev/null || echo "No secrets configured"
	@echo

# Create basic CI workflow
create-ci:
	@echo "Creating basic CI workflow..."
	@mkdir -p ../../.github/workflows
	@cp ci.yml ../../.github/workflows/ 2>/dev/null || echo "ci.yml template not found"
	@echo "Workflow created at .github/workflows/ci.yml"

# Log workflow activity
log:
	@echo "=== Logging GitHub Actions Activity ===" >> activity.log
	@date >> activity.log
	@echo "Workflow Runs:" >> activity.log
	@gh run list -R aygp-dr/gemini-repl-009 --limit 10 >> activity.log 2>&1
	@echo "" >> activity.log
	@echo "Activity logged to activity.log"