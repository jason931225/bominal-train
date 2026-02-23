#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
DEV_COMPOSE="$ROOT_DIR/infra/docker-compose.yml"
PROD_COMPOSE="$ROOT_DIR/infra/docker-compose.prod.yml"
LOCAL_CHECK="$ROOT_DIR/infra/scripts/local-check.sh"

if ! rg -n 'python -m app\.worker_entrypoint app\.worker_train\.WorkerTrainSettings' "$DEV_COMPOSE" >/dev/null; then
  echo "FAIL: dev compose worker-train must use app.worker_entrypoint." >&2
  exit 1
fi

if ! rg -n 'python -m app\.worker_entrypoint app\.worker_restaurant\.WorkerRestaurantSettings' "$DEV_COMPOSE" >/dev/null; then
  echo "FAIL: dev compose worker-restaurant must use app.worker_entrypoint." >&2
  exit 1
fi

if ! rg -n 'python -m app\.worker_entrypoint app\.worker_train\.WorkerTrainSettings' "$PROD_COMPOSE" >/dev/null; then
  echo "FAIL: prod compose worker-train must use app.worker_entrypoint." >&2
  exit 1
fi

if ! rg -n 'python -m app\.worker_entrypoint app\.worker_restaurant\.WorkerRestaurantSettings' "$PROD_COMPOSE" >/dev/null; then
  echo "FAIL: prod compose worker-restaurant must use app.worker_entrypoint." >&2
  exit 1
fi

if ! rg -n 'WorkerTrainSettings' "$PROD_COMPOSE" >/dev/null; then
  echo "FAIL: prod compose worker-train healthcheck must verify WorkerTrainSettings process." >&2
  exit 1
fi

if ! rg -n 'WorkerRestaurantSettings' "$PROD_COMPOSE" >/dev/null; then
  echo "FAIL: prod compose worker-restaurant healthcheck must verify WorkerRestaurantSettings process." >&2
  exit 1
fi

if ! rg -n 'check_worker_service "worker-train" "app\.worker_train\.WorkerTrainSettings"' "$LOCAL_CHECK" >/dev/null; then
  echo "FAIL: local-check must validate worker-train health." >&2
  exit 1
fi

if ! rg -n 'check_worker_service "worker-restaurant" "app\.worker_restaurant\.WorkerRestaurantSettings"' "$LOCAL_CHECK" >/dev/null; then
  echo "FAIL: local-check must validate worker-restaurant health." >&2
  exit 1
fi

echo "PASS: worker runtime guards verified (compose + local-check)."
