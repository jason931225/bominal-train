#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
README_PATH="$ROOT_DIR/docs/README.md"
TMP_POINTERS="$(mktemp)"
trap 'rm -f "$TMP_POINTERS"' EXIT

if [[ ! -f "$README_PATH" ]]; then
  echo "ERROR: missing docs index: $README_PATH" >&2
  exit 1
fi

if ! grep -q '^## Canonical Pointer Library' "$README_PATH"; then
  echo "ERROR: docs/README.md missing '## Canonical Pointer Library' section" >&2
  exit 1
fi

SECTION_CONTENT="$({
  awk '
    /^## Canonical Pointer Library/ {in_section=1; next}
    /^## / && in_section {exit}
    in_section {print}
  ' "$README_PATH"
})"

POINTER_LINES="$(printf '%s\n' "$SECTION_CONTENT" | grep -E '^- \[PTR-' || true)"
if [[ -z "$POINTER_LINES" ]]; then
  echo "ERROR: no pointer entries found (expected lines starting with '- [PTR-')" >&2
  exit 1
fi

BAD_FORMAT="$(printf '%s\n' "$POINTER_LINES" | grep -Ev '^- \[PTR-[A-Z]+-[0-9]{3}\] `[^`]+` - .+$' || true)"
if [[ -n "$BAD_FORMAT" ]]; then
  echo "ERROR: pointer lines do not follow required format" >&2
  echo "$BAD_FORMAT" >&2
  exit 1
fi

printf '%s\n' "$POINTER_LINES" | grep -oE '`[^`]+`' | tr -d '`' > "$TMP_POINTERS"

if [[ ! -s "$TMP_POINTERS" ]]; then
  echo "ERROR: no pointer paths parsed from pointer entries" >&2
  exit 1
fi

DUPES="$(sort "$TMP_POINTERS" | uniq -d || true)"
if [[ -n "$DUPES" ]]; then
  echo "ERROR: duplicate pointers found:" >&2
  echo "$DUPES" >&2
  exit 1
fi

while IFS= read -r path; do
  if [[ ! -e "$ROOT_DIR/$path" ]]; then
    echo "ERROR: pointer target does not exist: $path" >&2
    exit 1
  fi
done < "$TMP_POINTERS"

required=(
  "AGENTS.md"
  "docs/START_HERE.md"
  "docs/README.md"
  "docs/INTENT_ROUTING.md"
  "docs/agents/EXECUTION_PROTOCOL.md"
  "docs/agents/PERMISSIONS.md"
  "docs/agents/GUARDRAILS.md"
  "docs/governance/PRODUCTION_POLICY.md"
  "docs/governance/DEPRECATION_POLICY.md"
  "docs/humans/operations/DEPLOYMENT.md"
  "docs/humans/operations/RUNBOOK.md"
  "docs/plans/active/README.md"
  "CHANGELOG.md"
  "infra/tests/test_intent_routing.sh"
  "infra/tests/test_docs_consistency.sh"
  "infra/tests/test_docs_audience_split.sh"
)

for req in "${required[@]}"; do
  if ! grep -Fxq "$req" "$TMP_POINTERS"; then
    echo "ERROR: required pointer missing from Canonical Pointer Library: $req" >&2
    exit 1
  fi
done

count="$(wc -l < "$TMP_POINTERS" | tr -d ' ')"
echo "OK: Canonical pointer library is valid (${count} pointers)."
