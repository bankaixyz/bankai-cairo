#!/bin/bash

PROGRAM_PATH=${1:-"cairo/src/recursive_update.cairo"}  # Default to main.cairo if no argument provided
OUTPUT_NAME=$(basename "$PROGRAM_PATH" .cairo)  # Extract filename without path and extension

echo "Compiling Cairo Program: $PROGRAM_PATH"
cairo-compile --cairo_path=cairo/packages/garaga-zero/src "$PROGRAM_PATH" --output "cairo/build/${OUTPUT_NAME}.json" --no_debug_info
# cairo-compile "$PROGRAM_PATH" --output "cairo/build/${OUTPUT_NAME}.json"

if [ $? -eq 0 ]; then
    echo "Compilation Successful!"
fi
