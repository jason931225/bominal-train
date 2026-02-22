#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
SCRIPT="$ROOT_DIR/infra/scripts/version_guard.py"

if [[ ! -f "$SCRIPT" ]]; then
  echo "ERROR: missing version guard script: $SCRIPT" >&2
  exit 1
fi

python3 "$SCRIPT" validate

baseline_version="$(python3 "$SCRIPT" baseline-version)"
if [[ ! "$baseline_version" =~ ^v0\.0\.[0-9]+$ ]]; then
  echo "ERROR: baseline version must remain in v0.0.N series, got: $baseline_version" >&2
  exit 1
fi

baseline_commit="$(python3 "$SCRIPT" baseline-commit)"
resolved_baseline="$(python3 "$SCRIPT" resolve --commit "$baseline_commit")"
if [[ "$resolved_baseline" != "$baseline_version" ]]; then
  echo "ERROR: baseline commit/version parity mismatch: $baseline_commit -> $resolved_baseline" >&2
  exit 1
fi

if git rev-parse --verify "${baseline_commit}^" >/dev/null 2>&1; then
  pre_baseline_commit="$(git rev-parse "${baseline_commit}^")"
  pre_baseline_version="$(python3 "$SCRIPT" resolve --commit "$pre_baseline_commit")"
  if [[ ! "$pre_baseline_version" =~ ^v0\.0\.[0-9]+$ ]]; then
    echo "ERROR: pre-baseline commits must resolve to v0.0.N, got: $pre_baseline_version" >&2
    exit 1
  fi
fi

head_version="$(python3 "$SCRIPT" resolve --commit HEAD)"
if [[ ! "$head_version" =~ ^v0\.0\.[0-9]+$ ]]; then
  echo "ERROR: current version must remain in v0.0.N series, got: $head_version" >&2
  exit 1
fi

echo "OK: semantic version mapping and commit parity checks passed."
