#!/usr/bin/env bash
set -euo pipefail

# Bominal — Local dev build script
# Uses cargo-leptos as the single build entry point for the Leptos app + Axum server.

PROJECT_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$PROJECT_ROOT"

echo "=== cargo-leptos build ==="
cargo leptos build "$@"

echo ""
echo "=== Build complete ==="
if [ -f target/release/bominal-server ]; then
    ls -lh target/release/bominal-server
elif [ -f target/debug/bominal-server ]; then
    ls -lh target/debug/bominal-server
fi
echo "Static assets: target/site/"
find target/site -maxdepth 2 -type f | sed -n '1,20p'
