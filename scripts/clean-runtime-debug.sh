#!/usr/bin/env bash
set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
DEBUG_DIR="${REPO_ROOT}/runtime/target/debug"

usage() {
  cat <<EOF
Usage: ./scripts/clean-runtime-debug.sh

Removes local Rust dev build artifacts under runtime/target/debug.
EOF
}

fail() {
  printf 'error: %s\n' "$1" >&2
  exit 1
}

dir_size_human() {
  local path="$1"
  if command -v du >/dev/null 2>&1; then
    du -sh "${path}" 2>/dev/null | awk '{print $1}' || true
  fi
}

main() {
  if [ "${1:-}" = "--help" ] || [ "${1:-}" = "-h" ]; then
    usage
    exit 0
  fi

  if [ $# -gt 0 ]; then
    usage >&2
    exit 2
  fi

  if [ ! -d "${DEBUG_DIR}" ]; then
    printf '[clean-runtime-debug] no-op: %s is already absent\n' "${DEBUG_DIR}"
    exit 0
  fi

  local before_size
  before_size="$(dir_size_human "${DEBUG_DIR}")"

  rm -rf "${DEBUG_DIR}"

  if [ -e "${DEBUG_DIR}" ]; then
    fail "failed to remove debug artifacts: ${DEBUG_DIR}"
  fi

  if [ -n "${before_size}" ]; then
    printf '[clean-runtime-debug] removed %s (%s)\n' "${DEBUG_DIR}" "${before_size}"
  else
    printf '[clean-runtime-debug] removed %s\n' "${DEBUG_DIR}"
  fi
}

main "$@"
