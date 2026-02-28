#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"

API_ENV_EXAMPLE="$ROOT_DIR/infra/env/prod/api.env.example"
CONFIG_FILE="$ROOT_DIR/api/app/core/config.py"
WALLET_SCHEMA_FILE="$ROOT_DIR/api/app/schemas/wallet.py"
ADMIN_ROUTES_FILE="$ROOT_DIR/api/app/http/routes/admin.py"
PREDEPLOY_FILE="$ROOT_DIR/infra/scripts/predeploy-check.sh"
PROD_COMPOSE_FILE="$ROOT_DIR/infra/docker-compose.prod.yml"
TRAIN_SERVICE_FILE="$ROOT_DIR/api/app/modules/train/service.py"
TRAIN_WORKER_FILE="$ROOT_DIR/api/app/modules/train/worker.py"

fail_if_matches() {
  local pattern="$1"
  local target="$2"
  local message="$3"
  if rg -n -- "$pattern" "$target" >/dev/null; then
    echo "FAIL: $message" >&2
    rg -n -- "$pattern" "$target" >&2 || true
    exit 1
  fi
}

assert_matches() {
  local pattern="$1"
  local target="$2"
  local message="$3"
  if ! rg -n -- "$pattern" "$target" >/dev/null; then
    echo "FAIL: $message" >&2
    exit 1
  fi
}

# Production/default config must not default to server-side fallback.
assert_matches 'AUTOPAY_ALLOW_SERVER_FALLBACK"\)' "$CONFIG_FILE" "config missing AUTOPAY_ALLOW_SERVER_FALLBACK setting"
assert_matches 'default=False, alias="AUTOPAY_ALLOW_SERVER_FALLBACK"' "$CONFIG_FILE" "AUTOPAY_ALLOW_SERVER_FALLBACK default must remain false"
fail_if_matches '^AUTOPAY_ALLOW_SERVER_FALLBACK=true$' "$API_ENV_EXAMPLE" "api.env.example must not enable server fallback"
assert_matches '^PAYMENT_PROVIDER=evervault$' "$API_ENV_EXAMPLE" "api.env.example must default to PAYMENT_PROVIDER=evervault"
fail_if_matches '^PAYMENT_PROVIDER=legacy$' "$API_ENV_EXAMPLE" "legacy payment provider default must not be present in api.env.example"
fail_if_matches 'alias=\"CARDNUMBER\"|alias=\"EXPIRYMM\"|alias=\"EXPIRYYY\"|alias=\"DOB\"|alias=\"NN\"' "$CONFIG_FILE" "legacy backend card aliases must not exist in runtime config"

# Wallet API contract must continue rejecting plaintext CVV and plaintext card fields in evervault mode.
assert_matches 'field is no longer accepted' "$WALLET_SCHEMA_FILE" "wallet schema must reject cvv/cvc/security_code input"
assert_matches 'plaintext card fields are not accepted when PAYMENT_PROVIDER=evervault' "$WALLET_SCHEMA_FILE" "wallet schema must reject plaintext fallback in evervault mode"

# CVV cache writes must not be called from runtime paths.
cvv_cache_calls="$(rg -n "_cache_cvv\(" "$ROOT_DIR/api/app" || true)"
if [[ -n "$cvv_cache_calls" ]]; then
  non_definition_calls="$(printf '%s\n' "$cvv_cache_calls" | grep -v 'def _cache_cvv' || true)"
  if [[ -n "$non_definition_calls" ]]; then
    echo "FAIL: detected runtime calls to _cache_cvv (CVV cache write-path regression)" >&2
    printf '%s\n' "$non_definition_calls" >&2
    exit 1
  fi
fi

# Admin card override endpoints must stay retired (410 contract).
assert_matches 'HTTP_410_GONE' "$ADMIN_ROUTES_FILE" "admin payment card routes must return 410 Gone"
fail_if_matches 'AdminPaymentCardUpdateRequest' "$ADMIN_ROUTES_FILE" "admin card update request schema must be removed"
fail_if_matches 'set_system_payment_card|clear_system_payment_card' "$ADMIN_ROUTES_FILE" "admin routes must not call serverwide card custody services"
assert_matches 'wallet_only: bool' "$ADMIN_ROUTES_FILE" "admin payment settings response must expose wallet_only contract"
fail_if_matches 'source: Literal\["server_override", "pay_env", "none"\]' "$ADMIN_ROUTES_FILE" "admin payment settings response must not expose legacy source variants"

# Legacy alias and pay.env deploy contracts must remain blocked.
assert_matches 'Legacy backend card alias' "$PREDEPLOY_FILE" "predeploy must fail on legacy backend alias usage"
fail_if_matches '\./env/prod/pay\.env' "$PROD_COMPOSE_FILE" "prod compose must not load pay.env"

# Runtime kill switch must guard payment dispatch before provider/Evervault pay calls.
assert_matches 'if not await is_payment_runtime_enabled\(db\):' "$TRAIN_SERVICE_FILE" "manual pay path must enforce runtime kill switch"
worker_guard_count="$(rg -c 'if not await is_payment_runtime_enabled\(db\):' "$TRAIN_WORKER_FILE" || true)"
if [[ "${worker_guard_count:-0}" -lt 2 ]]; then
  echo "FAIL: worker pay paths must enforce runtime kill switch before dispatch" >&2
  exit 1
fi

# pay.env card aliases should not leak into request/runtime orchestration paths.
runtime_paths=(
  "$ROOT_DIR/api/app/http/routes"
  "$ROOT_DIR/api/app/modules/train"
)
for path in "${runtime_paths[@]}"; do
  if [[ -d "$path" ]]; then
    fail_if_matches '\b(CARDNUMBER|EXPIRYMM|EXPIRYYY|DOB|NN)\b' "$path" "pay.env card aliases leaked into runtime path: $path"
  fi
done

echo "OK: payment boundary regression checks passed."
