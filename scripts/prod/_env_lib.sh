#!/usr/bin/env bash
set -euo pipefail

log() {
  printf '[prod-deploy] %s\n' "$*"
}

fail() {
  printf '[prod-deploy] error: %s\n' "$*" >&2
  exit 1
}

require_cmd() {
  local cmd="$1"
  command -v "${cmd}" >/dev/null 2>&1 || fail "missing required command: ${cmd}"
}

require_var() {
  local key="$1"
  local value="${!key:-}"
  [ -n "${value}" ] || fail "missing required env var: ${key}"
}

read_env_key() {
  local file="$1"
  local key="$2"
  if [ ! -f "${file}" ]; then
    return 0
  fi
  grep -m1 "^${key}=" "${file}" | sed -E "s/^${key}=//" || true
}

set_env_key() {
  local file="$1"
  local key="$2"
  local value="$3"
  local tmp
  local updated=0

  tmp="$(mktemp)"
  if [ -f "${file}" ]; then
    while IFS= read -r line || [ -n "${line}" ]; do
      if [[ "${line}" == "${key}="* ]]; then
        printf '%s=%s\n' "${key}" "${value}" >> "${tmp}"
        updated=1
      else
        printf '%s\n' "${line}" >> "${tmp}"
      fi
    done < "${file}"
  fi

  if [ "${updated}" = "0" ]; then
    printf '%s=%s\n' "${key}" "${value}" >> "${tmp}"
  fi

  mkdir -p "$(dirname "${file}")"
  mv "${tmp}" "${file}"
}

compose_cmd() {
  local runtime_env_path="$1"
  local compose_file="$2"
  shift 2

  local args=()
  if [ -n "${BOMINAL_COMPOSE_PROJECT_NAME:-}" ]; then
    args+=(--project-name "${BOMINAL_COMPOSE_PROJECT_NAME}")
  fi
  args+=(--env-file "${runtime_env_path}" -f "${compose_file}")

  docker compose "${args[@]}" "$@"
}
