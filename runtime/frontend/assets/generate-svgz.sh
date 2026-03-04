#!/usr/bin/env bash
set -euo pipefail

ASSET_ROOT="${1:-$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)}"

if ! command -v gzip >/dev/null 2>&1; then
  echo "[generate-svgz] gzip is required but not available on PATH" >&2
  exit 1
fi

if [[ ! -d "$ASSET_ROOT" ]]; then
  echo "[generate-svgz] asset directory not found: $ASSET_ROOT" >&2
  exit 1
fi

generated_count=0

while IFS= read -r -d '' svg_path; do
  svgz_path="${svg_path%.svg}.svgz"
  gzip -9 -c "$svg_path" > "$svgz_path"
  generated_count=$((generated_count + 1))
done < <(find "$ASSET_ROOT" -type f -name "*.svg" -print0)

echo "[generate-svgz] generated $generated_count svgz files under $ASSET_ROOT"
