#!/usr/bin/env bash
set -euo pipefail

SCRIPT_NAME="$(basename "$0")"

BASELINE_JSON=""
CANDIDATE_JSON=""
RELATIVE_P95_MIN_IMPROVEMENT="15"
RELATIVE_MEAN_MIN_IMPROVEMENT="10"
ABSOLUTE_P95_MAX="12"
ABSOLUTE_MEAN_MAX="10"

usage() {
  cat <<EOF
Usage: $SCRIPT_NAME --baseline-json FILE --candidate-json FILE [options]

Validate hybrid performance gate (relative improvement + absolute ceilings)
for train task list metrics.

Required:
  --baseline-json FILE              Baseline metrics JSON path
  --candidate-json FILE             Candidate metrics JSON path

Options:
  --relative-p95-min-improvement N  Minimum p95 improvement percent (default: ${RELATIVE_P95_MIN_IMPROVEMENT})
  --relative-mean-min-improvement N Minimum mean improvement percent (default: ${RELATIVE_MEAN_MIN_IMPROVEMENT})
  --absolute-p95-max N              Absolute maximum p95 in ms (default: ${ABSOLUTE_P95_MAX})
  --absolute-mean-max N             Absolute maximum mean in ms (default: ${ABSOLUTE_MEAN_MAX})
  --help                            Show this help message
EOF
}

error() {
  echo "[ERROR] $*" >&2
}

require_number() {
  local value="$1"
  local flag="$2"
  if [[ ! "$value" =~ ^[0-9]+([.][0-9]+)?$ ]]; then
    error "$flag must be a non-negative number (got: $value)"
    exit 1
  fi
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
    --help|-h)
      usage
      exit 0
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
if [[ -z "$CANDIDATE_JSON" ]]; then
  error "--candidate-json is required"
  exit 1
fi
if [[ ! -f "$BASELINE_JSON" ]]; then
  error "Baseline metrics file not found: $BASELINE_JSON"
  exit 1
fi
if [[ ! -f "$CANDIDATE_JSON" ]]; then
  error "Candidate metrics file not found: $CANDIDATE_JSON"
  exit 1
fi

require_number "$RELATIVE_P95_MIN_IMPROVEMENT" "--relative-p95-min-improvement"
require_number "$RELATIVE_MEAN_MIN_IMPROVEMENT" "--relative-mean-min-improvement"
require_number "$ABSOLUTE_P95_MAX" "--absolute-p95-max"
require_number "$ABSOLUTE_MEAN_MAX" "--absolute-mean-max"

python3 - "$BASELINE_JSON" "$CANDIDATE_JSON" "$RELATIVE_P95_MIN_IMPROVEMENT" "$RELATIVE_MEAN_MIN_IMPROVEMENT" "$ABSOLUTE_P95_MAX" "$ABSOLUTE_MEAN_MAX" <<'PY'
import json
import sys
from typing import Dict, List

baseline_path, candidate_path, rel_p95_req, rel_mean_req, abs_p95_max, abs_mean_max = sys.argv[1:]
rel_p95_req = float(rel_p95_req)
rel_mean_req = float(rel_mean_req)
abs_p95_max = float(abs_p95_max)
abs_mean_max = float(abs_mean_max)

with open(baseline_path, encoding="utf-8") as fh:
    baseline = json.load(fh)
with open(candidate_path, encoding="utf-8") as fh:
    candidate = json.load(fh)

statuses = ("active", "completed")
required_metrics = ("mean", "p95")

def read_metric(payload, status, metric):
    if status not in payload or not isinstance(payload[status], dict):
        raise ValueError(f"missing status '{status}'")
    if metric not in payload[status]:
        raise ValueError(f"missing metric '{status}.{metric}'")
    value = float(payload[status][metric])
    if value < 0:
        raise ValueError(f"metric '{status}.{metric}' cannot be negative")
    return value

failures: List[str] = []
rows: List[Dict[str, float]] = []
for status in statuses:
    baseline_mean = read_metric(baseline, status, "mean")
    baseline_p95 = read_metric(baseline, status, "p95")
    candidate_mean = read_metric(candidate, status, "mean")
    candidate_p95 = read_metric(candidate, status, "p95")

    if baseline_mean <= 0:
        failures.append(f"{status}.mean baseline must be > 0 for relative check")
        continue
    if baseline_p95 <= 0:
        failures.append(f"{status}.p95 baseline must be > 0 for relative check")
        continue

    mean_improvement = ((baseline_mean - candidate_mean) / baseline_mean) * 100.0
    p95_improvement = ((baseline_p95 - candidate_p95) / baseline_p95) * 100.0

    rows.append(
        {
            "status": status,
            "baseline_mean": baseline_mean,
            "candidate_mean": candidate_mean,
            "mean_improvement": mean_improvement,
            "baseline_p95": baseline_p95,
            "candidate_p95": candidate_p95,
            "p95_improvement": p95_improvement,
        }
    )

    if mean_improvement < rel_mean_req:
        failures.append(
            f"{status}.mean improvement {mean_improvement:.2f}% is below required {rel_mean_req:.2f}%"
        )
    if p95_improvement < rel_p95_req:
        failures.append(
            f"{status}.p95 improvement {p95_improvement:.2f}% is below required {rel_p95_req:.2f}%"
        )
    if candidate_mean > abs_mean_max:
        failures.append(
            f"{status}.mean {candidate_mean:.2f}ms exceeds absolute max {abs_mean_max:.2f}ms"
        )
    if candidate_p95 > abs_p95_max:
        failures.append(
            f"{status}.p95 {candidate_p95:.2f}ms exceeds absolute max {abs_p95_max:.2f}ms"
        )

print("Hybrid benchmark gate summary:")
for row in rows:
    print(
        f"- {row['status']}: mean {row['baseline_mean']:.2f}->{row['candidate_mean']:.2f} "
        f"({row['mean_improvement']:.2f}%); p95 {row['baseline_p95']:.2f}->{row['candidate_p95']:.2f} "
        f"({row['p95_improvement']:.2f}%)"
    )

if failures:
    print("Gate result: FAIL")
    for failure in failures:
        print(f"  - {failure}")
    sys.exit(1)

print("Gate result: PASS")
PY
