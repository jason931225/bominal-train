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

  local repo_root
  if ! repo_root="$(git rev-parse --show-toplevel 2>/dev/null)"; then
    fail "must be run from within a git repository"
  fi

  local tracked_files
  if ! tracked_files="$(git -C "${repo_root}" ls-files -- "runtime/frontend/dist/**")"; then
    fail "failed to query tracked files under runtime/frontend/dist/**"
  fi

  if [ -n "${tracked_files}" ]; then
    printf '%s\n' "::error::runtime/frontend/dist must be generated-only and untracked."
    printf '%s\n' "tracked files detected:"
    while IFS= read -r tracked_file; do
      printf ' - %s\n' "${tracked_file}"
    done <<< "${tracked_files}"
    exit 1
  fi
}

main "$@"
