#!/usr/bin/env bash
set -euo pipefail

COMPARE_SCRIPT="infra/scripts/benchmark-train-task-list-compare.sh"
THRESHOLD_SCRIPT="infra/scripts/benchmark-threshold-check.sh"

if [[ ! -x "$COMPARE_SCRIPT" ]]; then
  echo "FAIL: compare script is missing or not executable: $COMPARE_SCRIPT" >&2
  exit 1
fi

if [[ ! -x "$THRESHOLD_SCRIPT" ]]; then
  echo "FAIL: threshold script is missing or not executable: $THRESHOLD_SCRIPT" >&2
  exit 1
fi

tmp_dir="$(mktemp -d)"
trap 'rm -rf "$tmp_dir"' EXIT

cat >"$tmp_dir/baseline.json" <<'JSON'
{
  "active": { "mean": 10.0, "p50": 9.0, "p95": 12.0, "min": 7.0, "max": 18.0 },
  "completed": { "mean": 11.0, "p50": 10.0, "p95": 13.0, "min": 8.0, "max": 19.0 }
}
JSON

cat >"$tmp_dir/candidate-pass.json" <<'JSON'
{
  "active": { "mean": 8.9, "p50": 8.0, "p95": 9.8, "min": 6.5, "max": 15.0 },
  "completed": { "mean": 9.7, "p50": 9.1, "p95": 10.8, "min": 7.2, "max": 16.0 }
}
JSON

cat >"$tmp_dir/candidate-relative-fail.json" <<'JSON'
{
  "active": { "mean": 9.5, "p50": 8.5, "p95": 10.5, "min": 7.0, "max": 16.0 },
  "completed": { "mean": 10.8, "p50": 9.8, "p95": 12.4, "min": 8.1, "max": 17.0 }
}
JSON

cat >"$tmp_dir/candidate-absolute-fail.json" <<'JSON'
{
  "active": { "mean": 8.8, "p50": 8.0, "p95": 12.3, "min": 6.5, "max": 16.0 },
  "completed": { "mean": 9.8, "p50": 9.2, "p95": 12.1, "min": 7.4, "max": 16.0 }
}
JSON

if ! "$THRESHOLD_SCRIPT" \
  --baseline-json "$tmp_dir/baseline.json" \
  --candidate-json "$tmp_dir/candidate-pass.json" \
  --relative-p95-min-improvement 15 \
  --relative-mean-min-improvement 10 \
  --absolute-p95-max 12 \
  --absolute-mean-max 10 >/dev/null; then
  echo "FAIL: threshold script should pass hybrid gate for candidate-pass fixture" >&2
  exit 1
fi

if "$THRESHOLD_SCRIPT" \
  --baseline-json "$tmp_dir/baseline.json" \
  --candidate-json "$tmp_dir/candidate-relative-fail.json" \
  --relative-p95-min-improvement 15 \
  --relative-mean-min-improvement 10 \
  --absolute-p95-max 12 \
  --absolute-mean-max 10 >/dev/null 2>&1; then
  echo "FAIL: threshold script should fail when relative improvement is below threshold" >&2
  exit 1
fi

if "$THRESHOLD_SCRIPT" \
  --baseline-json "$tmp_dir/baseline.json" \
  --candidate-json "$tmp_dir/candidate-absolute-fail.json" \
  --relative-p95-min-improvement 15 \
  --relative-mean-min-improvement 10 \
  --absolute-p95-max 12 \
  --absolute-mean-max 10 >/dev/null 2>&1; then
  echo "FAIL: threshold script should fail when absolute SLO thresholds are exceeded" >&2
  exit 1
fi

if ! "$COMPARE_SCRIPT" \
  --baseline-json "$tmp_dir/baseline.json" \
  --candidate-json "$tmp_dir/candidate-pass.json" \
  --relative-p95-min-improvement 15 \
  --relative-mean-min-improvement 10 \
  --absolute-p95-max 12 \
  --absolute-mean-max 10 >/dev/null; then
  echo "FAIL: compare script should pass when provided hybrid-pass fixtures" >&2
  exit 1
fi

echo "OK: benchmark compare/threshold hybrid gate checks passed."
