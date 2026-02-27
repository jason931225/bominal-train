#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"

API_ENV_EXAMPLE="$ROOT_DIR/infra/env/prod/api.env.example"
CONFIG_FILE="$ROOT_DIR/api/app/core/config.py"
WALLET_SCHEMA_FILE="$ROOT_DIR/api/app/schemas/wallet.py"

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

# Wallet API contract must continue rejecting plaintext CVV and plaintext card fields in evervault mode.
assert_matches 'field is no longer accepted' "$WALLET_SCHEMA_FILE" "wallet schema must reject cvv/cvc/security_code input"
assert_matches 'plaintext card fields are not accepted when PAYMENT_PROVIDER=evervault' "$WALLET_SCHEMA_FILE" "wallet schema must reject plaintext fallback in evervault mode"

# CVV cache writes must not be called from runtime paths.
cvv_cache_calls="$(rg -n "_cache_cvv\(" "$ROOT_DIR/api/app" || true)"
if [[ -n "$cvv_cache_calls" ]]; then
  non_definition_calls="$(printf '%s\n' "$cvv_cache_calls" | grep -v 'api/app/services/wallet.py:261:' || true)"
  if [[ -n "$non_definition_calls" ]]; then
    echo "FAIL: detected runtime calls to _cache_cvv (CVV cache write-path regression)" >&2
    printf '%s\n' "$non_definition_calls" >&2
    exit 1
  fi
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
