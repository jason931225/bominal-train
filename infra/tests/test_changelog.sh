#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
CHANGELOG_PATH="$ROOT_DIR/CHANGELOG.md"

if [[ ! -f "$CHANGELOG_PATH" ]]; then
  echo "ERROR: missing CHANGELOG.md at repository root" >&2
  exit 1
fi

if ! grep -q '^## Unreleased$' "$CHANGELOG_PATH"; then
  echo "ERROR: CHANGELOG.md missing '## Unreleased' section" >&2
  exit 1
fi

UNRELEASED="$({ awk '
  /^## Unreleased$/ {in_section=1; next}
  /^## / && in_section {exit}
  in_section {print}
' "$CHANGELOG_PATH"; } )"

if [[ -z "${UNRELEASED//[[:space:]]/}" ]]; then
  echo "ERROR: Unreleased section is empty" >&2
  exit 1
fi

CATEGORY_LINES="$(printf '%s\n' "$UNRELEASED" | grep -E '^### ' || true)"
if [[ -z "$CATEGORY_LINES" ]]; then
  echo "ERROR: Unreleased section must contain at least one Keep a Changelog category header (e.g., ### Added)" >&2
  exit 1
fi

INVALID_CATEGORIES="$(printf '%s\n' "$CATEGORY_LINES" | grep -Ev '^### (Added|Changed|Deprecated|Removed|Fixed|Security)$' || true)"
if [[ -n "$INVALID_CATEGORIES" ]]; then
  echo "ERROR: invalid changelog categories in Unreleased:" >&2
  echo "$INVALID_CATEGORIES" >&2
  exit 1
fi

ENTRY_LINES="$(printf '%s\n' "$UNRELEASED" | grep -E '^- ' || true)"
if [[ -z "$ENTRY_LINES" ]]; then
  echo "ERROR: Unreleased section must contain at least one bullet entry" >&2
  exit 1
fi

BAD_ENTRIES="$(printf '%s\n' "$ENTRY_LINES" | grep -Ev '^- \[[0-9a-f]{7,40}\] .+' || true)"
if [[ -n "$BAD_ENTRIES" ]]; then
  echo "ERROR: changelog entries must be commit-based and formatted as '- [<sha>] <description>'" >&2
  echo "$BAD_ENTRIES" >&2
  exit 1
fi

echo "OK: CHANGELOG.md structure and commit-based Unreleased entries are valid."
