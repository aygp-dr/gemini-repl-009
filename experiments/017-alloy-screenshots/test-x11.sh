#!/bin/sh
# Test X11 virtual display setup

echo "Testing X11 virtual display..."

# Start Xvfb
Xvfb :99 -screen 0 1024x768x24 &
XVFB_PID=$!
export DISPLAY=:99
echo "Started Xvfb on display :99 (PID: $XVFB_PID)"
sleep 2

# Test with a simple X11 app
echo "Testing with xclock..."
xclock &
XCLOCK_PID=$!
sleep 2

# Check if window exists
if xwininfo -root -tree | grep -i "xclock"; then
    echo "✓ X11 is working - xclock window found"
else
    echo "✗ X11 test failed - no xclock window"
fi

# Try a screenshot
echo "Testing screenshot capture..."
if import -window root test-screenshot.png; then
    echo "✓ Screenshot captured"
    ls -lh test-screenshot.png
    rm -f test-screenshot.png
else
    echo "✗ Screenshot failed"
fi

# Cleanup
kill $XCLOCK_PID $XVFB_PID 2>/dev/null

echo "X11 test complete"