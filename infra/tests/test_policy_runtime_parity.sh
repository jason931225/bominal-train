#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"

PREDEPLOY_FILE="$ROOT_DIR/infra/scripts/predeploy-check.sh"
DEPLOY_FILE="$ROOT_DIR/infra/scripts/deploy.sh"
SYNC_SUPABASE_AUTH_TEMPLATES_SCRIPT="$ROOT_DIR/infra/scripts/sync-supabase-auth-templates.sh"
DEV_COMPOSE_FILE="$ROOT_DIR/infra/docker-compose.yml"
PROD_COMPOSE_FILE="$ROOT_DIR/infra/docker-compose.prod.yml"
CADDY_FILE="$ROOT_DIR/infra/caddy/Caddyfile"
CADDY_ACTIVE_UPSTREAM_FILE="$ROOT_DIR/infra/caddy/upstreams/active.caddy"
CADDY_WEB_CANARY_UPSTREAM_FILE="$ROOT_DIR/infra/caddy/upstreams/web-with-canary.caddy"
RUST_ENV_EXAMPLE_FILE="$ROOT_DIR/rust/env.example"
RUST_API_DOCKERFILE="$ROOT_DIR/rust/Dockerfile.api"
RUST_WORKER_DOCKERFILE="$ROOT_DIR/rust/Dockerfile.worker"

assert_contains() {
  local pattern="$1"
  local file="$2"
  local message="$3"
  if ! rg -n -- "$pattern" "$file" >/dev/null; then
    echo "FAIL: $message ($file)" >&2
    exit 1
  fi
}

assert_not_contains() {
  local pattern="$1"
  local file="$2"
  local message="$3"
  if rg -n -- "$pattern" "$file" >/dev/null; then
    echo "FAIL: $message ($file)" >&2
    exit 1
  fi
}

assert_contains "cargo test --workspace" "$PREDEPLOY_FILE" "predeploy must run Rust workspace tests"
assert_contains "cargo check --workspace" "$PREDEPLOY_FILE" "predeploy must run Rust workspace checks"
assert_not_contains "pytest -q tests/test_auth_flow.py tests/test_train_provider_crud.py" "$PREDEPLOY_FILE" "predeploy must not run legacy Python smoke tests"
assert_not_contains "uv run --python" "$PREDEPLOY_FILE" "predeploy must not require uv/python fallback execution"
assert_not_contains "npx tsc --noEmit" "$PREDEPLOY_FILE" "predeploy must not run legacy frontend typecheck"

assert_contains "load_runtime_api_secrets_from_gsm" "$DEPLOY_FILE" "deploy missing runtime secret loader"
assert_contains "INTERNAL_API_KEY_SECRET_ID" "$DEPLOY_FILE" "deploy missing internal gsm reference resolution"
assert_contains "RESEND_API_KEY_SECRET_ID" "$DEPLOY_FILE" "deploy missing resend gsm reference resolution"
assert_contains "export INTERNAL_API_KEY" "$DEPLOY_FILE" "deploy does not export INTERNAL_API_KEY runtime value"
assert_contains "DEPLOY_WEB_CANARY_ENABLED" "$DEPLOY_FILE" "deploy missing web canary toggle"
assert_contains "start_web_canary" "$DEPLOY_FILE" "deploy missing web canary bootstrap"
assert_contains "check_auth_callback_route" "$DEPLOY_FILE" "deploy missing auth callback smoke guard"
assert_contains "rust-api" "$DEPLOY_FILE" "deploy must reference Rust API image names"
assert_contains "rust-worker" "$DEPLOY_FILE" "deploy must reference Rust worker image names"

assert_contains "SUPABASE_MANAGEMENT_API_TOKEN_SECRET_ID" "$SYNC_SUPABASE_AUTH_TEMPLATES_SCRIPT" "sync script missing supabase management token secret reference"
assert_contains "gcloud secrets versions access" "$SYNC_SUPABASE_AUTH_TEMPLATES_SCRIPT" "sync script missing gsm token resolution"
assert_contains "--apply in production requires SUPABASE_MANAGEMENT_API_TOKEN_SECRET_ID" "$SYNC_SUPABASE_AUTH_TEMPLATES_SCRIPT" "sync script missing production fail-closed token guard"

assert_contains "image: \\$\\{API_IMAGE:-\\$\\{GHCR_NAMESPACE:-ghcr\\.io/jason931225/bominal\\}/rust-api:latest\\}" "$PROD_COMPOSE_FILE" "prod compose api default image must be rust-api"
assert_contains "image: \\$\\{WORKER_IMAGE:-\\$\\{GHCR_NAMESPACE:-ghcr\\.io/jason931225/bominal\\}/rust-worker:latest\\}" "$PROD_COMPOSE_FILE" "prod compose worker default image must be rust-worker"
assert_contains "image: \\$\\{WEB_IMAGE:-\\$\\{GHCR_NAMESPACE:-ghcr\\.io/jason931225/bominal\\}/rust-api:latest\\}" "$PROD_COMPOSE_FILE" "prod compose web default image must be rust-api"
assert_contains "command: \\[\"/usr/local/bin/bominal-rust-api\"\\]" "$PROD_COMPOSE_FILE" "prod compose api/web must execute rust api binary"
assert_contains "command: \\[\"/usr/local/bin/bominal-rust-worker\"\\]" "$PROD_COMPOSE_FILE" "prod compose worker must execute rust worker binary"
assert_contains "web-canary:" "$PROD_COMPOSE_FILE" "compose missing web-canary service"
assert_contains "127\\.0\\.0\\.1:3001:3000" "$PROD_COMPOSE_FILE" "compose web-canary port mapping missing"
assert_contains "host\\.docker\\.internal:host-gateway" "$PROD_COMPOSE_FILE" "compose caddy host-gateway mapping missing"
assert_contains "\\./caddy/upstreams:/etc/caddy/upstreams:ro" "$PROD_COMPOSE_FILE" "compose missing caddy upstream include mount"
assert_not_contains "\\./env/prod/pay\\.env" "$PROD_COMPOSE_FILE" "compose must not reference retired pay.env"
assert_not_contains "python -m app\\.worker_entrypoint app\\.worker\\.WorkerSettings" "$PROD_COMPOSE_FILE" "prod compose must not reference Python worker entrypoint"

assert_contains "image: rust:1\\.93-bookworm" "$DEV_COMPOSE_FILE" "dev compose must use Rust toolchain image for rust services"
assert_contains "cargo run -p bominal-rust-api" "$DEV_COMPOSE_FILE" "dev compose api/web must run rust api package"
assert_contains "cargo run -p bominal-rust-worker" "$DEV_COMPOSE_FILE" "dev compose worker must run rust worker package"
assert_not_contains "python -m app\\.worker_entrypoint app\\.worker\\.WorkerSettings" "$DEV_COMPOSE_FILE" "dev compose must not reference Python worker entrypoint"

assert_contains "/health/live /health/ready" "$CADDY_FILE" "caddy api matcher missing live/ready routes"
assert_contains "handle_errors" "$CADDY_FILE" "caddy missing callback unavailability error handler"
assert_contains "path /auth/verify\\* /auth/confirm\\*" "$CADDY_FILE" "caddy missing callback error scope matcher"
assert_contains "import /etc/caddy/upstreams/active\\.caddy" "$CADDY_FILE" "caddy missing runtime upstream include"
assert_contains "output stdout" "$CADDY_FILE" "caddy missing request log output config"
assert_contains "to web:3000" "$CADDY_ACTIVE_UPSTREAM_FILE" "caddy active upstream default must target primary web"
assert_contains "to web:3000 web-canary:3000" "$CADDY_WEB_CANARY_UPSTREAM_FILE" "caddy canary upstream template missing failover target"

assert_contains "SUPABASE_JWT_AUDIENCE" "$RUST_ENV_EXAMPLE_FILE" "rust env example missing JWT audience contract"
assert_contains "SUPABASE_JWKS_CACHE_SECONDS" "$RUST_ENV_EXAMPLE_FILE" "rust env example missing JWKS cache contract"
assert_contains "RUNTIME_QUEUE_DLQ_KEY" "$RUST_ENV_EXAMPLE_FILE" "rust env example missing queue DLQ contract"
assert_contains "CMD \\[\"/usr/local/bin/bominal-rust-api\"\\]" "$RUST_API_DOCKERFILE" "rust api dockerfile missing binary entrypoint"
assert_contains "CMD \\[\"/usr/local/bin/bominal-rust-worker\"\\]" "$RUST_WORKER_DOCKERFILE" "rust worker dockerfile missing binary entrypoint"

echo "OK: policy/runtime parity checks passed."
