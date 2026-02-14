#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
ROUTING_PATH="$ROOT_DIR/docs/INTENT_ROUTING.md"

if [[ ! -f "$ROUTING_PATH" ]]; then
  echo "ERROR: missing intent routing file: $ROUTING_PATH" >&2
  exit 1
fi

required_keywords=(
  "read"
  "clean"
  "hygiene"
  "deploy"
  "rollback"
  "deprecate"
  "resy"
)

for kw in "${required_keywords[@]}"; do
  if ! grep -Eiq "\\b${kw}\\b" "$ROUTING_PATH"; then
    echo "ERROR: required keyword missing from intent routing: $kw" >&2
    exit 1
  fi
done

TMP_PATHS="$(mktemp)"
trap 'rm -f "$TMP_PATHS"' EXIT

grep -oE '`[^`]+`' "$ROUTING_PATH" | tr -d '`' | sort -u > "$TMP_PATHS"

while IFS= read -r path; do
  case "$path" in
    docs/*|AGENTS.md|CHANGELOG.md)
      if [[ ! -e "$ROOT_DIR/$path" ]]; then
        echo "ERROR: intent routing references missing path: $path" >&2
        exit 1
      fi
      ;;
    *)
      ;;
  esac
done < "$TMP_PATHS"

echo "OK: intent routing is valid."
