#!/usr/bin/env bash
# Shared environment-file helpers for infra scripts.

set -euo pipefail

log_info() {
  echo "[INFO] $*"
}

log_warn() {
  echo "[WARN] $*" >&2
}

log_error() {
  echo "[ERROR] $*" >&2
}

detect_compose_cmd() {
  if docker compose version >/dev/null 2>&1; then
    COMPOSE_CMD=(docker compose)
    return 0
  fi
  if command -v docker-compose >/dev/null 2>&1; then
    COMPOSE_CMD=(docker-compose)
    return 0
  fi
  log_error "docker compose (v2) or docker-compose (v1) is required"
  return 1
}

require_file() {
  local file="$1"
  if [[ ! -f "$file" ]]; then
    log_error "Missing required file: $file"
    return 1
  fi
  if [[ ! -r "$file" ]]; then
    log_error "Required file is not readable: $file"
    return 1
  fi
  return 0
}

require_nonempty_file() {
  local file="$1"
  require_file "$file" || return 1
  if [[ ! -s "$file" ]]; then
    log_error "Required file is empty: $file"
    return 1
  fi
  return 0
}

copy_env_from_examples() {
  local env_dir="$1"
  local copied=0
  local found=0
  local example_file target_file

  for example_file in "$env_dir"/*.env.example; do
    if [[ -f "$example_file" ]]; then
      found=1
      target_file="${example_file%.example}"
      if [[ ! -f "$target_file" ]]; then
        cp "$example_file" "$target_file"
        log_info "Created env file from example: $(basename "$target_file")"
        copied=1
      fi
    fi
  done

  if [[ "$found" -eq 0 ]]; then
    log_warn "No .env.example files found in $env_dir"
  fi

  if [[ "$copied" -eq 0 ]]; then
    return 1
  fi
  return 0
}

env_key_value() {
  local file="$1"
  local key="$2"
  awk -F'=' -v key="$key" '
    /^[[:space:]]*#/ {next}
    /^[[:space:]]*$/ {next}
    {
      k=$1
      gsub(/^[[:space:]]+|[[:space:]]+$/, "", k)
      if (k == key) {
        v=$0
        sub(/^[^=]*=/, "", v)
        gsub(/^[[:space:]]+|[[:space:]]+$/, "", v)
        gsub(/^"/, "", v)
        gsub(/"$/, "", v)
        print v
        exit
      }
    }
  ' "$file"
}

require_env_key_nonempty() {
  local file="$1"
  local key="$2"
  local value

  require_file "$file" || return 1
  value="$(env_key_value "$file" "$key")"
  if [[ -z "${value}" ]]; then
    log_error "Missing or empty ${key} in ${file}"
    return 1
  fi
  return 0
}

require_no_env_placeholders() {
  local file="$1"
  local pattern="${2:-CHANGE_ME|REPLACE_ME|TODO|<no value>}"

  require_file "$file" || return 1
  if grep -E -n "$pattern" "$file" >/tmp/bominal-env-placeholders.txt 2>/dev/null; then
    log_error "Found unresolved placeholder values in ${file}:"
    cat /tmp/bominal-env-placeholders.txt >&2
    return 1
  fi
  return 0
}
