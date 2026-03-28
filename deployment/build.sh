#!/usr/bin/env bash
set -euo pipefail

# Bominal — Release build script
# Uses cargo-leptos to build the server, WASM bundle, and static assets.

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
cd "$PROJECT_ROOT"

echo "=== cargo-leptos release build ==="
cargo leptos build --release --precompress

echo ""
echo "=== Build complete ==="
ls -lh target/release/bominal-server 2>/dev/null || echo "Binary not found"
echo "Static assets in target/site/"
find target/site -maxdepth 2 -type f | sed -n '1,40p'
