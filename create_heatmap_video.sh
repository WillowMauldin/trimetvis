#!/bin/bash

# Script to create a video from heatmap images using ffmpeg
# Usage: ./create_heatmap_video.sh [output_filename] [framerate]

set -e

HEATMAPS_DIR="heatmaps"
OUTPUT_FILE="${1:-heatmap_video.mkv}"
FRAMERATE="${2:-30}"

if [ ! -d "$HEATMAPS_DIR" ]; then
    echo "Error: $HEATMAPS_DIR directory not found"
    exit 1
fi

PNG_COUNT=$(find "$HEATMAPS_DIR" -name "*.png" | wc -l)
if [ "$PNG_COUNT" -eq 0 ]; then
    echo "Error: No PNG files found in $HEATMAPS_DIR"
    exit 1
fi

echo "Found $PNG_COUNT heatmap images"
echo "Creating video: $OUTPUT_FILE"
echo "Framerate: $FRAMERATE fps"

# Use ffmpeg to create lossless video from PNG sequence
# -y: overwrite output file if it exists
# -framerate: input framerate
# -pattern_type glob: use glob pattern for input
# -i: input pattern
# -c:v libx264: use H.264 codec
# -preset ultrafast: fastest encoding
# -crf 0: lossless encoding
# -pix_fmt yuv420p: pixel format for compatibility
# -r: output framerate
ffmpeg -y \
    -framerate "$FRAMERATE" \
    -pattern_type glob \
    -i "$HEATMAPS_DIR/heatmap_*.png" \
    -c:v libx264 \
    -qp 0 \
    -r "$FRAMERATE" \
    "$OUTPUT_FILE"

echo "Video created successfully: $OUTPUT_FILE"
