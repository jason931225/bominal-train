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

# Install Tailwind CSS v4 CLI globally
RUN npm install -g @tailwindcss/cli esbuild

# Compile TypeScript interop
RUN esbuild crates/bominal-frontend/ts/interop.ts \
    --bundle --outfile=crates/bominal-frontend/ts/interop.js \
    --format=iife --minify

# Compile Tailwind CSS
RUN tailwindcss -i crates/bominal-frontend/style/main.css \
    -o crates/bominal-frontend/style/output.css --minify

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

EXPOSE 3000

CMD ["/app/bominal-server"]
