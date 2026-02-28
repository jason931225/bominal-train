#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
SCRIPT_PATH="$ROOT_DIR/infra/scripts/db-deep-dive.sh"

source "$SCRIPT_PATH" >/dev/null 2>&1 || true

assert_eq() {
  local expected="$1"
  local actual="$2"
  local message="$3"
  if [[ "$expected" != "$actual" ]]; then
    echo "FAIL: $message" >&2
    echo "expected: $expected" >&2
    echo "actual:   $actual" >&2
    exit 1
  fi
}

normalized="$(normalize_db_url_for_asyncpg 'postgresql+asyncpg://u:p@db.example.com:5432/postgres?sslmode=require')"
assert_eq \
  "postgresql://u:p@db.example.com:5432/postgres?ssl=require" \
  "$normalized" \
  "postgresql+asyncpg + sslmode should normalize for asyncpg"

normalized_existing_ssl="$(normalize_db_url_for_asyncpg 'postgresql+asyncpg://u:p@db.example.com:5432/postgres?ssl=require&sslmode=require')"
assert_eq \
  "postgresql://u:p@db.example.com:5432/postgres?ssl=require" \
  "$normalized_existing_ssl" \
  "existing ssl should win and sslmode should be dropped"

normalized_plain_postgres="$(normalize_db_url_for_asyncpg 'postgresql://u:p@db.example.com:5432/postgres?sslmode=require')"
assert_eq \
  "postgresql://u:p@db.example.com:5432/postgres?ssl=require" \
  "$normalized_plain_postgres" \
  "postgresql URL should also drop sslmode for asyncpg DSN compatibility"

echo "OK: db-deep-dive DSN normalization validated."
