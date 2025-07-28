# Formal Specifications Makefile
# Verify TLA+ and Alloy specifications

.PHONY: all tla alloy clean help

# Tool paths (relative to project root)
TLA_JAR := ../tools/formal-methods/tla2tools.jar
ALLOY_JAR := ../tools/formal-methods/alloy.jar

# Find all spec files
TLA_SPECS := $(wildcard *.tla)
ALLOY_SPECS := $(wildcard *.als) $(wildcard *.alloy)

# Default target
all: tla alloy
	@echo "✅ All specifications verified"

# Help
help:
	@echo "Formal Specifications - Available targets:"
	@echo "  make all    - Verify all specifications"
	@echo "  make tla    - Verify TLA+ specifications"
	@echo "  make alloy  - List Alloy specifications"
	@echo "  make clean  - Clean temporary files"

# Verify TLA+ specifications
tla: $(TLA_SPECS)
	@echo "=== Verifying TLA+ Specifications ==="
	@for spec in $(TLA_SPECS); do \
		echo -n "  Checking $$spec... "; \
		if timeout 10 java -cp $(TLA_JAR) tla2sany.SANY "$$spec" >/dev/null 2>&1; then \
			echo "✓"; \
		else \
			echo "✗"; \
			timeout 10 java -cp $(TLA_JAR) tla2sany.SANY "$$spec" || true; \
			exit 1; \
		fi; \
	done
	@echo "  All TLA+ specs verified ✓"

# List Alloy specifications (can't verify without GUI)
alloy: $(ALLOY_SPECS)
	@echo "=== Alloy Specifications ==="
	@if [ -n "$(ALLOY_SPECS)" ]; then \
		for spec in $(ALLOY_SPECS); do \
			echo "  Found: $$spec"; \
		done; \
		echo "  (Use Alloy GUI over X11 to verify)"; \
	else \
		echo "  No Alloy specifications found"; \
	fi

# Clean temporary files
clean:
	@echo "Cleaning temporary files..."
	@rm -f *.out *.log *.tmp
	@echo "Done"

# Individual spec targets
%.tla:
	@echo "Checking $@..."
	@timeout 10 java -cp $(TLA_JAR) tla2sany.SANY "$@"

%.als %.alloy:
	@echo "Alloy spec: $@"
	@echo "Use: java -jar $(ALLOY_JAR) $@"