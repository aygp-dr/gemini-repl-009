# Emacs Specification Screenshots

.PHONY: all capture-tla capture-alloy clean help

help:
	@echo "Emacs Spec Screenshot Commands:"
	@echo "  make capture-tla   - Capture TLA+ spec in Emacs"
	@echo "  make capture-alloy - Capture Alloy spec in Emacs"
	@echo "  make clean         - Remove screenshots"

capture-tla:
	@echo "Capturing TLA+ specification..."
	@chmod +x capture-emacs-spec.sh
	@./capture-emacs-spec.sh ../../specs/GeminiProtocol.tla

capture-alloy:
	@echo "Capturing Alloy specification..."
	@chmod +x capture-emacs-spec.sh
	@./capture-emacs-spec.sh ../../specs/FrequencyAnalysis.als

all: capture-tla capture-alloy

clean:
	@rm -rf screenshots/
	@echo "Screenshots removed"