#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"

CONFIG_FILE="$ROOT_DIR/api/app/core/config.py"
PREDEPLOY_FILE="$ROOT_DIR/infra/scripts/predeploy-check.sh"
DEPLOY_FILE="$ROOT_DIR/infra/scripts/deploy.sh"
SYNC_SUPABASE_AUTH_TEMPLATES_SCRIPT="$ROOT_DIR/infra/scripts/sync-supabase-auth-templates.sh"
COMPOSE_FILE="$ROOT_DIR/infra/docker-compose.prod.yml"
CADDY_FILE="$ROOT_DIR/infra/caddy/Caddyfile"
WALLET_SCHEMA_FILE="$ROOT_DIR/api/app/schemas/wallet.py"
ADMIN_ROUTES_FILE="$ROOT_DIR/api/app/http/routes/admin.py"

assert_contains() {
  local pattern="$1"
  local file="$2"
  local message="$3"
  if ! rg -n -- "$pattern" "$file" >/dev/null; then
    echo "FAIL: $message ($file)" >&2
    exit 1
  fi
}

assert_contains "alias=\"INTERNAL_API_KEY_SECRET_ID\"" "$CONFIG_FILE" "config missing INTERNAL_API_KEY secret id alias"
assert_contains "alias=\"INTERNAL_API_KEY_SECRET_VERSION\"" "$CONFIG_FILE" "config missing INTERNAL_API_KEY secret version alias"
assert_contains "alias=\"RESEND_API_KEY_SECRET_ID\"" "$CONFIG_FILE" "config missing RESEND_API_KEY secret id alias"
assert_contains "alias=\"RESEND_API_KEY_SECRET_VERSION\"" "$CONFIG_FILE" "config missing RESEND_API_KEY secret version alias"

assert_contains "INTERNAL_API_KEY and INTERNAL_API_KEY_SECRET_ID cannot both be set" "$PREDEPLOY_FILE" "predeploy missing internal key ambiguity guard"
assert_contains "RESEND API key source is ambiguous" "$PREDEPLOY_FILE" "predeploy missing resend source ambiguity guard"
assert_contains "RESEND_API_KEY_VAULT_NAME is allowed only when EDGE_TASK_NOTIFY_ENABLED=true" "$PREDEPLOY_FILE" "predeploy missing resend vault edge-mode guard"
assert_contains "RESEND_API_KEY_SECRET_VERSION" "$PREDEPLOY_FILE" "predeploy missing resend secret version contract"
assert_contains "INTERNAL_API_KEY_SECRET_VERSION" "$PREDEPLOY_FILE" "predeploy missing internal secret version contract"
assert_contains "SUPABASE_MANAGEMENT_API_TOKEN_SECRET_ID" "$PREDEPLOY_FILE" "predeploy missing supabase management token gsm source contract"
assert_contains "SUPABASE_MANAGEMENT_API_TOKEN must not be set in infra/env/prod/api.env" "$PREDEPLOY_FILE" "predeploy missing plaintext management token guard"
assert_contains "PAYMENT_PROVIDER" "$PREDEPLOY_FILE" "predeploy missing payment-provider contract checks"

assert_contains "load_runtime_api_secrets_from_gsm" "$DEPLOY_FILE" "deploy missing runtime secret loader"
assert_contains "INTERNAL_API_KEY_SECRET_ID" "$DEPLOY_FILE" "deploy missing internal gsm reference resolution"
assert_contains "RESEND_API_KEY_SECRET_ID" "$DEPLOY_FILE" "deploy missing resend gsm reference resolution"
assert_contains "export INTERNAL_API_KEY" "$DEPLOY_FILE" "deploy does not export INTERNAL_API_KEY runtime value"
assert_contains "DEPLOY_WEB_CANARY_ENABLED" "$DEPLOY_FILE" "deploy missing web canary toggle"
assert_contains "start_web_canary" "$DEPLOY_FILE" "deploy missing web canary bootstrap"
assert_contains "check_auth_callback_route" "$DEPLOY_FILE" "deploy missing auth callback smoke guard"

assert_contains "SUPABASE_MANAGEMENT_API_TOKEN_SECRET_ID" "$SYNC_SUPABASE_AUTH_TEMPLATES_SCRIPT" "sync script missing supabase management token secret reference"
assert_contains "gcloud secrets versions access" "$SYNC_SUPABASE_AUTH_TEMPLATES_SCRIPT" "sync script missing gsm token resolution"
assert_contains "--apply in production requires SUPABASE_MANAGEMENT_API_TOKEN_SECRET_ID" "$SYNC_SUPABASE_AUTH_TEMPLATES_SCRIPT" "sync script missing production fail-closed token guard"

assert_contains "INTERNAL_API_KEY:[[:space:]]+\\$\\{INTERNAL_API_KEY:-\\}" "$COMPOSE_FILE" "compose api/worker missing INTERNAL_API_KEY passthrough"
assert_contains "RESEND_API_KEY:[[:space:]]+\\$\\{RESEND_API_KEY:-\\}" "$COMPOSE_FILE" "compose api/worker missing RESEND_API_KEY passthrough"
assert_contains "web-canary:" "$COMPOSE_FILE" "compose missing web-canary service"
assert_contains "127\\.0\\.0\\.1:3001:3000" "$COMPOSE_FILE" "compose web-canary port mapping missing"
assert_contains "host\\.docker\\.internal:host-gateway" "$COMPOSE_FILE" "compose caddy host-gateway mapping missing"
if rg -n -- '\./env/prod/pay\.env' "$COMPOSE_FILE" >/dev/null; then
  echo "FAIL: compose must not reference retired pay.env" >&2
  exit 1
fi

assert_contains "/health/live /health/ready" "$CADDY_FILE" "caddy api matcher missing live/ready routes"
assert_contains "handle_errors" "$CADDY_FILE" "caddy missing callback unavailability error handler"
assert_contains "path /auth/verify\\* /auth/confirm\\*" "$CADDY_FILE" "caddy missing callback error scope matcher"
assert_contains "web:3000 web-canary:3000" "$CADDY_FILE" "caddy missing dual web upstream failover"

assert_contains "field is no longer accepted" "$WALLET_SCHEMA_FILE" "wallet schema must reject cvv plaintext input"
assert_contains "plaintext card fields are not accepted when PAYMENT_PROVIDER=evervault" "$WALLET_SCHEMA_FILE" "wallet schema must reject plaintext fallback in evervault mode"
assert_contains "wallet_only: bool" "$ADMIN_ROUTES_FILE" "admin payment settings response must expose wallet_only"
assert_contains "HTTP_410_GONE" "$ADMIN_ROUTES_FILE" "admin serverwide card routes must be retired with 410 responses"

echo "OK: policy/runtime parity checks passed."
