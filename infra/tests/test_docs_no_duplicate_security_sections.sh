#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
SECURITY_DOC="$ROOT_DIR/docs/humans/security/SECURITY.md"

if [[ ! -f "$SECURITY_DOC" ]]; then
  echo "FAIL: missing security doc: $SECURITY_DOC" >&2
  exit 1
fi

duplicate_headers="$(
  awk '
    /^##[[:space:]]+/ {
      header=$0
      sub(/^##[[:space:]]+/, "", header)
      gsub(/[[:space:]]+/, " ", header)
      key=tolower(header)
      seen[key]++
      if (seen[key] == 2) {
        print header
      }
    }
  ' "$SECURITY_DOC"
)"

if [[ -n "$duplicate_headers" ]]; then
  echo "FAIL: duplicate level-2 security sections detected:" >&2
  while IFS= read -r line; do
    [[ -n "$line" ]] || continue
    echo "  - $line" >&2
  done <<<"$duplicate_headers"
  exit 1
fi

pci_heading_count="$(grep -c '^## PCI Relay Worker Isolation Policy$' "$SECURITY_DOC" || true)"
if [[ "$pci_heading_count" -ne 1 ]]; then
  echo "FAIL: expected exactly one 'PCI Relay Worker Isolation Policy' section, found $pci_heading_count" >&2
  exit 1
fi

echo "OK: security doc contains no duplicate section headings."
