#!/bin/bash

set -euo pipefail

TEST_DIR="cairo/tests"
MANIFEST_PATH="$TEST_DIR/fixture_manifest.json"

for file in "$TEST_DIR"/*.cairo; do
  if [ -f "$file" ]; then
    ./scripts/cairo_compile.sh "$file"
  fi
done

cargo run -p cairo-runner --bin fixture-harness -- --manifest "$MANIFEST_PATH"
