#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
DEV_COMPOSE="$ROOT_DIR/infra/docker-compose.yml"
PROD_COMPOSE="$ROOT_DIR/infra/docker-compose.prod.yml"
LOCAL_CHECK="$ROOT_DIR/infra/scripts/local-check.sh"

matches_pattern() {
  local pattern="$1"
  local file="$2"
  if command -v rg >/dev/null 2>&1; then
    rg -n -- "$pattern" "$file" >/dev/null
    return $?
  fi
  grep -En -- "$pattern" "$file" >/dev/null
}

if ! matches_pattern 'python -m app\.worker_entrypoint app\.worker\.WorkerSettings' "$DEV_COMPOSE"; then
  echo "FAIL: dev compose worker must use app.worker_entrypoint app.worker.WorkerSettings." >&2
  exit 1
fi

if ! matches_pattern 'python -m app\.worker_entrypoint app\.worker\.WorkerSettings' "$PROD_COMPOSE"; then
  echo "FAIL: prod compose worker must use app.worker_entrypoint app.worker.WorkerSettings." >&2
  exit 1
fi

if ! matches_pattern 'WorkerSettings' "$PROD_COMPOSE"; then
  echo "FAIL: prod compose worker healthcheck must verify WorkerSettings process." >&2
  exit 1
fi

if ! matches_pattern 'check_worker_service "worker" "app\.worker\.WorkerSettings"' "$LOCAL_CHECK"; then
  echo "FAIL: local-check must validate worker health." >&2
  exit 1
fi

echo "PASS: worker runtime guards verified (compose + local-check)."
