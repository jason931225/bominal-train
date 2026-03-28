# syntax=docker/dockerfile:1

# Build from the repo root and provide the sibling `bominal-ui` crate as an
# extra BuildKit context:
#   docker buildx build \
#     --build-context bominal_ui=../bominal-ui \
#     -f Dockerfile \
#     -t bominal-train:latest \
#     .

# ---------- Stage 1: Build ----------
FROM rust:1.85-bookworm AS builder

RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates \
    clang \
    curl \
    libssl-dev \
    lld \
    pkg-config \
    && rm -rf /var/lib/apt/lists/*

RUN rustup target add wasm32-unknown-unknown \
    && cargo install cargo-leptos --locked

# Tailwind CSS v4 standalone CLI. cargo-leptos invokes the binary directly.
RUN set -eux; \
    ARCH="$(uname -m)"; \
    case "$ARCH" in \
        x86_64) TW_ARCH="x64" ;; \
        aarch64|arm64) TW_ARCH="arm64" ;; \
        *) echo "Unsupported arch: $ARCH" && exit 1 ;; \
    esac; \
    curl -fsSL "https://github.com/tailwindlabs/tailwindcss/releases/latest/download/tailwindcss-linux-${TW_ARCH}" \
        -o /usr/local/bin/tailwindcss; \
    chmod +x /usr/local/bin/tailwindcss

WORKDIR /app

COPY --from=bominal_ui . /app/bominal-ui
COPY . /app/bominal-train

WORKDIR /app/bominal-train
RUN cargo leptos build --release --precompress

# ---------- Stage 2: Runtime ----------
FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

COPY --from=builder /app/bominal-train/target/release/bominal-server /app/bominal-server
COPY --from=builder /app/bominal-train/target/site /app/target/site

EXPOSE 3000

CMD ["/app/bominal-server"]
