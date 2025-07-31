# Experiments Runner
.PHONY: help run-all run-sample test-all clean-all list

help:
	@echo "Experiments Management:"
	@echo "  make list        - List all experiments"
	@echo "  make run-sample  - Run first 5 experiments (quick test)"
	@echo "  make run-all     - Run all experiments sequentially"
	@echo "  make test-all    - Test all experiments"
	@echo "  make clean-all   - Clean all experiment artifacts"

list:
	@echo "Available experiments:"
	@find . -maxdepth 1 -type d -name '[0-9]*' | sort | sed 's|./||'

run-sample:
	@echo "Running sample experiments (first 5)..."
	@for exp in $$(find . -maxdepth 1 -type d -name '[0-9]*' | sort | head -5); do \
		exp_name=$$(basename $$exp); \
		echo ""; \
		echo "=== Running $$exp_name ==="; \
		if [ -f "$$exp/Makefile" ]; then \
			timeout 30 $(MAKE) -C $$exp run || echo "⚠️  $$exp_name failed or timed out"; \
		else \
			echo "⚠️  No Makefile in $$exp_name"; \
		fi; \
	done

run-all:
	@echo "Running all experiments..."
	@for exp in $$(find . -maxdepth 1 -type d -name '[0-9]*' | sort); do \
		exp_name=$$(basename $$exp); \
		echo ""; \
		echo "=== Running $$exp_name ==="; \
		if [ -f "$$exp/Makefile" ]; then \
			timeout 60 $(MAKE) -C $$exp run || echo "⚠️  $$exp_name failed or timed out"; \
		else \
			echo "⚠️  No Makefile in $$exp_name"; \
		fi; \
	done

test-all:
	@echo "Testing all experiments..."
	@for exp in $$(find . -maxdepth 1 -type d -name '[0-9]*' | sort); do \
		exp_name=$$(basename $$exp); \
		echo ""; \
		echo "=== Testing $$exp_name ==="; \
		if [ -f "$$exp/Makefile" ]; then \
			$(MAKE) -C $$exp test || echo "⚠️  $$exp_name tests failed"; \
		else \
			echo "⚠️  No Makefile in $$exp_name"; \
		fi; \
	done

clean-all:
	@echo "Cleaning all experiments..."
	@for exp in $$(find . -maxdepth 1 -type d -name '[0-9]*' | sort); do \
		exp_name=$$(basename $$exp); \
		echo "Cleaning $$exp_name..."; \
		if [ -f "$$exp/Makefile" ]; then \
			$(MAKE) -C $$exp clean || echo "⚠️  $$exp_name clean failed"; \
		fi; \
	done