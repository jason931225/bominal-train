#!/usr/bin/env bash
set -euo pipefail

fail() {
  printf 'error: %s\n' "$1" >&2
  exit 1
}

main() {
  local repo_root
  if ! repo_root="$(git rev-parse --show-toplevel 2>/dev/null)"; then
    fail "must be run from within a git repository"
  fi

  local script_path="${repo_root}/scripts/clean-runtime-debug.sh"
  if [ ! -x "${script_path}" ]; then
    fail "cleanup script is missing or not executable: ${script_path}"
  fi

  local runtime_target="${repo_root}/runtime/target"
  local debug_dir="${runtime_target}/debug"
  local release_dir="${runtime_target}/release"
  local debug_marker="${debug_dir}/tdd-marker.txt"
  local release_marker="${release_dir}/tdd-release-marker.txt"

  mkdir -p "${debug_dir}" "${release_dir}"
  printf 'debug-artifact\n' > "${debug_marker}"
  printf 'release-artifact\n' > "${release_marker}"

  "${script_path}"

  if [ -d "${debug_dir}" ]; then
    fail "expected debug directory to be removed: ${debug_dir}"
  fi

  if [ ! -f "${release_marker}" ]; then
    fail "release artifacts should not be removed by debug cleanup"
  fi

  "${script_path}"

  rm -f "${release_marker}"
  rmdir "${release_dir}" 2>/dev/null || true
}

main "$@"
