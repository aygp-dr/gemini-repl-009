# Ollama Validation Experiment

.PHONY: all deps init run clean ollama-setup setup test-resp

all: deps init run

# Install dependencies using pip
deps:
	@echo "Installing dependencies..."
	@if command -v python3 >/dev/null; then \
		python3 -m pip install -r requirements.txt; \
	elif command -v python >/dev/null; then \
		python -m pip install -r requirements.txt; \
	else \
		echo "Python not found!"; \
		exit 1; \
	fi

# Initialize environment
init:
	@echo "Initializing environment..."
	@echo "Checking for Ollama..."
	@if ! command -v ollama >/dev/null; then \
		echo "Ollama not found. Install from https://ollama.ai"; \
		echo "Or run: make ollama-setup"; \
	else \
		echo "✓ Ollama is installed"; \
	fi
	@echo "Checking Ollama service..."
	@if curl -s http://localhost:11434/api/tags >/dev/null 2>&1; then \
		echo "✓ Ollama service is running"; \
	else \
		echo "✗ Ollama service not running. Start with: ollama serve"; \
	fi

# Run validation tests
run:
	@echo "Running Ollama validation tests..."
	@if command -v python3 >/dev/null; then \
		python3 validate_ollama.py; \
	else \
		python validate_ollama.py; \
	fi

# Legacy setup command (alias for deps)
setup: deps

# Install and setup Ollama
ollama-setup:
	@echo "Setting up Ollama..."
	@if ! command -v ollama >/dev/null; then \
		echo "Installing Ollama..."; \
		curl -fsSL https://ollama.ai/install.sh | sh; \
	fi
	@echo "Starting Ollama service in background..."
	@nohup ollama serve > ollama.log 2>&1 &
	@echo "Waiting for service to start..."
	@sleep 3
	@echo "Pulling llama3.2 model (this may take a while)..."
	@ollama pull llama3.2
	@echo "✓ Ollama setup complete!"

clean:
	@rm -f ollama_schemas.json ollama.log
	@rm -rf __pycache__ *.egg-info .pytest_cache
	@find . -name "*.pyc" -delete

# Test Ollama response to system info request
test-resp:
	@echo "Testing Ollama response for system info..."
	@echo "show core pkg installs for $$(uname -a)" | ollama run llama3.2 || echo "Ollama not available or model not pulled"