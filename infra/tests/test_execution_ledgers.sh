#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
LOCK_FILE="$ROOT_DIR/docs/LOCK.md"
REQUEST_FILE="$ROOT_DIR/docs/REQUEST.md"

for f in "$LOCK_FILE" "$REQUEST_FILE"; do
  if [[ ! -f "$f" ]]; then
    echo "ERROR: missing ledger file: $f" >&2
    exit 1
  fi
done

extract_section() {
  local file="$1"
  local section="$2"
  awk -v section="$section" '
    $0 == section {in_section=1; next}
    /^## / && in_section {exit}
    in_section {print}
  ' "$file"
}

if ! grep -Fq "## Current Entries" "$LOCK_FILE"; then
  echo "ERROR: docs/LOCK.md must include '## Current Entries'" >&2
  exit 1
fi
if ! grep -Fq "## Current Entries" "$REQUEST_FILE"; then
  echo "ERROR: docs/REQUEST.md must include '## Current Entries'" >&2
  exit 1
fi
if ! grep -Fq "## Template" "$LOCK_FILE"; then
  echo "ERROR: docs/LOCK.md must include a template section" >&2
  exit 1
fi
if ! grep -Fq "## Template" "$REQUEST_FILE"; then
  echo "ERROR: docs/REQUEST.md must include a template section" >&2
  exit 1
fi

lock_template="$(extract_section "$LOCK_FILE" "## Template (Non-live Example)")"
request_template="$(extract_section "$REQUEST_FILE" "## Template (Non-live Example)")"
lock_current="$(extract_section "$LOCK_FILE" "## Current Entries")"
request_current="$(extract_section "$REQUEST_FILE" "## Current Entries")"

if [[ -z "${lock_template//[[:space:]]/}" ]]; then
  echo "ERROR: docs/LOCK.md template section is empty" >&2
  exit 1
fi
if [[ -z "${request_template//[[:space:]]/}" ]]; then
  echo "ERROR: docs/REQUEST.md template section is empty" >&2
  exit 1
fi
if [[ -z "${lock_current//[[:space:]]/}" ]]; then
  echo "ERROR: docs/LOCK.md current entries section is empty" >&2
  exit 1
fi
if [[ -z "${request_current//[[:space:]]/}" ]]; then
  echo "ERROR: docs/REQUEST.md current entries section is empty" >&2
  exit 1
fi

if printf '%s\n' "$lock_template" | grep -Eq '^- status: ACTIVE$'; then
  echo "ERROR: docs/LOCK.md template must not use live status ACTIVE" >&2
  exit 1
fi
if printf '%s\n' "$request_template" | grep -Eq '^- status: OPEN$'; then
  echo "ERROR: docs/REQUEST.md template must not use live status OPEN" >&2
  exit 1
fi

if printf '%s\n' "$lock_current" | grep -Eiq 'Session A|Session B|<[^>]+>|LOCK-EXAMPLE'; then
  echo "ERROR: docs/LOCK.md current entries contains template placeholders" >&2
  exit 1
fi
if printf '%s\n' "$request_current" | grep -Eiq 'Session A|Session B|<[^>]+>|REQ-EXAMPLE'; then
  echo "ERROR: docs/REQUEST.md current entries contains template placeholders" >&2
  exit 1
fi

echo "OK: execution ledgers are structurally valid and template-safe."
