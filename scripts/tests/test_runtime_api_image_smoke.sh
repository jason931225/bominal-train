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
  require_command docker

  local repo_root
  if ! repo_root="$(git rev-parse --show-toplevel 2>/dev/null)"; then
    fail "must be run from within a git repository"
  fi

  local frontend_dist
  local dockerfile_path
  local build_context
  frontend_dist="${repo_root}/runtime/frontend/dist"
  dockerfile_path="${repo_root}/runtime/Dockerfile.api"
  build_context="${repo_root}/runtime"

  if [ ! -f "${dockerfile_path}" ]; then
    fail "missing Dockerfile: ${dockerfile_path}"
  fi

  if ! docker info >/dev/null 2>&1; then
    fail "docker daemon is not available; start Docker and retry"
  fi

  rm -rf "${frontend_dist}"
  if [ -e "${frontend_dist}" ]; then
    fail "failed to remove frontend dist directory: ${frontend_dist}"
  fi

  docker build --file "${dockerfile_path}" "${build_context}"
}

main "$@"
