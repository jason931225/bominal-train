#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
SCRIPT="$ROOT_DIR/infra/scripts/db-slo-check.sh"

TMP_DIR="$(mktemp -d)"
trap 'rm -rf "$TMP_DIR"' EXIT

cat >"$TMP_DIR/samples-ok.txt" <<'EOF_SAMPLES_OK'
120
150
170
180
EOF_SAMPLES_OK

cat >"$TMP_DIR/samples-slow.txt" <<'EOF_SAMPLES_SLOW'
250
900
1200
1500
EOF_SAMPLES_SLOW

cat >"$TMP_DIR/logs-ok.txt" <<'EOF_LOG_OK'
INFO request complete path=/api/auth/me status=200
ERROR TimeoutError on upstream auth call
ERROR QueryCanceledError: canceling statement due to statement timeout
EOF_LOG_OK

cat >"$TMP_DIR/logs-bad.txt" <<'EOF_LOG_BAD'
ERROR TimeoutError on upstream auth call
ERROR TimeoutError on upstream auth call
ERROR QueryCanceledError: canceling statement due to statement timeout
EOF_LOG_BAD

bash "$SCRIPT" \
  --samples-file "$TMP_DIR/samples-ok.txt" \
  --connect-p95-max-ms 250 \
  --api-log-file "$TMP_DIR/logs-ok.txt" \
  --auth-timeout-max 2 >/dev/null

if bash "$SCRIPT" \
  --samples-file "$TMP_DIR/samples-slow.txt" \
  --connect-p95-max-ms 500 \
  --api-log-file "$TMP_DIR/logs-ok.txt" \
  --auth-timeout-max 2 >/dev/null 2>&1; then
  echo "FAIL: expected p95 threshold failure" >&2
  exit 1
fi

if bash "$SCRIPT" \
  --samples-file "$TMP_DIR/samples-ok.txt" \
  --connect-p95-max-ms 250 \
  --api-log-file "$TMP_DIR/logs-bad.txt" \
  --auth-timeout-max 1 >/dev/null 2>&1; then
  echo "FAIL: expected timeout-count threshold failure" >&2
  exit 1
fi

echo "OK: db-slo-check thresholds validated."
