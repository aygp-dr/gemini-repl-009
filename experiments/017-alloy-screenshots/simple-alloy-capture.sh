#!/bin/sh
# Simple Alloy screenshot capture

set -e

ALLOY_JAR="../016-alloy-demo/alloy.jar"
DEMO_SPEC="../016-alloy-demo/demo.als"

echo "Simple Alloy screenshot capture"

# Clean up any existing X server
kill $(ps aux | grep 'Xvfb :99' | grep -v grep | awk '{print $2}') 2>/dev/null || true
rm -f /tmp/.X99-lock

# Start fresh Xvfb
echo "Starting X virtual framebuffer..."
Xvfb :99 -screen 0 1600x1000x24 &
XVFB_PID=$!
export DISPLAY=:99
sleep 3

# Start Alloy
echo "Starting Alloy (this may take a moment)..."
java -jar "$ALLOY_JAR" "$DEMO_SPEC" &
ALLOY_PID=$!

# Give Alloy time to start
echo "Waiting for Alloy to load..."
sleep 8

# Take a screenshot
echo "Capturing screenshot..."
mkdir -p screenshots
import -window root screenshots/alloy-simple.png

echo "Screenshot saved to: screenshots/alloy-simple.png"
ls -lh screenshots/alloy-simple.png

# Try to find Alloy window specifically
echo "Looking for Alloy window..."
xwininfo -root -tree | grep -i alloy || echo "No Alloy window found in window list"

# Cleanup
echo "Cleaning up..."
kill $ALLOY_PID 2>/dev/null || true
kill $XVFB_PID 2>/dev/null || true

echo "Done!"