#!/bin/sh
# Capture screenshots of TLA+/Alloy specs in Emacs

set -e

SPEC_FILE="${1:-../../specs/GeminiProtocol.tla}"
OUTPUT_DIR="screenshots"
ATTEMPTS=3

echo "Capturing Emacs screenshot for: $SPEC_FILE"

# Clean up any existing X server
kill $(ps aux | grep 'Xvfb :99' | grep -v grep | awk '{print $2}') 2>/dev/null || true
rm -f /tmp/.X99-lock

# Create output directory
mkdir -p "$OUTPUT_DIR"

# Function to capture screenshot
capture_attempt() {
    local attempt=$1
    echo "Attempt $attempt of $ATTEMPTS"
    
    # Start fresh Xvfb
    Xvfb :99 -screen 0 1400x900x24 &
    XVFB_PID=$!
    export DISPLAY=:99
    sleep 2
    
    # Set nice background
    xsetroot -solid '#2e3440' 2>/dev/null || true
    
    # Start Emacs with the spec file
    echo "Starting Emacs with $SPEC_FILE..."
    timeout 30 emacs -Q -nw "$SPEC_FILE" &
    EMACS_PID=$!
    
    # Wait for Emacs to load
    sleep 5
    
    # Try to find Emacs terminal window
    EMACS_WINDOW=$(xwininfo -root -tree 2>/dev/null | grep -i "emacs\|xterm\|terminal" | head -1 | awk '{print $1}' || true)
    
    if [ -n "$EMACS_WINDOW" ]; then
        echo "Found Emacs window: $EMACS_WINDOW"
        # Capture the specific window
        xwd -id "$EMACS_WINDOW" -out emacs.xwd 2>/dev/null
        magick emacs.xwd "$OUTPUT_DIR/emacs-spec-attempt-$attempt.png"
        rm -f emacs.xwd
    else
        echo "No Emacs window found, capturing full screen"
        import -window root "$OUTPUT_DIR/emacs-spec-attempt-$attempt.png"
    fi
    
    # Also try xterm approach
    echo "Trying xterm approach..."
    xterm -e "timeout 25 emacs -Q -nw '$SPEC_FILE'" &
    XTERM_PID=$!
    sleep 3
    
    # Capture xterm
    XTERM_WINDOW=$(xwininfo -root -tree 2>/dev/null | grep -i "xterm" | head -1 | awk '{print $1}' || true)
    if [ -n "$XTERM_WINDOW" ]; then
        echo "Found xterm window: $XTERM_WINDOW"
        xwd -id "$XTERM_WINDOW" -out xterm.xwd 2>/dev/null
        magick xterm.xwd "$OUTPUT_DIR/xterm-emacs-attempt-$attempt.png"
        rm -f xterm.xwd
    fi
    
    # Full desktop capture as backup
    import -window root "$OUTPUT_DIR/desktop-attempt-$attempt.png"
    
    # Cleanup this attempt
    kill $EMACS_PID $XTERM_PID 2>/dev/null || true
    kill $XVFB_PID 2>/dev/null || true
    sleep 1
}

# Try multiple times until stable
for i in $(seq 1 $ATTEMPTS); do
    capture_attempt $i
    sleep 2
done

echo "\nScreenshots captured:"
ls -lh "$OUTPUT_DIR"/*.png

echo "\nAttempting to identify best screenshot..."
for png in "$OUTPUT_DIR"/*.png; do
    size=$(stat -f%z "$png" 2>/dev/null || stat -c%s "$png")
    echo "$png: ${size} bytes"
done