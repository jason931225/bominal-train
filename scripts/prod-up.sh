#!/usr/bin/env bash
set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
PROD_SCRIPT_DIR="${REPO_ROOT}/scripts/prod"

# shellcheck source=scripts/prod/_env_lib.sh
source "${PROD_SCRIPT_DIR}/_env_lib.sh"

log() {
  printf '[prod-up] %s\n' "$*"
}

EXIT_USAGE=2
EXIT_PRECONDITION=3
EXIT_HEALTH=4
EXIT_DEPLOY=5
EXIT_ROLLBACK=6

runtime_env_path="${BOMINAL_RUNTIME_ENV_PATH:-/opt/bominal/repo/env/prod/runtime.env}"
compose_file="${BOMINAL_COMPOSE_FILE:-/opt/bominal/repo/runtime/compose.prod.yml}"
vm_secret_env_file="${BOMINAL_VM_SECRET_ENV_FILE:-/opt/bominal/env/prod/vm-secrets.env}"
api_service="${BOMINAL_API_SERVICE:-api}"
worker_service="${BOMINAL_WORKER_SERVICE:-worker}"
redis_service="${BOMINAL_REDIS_SERVICE:-redis}"
compose_project_name="${BOMINAL_COMPOSE_PROJECT_NAME:-}"
rollback_state_path="${BOMINAL_ROLLBACK_STATE_PATH:-}"

deploy_script="${BOMINAL_PROD_DEPLOY_SCRIPT_PATH:-${PROD_SCRIPT_DIR}/deploy-runtime.sh}"
health_script="${BOMINAL_PROD_HEALTHCHECK_SCRIPT_PATH:-${PROD_SCRIPT_DIR}/healthcheck-runtime.sh}"
rollback_script="${BOMINAL_PROD_ROLLBACK_SCRIPT_PATH:-${PROD_SCRIPT_DIR}/rollback-runtime.sh}"

yes_flag=0
global_consumed=0

usage() {
  cat <<EOF
Usage: ./scripts/prod-up.sh <subcommand> [options]

Subcommands:
  start       Start existing runtime containers only, then run health checks.
  status      Show runtime service status and pinned image refs from runtime env.
  health      Run runtime health checks.
  logs        Show runtime service logs (defaults to api+worker).
  deploy      Run deploy flow (requires --yes), then health check and rollback on failure.
  rollback    Run rollback flow (requires --yes), then health check.
  help        Show this help.

Global options:
  --runtime-env <path>     Runtime env file path.
  --compose-file <path>    Compose file path.
  --vm-secret-env <path>   VM secret env path.
  --api-service <name>     API service name in compose.
  --worker-service <name>  Worker service name in compose.
  --redis-service <name>   Redis service name in compose.
  --project-name <name>    Compose project name override.
  --rollback-state <path>  Rollback state file path (used by deploy/rollback scripts).
  --yes                    Required for deploy and rollback.
  -h, --help               Show help.

Examples:
  ./scripts/prod-up.sh start
  ./scripts/prod-up.sh status --project-name bominal
  ./scripts/prod-up.sh logs -f --since 30m --service api
  ./scripts/prod-up.sh deploy --yes
  ./scripts/prod-up.sh rollback --yes
EOF
}

usage_error() {
  printf '[prod-up] error: %s\n' "$1" >&2
  usage >&2
  exit "${EXIT_USAGE}"
}

precondition_error() {
  printf '[prod-up] error: %s\n' "$1" >&2
  exit "${EXIT_PRECONDITION}"
}

require_option_value() {
  local opt="$1"
  local maybe_value="${2:-}"
  if [ -z "${maybe_value}" ]; then
    usage_error "missing value for ${opt}"
  fi
}

parse_global_option() {
  global_consumed=0
  case "${1:-}" in
    --runtime-env)
      require_option_value "--runtime-env" "${2:-}"
      runtime_env_path="${2}"
      global_consumed=2
      ;;
    --compose-file)
      require_option_value "--compose-file" "${2:-}"
      compose_file="${2}"
      global_consumed=2
      ;;
    --vm-secret-env)
      require_option_value "--vm-secret-env" "${2:-}"
      vm_secret_env_file="${2}"
      global_consumed=2
      ;;
    --api-service)
      require_option_value "--api-service" "${2:-}"
      api_service="${2}"
      global_consumed=2
      ;;
    --worker-service)
      require_option_value "--worker-service" "${2:-}"
      worker_service="${2}"
      global_consumed=2
      ;;
    --redis-service)
      require_option_value "--redis-service" "${2:-}"
      redis_service="${2}"
      global_consumed=2
      ;;
    --project-name)
      require_option_value "--project-name" "${2:-}"
      compose_project_name="${2}"
      global_consumed=2
      ;;
    --rollback-state)
      require_option_value "--rollback-state" "${2:-}"
      rollback_state_path="${2}"
      global_consumed=2
      ;;
    --yes)
      yes_flag=1
      global_consumed=1
      ;;
    -h|--help)
      usage
      exit 0
      ;;
  esac
}

export_common_env() {
  export BOMINAL_RUNTIME_ENV_PATH="${runtime_env_path}"
  export BOMINAL_COMPOSE_FILE="${compose_file}"
  export BOMINAL_VM_SECRET_ENV_FILE="${vm_secret_env_file}"
  export BOMINAL_API_SERVICE="${api_service}"
  export BOMINAL_WORKER_SERVICE="${worker_service}"
  if [ -n "${compose_project_name}" ]; then
    export BOMINAL_COMPOSE_PROJECT_NAME="${compose_project_name}"
  else
    unset BOMINAL_COMPOSE_PROJECT_NAME 2>/dev/null || true
  fi
  if [ -n "${rollback_state_path}" ]; then
    export BOMINAL_ROLLBACK_STATE_PATH="${rollback_state_path}"
  fi
}

require_runtime_inputs() {
  require_cmd docker
  [ -f "${runtime_env_path}" ] || precondition_error "runtime env file not found: ${runtime_env_path}"
  [ -f "${compose_file}" ] || precondition_error "compose file not found: ${compose_file}"
}

require_executable_script() {
  local script_path="$1"
  [ -f "${script_path}" ] || precondition_error "script not found: ${script_path}"
  [ -x "${script_path}" ] || precondition_error "script not executable: ${script_path}"
}

run_healthcheck() {
  require_executable_script "${health_script}"
  export_common_env
  if ! "${health_script}"; then
    return 1
  fi
  return 0
}

run_rollback_script() {
  require_executable_script "${rollback_script}"
  export_common_env
  if ! "${rollback_script}"; then
    return 1
  fi
  return 0
}

ensure_service_containers_exist() {
  local service_listing=""
  local missing_services=()
  local service

  if ! service_listing="$(compose_cmd "${runtime_env_path}" "${compose_file}" ps --all --services | tr -d '\r')"; then
    precondition_error "failed to inspect runtime containers via docker compose"
  fi

  for service in "${redis_service}" "${api_service}" "${worker_service}"; do
    if ! printf '%s\n' "${service_listing}" | grep -qx "${service}"; then
      missing_services+=("${service}")
    fi
  done

  if [ "${#missing_services[@]}" -gt 0 ]; then
    precondition_error "runtime containers missing (${missing_services[*]}). Run './scripts/prod-up.sh deploy --yes' (with required BOMINAL_* deploy vars) or CI/CD deploy first."
  fi
}

cmd_start() {
  while [ "$#" -gt 0 ]; do
    parse_global_option "$@"
    if [ "${global_consumed}" -gt 0 ]; then
      shift "${global_consumed}"
      continue
    fi
    usage_error "unknown option for start: $1"
  done

  require_runtime_inputs
  export_common_env
  ensure_service_containers_exist

  log "starting existing runtime services: ${redis_service} ${api_service} ${worker_service}"
  compose_cmd "${runtime_env_path}" "${compose_file}" start \
    "${redis_service}" "${api_service}" "${worker_service}" >/dev/null

  if ! run_healthcheck; then
    precondition_error "start completed but health checks failed"
  fi
  log "start completed"
}

cmd_status() {
  while [ "$#" -gt 0 ]; do
    parse_global_option "$@"
    if [ "${global_consumed}" -gt 0 ]; then
      shift "${global_consumed}"
      continue
    fi
    usage_error "unknown option for status: $1"
  done

  require_runtime_inputs
  export_common_env

  printf 'runtime_env=%s\n' "${runtime_env_path}"
  printf 'compose_file=%s\n' "${compose_file}"
  printf 'api_image=%s\n' "$(read_env_key "${runtime_env_path}" "BOMINAL_API_IMAGE")"
  printf 'worker_image=%s\n' "$(read_env_key "${runtime_env_path}" "BOMINAL_WORKER_IMAGE")"
  printf 'database_url_present=%s\n' "$([ -n "$(read_env_key "${runtime_env_path}" "DATABASE_URL")" ] && printf true || printf false)"

  compose_cmd "${runtime_env_path}" "${compose_file}" ps
}

cmd_health() {
  while [ "$#" -gt 0 ]; do
    parse_global_option "$@"
    if [ "${global_consumed}" -gt 0 ]; then
      shift "${global_consumed}"
      continue
    fi
    usage_error "unknown option for health: $1"
  done

  require_runtime_inputs
  export_common_env
  if ! run_healthcheck; then
    printf '[prod-up] error: health checks failed\n' >&2
    exit "${EXIT_HEALTH}"
  fi
  log "health checks passed"
}

cmd_logs() {
  local logs_args=()
  local logs_services=()
  local compose_logs_cmd=("logs")

  while [ "$#" -gt 0 ]; do
    parse_global_option "$@"
    if [ "${global_consumed}" -gt 0 ]; then
      shift "${global_consumed}"
      continue
    fi

    case "$1" in
      -f|--follow)
        logs_args+=("--follow")
        shift
        ;;
      --since)
        require_option_value "--since" "${2:-}"
        logs_args+=("--since" "${2}")
        shift 2
        ;;
      --tail)
        require_option_value "--tail" "${2:-}"
        logs_args+=("--tail" "${2}")
        shift 2
        ;;
      --service)
        require_option_value "--service" "${2:-}"
        logs_services+=("${2}")
        shift 2
        ;;
      --)
        shift
        while [ "$#" -gt 0 ]; do
          logs_services+=("$1")
          shift
        done
        ;;
      -*)
        usage_error "unknown option for logs: $1"
        ;;
      *)
        logs_services+=("$1")
        shift
        ;;
    esac
  done

  if [ "${#logs_services[@]}" -eq 0 ]; then
    logs_services=("${api_service}" "${worker_service}")
  fi

  require_runtime_inputs
  export_common_env
  if [ "${#logs_args[@]}" -gt 0 ]; then
    compose_logs_cmd+=("${logs_args[@]}")
  fi
  compose_logs_cmd+=("${logs_services[@]}")
  compose_cmd "${runtime_env_path}" "${compose_file}" "${compose_logs_cmd[@]}"
}

cmd_deploy() {
  while [ "$#" -gt 0 ]; do
    parse_global_option "$@"
    if [ "${global_consumed}" -gt 0 ]; then
      shift "${global_consumed}"
      continue
    fi
    usage_error "unknown option for deploy: $1"
  done

  if [ "${yes_flag}" -ne 1 ]; then
    usage_error "deploy requires --yes"
  fi

  require_runtime_inputs
  require_executable_script "${deploy_script}"
  export_common_env

  if ! "${deploy_script}"; then
    printf '[prod-up] error: deploy script failed\n' >&2
    exit "${EXIT_DEPLOY}"
  fi

  if run_healthcheck; then
    log "deploy completed and health checks passed"
    return
  fi

  log "health checks failed after deploy; attempting rollback"
  if ! run_rollback_script; then
    printf '[prod-up] error: rollback failed after deploy health-check failure\n' >&2
    exit "${EXIT_ROLLBACK}"
  fi
  printf '[prod-up] error: deploy health checks failed; rollback applied\n' >&2
  exit "${EXIT_HEALTH}"
}

cmd_rollback() {
  while [ "$#" -gt 0 ]; do
    parse_global_option "$@"
    if [ "${global_consumed}" -gt 0 ]; then
      shift "${global_consumed}"
      continue
    fi
    usage_error "unknown option for rollback: $1"
  done

  if [ "${yes_flag}" -ne 1 ]; then
    usage_error "rollback requires --yes"
  fi

  require_runtime_inputs
  if ! run_rollback_script; then
    printf '[prod-up] error: rollback script failed\n' >&2
    exit "${EXIT_ROLLBACK}"
  fi

  if ! run_healthcheck; then
    printf '[prod-up] error: rollback completed but health checks failed\n' >&2
    exit "${EXIT_HEALTH}"
  fi
  log "rollback completed and health checks passed"
}

main() {
  local subcommand="${1:-help}"
  if [ "$#" -gt 0 ]; then
    shift
  fi

  case "${subcommand}" in
    start)
      cmd_start "$@"
      ;;
    status)
      cmd_status "$@"
      ;;
    health)
      cmd_health "$@"
      ;;
    logs)
      cmd_logs "$@"
      ;;
    deploy)
      cmd_deploy "$@"
      ;;
    rollback)
      cmd_rollback "$@"
      ;;
    help|-h|--help)
      usage
      ;;
    *)
      usage_error "unknown subcommand: ${subcommand}"
      ;;
  esac
}

main "$@"
