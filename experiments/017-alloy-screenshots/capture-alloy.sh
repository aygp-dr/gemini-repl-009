#!/bin/sh
# Capture screenshots of Alloy analyzer for documentation

set -e

ALLOY_JAR="../016-alloy-demo/alloy.jar"
DEMO_SPEC="../016-alloy-demo/demo.als"
OUTPUT_DIR="screenshots"

# Check dependencies
check_deps() {
    for cmd in Xvfb xwininfo xdotool import magick; do
        if ! command -v $cmd >/dev/null 2>&1; then
            echo "Error: $cmd not found. Please install it."
            exit 1
        fi
    done
    
    if [ ! -f "$ALLOY_JAR" ]; then
        echo "Error: Alloy JAR not found. Run 'make -C ../016-alloy-demo download' first"
        exit 1
    fi
}

# Start virtual display
start_xvfb() {
    echo "Starting virtual display..."
    Xvfb :99 -screen 0 1600x1000x24 &
    XVFB_PID=$!
    export DISPLAY=:99
    sleep 2
    
    # Set nice background
    xsetroot -solid '#f5f5f5' 2>/dev/null || true
}

# Capture Alloy editor view
capture_editor() {
    echo "Capturing Alloy editor view..."
    
    java -jar "$ALLOY_JAR" "$DEMO_SPEC" &
    ALLOY_PID=$!
    
    # Wait for Alloy to load
    sleep 5
    
    # Find Alloy window
    WINDOW_ID=$(xwininfo -root -tree 2>/dev/null | grep -i "alloy" | head -1 | awk '{print $1}' || true)
    
    if [ -n "$WINDOW_ID" ]; then
        echo "Found Alloy window: $WINDOW_ID"
        # Capture just the Alloy window
        xwd -id "$WINDOW_ID" -out alloy.xwd 2>/dev/null
        magick alloy.xwd "$OUTPUT_DIR/alloy-editor.png"
        rm -f alloy.xwd
    else
        echo "Warning: Could not find Alloy window, capturing full screen"
        import -window root "$OUTPUT_DIR/alloy-editor.png"
    fi
    
    kill $ALLOY_PID 2>/dev/null || true
}

# Capture Alloy with instance visualization
capture_instance() {
    echo "Capturing Alloy instance visualization..."
    
    java -jar "$ALLOY_JAR" "$DEMO_SPEC" &
    ALLOY_PID=$!
    sleep 5
    
    # Find and focus Alloy window
    WINDOW_ID=$(xwininfo -root -tree 2>/dev/null | grep -i "alloy" | head -1 | awk '{print $1}' || true)
    
    if [ -n "$WINDOW_ID" ]; then
        # Execute the first run command
        xdotool windowfocus "$WINDOW_ID" 2>/dev/null || true
        sleep 1
        xdotool key ctrl+e 2>/dev/null || true
        sleep 4
        
        # Capture full desktop to get both windows
        import -window root "$OUTPUT_DIR/alloy-instance.png"
        
        # Try to capture just the viz window
        VIZ_ID=$(xwininfo -root -tree 2>/dev/null | grep -i "viz" | head -1 | awk '{print $1}' || true)
        if [ -n "$VIZ_ID" ]; then
            echo "Found viz window: $VIZ_ID"
            xwd -id "$VIZ_ID" -out viz.xwd 2>/dev/null
            magick viz.xwd "$OUTPUT_DIR/alloy-viz-only.png"
            rm -f viz.xwd
        fi
    fi
    
    kill $ALLOY_PID 2>/dev/null || true
}

# Create documentation montage
create_montage() {
    echo "Creating documentation montage..."
    
    if [ -f "$OUTPUT_DIR/alloy-editor.png" ] && [ -f "$OUTPUT_DIR/alloy-instance.png" ]; then
        # Add drop shadows and create montage
        for img in "$OUTPUT_DIR"/*.png; do
            base=$(basename "$img" .png)
            magick "$img" \
                \( +clone -background black -shadow 60x5+5+5 \) \
                +swap -background white -layers merge +repage \
                "$OUTPUT_DIR/${base}-shadow.png"
        done
        
        # Create side-by-side montage
        magick montage \
            "$OUTPUT_DIR/alloy-editor-shadow.png" \
            "$OUTPUT_DIR/alloy-instance-shadow.png" \
            -tile 2x1 -geometry +20+20 \
            -background '#ffffff' \
            "$OUTPUT_DIR/alloy-demo-montage.png"
            
        echo "Montage created: $OUTPUT_DIR/alloy-demo-montage.png"
    fi
}

# Main execution
main() {
    check_deps
    
    # Create output directory
    mkdir -p "$OUTPUT_DIR"
    
    # Start virtual display
    start_xvfb
    
    # Capture screenshots
    capture_editor
    capture_instance
    
    # Create montage
    create_montage
    
    # Cleanup
    kill $XVFB_PID 2>/dev/null || true
    
    echo "\nScreenshots captured:"
    ls -la "$OUTPUT_DIR"/*.png
}

# Run main
main