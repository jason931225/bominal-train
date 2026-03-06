#!/usr/bin/env bash
set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
PROD_ENV_DIR="${REPO_ROOT}/env/prod"

INTERACTIVE_MODE="auto"
FORCE_WRITE=0
DRY_RUN=0

SELECT_RUNTIME=0
SELECT_CADDY=0
SELECT_DEPLOY=0
SELECT_CLOUDRUN_API=0
SELECT_ANY=0

OVERRIDE_KEYS=()
OVERRIDE_VALUES=()

log() {
  printf '[bootstrap-prod] %s\n' "$*"
}

usage() {
  cat <<EOF
Usage: ./scripts/bootstrap-prod.sh [options]

Generates production env files from templates with two modes:
  - Interactive (default when TTY): prompts for unresolved CHANGE_ME values
  - Non-interactive (CI): fail-closed unless all required values are provided

Options:
  --interactive           Force interactive prompts.
  --non-interactive       Disable prompts (CI-safe fail-closed mode).
  --set KEY=VALUE         Provide/override a value (repeatable).
  --only TARGET           Limit output target: runtime | caddy | deploy | cloudrun-api (repeatable).
  --force                 Overwrite existing target files without prompt.
  --dry-run               Print rendered files to stdout, do not write.
  --help                  Show this help.

Examples:
  ./scripts/bootstrap-prod.sh --interactive
  ./scripts/bootstrap-prod.sh --non-interactive --force \\
    --only deploy \\
    --set GCP_PROJECT_ID=my-project \\
    --set DEPLOY_VM_NAME=bominal-deploy \\
    --set DEPLOY_VM_ZONE=us-central1-a \\
    --set DEPLOY_WORKDIR=/opt/bominal/repo \\
    --set DEPLOY_MIGRATIONS_DIR=/opt/bominal/repo/runtime/migrations

  ./scripts/bootstrap-prod.sh --non-interactive --force \\
    --only cloudrun-api \\
    --set CLOUDRUN_API_IMAGE=us-central1-docker.pkg.dev/my-project/runtime/bominal-api:sha \\
    --set CLOUDRUN_API_SERVICE_ACCOUNT=bominal-api@my-project.iam.gserviceaccount.com \\
    --set CLOUDRUN_API_VPC_NETWORK=default \\
    --set CLOUDRUN_API_VPC_SUBNET=serverless-us-central1
EOF
}

require_cmd() {
  local cmd="$1"
  if ! command -v "${cmd}" >/dev/null 2>&1; then
    echo "missing required command: ${cmd}" >&2
    exit 1
  fi
}

add_override() {
  local key="$1"
  local value="$2"
  local i

  for i in "${!OVERRIDE_KEYS[@]}"; do
    if [ "${OVERRIDE_KEYS[i]}" = "${key}" ]; then
      OVERRIDE_VALUES[i]="${value}"
      return
    fi
  done

  OVERRIDE_KEYS+=("${key}")
  OVERRIDE_VALUES+=("${value}")
}

lookup_override() {
  local key="$1"
  local i
  for i in "${!OVERRIDE_KEYS[@]}"; do
    if [ "${OVERRIDE_KEYS[i]}" = "${key}" ]; then
      printf '%s' "${OVERRIDE_VALUES[i]}"
      return 0
    fi
  done
  return 1
}

is_placeholder_value() {
  local value="$1"
  [[ "${value}" == *"CHANGE_ME"* ]]
}

is_sensitive_key() {
  local key="$1"
  [[ "${key}" == *"PASSWORD"* || "${key}" == *"SECRET"* || "${key}" == *"TOKEN"* || "${key}" == *"MASTER_KEY"* || "${key}" == *"API_KEY"* ]]
}

confirm_overwrite() {
  local path="$1"

  if [ "${FORCE_WRITE}" = "1" ]; then
    return 0
  fi

  if [ "${INTERACTIVE_MODE}" = "1" ]; then
    local answer
    while true; do
      printf 'Overwrite existing file %s? [y/N]: ' "${path}" >/dev/tty
      read -r answer </dev/tty
      case "${answer}" in
        y|Y|yes|YES) return 0 ;;
        n|N|no|NO|'') return 1 ;;
        *) ;;
      esac
    done
  fi

  echo "refusing to overwrite existing file in non-interactive mode: ${path}" >&2
  exit 1
}

prompt_value() {
  local key="$1"
  local default_value="$2"
  local required="$3"
  local sensitive=0
  local value

  if is_sensitive_key "${key}"; then
    sensitive=1
  fi

  while true; do
    if [ -n "${default_value}" ]; then
      printf '%s [%s]: ' "${key}" "${default_value}" >/dev/tty
    else
      printf '%s: ' "${key}" >/dev/tty
    fi

    if [ "${sensitive}" = "1" ]; then
      read -r -s value </dev/tty
      printf '\n' >/dev/tty
    else
      read -r value </dev/tty
    fi

    if [ -z "${value}" ]; then
      value="${default_value}"
    fi

    if [ "${required}" = "1" ] && [ -z "${value}" ]; then
      printf 'value is required for %s\n' "${key}" >/dev/tty
      continue
    fi

    printf '%s' "${value}"
    return 0
  done
}

process_template() {
  local label="$1"
  local template_path="$2"
  local output_path="$3"
  local tmp
  local line
  local key
  local value
  local resolved_value
  local missing_keys=()
  local env_value
  local override_value

  if [ ! -f "${template_path}" ]; then
    echo "missing template file: ${template_path}" >&2
    exit 1
  fi

  tmp="$(mktemp)"
  while IFS= read -r line || [ -n "${line}" ]; do
    if [[ "${line}" =~ ^([A-Za-z_][A-Za-z0-9_]*)=(.*)$ ]]; then
      key="${BASH_REMATCH[1]}"
      value="${BASH_REMATCH[2]}"
      resolved_value="${value}"

      if override_value="$(lookup_override "${key}" 2>/dev/null)"; then
        resolved_value="${override_value}"
      elif [ -n "${!key:-}" ]; then
        env_value="${!key}"
        resolved_value="${env_value}"
      elif is_placeholder_value "${value}"; then
        if [ "${INTERACTIVE_MODE}" = "1" ]; then
          resolved_value="$(prompt_value "${key}" "" "1")"
        else
          missing_keys+=("${key}")
        fi
      fi

      printf '%s=%s\n' "${key}" "${resolved_value}" >> "${tmp}"
    else
      printf '%s\n' "${line}" >> "${tmp}"
    fi
  done < "${template_path}"

  if [ "${#missing_keys[@]}" -gt 0 ]; then
    rm -f "${tmp}"
    echo "missing required values for ${label} (${output_path}) in non-interactive mode:" >&2
    printf '  - %s\n' "${missing_keys[@]}" >&2
    echo "supply with --set KEY=VALUE or environment variables." >&2
    exit 1
  fi

  if [ "${DRY_RUN}" = "1" ]; then
    log "dry-run: ${output_path}"
    printf '%s\n' "----- ${output_path} -----"
    cat "${tmp}"
    printf '%s\n' "----- end ${output_path} -----"
    rm -f "${tmp}"
    return
  fi

  if [ -f "${output_path}" ] && ! confirm_overwrite "${output_path}"; then
    log "skipped existing file: ${output_path}"
    rm -f "${tmp}"
    return
  fi

  mkdir -p "$(dirname "${output_path}")"
  mv "${tmp}" "${output_path}"
  log "wrote ${output_path}"
}

bootstrap_vm_secret_env() {
  local runtime_env_path="${PROD_ENV_DIR}/runtime.env"
  local vm_secret_path="${PROD_ENV_DIR}/vm-secrets.env"
  local database_url=""
  local ghcr_username="${GHCR_USERNAME:-CHANGE_ME_GHCR_USERNAME}"
  local ghcr_token="${GHCR_TOKEN:-CHANGE_ME_GHCR_TOKEN}"
  local secret_line=""

  if [ -f "${vm_secret_path}" ]; then
    log "vm secret env already exists: ${vm_secret_path}"
    return
  fi

  if [ -f "${runtime_env_path}" ]; then
    database_url="$(grep -m1 '^DATABASE_URL=' "${runtime_env_path}" | sed -E 's/^DATABASE_URL=//' || true)"
  fi

  if [ -n "${database_url}" ] && ! is_placeholder_value "${database_url}"; then
    secret_line="BOMINAL_DATABASE_URL=${database_url}"
  else
    secret_line="BOMINAL_DATABASE_URL=CHANGE_ME_DATABASE_URL"
  fi

  if [ "${DRY_RUN}" = "1" ]; then
    log "dry-run: would write ${vm_secret_path}"
    if [[ "${secret_line}" == *"CHANGE_ME"* ]]; then
      log "dry-run: vm secret env would contain placeholder; set BOMINAL_DATABASE_URL before deploy."
    fi
    return
  fi

  umask 077
  {
    printf '%s\n' "${secret_line}"
    printf 'GHCR_USERNAME=%s\n' "${ghcr_username}"
    printf 'GHCR_TOKEN=%s\n' "${ghcr_token}"
  } > "${vm_secret_path}"
  chmod 600 "${vm_secret_path}"
  if [[ "${secret_line}" == *"CHANGE_ME"* ]]; then
    log "wrote ${vm_secret_path} with placeholder BOMINAL_DATABASE_URL; update before deploy"
  else
    log "wrote ${vm_secret_path} from runtime.env DATABASE_URL"
  fi
  if [[ "${ghcr_username}" == *"CHANGE_ME"* || "${ghcr_token}" == *"CHANGE_ME"* ]]; then
    log "vm secret env includes GHCR placeholders; set GHCR_USERNAME and GHCR_TOKEN before deploy if registry is private"
  fi
}

select_target() {
  local target="$1"
  case "${target}" in
    runtime) SELECT_RUNTIME=1 ;;
    caddy) SELECT_CADDY=1 ;;
    deploy) SELECT_DEPLOY=1 ;;
    cloudrun-api) SELECT_CLOUDRUN_API=1 ;;
    *)
      echo "invalid --only target: ${target} (expected runtime|caddy|deploy|cloudrun-api)" >&2
      exit 1
      ;;
  esac
  SELECT_ANY=1
}

parse_args() {
  local kv
  local key
  local value

  while [ "$#" -gt 0 ]; do
    case "$1" in
      --interactive)
        INTERACTIVE_MODE="1"
        ;;
      --non-interactive)
        INTERACTIVE_MODE="0"
        ;;
      --set)
        if [ "$#" -lt 2 ]; then
          echo "--set requires KEY=VALUE" >&2
          exit 1
        fi
        kv="$2"
        shift
        if [[ "${kv}" != *=* ]]; then
          echo "--set must be KEY=VALUE, got: ${kv}" >&2
          exit 1
        fi
        key="${kv%%=*}"
        value="${kv#*=}"
        if [[ ! "${key}" =~ ^[A-Za-z_][A-Za-z0-9_]*$ ]]; then
          echo "invalid key in --set: ${key}" >&2
          exit 1
        fi
        add_override "${key}" "${value}"
        ;;
      --only)
        if [ "$#" -lt 2 ]; then
          echo "--only requires a target" >&2
          exit 1
        fi
        select_target "$2"
        shift
        ;;
      --force)
        FORCE_WRITE=1
        ;;
      --dry-run)
        DRY_RUN=1
        ;;
      --help|-h)
        usage
        exit 0
        ;;
      *)
        echo "unknown option: $1" >&2
        usage
        exit 1
        ;;
    esac
    shift
  done

  if [ "${SELECT_ANY}" = "0" ]; then
    SELECT_RUNTIME=1
    SELECT_CADDY=1
    SELECT_DEPLOY=1
  fi

  if [ "${INTERACTIVE_MODE}" = "auto" ]; then
    if [ -t 0 ] && [ -t 1 ]; then
      INTERACTIVE_MODE="1"
    else
      INTERACTIVE_MODE="0"
    fi
  fi
}

main() {
  require_cmd mktemp
  parse_args "$@"

  log "mode: $( [ "${INTERACTIVE_MODE}" = "1" ] && echo interactive || echo non-interactive )"
  if [ "${DRY_RUN}" = "1" ]; then
    log "dry-run enabled (no files will be written)"
  fi

  if [ "${SELECT_RUNTIME}" = "1" ]; then
    process_template \
      "runtime" \
      "${PROD_ENV_DIR}/runtime.env.example" \
      "${PROD_ENV_DIR}/runtime.env"
  fi

  if [ "${SELECT_CADDY}" = "1" ]; then
    process_template \
      "caddy" \
      "${PROD_ENV_DIR}/caddy.env.example" \
      "${PROD_ENV_DIR}/caddy.env"
  fi

  if [ "${SELECT_DEPLOY}" = "1" ]; then
    process_template \
      "deploy" \
      "${PROD_ENV_DIR}/deploy.env.example" \
      "${PROD_ENV_DIR}/deploy.env"
    bootstrap_vm_secret_env

    log "reminder: configure GitHub production variables used by .github/workflows/cd.yml."
    log "reminder: deploy bootstrap will create VM secret env file if missing and persist BOMINAL_DATABASE_URL."
    log "reminder: deploy script enforces VM baseline swap/sysctl guard by default."
  fi

  if [ "${SELECT_CLOUDRUN_API}" = "1" ]; then
    process_template \
      "cloudrun-api" \
      "${PROD_ENV_DIR}/cloudrun-api.env.example" \
      "${PROD_ENV_DIR}/cloudrun-api.env"

    log "reminder: runtime/cloudrun/api/bootstrap.sh renders the Cloud Run service YAML from env/prod/cloudrun-api.env."
    log "reminder: Cloud Run prep is opt-in; current production deployment remains VM-first."
  fi

  log "bootstrap-prod complete"
}

main "$@"
