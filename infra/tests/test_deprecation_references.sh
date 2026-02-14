#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT_DIR"

DEPRECATED_PATH="infra/docker-compose.deploy.yml.deprecated"

if [[ -e "$DEPRECATED_PATH" ]]; then
  echo "FAIL: deprecated artifact still exists: $DEPRECATED_PATH" >&2
  exit 1
fi

active_refs="$({ rg -n "docker-compose\.deploy\.yml\.deprecated" infra/scripts infra/docker-compose*.yml .github/workflows 2>/dev/null || true; } | sed '/^$/d')"
if [[ -n "$active_refs" ]]; then
  echo "FAIL: active runtime references to deprecated artifact remain:" >&2
  echo "$active_refs" >&2
  exit 1
fi

echo "OK: no active references to removed deprecated deploy artifact."
