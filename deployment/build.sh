#!/usr/bin/env bash
set -euo pipefail

# Bominal — Release build script
# Compiles Leptos, processes CSS, and pre-compresses assets.

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
cd "$PROJECT_ROOT"

echo "=== Step 1: Build Leptos (server binary + client WASM + static assets) ==="
cargo leptos build --release

echo "=== Step 2: Process CSS ==="
if command -v tailwindcss &>/dev/null; then
    tailwindcss -i style/main.css -o target/site/style.css 2>/dev/null || true
fi
if command -v lightningcss &>/dev/null; then
    lightningcss --minify --bundle target/site/style.css -o target/site/style.min.css 2>/dev/null || true
fi

echo "=== Step 3: Pre-compress static assets ==="
if command -v brotli &>/dev/null; then
    find target/site -type f \( -name '*.css' -o -name '*.js' -o -name '*.wasm' -o -name '*.svg' -o -name '*.html' \) \
        -exec brotli -f -q 11 {} \;
    echo "  Brotli compression complete"
fi

find target/site -type f \( -name '*.css' -o -name '*.js' -o -name '*.wasm' -o -name '*.svg' -o -name '*.html' \) \
    -exec gzip -k -9 {} \; 2>/dev/null || true
echo "  Gzip compression complete"

echo ""
echo "=== Build complete ==="
ls -lh target/release/bominal-server 2>/dev/null || echo "Binary not found (expected with cargo-leptos)"
echo "Static assets in target/site/"
