#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR=$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)
TRAIN_CADDYFILE="$ROOT_DIR/infra/egress/train/Caddyfile"
RESTAURANT_CADDYFILE="$ROOT_DIR/infra/egress/restaurant/Caddyfile"

assert_contains() {
  local file_path="$1"
  local pattern="$2"
  if ! grep -Eq "$pattern" "$file_path"; then
    echo "FAIL: expected pattern '$pattern' in $file_path" >&2
    exit 1
  fi
}

for file_path in "$TRAIN_CADDYFILE" "$RESTAURANT_CADDYFILE"; do
  if [[ ! -f "$file_path" ]]; then
    echo "FAIL: missing Caddyfile: $file_path" >&2
    exit 1
  fi

  assert_contains "$file_path" "admin off"
  assert_contains "$file_path" "auto_https off"
  assert_contains "$file_path" "not method GET POST HEAD"
  assert_contains "$file_path" "respond @method_disallowed \"method not allowed\" 405"
  assert_contains "$file_path" "@health path /health"
  assert_contains "$file_path" "respond \"egress route not allowed\" 403"
done

# Train egress routes
assert_contains "$TRAIN_CADDYFILE" "path /srt/\\*"
assert_contains "$TRAIN_CADDYFILE" "path /korail/\\*"
assert_contains "$TRAIN_CADDYFILE" "path /netfunnel/\\*"

# Restaurant egress routes
assert_contains "$RESTAURANT_CADDYFILE" "path /opentable/\\*"
assert_contains "$RESTAURANT_CADDYFILE" "path /resy/\\*"

echo "PASS: egress Caddyfiles enforce deny-by-default route and method gates"
