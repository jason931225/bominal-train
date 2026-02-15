#!/usr/bin/env bash
set -euo pipefail

SCRIPT="infra/scripts/benchmark-train-task-list.sh"

if [[ ! -x "$SCRIPT" ]]; then
  echo "FAIL: benchmark script is missing or not executable: $SCRIPT" >&2
  exit 1
fi

if ! "$SCRIPT" --help | grep -q "p50/p95"; then
  echo "FAIL: help output must mention p50/p95 metrics" >&2
  exit 1
fi

if "$SCRIPT" --iterations 0 >/dev/null 2>&1; then
  echo "FAIL: --iterations 0 should fail validation" >&2
  exit 1
fi

if "$SCRIPT" --refresh-completed maybe >/dev/null 2>&1; then
  echo "FAIL: invalid boolean for --refresh-completed should fail" >&2
  exit 1
fi

echo "OK: benchmark script validation checks passed."
