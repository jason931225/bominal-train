#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
DEV_COMPOSE="$ROOT_DIR/infra/docker-compose.yml"
PROD_COMPOSE="$ROOT_DIR/infra/docker-compose.prod.yml"
LOCAL_CHECK="$ROOT_DIR/infra/scripts/local-check.sh"
RUST_API_DOCKERFILE="$ROOT_DIR/rust/Dockerfile.api"
RUST_WORKER_DOCKERFILE="$ROOT_DIR/rust/Dockerfile.worker"

matches_pattern() {
  local pattern="$1"
  local file="$2"
  if command -v rg >/dev/null 2>&1; then
    rg -n -- "$pattern" "$file" >/dev/null
    return $?
  fi
  grep -En -- "$pattern" "$file" >/dev/null
}

if ! matches_pattern 'cargo run -p bominal-rust-worker' "$DEV_COMPOSE"; then
  echo "FAIL: dev compose worker must run cargo for bominal-rust-worker." >&2
  exit 1
fi

if ! matches_pattern '/usr/local/bin/bominal-rust-worker' "$PROD_COMPOSE"; then
  echo "FAIL: prod compose worker must execute /usr/local/bin/bominal-rust-worker." >&2
  exit 1
fi

if ! matches_pattern '/usr/local/bin/bominal-rust-api' "$PROD_COMPOSE"; then
  echo "FAIL: prod compose api/web must execute /usr/local/bin/bominal-rust-api." >&2
  exit 1
fi

if matches_pattern 'python -m app\.worker_entrypoint app\.worker\.WorkerSettings' "$DEV_COMPOSE"; then
  echo "FAIL: dev compose must not reference legacy Python worker entrypoints." >&2
  exit 1
fi

if matches_pattern 'python -m app\.worker_entrypoint app\.worker\.WorkerSettings' "$PROD_COMPOSE"; then
  echo "FAIL: prod compose must not reference legacy Python worker entrypoints." >&2
  exit 1
fi

pay_env_refs="$(grep -c '\./env/prod/pay\.env' "$PROD_COMPOSE" || true)"
if [[ "$pay_env_refs" -ne 0 ]]; then
  echo "FAIL: prod compose must not include ./env/prod/pay.env (wallet-only payment contract)." >&2
  exit 1
fi

if ! matches_pattern 'check_worker_service "worker" "bominal-rust-worker"' "$LOCAL_CHECK"; then
  echo "FAIL: local-check must validate worker health." >&2
  exit 1
fi

if ! matches_pattern 'cargo test --workspace' "$LOCAL_CHECK"; then
  echo "FAIL: local-check must run Rust workspace tests." >&2
  exit 1
fi

if ! matches_pattern 'cargo check --workspace' "$LOCAL_CHECK"; then
  echo "FAIL: local-check must run Rust workspace checks." >&2
  exit 1
fi

if matches_pattern 'pytest -q' "$LOCAL_CHECK"; then
  echo "FAIL: local-check must not run legacy Python pytest checks." >&2
  exit 1
fi

if matches_pattern 'npx tsc --noEmit' "$LOCAL_CHECK"; then
  echo "FAIL: local-check must not run legacy Next.js typecheck commands." >&2
  exit 1
fi

if ! matches_pattern 'CMD \["/usr/local/bin/bominal-rust-api"\]' "$RUST_API_DOCKERFILE"; then
  echo "FAIL: rust/Dockerfile.api must execute /usr/local/bin/bominal-rust-api." >&2
  exit 1
fi

if ! matches_pattern 'CMD \["/usr/local/bin/bominal-rust-worker"\]' "$RUST_WORKER_DOCKERFILE"; then
  echo "FAIL: rust/Dockerfile.worker must execute /usr/local/bin/bominal-rust-worker." >&2
  exit 1
fi

echo "PASS: worker runtime guards verified (compose + local-check)."
