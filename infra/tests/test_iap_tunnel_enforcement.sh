#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT_DIR"

scan_paths=(
  "docs"
  "infra"
  ".github"
)

matches="$(rg -n \
  --glob '!third_party/**' \
  --glob '!infra/tests/test_iap_tunnel_enforcement.sh' \
  'gcloud[[:space:]]+(beta[[:space:]]+)?compute[[:space:]]+ssh' \
  "${scan_paths[@]}" || true)"

if [[ -z "$matches" ]]; then
  echo "PASS: no gcloud compute ssh references found"
  exit 0
fi

status=0

while IFS= read -r match; do
  [[ -z "$match" ]] && continue

  file="${match%%:*}"
  remainder="${match#*:}"
  line="${remainder%%:*}"

  if sed -n "${line},$((line + 6))p" "$file" | rg -q -- '--tunnel-through-iap'; then
    continue
  fi

  echo "FAIL: ${file}:${line} must include --tunnel-through-iap in the SSH command block" >&2
  status=1
done <<<"$matches"

if [[ "$status" -ne 0 ]]; then
  exit 1
fi

echo "PASS: every gcloud compute ssh command enforces --tunnel-through-iap"
