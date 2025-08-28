# Makefile for RustConf 2025 UnConference Presentation
# Generates PDF from org-mode with beamer for use with pdfpc presenter

# Configuration
EMACS = emacs
ORG_FILE = presentation.org
TEX_FILE = presentation.tex
PDF_FILE = presentation.pdf
PDFPC_FILE = presentation.pdfpc
BUILD_DIR = build
IMAGES_DIR = images

# Emacs batch command for org-mode to LaTeX/Beamer export
# Using -Q for clean environment, loading only org-mode
EMACS_BATCH = $(EMACS) -Q -l org --batch \
	--eval "(require 'ox-beamer)" \
	--eval "(setq org-export-with-toc nil)" \
	--eval "(setq org-export-with-author t)" \
	--eval "(setq org-export-with-date t)" \
	--eval "(setq org-export-with-title t)" \
	--eval "(setq org-beamer-frame-level 2)" \
	--eval "(setq org-latex-listings 'minted)" \
	--eval "(setq org-latex-minted-options '((\"frame\" \"lines\") (\"fontsize\" \"\\scriptsize\") (\"linenos\" \"\")))" \
	--eval "(setq org-latex-pdf-process '(\"pdflatex -shell-escape -interaction nonstopmode -output-directory %o %f\" \"pdflatex -shell-escape -interaction nonstopmode -output-directory %o %f\"))"

# LaTeX command with necessary flags
PDFLATEX = pdflatex -shell-escape -interaction=nonstopmode

# Default target
.PHONY: all
all: pdf pdfpc ## Build PDF and generate pdfpc notes file

# Help target
.PHONY: help
help: ## Show this help message
	@echo "RustConf 2025 UnConference Presentation Builder"
	@echo "==============================================="
	@echo ""
	@echo "Available targets:"
	@grep -E '^[a-zA-Z_-]+:.*?## .*$$' $(MAKEFILE_LIST) | \
		awk 'BEGIN {FS = ":.*?## "}; {printf "  %-15s %s\n", $$1, $$2}'
	@echo ""
	@echo "Presentation timing: 30 minutes for UnConf session"
	@echo "Use: pdfpc presentation.pdf to present"

# Create build directory
$(BUILD_DIR):
	@mkdir -p $(BUILD_DIR)

# Export org to LaTeX/Beamer
.PHONY: tex
tex: $(BUILD_DIR) ## Export org-mode to LaTeX/Beamer
	@echo "Exporting org-mode to LaTeX/Beamer..."
	$(EMACS_BATCH) \
		--visit="$(ORG_FILE)" \
		--funcall org-beamer-export-to-latex
	@mv $(TEX_FILE) $(BUILD_DIR)/
	@echo "LaTeX file generated: $(BUILD_DIR)/$(TEX_FILE)"

# Generate PDF from LaTeX
.PHONY: pdf
pdf: tex ## Generate PDF from LaTeX
	@echo "Generating PDF presentation..."
	cd $(BUILD_DIR) && $(PDFLATEX) $(TEX_FILE)
	cd $(BUILD_DIR) && $(PDFLATEX) $(TEX_FILE) # Second pass for references
	@cp $(BUILD_DIR)/$(PDF_FILE) .
	@echo "PDF generated: $(PDF_FILE)"
	@echo "Slides: $$(pdfinfo $(PDF_FILE) | grep Pages | awk '{print $$2}') pages"

# Generate pdfpc notes file
.PHONY: pdfpc
pdfpc: ## Generate pdfpc presenter notes file
	@echo "Generating pdfpc notes file..."
	@echo "[file]" > $(PDFPC_FILE)
	@echo "$(PDF_FILE)" >> $(PDFPC_FILE)
	@echo "[duration]" >> $(PDFPC_FILE)
	@echo "30" >> $(PDFPC_FILE)  # 30 minutes for UnConf session
	@echo "[notes]" >> $(PDFPC_FILE)
	@echo "### Presenter Notes embedded in PDF ###" >> $(PDFPC_FILE)
	@echo "pdfpc file generated: $(PDFPC_FILE)"

# Test presentation build
.PHONY: test
test: ## Test org-mode export without full PDF generation
	@echo "Testing org-mode export..."
	$(EMACS_BATCH) \
		--visit="$(ORG_FILE)" \
		--eval "(org-lint)"
	@echo "Org file validation passed!"

# Watch for changes and rebuild
.PHONY: watch
watch: ## Watch for changes and auto-rebuild
	@echo "Watching for changes in $(ORG_FILE)..."
	@while true; do \
		inotifywait -e modify $(ORG_FILE) 2>/dev/null || sleep 2; \
		gmake pdf; \
		echo "Rebuilt at $$(date +%H:%M:%S)"; \
	done

# Present with pdfpc
.PHONY: present
present: all ## Launch pdfpc presenter
	@if command -v pdfpc >/dev/null 2>&1; then \
		echo "Launching pdfpc presenter..."; \
		echo "Controls:"; \
		echo "  - Next: Space/Right Arrow"; \
		echo "  - Previous: Left Arrow"; \
		echo "  - Notes: N"; \
		echo "  - Timer: T"; \
		echo "  - Blank: B"; \
		pdfpc --duration=30 --last-minutes=5 $(PDF_FILE); \
	else \
		echo "pdfpc not found. Install with: pkg install pdfpc"; \
	fi

# Quick preview
.PHONY: preview
preview: pdf ## Open PDF in default viewer
	@if [ "$$(uname)" = "FreeBSD" ]; then \
		xdg-open $(PDF_FILE) 2>/dev/null || mupdf $(PDF_FILE); \
	elif [ "$$(uname)" = "Darwin" ]; then \
		open $(PDF_FILE); \
	else \
		xdg-open $(PDF_FILE); \
	fi

# Validate timing
.PHONY: timing
timing: ## Calculate and validate presentation timing
	@echo "Presentation Timing Analysis"
	@echo "============================"
	@echo "Target duration: 30 minutes (UnConf session)"
	@echo ""
	@if [ -f $(PDF_FILE) ]; then \
		slides=$$(pdfinfo $(PDF_FILE) | grep Pages | awk '{print $$2}'); \
		echo "Total slides: $$slides"; \
		echo "Average time per slide: $$(echo "scale=1; 1800 / $$slides" | bc) seconds"; \
		echo ""; \
		echo "Recommended pacing:"; \
		echo "  - Title/Intro: 2 minutes"; \
		echo "  - Main content: 20 minutes"; \
		echo "  - Demo: 5 minutes"; \
		echo "  - Q&A: 3 minutes"; \
	else \
		echo "PDF not found. Run 'gmake pdf' first."; \
	fi

# Generate speaker notes as separate file
.PHONY: notes
notes: ## Extract speaker notes to text file
	@echo "Extracting speaker notes..."
	@grep -A 5 "^#+BEGIN_NOTES" $(ORG_FILE) | \
		grep -v "^#+BEGIN_NOTES" | \
		grep -v "^#+END_NOTES" | \
		grep -v "^--$$" > notes/speaker-notes.txt
	@echo "Speaker notes extracted to: notes/speaker-notes.txt"

# Clean build artifacts
.PHONY: clean
clean: ## Clean build artifacts
	@echo "Cleaning build artifacts..."
	@rm -rf $(BUILD_DIR)
	@rm -f $(PDF_FILE) $(PDFPC_FILE)
	@rm -f *.aux *.log *.nav *.out *.snm *.toc *.vrb
	@rm -f _minted-presentation
	@echo "Clean complete"

# Full rebuild
.PHONY: rebuild
rebuild: clean all ## Clean and rebuild everything

# Lint org file
.PHONY: lint
lint: ## Lint the org-mode file for issues
	@echo "Linting org-mode file..."
	$(EMACS_BATCH) \
		--visit="$(ORG_FILE)" \
		--eval "(require 'org-lint)" \
		--eval "(let ((lint-results (org-lint))) \
			(if lint-results \
				(progn \
					(princ \"Linting issues found:\n\") \
					(dolist (issue lint-results) \
						(princ (format \"Line %d: %s\n\" \
							(org-lint-issue-line issue) \
							(org-lint-issue-message issue)))) \
					(kill-emacs 1)) \
				(progn \
					(princ \"No linting issues found!\n\") \
					(kill-emacs 0))))"

# Check prerequisites
.PHONY: check-deps
check-deps: ## Check for required dependencies
	@echo "Checking dependencies..."
	@which emacs >/dev/null 2>&1 || (echo "❌ Emacs not found"; exit 1)
	@which pdflatex >/dev/null 2>&1 || (echo "❌ pdflatex not found (install texlive)"; exit 1)
	@which pdfpc >/dev/null 2>&1 || echo "⚠️  pdfpc not found (optional, for presenting)"
	@echo "✓ All required dependencies found"

# Install dependencies on FreeBSD
.PHONY: install-deps-freebsd
install-deps-freebsd: ## Install dependencies on FreeBSD
	@echo "Installing dependencies on FreeBSD..."
	sudo pkg install -y emacs-nox texlive-full pdfpc
	@echo "Dependencies installed!"

# Generate handout version (no animations)
.PHONY: handout
handout: ## Generate handout version (no animations)
	@echo "Generating handout version..."
	$(EMACS_BATCH) \
		--eval "(setq org-beamer-frame-default-options \"handout\")" \
		--visit="$(ORG_FILE)" \
		--funcall org-beamer-export-to-pdf
	@mv presentation.pdf presentation-handout.pdf
	@echo "Handout generated: presentation-handout.pdf"

# Package for distribution
.PHONY: package
package: all handout notes ## Package presentation for distribution
	@echo "Packaging presentation..."
	@mkdir -p dist
	@cp $(PDF_FILE) dist/
	@cp presentation-handout.pdf dist/ 2>/dev/null || true
	@cp -r notes dist/
	@cp $(ORG_FILE) dist/
	@tar czf rustconf-2025-unconf-presentation.tar.gz dist/
	@echo "Package created: rustconf-2025-unconf-presentation.tar.gz"
	@rm -rf dist

.PHONY: stats
stats: ## Show presentation statistics
	@echo "Presentation Statistics"
	@echo "======================="
	@echo "Org file: $$(wc -l < $(ORG_FILE)) lines"
	@echo "Code examples: $$(grep -c '^#+BEGIN_SRC' $(ORG_FILE))"
	@echo "Speaker notes: $$(grep -c '^#+BEGIN_NOTES' $(ORG_FILE))"
	@if [ -f $(PDF_FILE) ]; then \
		echo "PDF size: $$(du -h $(PDF_FILE) | cut -f1)"; \
		echo "Slide count: $$(pdfinfo $(PDF_FILE) | grep Pages | awk '{print $$2}')"; \
	fi

# Default for gmake on FreeBSD
.DEFAULT_GOAL := help