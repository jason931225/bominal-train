#!/usr/bin/env bash
set -euo pipefail

usage() {
  cat <<USAGE
Usage: $(basename "$0") [--repo owner/name] [--labels-file path]

Sync labels from .github/labels.yml to a GitHub repository.
Requires: gh, jq, ruby.
USAGE
}

repo=""
labels_file=".github/labels.yml"

while [[ $# -gt 0 ]]; do
  case "$1" in
    --repo)
      repo="${2:-}"
      shift 2
      ;;
    --labels-file)
      labels_file="${2:-}"
      shift 2
      ;;
    -h|--help)
      usage
      exit 0
      ;;
    *)
      echo "Unknown argument: $1" >&2
      usage >&2
      exit 1
      ;;
  esac
done

if [[ -z "$repo" ]]; then
  repo="$(gh repo view --json nameWithOwner -q .nameWithOwner)"
fi

if [[ ! -f "$labels_file" ]]; then
  echo "labels file not found: $labels_file" >&2
  exit 1
fi

if ! command -v gh >/dev/null 2>&1; then
  echo "gh is required" >&2
  exit 1
fi
if ! command -v jq >/dev/null 2>&1; then
  echo "jq is required" >&2
  exit 1
fi
if ! command -v ruby >/dev/null 2>&1; then
  echo "ruby is required" >&2
  exit 1
fi

labels_json="$(ruby -ryaml -rjson -e 'data = YAML.load_file(ARGV[0]); labels = data.fetch("labels"); puts JSON.generate(labels)' "$labels_file")"

count="$(jq 'length' <<<"$labels_json")"
echo "Syncing ${count} labels to ${repo}"

jq -c '.[]' <<<"$labels_json" | while IFS= read -r row; do
  name="$(jq -r '.name' <<<"$row")"
  color="$(jq -r '.color' <<<"$row")"
  description="$(jq -r '.description' <<<"$row")"

  if [[ -z "$name" || -z "$color" || -z "$description" ]]; then
    echo "Skipping invalid label entry: $row" >&2
    continue
  fi

  encoded="$(jq -rn --arg v "$name" '$v|@uri')"
  if gh api "repos/${repo}/labels/${encoded}" >/dev/null 2>&1; then
    gh label edit "$name" --repo "$repo" --color "$color" --description "$description" >/dev/null
    echo "updated: $name"
  else
    gh label create "$name" --repo "$repo" --color "$color" --description "$description" >/dev/null
    echo "created: $name"
  fi
done

echo "Label sync complete."
