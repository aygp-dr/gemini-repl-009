# Alloy Screenshot Capture Experiment

.PHONY: all capture deps clean help

help:
	@echo "Alloy Screenshot Commands:"
	@echo "  make deps    - Check/install dependencies"
	@echo "  make capture - Capture Alloy screenshots"
	@echo "  make clean   - Remove screenshots"

deps:
	@echo "Checking dependencies..."
	@which Xvfb >/dev/null || (echo "Installing Xvfb..." && sudo pkg install -y xorg-vfbserver)
	@which xwininfo >/dev/null || (echo "Installing x11 utils..." && sudo pkg install -y xwininfo)
	@which xdotool >/dev/null || (echo "Installing xdotool..." && sudo pkg install -y xdotool)
	@which import >/dev/null || (echo "Installing ImageMagick..." && sudo pkg install -y ImageMagick7)
	@which magick >/dev/null || (echo "Installing ImageMagick..." && sudo pkg install -y ImageMagick7)
	@echo "Dependencies checked"

capture: deps
	@echo "Ensuring Alloy is downloaded..."
	@$(MAKE) -C ../016-alloy-demo download
	@echo "Capturing screenshots..."
	@chmod +x capture-alloy.sh
	@./capture-alloy.sh

clean:
	@rm -rf screenshots/
	@echo "Screenshots removed"

all: capture