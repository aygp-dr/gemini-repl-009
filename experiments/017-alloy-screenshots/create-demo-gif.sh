#!/bin/sh
# Create an animated GIF demo of Alloy

set -e

ALLOY_JAR="../016-alloy-demo/alloy.jar"
DEMO_SPEC="../016-alloy-demo/demo.als"
OUTPUT_DIR="screenshots"

echo "Creating Alloy demo GIF..."

# Start Xvfb
Xvfb :99 -screen 0 1400x900x24 &
XVFB_PID=$!
export DISPLAY=:99
sleep 2

mkdir -p "$OUTPUT_DIR/frames"

# Start Alloy
java -jar "$ALLOY_JAR" "$DEMO_SPEC" &
ALLOY_PID=$!
sleep 5

# Capture frames
frame=0
capture_frame() {
    import -window root "$OUTPUT_DIR/frames/frame_$(printf '%03d' $frame).png"
    frame=$((frame + 1))
}

# Initial view
capture_frame
sleep 1

# Execute first predicate
xdotool key ctrl+e 2>/dev/null || true
sleep 2
capture_frame
sleep 1
capture_frame

# Navigate instances
for i in 1 2 3; do
    xdotool key Right 2>/dev/null || true
    sleep 1
    capture_frame
done

# Clean up
kill $ALLOY_PID $XVFB_PID 2>/dev/null || true

# Create GIF
if command -v magick >/dev/null; then
    echo "Creating animated GIF..."
    magick -delay 100 -loop 0 "$OUTPUT_DIR/frames/frame_*.png" \
        -resize 800x600 "$OUTPUT_DIR/alloy-demo.gif"
    echo "GIF created: $OUTPUT_DIR/alloy-demo.gif"
fi

# Clean up frames
rm -rf "$OUTPUT_DIR/frames"