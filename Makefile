# Gemini REPL Makefile
# Quick install/run targets for development

.PHONY: help install run test clean build test-all test-unit test-integration pre-commit push-all

# Default target
help:
	@echo "Gemini REPL 009 - Available targets:"
	@echo "  make install  - Build and install the REPL"
	@echo "  make run      - Run the REPL"
	@echo "  make test     - Run tests"
	@echo "  make clean    - Clean build artifacts"
	@echo "  make build    - Build debug version"

# Install the REPL (build release and put in path)
install: build
	@echo "Installing Gemini REPL..."
	cargo build --release
	@echo "Binary available at: target/release/gemini-repl"
	@echo "Run with: ./target/release/gemini-repl"

# Run the REPL
run:
	@echo "Running Gemini REPL..."
	cargo run --bin gemini-repl

# Run tests
test: test-unit test-integration

# Run unit tests
test-unit:
	@echo "Running unit tests..."
	cargo test

# Run integration tests
test-integration:
	@echo "Running integration tests..."
	$(MAKE) -C tests/integration test

# Clean build artifacts
clean:
	@echo "Cleaning..."
	cargo clean

# Build debug version
build:
	@echo "Building..."
	cargo build

# Push all changes
push-all:
	@echo "Pushing all changes to GitHub..."
	@git push origin HEAD

# Run all tests
test-all: test-unit test-integration
	@echo "All tests passed!"

.tmp/files-to-prompt.xml: | .tmp ## Collect all files for prompting
	@echo "Collecting files..."
	@timeout 5 time uv run files-to-prompt -c *.org *.md *.toml *.sh Makefile src/*.rs experiments/*/*.org > $@ || true
	@echo "Done: $@"

.tmp: ## Create temp directory
	@install -d $@

# Pre-commit checks
pre-commit:
	@echo "Running pre-commit checks..."
	@cargo fmt --check || (echo "Run 'cargo fmt' to fix formatting" && exit 1)
	@cargo clippy -- -D warnings || (echo "Fix clippy warnings" && exit 1)
	@cargo test || (echo "Unit tests failed" && exit 1)
	@$(MAKE) -C tests/integration test || (echo "Integration tests failed" && exit 1)
	@echo "✅ All pre-commit checks passed!"

# Tool versions
TLA_VERSION := 1.8.0
ALLOY_VERSION := 5.1.0

# Local tool paths
TOOLS_DIR := tools/formal-methods
TLA_JAR := $(TOOLS_DIR)/tla2tools.jar
ALLOY_JAR := $(TOOLS_DIR)/alloy.jar

# Phony targets
.PHONY: release release-patch release-minor release-major

# Default target
all: help

# Create tools directory
$(TOOLS_DIR):
	@mkdir -p $@

# Download TLA+ tools (file target, depends on directory)
$(TLA_JAR): | $(TOOLS_DIR)
	@echo "Downloading TLA+ tools $(TLA_VERSION)..."
	cd $(TOOLS_DIR) && \
		fetch -o tla2tools.jar https://github.com/tlaplus/tlaplus/releases/download/v$(TLA_VERSION)/tla2tools.jar

# Download Alloy analyzer (file target, depends on directory)
$(ALLOY_JAR): | $(TOOLS_DIR)
	@echo "Downloading Alloy analyzer $(ALLOY_VERSION)..."
	cd $(TOOLS_DIR) && \
		fetch -o alloy.jar https://github.com/AlloyTools/org.alloytools.alloy/releases/download/v$(ALLOY_VERSION)/org.alloytools.alloy.dist.jar

# Verify all specifications
verify: verify-tla verify-alloy
	@echo "✅ All specifications verified"

# Verify TLA+ specifications (depends on jar)
verify-tla: $(TLA_JAR)
	@echo "=== Verifying TLA+ Specifications ==="
	@for spec in specs/*.tla; do \
		if [ -f "$$spec" ]; then \
			echo -n "  Checking $$(basename $$spec)... "; \
			if java -cp $(TLA_JAR) tla2sany.SANY "$$spec" >/dev/null 2>&1; then \
				echo "✓"; \
			else \
				echo "✗"; \
				java -cp $(TLA_JAR) tla2sany.SANY "$$spec"; \
				exit 1; \
			fi; \
		fi; \
	done

# Verify Alloy specifications (depends on jar)
verify-alloy: $(ALLOY_JAR)
	@echo "=== Verifying Alloy Specifications ==="
	@for spec in specs/*.als specs/*.alloy; do \
		if [ -f "$$spec" ]; then \
			echo "  Found: $$spec"; \
		fi; \
	done
	@echo "  (Run 'java -jar $(ALLOY_JAR) <spec>' to check individually)"

# Start tmux development dashboard
dashboard:
	./scripts/tmux-dashboard.sh

# Restart dashboard
dashboard-restart:
	./scripts/tmux-dashboard.sh --restart

# Generate ASCII art banner
resources/:
	mkdir -p $@

resources/repl-banner.txt: || resources/
	@echo "Generating REPL banner..."
	@if command -v toilet >/dev/null 2>&1; then \
		toilet -f future "Gemini REPL" > $@; \
		echo "" >> $@; \
		echo "  📝 Logging enabled via GEMINI_LOG_ENABLED" >> $@; \
		echo "  🔍 Type /help for commands" >> $@; \
		echo "" >> $@; \
		echo "✅ Generated banner at $@"; \
	else \
		echo "Warning: toilet not found, creating simple banner"; \
		echo "Gemini REPL" > $@; \
		echo "==========" >> $@; \
		echo "" >> $@; \
		echo "📝 Logging enabled via GEMINI_LOG_ENABLED" >> $@; \
		echo "🔍 Type /help for commands" >> $@; \
		echo "" >> $@; \
	fi

banner: resources/repl-banner.txt

# Release targets
release: release-patch

release-patch: _release-precheck _release-bump-patch _release-create

release-minor: _release-precheck _release-bump-minor _release-create

release-major: _release-precheck _release-bump-major _release-create

# Internal release helpers
_release-bump-patch:
	@echo "📦 Bumping patch version..."
	@npm version patch --no-git-tag-version

_release-bump-minor:
	@echo "📦 Bumping minor version..."
	@npm version minor --no-git-tag-version

_release-bump-major:
	@echo "📦 Bumping major version..."
	@npm version major --no-git-tag-version

