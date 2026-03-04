#!/usr/bin/env bash
set -euo pipefail

fail() {
  printf 'error: %s\n' "$1" >&2
  exit 1
}

require_command() {
  local command_name
  command_name="$1"
  if ! command -v "${command_name}" >/dev/null 2>&1; then
    fail "required command not found: ${command_name}"
  fi
}

main() {
  require_command git

  if ! git rev-parse --show-toplevel >/dev/null 2>&1; then
    fail "must be run from within a git repository"
  fi

  local has_tracked_files
  has_tracked_files=0

  while IFS= read -r tracked_file; do
    if [ "${has_tracked_files}" -eq 0 ]; then
      printf '%s\n' "::error::runtime/frontend/dist must be generated-only and untracked."
      printf '%s\n' "tracked files detected:"
      has_tracked_files=1
    fi
    printf ' - %s\n' "${tracked_file}"
  done < <(git ls-files -- "runtime/frontend/dist/**")

  if [ "${has_tracked_files}" -eq 1 ]; then
    exit 1
  fi
}

main "$@"
