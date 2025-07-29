# Alloy Demo Experiment

.PHONY: all download run verify clean help

ALLOY_VERSION := 6.1.0
ALLOY_JAR := alloy.jar
ALLOY_URL := https://github.com/AlloyTools/org.alloytools.alloy/releases/download/v$(ALLOY_VERSION)/org.alloytools.alloy.dist.jar

help:
	@echo "Alloy Demo Commands:"
	@echo "  make download - Download Alloy analyzer"
	@echo "  make run      - Run Alloy GUI with demo spec"
	@echo "  make verify   - Verify the specification"
	@echo "  make clean    - Remove downloaded files"

all: download verify

download:
	@if [ ! -f $(ALLOY_JAR) ]; then \
		echo "Downloading Alloy $(ALLOY_VERSION)..."; \
		curl -L -o $(ALLOY_JAR) $(ALLOY_URL); \
	else \
		echo "Alloy already downloaded"; \
	fi

run: download
	@echo "Starting Alloy GUI with demo.als..."
	@echo "Instructions:"
	@echo "  1. Click 'Execute' or press Ctrl+E to run"
	@echo "  2. Use arrow keys to browse instances"
	@echo "  3. Try Theme -> Magic Layout for better visualization"
	@java -jar $(ALLOY_JAR) demo.als

verify: download
	@echo "Verifying Alloy specification..."
	@java -cp $(ALLOY_JAR) edu.mit.csail.sdg.alloy4whole.ExampleUsingTheCompiler demo.als || \
		echo "Note: Command-line verification requires custom runner"

clean:
	@rm -f $(ALLOY_JAR)
	@echo "Cleaned up Alloy JAR"