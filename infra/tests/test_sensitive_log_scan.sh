#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR=$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)
cd "$ROOT_DIR"

TMP_DIR=$(mktemp -d)
trap 'rm -rf "$TMP_DIR"' EXIT

SAFE_LOG="$TMP_DIR/safe.log"
UNSAFE_LOG="$TMP_DIR/unsafe.log"

cat > "$SAFE_LOG" <<'LOG'
2026-02-22 INFO payment attempt meta={"authorization":"[REDACTED]","cvv":"[REDACTED]"}
2026-02-22 INFO payload card=[REDACTED_PAN_****1111]
LOG

cat > "$UNSAFE_LOG" <<'LOG'
2026-02-22 ERROR payment failed card=4111 1111 1111 1111
2026-02-22 ERROR headers Authorization: Bearer abc123
LOG

python3 infra/scripts/scan_sensitive_logs.py "$SAFE_LOG" >/dev/null

if python3 infra/scripts/scan_sensitive_logs.py "$UNSAFE_LOG" >/dev/null 2>&1; then
  echo "FAIL: unsafe log unexpectedly passed scanner" >&2
  exit 1
fi

echo "PASS: sensitive log scanner detects unsafe patterns"
