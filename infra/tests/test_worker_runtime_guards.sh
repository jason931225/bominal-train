#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
DEV_COMPOSE="$ROOT_DIR/infra/docker-compose.yml"
PROD_COMPOSE="$ROOT_DIR/infra/docker-compose.prod.yml"
LOCAL_CHECK="$ROOT_DIR/infra/scripts/local-check.sh"
WEB_DOCKERFILE="$ROOT_DIR/web/Dockerfile"

matches_pattern() {
  local pattern="$1"
  local file="$2"
  if command -v rg >/dev/null 2>&1; then
    rg -n -- "$pattern" "$file" >/dev/null
    return $?
  fi
  grep -En -- "$pattern" "$file" >/dev/null
}

web_service_has_init_true() {
  awk '
    /^  web:$/ {in_web=1; next}
    in_web && /^  [a-zA-Z0-9_-]+:/ {in_web=0}
    in_web && /^    init:[[:space:]]*true[[:space:]]*$/ {found=1}
    END {exit found ? 0 : 1}
  ' "$PROD_COMPOSE"
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

if ! web_service_has_init_true; then
  echo "FAIL: prod compose web service must enable init: true for child-process reaping." >&2
  exit 1
fi

if ! matches_pattern 'CMD \["node", "node_modules/next/dist/bin/next", "start"' "$WEB_DOCKERFILE"; then
  echo "FAIL: web Dockerfile must start Next directly (without npm wrapper)." >&2
  exit 1
fi

echo "PASS: worker runtime guards verified (compose + local-check)."
