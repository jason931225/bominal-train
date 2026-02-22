#!/usr/bin/env bash
set -euo pipefail

SCRIPT_NAME="$(basename "$0")"
BENCHMARK_SCRIPT="infra/scripts/benchmark-train-task-list.sh"
THRESHOLD_SCRIPT="infra/scripts/benchmark-threshold-check.sh"

BASELINE_JSON=""
CANDIDATE_JSON=""
OUTPUT_JSON=""
RELATIVE_P95_MIN_IMPROVEMENT="15"
RELATIVE_MEAN_MIN_IMPROVEMENT="10"
ABSOLUTE_P95_MAX="12"
ABSOLUTE_MEAN_MAX="10"

run_candidate_benchmark="false"
benchmark_args=()

usage() {
  cat <<EOF
Usage: $SCRIPT_NAME --baseline-json FILE [options]

Compare train task-list benchmark metrics with hybrid threshold gate.

Required:
  --baseline-json FILE                Baseline metrics JSON path

Options:
  --candidate-json FILE               Candidate metrics JSON path (skip live benchmark run)
  --output-json FILE                  Write candidate metrics JSON to this path
  --relative-p95-min-improvement N    Minimum p95 improvement percent (default: ${RELATIVE_P95_MIN_IMPROVEMENT})
  --relative-mean-min-improvement N   Minimum mean improvement percent (default: ${RELATIVE_MEAN_MIN_IMPROVEMENT})
  --absolute-p95-max N                Absolute max p95 in ms (default: ${ABSOLUTE_P95_MAX})
  --absolute-mean-max N               Absolute max mean in ms (default: ${ABSOLUTE_MEAN_MAX})
  --run-live                          Run benchmark script for candidate metrics
  --                                  Remaining args passed to benchmark script
  --help                              Show this help text
EOF
}

error() {
  echo "[ERROR] $*" >&2
}

while [[ $# -gt 0 ]]; do
  case "$1" in
    --baseline-json)
      BASELINE_JSON="$2"
      shift 2
      ;;
    --candidate-json)
      CANDIDATE_JSON="$2"
      shift 2
      ;;
    --output-json)
      OUTPUT_JSON="$2"
      shift 2
      ;;
    --relative-p95-min-improvement)
      RELATIVE_P95_MIN_IMPROVEMENT="$2"
      shift 2
      ;;
    --relative-mean-min-improvement)
      RELATIVE_MEAN_MIN_IMPROVEMENT="$2"
      shift 2
      ;;
    --absolute-p95-max)
      ABSOLUTE_P95_MAX="$2"
      shift 2
      ;;
    --absolute-mean-max)
      ABSOLUTE_MEAN_MAX="$2"
      shift 2
      ;;
    --run-live)
      run_candidate_benchmark="true"
      shift
      ;;
    --help|-h)
      usage
      exit 0
      ;;
    --)
      shift
      benchmark_args+=("$@")
      break
      ;;
    *)
      error "Unknown argument: $1"
      usage
      exit 1
      ;;
  esac
done

if [[ -z "$BASELINE_JSON" ]]; then
  error "--baseline-json is required"
  exit 1
fi
if [[ ! -f "$BASELINE_JSON" ]]; then
  error "Baseline metrics JSON not found: $BASELINE_JSON"
  exit 1
fi

if [[ -n "$CANDIDATE_JSON" && "$run_candidate_benchmark" == "true" ]]; then
  error "--candidate-json and --run-live cannot be used together"
  exit 1
fi

if [[ -z "$CANDIDATE_JSON" ]]; then
  run_candidate_benchmark="true"
fi

if [[ "$run_candidate_benchmark" == "true" ]]; then
  if [[ ! -x "$BENCHMARK_SCRIPT" ]]; then
    error "Benchmark script is missing or not executable: $BENCHMARK_SCRIPT"
    exit 1
  fi
fi

if [[ ! -x "$THRESHOLD_SCRIPT" ]]; then
  error "Threshold script is missing or not executable: $THRESHOLD_SCRIPT"
  exit 1
fi

tmp_dir="$(mktemp -d)"
trap 'rm -rf "$tmp_dir"' EXIT

candidate_json_path="$CANDIDATE_JSON"
if [[ "$run_candidate_benchmark" == "true" ]]; then
  benchmark_report="$tmp_dir/benchmark-report.txt"
  candidate_json_path="$tmp_dir/candidate-metrics.json"

  "$BENCHMARK_SCRIPT" "${benchmark_args[@]}" | tee "$benchmark_report"

  python3 - "$benchmark_report" "$candidate_json_path" <<'PY'
import json
import sys

report_path = sys.argv[1]
output_path = sys.argv[2]
metrics: dict[str, dict[str, float]] = {}

with open(report_path, encoding="utf-8") as fh:
    for raw_line in fh:
        line = raw_line.strip()
        if not line:
            continue
        parts = line.split()
        if parts[0] not in {"active", "completed"}:
            continue
        if len(parts) < 6:
            raise SystemExit(f"Invalid benchmark output row: {line}")
        status = parts[0]
        metrics[status] = {
            "mean": float(parts[1]),
            "p50": float(parts[2]),
            "p95": float(parts[3]),
            "min": float(parts[4]),
            "max": float(parts[5]),
        }

for status in ("active", "completed"):
    if status not in metrics:
        raise SystemExit(f"Missing benchmark row for status '{status}'")

with open(output_path, "w", encoding="utf-8") as out_fh:
    json.dump(metrics, out_fh, indent=2, sort_keys=True)
    out_fh.write("\n")
PY
fi

if [[ -z "$candidate_json_path" || ! -f "$candidate_json_path" ]]; then
  error "Candidate metrics JSON not found: $candidate_json_path"
  exit 1
fi

"$THRESHOLD_SCRIPT" \
  --baseline-json "$BASELINE_JSON" \
  --candidate-json "$candidate_json_path" \
  --relative-p95-min-improvement "$RELATIVE_P95_MIN_IMPROVEMENT" \
  --relative-mean-min-improvement "$RELATIVE_MEAN_MIN_IMPROVEMENT" \
  --absolute-p95-max "$ABSOLUTE_P95_MAX" \
  --absolute-mean-max "$ABSOLUTE_MEAN_MAX"

if [[ -n "$OUTPUT_JSON" ]]; then
  cp "$candidate_json_path" "$OUTPUT_JSON"
  echo "[INFO] Wrote candidate metrics JSON to $OUTPUT_JSON"
fi
