#!/bin/sh
# Simple Emacs TLA+ screenshot capture

set -e

SPEC_FILE="${1:-../../specs/GeminiProtocol.tla}"

echo "Simple Emacs capture for: $SPEC_FILE"

# Clean up
kill $(ps aux | grep 'Xvfb :99' | grep -v grep | awk '{print $2}') 2>/dev/null || true
rm -f /tmp/.X99-lock

mkdir -p screenshots

# Try 3 times for stability
for attempt in 1 2 3; do
    echo "\n=== Attempt $attempt ==="
    
    # Start X server
    Xvfb :99 -screen 0 1200x800x24 &
    XVFB_PID=$!
    export DISPLAY=:99
    sleep 2
    
    # Start xterm with Emacs
    echo "Starting xterm with Emacs..."
    xterm -geometry 120x40 -bg black -fg green -e "timeout 25 emacs -Q -nw '$SPEC_FILE'" &
    XTERM_PID=$!
    
    # Wait for load
    sleep 6
    
    # Capture
    echo "Capturing screenshot..."
    import -window root "screenshots/emacs-tla-attempt-$attempt.png"
    
    # Check window list
    echo "Windows found:"
    xwininfo -root -tree | grep -E "(xterm|emacs)" || echo "No xterm/emacs windows"
    
    # Cleanup
    kill $XTERM_PID $XVFB_PID 2>/dev/null || true
    sleep 2
done

echo "\nResults:"
ls -lh screenshots/*.png