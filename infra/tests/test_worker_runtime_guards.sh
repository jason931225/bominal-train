#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
DEV_COMPOSE="$ROOT_DIR/infra/docker-compose.yml"
PROD_COMPOSE="$ROOT_DIR/infra/docker-compose.prod.yml"
LOCAL_CHECK="$ROOT_DIR/infra/scripts/local-check.sh"

if ! rg -n 'python -m app\.worker_entrypoint app\.worker\.WorkerSettings' "$DEV_COMPOSE" >/dev/null; then
  echo "FAIL: dev compose worker must use app.worker_entrypoint app.worker.WorkerSettings." >&2
  exit 1
fi

if ! rg -n 'python -m app\.worker_entrypoint app\.worker\.WorkerSettings' "$PROD_COMPOSE" >/dev/null; then
  echo "FAIL: prod compose worker must use app.worker_entrypoint app.worker.WorkerSettings." >&2
  exit 1
fi

if ! rg -n 'WorkerSettings' "$PROD_COMPOSE" >/dev/null; then
  echo "FAIL: prod compose worker healthcheck must verify WorkerSettings process." >&2
  exit 1
fi

if ! rg -n 'check_worker_service "worker" "app\.worker\.WorkerSettings"' "$LOCAL_CHECK" >/dev/null; then
  echo "FAIL: local-check must validate worker health." >&2
  exit 1
fi

echo "PASS: worker runtime guards verified (compose + local-check)."
