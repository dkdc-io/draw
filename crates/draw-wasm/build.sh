#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$SCRIPT_DIR"

echo "Building draw-wasm..."
wasm-pack build --target web --release

echo "Optimizing WASM binary..."
if command -v wasm-opt &>/dev/null; then
    wasm-opt -Oz -o pkg/draw_wasm_bg.wasm pkg/draw_wasm_bg.wasm
    echo "wasm-opt applied"
else
    echo "wasm-opt not found, skipping optimization"
fi

echo "Build complete. Output in pkg/"
ls -lh pkg/*.wasm
