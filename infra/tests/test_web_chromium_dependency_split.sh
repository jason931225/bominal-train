#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
DEV_DOCKERFILE="$ROOT_DIR/web/Dockerfile.dev"
E2E_DOCKERFILE="$ROOT_DIR/web/Dockerfile.e2e"
COMPOSE_FILE="$ROOT_DIR/infra/docker-compose.yml"

if grep -Eiq '\bchromium\b' "$DEV_DOCKERFILE"; then
  echo "FAIL: web/Dockerfile.dev must remain Chromium-free" >&2
  exit 1
fi

if ! grep -Eiq '\bchromium\b' "$E2E_DOCKERFILE"; then
  echo "FAIL: web/Dockerfile.e2e must include Chromium for Playwright E2E" >&2
  exit 1
fi

if ! rg -n '^\s*web-e2e:\s*$' "$COMPOSE_FILE" >/dev/null; then
  echo "FAIL: infra/docker-compose.yml missing web-e2e service" >&2
  exit 1
fi

if ! rg -n '^\s*profiles:\s*\["e2e"\]\s*$' "$COMPOSE_FILE" >/dev/null; then
  echo "FAIL: web-e2e service must be gated behind profile e2e" >&2
  exit 1
fi

if ! rg -n '^\s*dockerfile:\s*Dockerfile\.e2e\s*$' "$COMPOSE_FILE" >/dev/null; then
  echo "FAIL: web-e2e service must use web/Dockerfile.e2e" >&2
  exit 1
fi

echo "PASS: Chromium dependency split verified (web dev no Chromium, web-e2e Chromium-enabled)."
