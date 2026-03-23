#!/usr/bin/env bash
set -euo pipefail

# Bominal — Local dev build script
# Replicates the Dockerfile WASM build steps for local development.

PROJECT_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$PROJECT_ROOT"

echo "=== Step 1: Compile TypeScript interop ==="
npx esbuild crates/bominal-frontend/ts/interop.ts \
    --bundle --outfile=crates/bominal-frontend/ts/interop.js \
    --format=iife --global-name=BominalInterop
echo "  interop.js compiled"

echo "=== Step 2: Process Tailwind CSS ==="
tailwindcss -i crates/bominal-frontend/style/main.css \
    -o crates/bominal-frontend/style/output.css \
    --content 'crates/bominal-frontend/src/**/*.rs'
echo "  output.css generated"

echo "=== Step 3: Build WASM hydration bundle ==="
cargo build --profile wasm-release --target wasm32-unknown-unknown \
    -p bominal-frontend --lib --features hydrate --no-default-features
echo "  WASM compiled"

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

echo "=== Step 5: Optimize WASM (optional) ==="
if command -v wasm-opt &>/dev/null; then
    wasm-opt -Os pkg/bominal_frontend_bg.wasm -o pkg/bominal_frontend_bg.wasm
    echo "  wasm-opt applied"
else
    echo "  wasm-opt not found, skipping (install binaryen for smaller bundles)"
fi

echo "=== Step 6: Build server binary ==="
cargo build --release
echo "  target/release/bominal-server built"

echo ""
echo "=== Build complete ==="
ls -lh target/release/bominal-server
echo "WASM bundle: pkg/"
ls -lh pkg/
