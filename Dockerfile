# syntax=docker/dockerfile:1

# ---------- Stage 1: Build ----------
FROM rust:1.85-bookworm AS builder

# Install Node.js 22 for esbuild (TS interop compilation)
RUN curl -fsSL https://deb.nodesource.com/setup_22.x | bash - \
    && apt-get install -y --no-install-recommends nodejs \
    && rm -rf /var/lib/apt/lists/*

# Add WASM target for Leptos (if needed in the future)
RUN rustup target add wasm32-unknown-unknown

WORKDIR /app

# Copy manifests first for dependency caching
COPY Cargo.toml Cargo.lock rust-toolchain.toml ./
COPY crates/bominal-server/Cargo.toml crates/bominal-server/Cargo.toml
COPY crates/bominal-frontend/Cargo.toml crates/bominal-frontend/Cargo.toml
COPY crates/bominal-db/Cargo.toml crates/bominal-db/Cargo.toml
COPY crates/bominal-domain/Cargo.toml crates/bominal-domain/Cargo.toml
COPY crates/bominal-provider/Cargo.toml crates/bominal-provider/Cargo.toml
COPY crates/bominal-email/Cargo.toml crates/bominal-email/Cargo.toml
COPY crates/bominal-service/Cargo.toml crates/bominal-service/Cargo.toml

# Create stub source files so cargo can resolve the workspace and cache deps
RUN mkdir -p crates/bominal-server/src \
    && echo "fn main() {}" > crates/bominal-server/src/main.rs \
    && echo "" > crates/bominal-server/src/lib.rs \
    && mkdir -p crates/bominal-frontend/src && echo "" > crates/bominal-frontend/src/lib.rs \
    && mkdir -p crates/bominal-db/src && echo "" > crates/bominal-db/src/lib.rs \
    && mkdir -p crates/bominal-domain/src && echo "" > crates/bominal-domain/src/lib.rs \
    && mkdir -p crates/bominal-provider/src && echo "" > crates/bominal-provider/src/lib.rs \
    && mkdir -p crates/bominal-email/src && echo "" > crates/bominal-email/src/lib.rs \
    && mkdir -p crates/bominal-service/src && echo "" > crates/bominal-service/src/lib.rs

# Build dependencies only (cached layer)
RUN cargo build --release 2>/dev/null || true

# Copy real source code
COPY crates/ crates/

# Touch source files so cargo detects changes over the stubs
RUN find crates -name "*.rs" -exec touch {} +

# Install esbuild globally for TypeScript compilation
RUN npm install -g esbuild

# Install Tailwind CSS v4 standalone binary — arch-aware, bundles framework internally
RUN set -eux; \
    ARCH="$(uname -m)"; \
    case "$ARCH" in \
        x86_64)  TW_ARCH="x64"   ;; \
        aarch64) TW_ARCH="arm64" ;; \
        *) echo "Unsupported arch: $ARCH" && exit 1 ;; \
    esac; \
    curl -fsSL "https://github.com/tailwindlabs/tailwindcss/releases/latest/download/tailwindcss-linux-${TW_ARCH}" \
        -o /usr/local/bin/tailwindcss; \
    chmod +x /usr/local/bin/tailwindcss

# Compile TypeScript interop
RUN esbuild crates/bominal-frontend/ts/interop.ts \
    --bundle --outfile=crates/bominal-frontend/ts/interop.js \
    --format=iife --minify

# Compile Tailwind CSS
RUN tailwindcss -i crates/bominal-frontend/style/main.css \
    -o crates/bominal-frontend/style/output.css --minify

# Build WASM hydration bundle
RUN cargo build --profile wasm-release --target wasm32-unknown-unknown \
    -p bominal-frontend --lib --features hydrate --no-default-features

# Install wasm-bindgen-cli (version must match wasm-bindgen crate exactly)
RUN cargo install wasm-bindgen-cli --version $(grep -A1 'name = "wasm-bindgen"' Cargo.lock | grep version | head -1 | cut -d'"' -f2)

# Generate JS/WASM bindings
RUN wasm-bindgen \
    target/wasm32-unknown-unknown/wasm-release/bominal_frontend.wasm \
    --out-dir /app/pkg \
    --target web \
    --no-typescript

# Optimize WASM bundle size
RUN apt-get update && apt-get install -y --no-install-recommends binaryen && rm -rf /var/lib/apt/lists/*
RUN wasm-opt -Os /app/pkg/bominal_frontend_bg.wasm -o /app/pkg/bominal_frontend_bg.wasm

# Build the server binary
RUN cargo build --release

# ---------- Stage 2: Runtime ----------
FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copy binary
COPY --from=builder /app/target/release/bominal-server /app/bominal-server

# Copy static assets the server serves
COPY --from=builder /app/crates/bominal-frontend/style/output.css /app/crates/bominal-frontend/style/output.css
COPY --from=builder /app/crates/bominal-frontend/ts/interop.js /app/crates/bominal-frontend/ts/interop.js
COPY --from=builder /app/pkg /app/pkg

EXPOSE 3000

CMD ["/app/bominal-server"]
