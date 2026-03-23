#!/usr/bin/env bash
set -euo pipefail

# Bominal — Release build script
# Compiles TypeScript, WASM hydration bundle, CSS, server binary, and
# pre-compresses assets.

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
cd "$PROJECT_ROOT"

echo "=== Step 1: Compile TypeScript interop ==="
if command -v npx &>/dev/null; then
    npx esbuild crates/bominal-frontend/ts/interop.ts --bundle --outfile=crates/bominal-frontend/ts/interop.js --format=iife --minify
    echo "  interop.js compiled"
fi

echo "=== Step 2: Process CSS ==="
if command -v tailwindcss &>/dev/null; then
    tailwindcss -i crates/bominal-frontend/style/main.css -o crates/bominal-frontend/style/output.css --content 'crates/bominal-frontend/src/**/*.rs' --minify 2>/dev/null || true
fi

echo "=== Step 3: Build WASM hydration bundle ==="
cargo build --profile wasm-release --target wasm32-unknown-unknown \
    -p bominal-frontend --lib --features hydrate --no-default-features

echo "=== Step 4: Generate JS/WASM bindings ==="
WASM_BINDGEN_VERSION=$(grep -A1 'name = "wasm-bindgen"' Cargo.lock | grep version | head -1 | cut -d'"' -f2)
if ! command -v wasm-bindgen &>/dev/null; then
    echo "  Installing wasm-bindgen-cli@${WASM_BINDGEN_VERSION}..."
    cargo install wasm-bindgen-cli --version "$WASM_BINDGEN_VERSION"
fi
wasm-bindgen \
    target/wasm32-unknown-unknown/wasm-release/bominal_frontend.wasm \
    --out-dir pkg \
    --target web \
    --no-typescript
echo "  pkg/bominal_frontend.js + pkg/bominal_frontend_bg.wasm generated"

echo "=== Step 5: Optimize WASM ==="
if command -v wasm-opt &>/dev/null; then
    wasm-opt -Os pkg/bominal_frontend_bg.wasm -o pkg/bominal_frontend_bg.wasm
    echo "  wasm-opt applied"
else
    echo "  wasm-opt not found, skipping"
fi

echo "=== Step 6: Build server binary ==="
cargo build --release

echo "=== Step 7: Pre-compress static assets ==="
if command -v brotli &>/dev/null; then
    find target/site pkg -type f \( -name '*.css' -o -name '*.js' -o -name '*.wasm' -o -name '*.svg' -o -name '*.html' \) \
        -exec brotli -f -q 11 {} \;
    echo "  Brotli compression complete"
fi

find target/site pkg -type f \( -name '*.css' -o -name '*.js' -o -name '*.wasm' -o -name '*.svg' -o -name '*.html' \) \
    -exec gzip -k -9 {} \; 2>/dev/null || true
echo "  Gzip compression complete"

echo ""
echo "=== Build complete ==="
ls -lh target/release/bominal-server 2>/dev/null || echo "Binary not found"
echo "WASM bundle in pkg/"
echo "Static assets in target/site/"
