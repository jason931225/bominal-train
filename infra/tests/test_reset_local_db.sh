#!/usr/bin/env bash
set -euo pipefail

SCRIPT="infra/scripts/reset-local-db.sh"

if [[ ! -x "$SCRIPT" ]]; then
  echo "FAIL: reset script is missing or not executable: $SCRIPT" >&2
  exit 1
fi

if ! "$SCRIPT" --help | grep -q -- "--fresh-schema"; then
  echo "FAIL: help output must mention --fresh-schema" >&2
  exit 1
fi

if "$SCRIPT" >/dev/null 2>&1; then
  echo "FAIL: script should require --yes confirmation" >&2
  exit 1
fi

if "$SCRIPT" --compose-file infra/docker-compose.prod.yml --yes >/dev/null 2>&1; then
  echo "FAIL: script should block non-dev compose files without --allow-non-dev" >&2
  exit 1
fi

echo "OK: reset-local-db script guard checks passed."
