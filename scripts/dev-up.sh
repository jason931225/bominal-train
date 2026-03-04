#!/usr/bin/env bash
set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
RUNTIME_DIR="${REPO_ROOT}/runtime"
BOOTSTRAP_SCRIPT="${REPO_ROOT}/scripts/bootstrap-local.sh"
DEV_ENV_FILE="${REPO_ROOT}/env/dev/runtime.env"
LOCAL_ENV_FILE="${REPO_ROOT}/env/local/runtime.env"

RUN_BOOTSTRAP=1
START_API=1
START_WORKER=1
START_CSS_WATCH=1
START_RUST_WATCH=1
APP_PORT_OVERRIDE=""
TAKEOVER_WATCHERS=1
LOCK_DIR="/tmp/bominal-dev-up.lock"
LOCK_PID_FILE="${LOCK_DIR}/pid"
LOCK_ACQUIRED=0

usage() {
  cat <<EOF
Usage: ./scripts/dev-up.sh [options]

Options:
  --no-bootstrap   Skip local dependency/bootstrap prep.
  --api-only       Start only bominal-api.
  --worker-only    Start only bominal-worker.
  --no-css-watch   Do not run Tailwind CSS watch loop.
  --rust-watch     Auto-restart Rust services on source changes (default; requires cargo-watch).
  --no-rust-watch  Disable Rust auto-restart and run services directly.
  --takeover-watchers
                  Stop stale cargo-watch processes for this repo before startup (default).
  --no-takeover-watchers
                  Fail if stale cargo-watch processes exist instead of taking over.
  --port <port>    Override APP_PORT for this dev-up session.
  --help           Show this help.

Environment:
  BOMINAL_RUN_MIGRATIONS=0|1  Passed to bootstrap (default: 1).
  BOMINAL_RUN_TESTS=0|1       Passed to bootstrap (default: 0 in dev-up).
EOF
}

log() {
  printf '[dev-up] %s\n' "$*"
}

require_cmd() {
  local cmd="$1"
  if ! command -v "${cmd}" >/dev/null 2>&1; then
    echo "missing required command: ${cmd}" >&2
    exit 1
  fi
}

require_cargo_watch() {
  if ! cargo watch --version >/dev/null 2>&1; then
    cat <<EOF >&2
missing cargo watch subcommand.

Install it with:
  cargo install cargo-watch
EOF
    exit 1
  fi
}

port_in_use_details() {
  local port="$1"

  if command -v lsof >/dev/null 2>&1; then
    lsof -nP -iTCP:"${port}" -sTCP:LISTEN 2>/dev/null \
      | awk '
          NR > 1 {
            printf "%s pid=%s %s\n", $1, $2, $9
            found = 1
          }
          END {
            if (!found) {
              exit 1
            }
          }
        '
    return
  fi

  if command -v ss >/dev/null 2>&1; then
    ss -ltnp "sport = :${port}" 2>/dev/null \
      | awk '
          NR > 1 {
            print $0
            found = 1
          }
          END {
            if (!found) {
              exit 1
            }
          }
        '
    return
  fi

  return 2
}

preflight_api_port() {
  if [ "${START_API}" != "1" ]; then
    return 0
  fi

  local port="${APP_PORT:-8000}"
  local next_port=$((port + 1))
  local listener_details=""
  if listener_details="$(port_in_use_details "${port}")"; then
    echo >&2
    log "error: APP_PORT ${port} is already in use"
    printf '%s\n' "${listener_details}" >&2
    echo >&2
    echo "resolve by stopping the listener, then re-run dev-up:" >&2
    echo "  lsof -nP -iTCP:${port} -sTCP:LISTEN" >&2
    echo "  kill <pid>" >&2
    echo >&2
    echo "or run on a different port:" >&2
    echo "  ./scripts/dev-up.sh --port ${next_port}" >&2
    echo >&2
    exit 1
  else
    local details_status=$?
    if [ "${details_status}" -eq 2 ]; then
      log "warning: lsof/ss unavailable; skipping APP_PORT preflight"
    fi
  fi
}

repo_cargo_watch_pids() {
  if ! command -v lsof >/dev/null 2>&1; then
    return 2
  fi

  local current_pid=""
  while IFS= read -r line; do
    case "${line}" in
      p*)
        current_pid="${line#p}"
        ;;
      n*)
        local cwd_path="${line#n}"
        if [ -n "${current_pid}" ] && {
          [ "${cwd_path}" = "${RUNTIME_DIR}" ] || [ "${cwd_path}" = "${REPO_ROOT}" ];
        }; then
          printf '%s\n' "${current_pid}"
        fi
        ;;
    esac
  done < <(lsof -a -c cargo-wat -d cwd -Fn 2>/dev/null || true)
}

preflight_cargo_watchers() {
  if [ "${START_API}" != "1" ] && [ "${START_WORKER}" != "1" ]; then
    return 0
  fi

  local watcher_status=0
  local watcher_pids=()
  while IFS= read -r pid; do
    [ -n "${pid}" ] && watcher_pids+=("${pid}")
  done < <(repo_cargo_watch_pids || watcher_status=$?)

  if [ "${watcher_status}" -eq 2 ]; then
    log "warning: lsof unavailable; skipping cargo-watch preflight"
    return 0
  fi

  if [ "${#watcher_pids[@]}" -eq 0 ]; then
    return 0
  fi

  # De-duplicate PIDs
  local unique_pids=()
  local seen=" "
  local pid
  for pid in "${watcher_pids[@]}"; do
    if [[ "${seen}" != *" ${pid} "* ]]; then
      unique_pids+=("${pid}")
      seen="${seen}${pid} "
    fi
  done

  if [ "${TAKEOVER_WATCHERS}" = "1" ]; then
    log "stopping stale cargo-watch supervisors: ${unique_pids[*]}"
    for pid in "${unique_pids[@]}"; do
      kill "${pid}" >/dev/null 2>&1 || true
    done
    sleep 1

    local survivors=()
    for pid in "${unique_pids[@]}"; do
      if kill -0 "${pid}" >/dev/null 2>&1; then
        survivors+=("${pid}")
      fi
    done

    if [ "${#survivors[@]}" -gt 0 ]; then
      log "error: could not stop cargo-watch supervisor(s): ${survivors[*]}"
      echo "terminate them manually, then re-run dev-up." >&2
      exit 1
    fi
    return 0
  fi

  log "error: stale cargo-watch supervisor(s) detected for this repo: ${unique_pids[*]}"
  echo "this can respawn bominal-api/worker unexpectedly." >&2
  echo "stop them, then re-run dev-up:" >&2
  for pid in "${unique_pids[@]}"; do
    echo "  kill ${pid}" >&2
  done
  echo >&2
  echo "or let dev-up take over automatically:" >&2
  echo "  ./scripts/dev-up.sh --takeover-watchers" >&2
  exit 1
}

pid_command_line() {
  local pid="$1"
  if ! command -v ps >/dev/null 2>&1; then
    return 1
  fi
  ps -p "${pid}" -o command= 2>/dev/null || true
}

pid_looks_like_dev_up() {
  local pid="$1"
  local command_line
  command_line="$(pid_command_line "${pid}")"
  if [ -z "${command_line}" ]; then
    return 1
  fi

  case "${command_line}" in
    *scripts/dev-up.sh*|*dev-up.sh*)
      return 0
      ;;
    *)
      return 1
      ;;
  esac
}

acquire_lock() {
  if mkdir "${LOCK_DIR}" >/dev/null 2>&1; then
    printf '%s\n' "$$" > "${LOCK_PID_FILE}"
    LOCK_ACQUIRED=1
    return
  fi

  local existing_pid=""
  if [ -f "${LOCK_PID_FILE}" ]; then
    existing_pid="$(cat "${LOCK_PID_FILE}" 2>/dev/null || true)"
  fi

  if [ -n "${existing_pid}" ] && kill -0 "${existing_pid}" >/dev/null 2>&1; then
    if pid_looks_like_dev_up "${existing_pid}"; then
      log "error: another dev-up instance is already running (pid ${existing_pid})"
      echo "stop the existing dev-up process or terminate it before starting a new one." >&2
      exit 1
    fi

    log "stale lock points to unrelated pid ${existing_pid}; clearing lock"
  fi

  rm -rf "${LOCK_DIR}" >/dev/null 2>&1 || true
  if mkdir "${LOCK_DIR}" >/dev/null 2>&1; then
    printf '%s\n' "$$" > "${LOCK_PID_FILE}"
    LOCK_ACQUIRED=1
    return
  fi

  log "error: unable to acquire dev-up lock at ${LOCK_DIR}"
  exit 1
}

release_lock() {
  if [ "${LOCK_ACQUIRED}" = "1" ]; then
    rm -rf "${LOCK_DIR}" >/dev/null 2>&1 || true
    LOCK_ACQUIRED=0
  fi
}

cleanup() {
  local code=$?

  stop_service_tree "${API_PID:-}" "bominal-api"
  stop_service_tree "${WORKER_PID:-}" "bominal-worker"
  stop_service_tree "${CSS_WATCH_PID:-}" "css watch"

  release_lock
  wait >/dev/null 2>&1 || true
  exit "${code}"
}

list_child_pids() {
  local parent_pid="$1"
  if [ -z "${parent_pid}" ] || ! command -v ps >/dev/null 2>&1; then
    return 0
  fi

  ps -axo pid=,ppid= 2>/dev/null \
    | awk -v parent="${parent_pid}" '$2 == parent { print $1 }'
}

terminate_process_tree() {
  local pid="$1"
  [ -z "${pid}" ] && return 0
  if ! kill -0 "${pid}" >/dev/null 2>&1; then
    return 0
  fi

  local child_pid=""
  while IFS= read -r child_pid; do
    [ -n "${child_pid}" ] && terminate_process_tree "${child_pid}"
  done < <(list_child_pids "${pid}")

  kill "${pid}" >/dev/null 2>&1 || true
}

force_terminate_process_tree() {
  local pid="$1"
  [ -z "${pid}" ] && return 0
  if ! kill -0 "${pid}" >/dev/null 2>&1; then
    return 0
  fi

  local child_pid=""
  while IFS= read -r child_pid; do
    [ -n "${child_pid}" ] && force_terminate_process_tree "${child_pid}"
  done < <(list_child_pids "${pid}")

  kill -9 "${pid}" >/dev/null 2>&1 || true
}

wait_for_pid_exit() {
  local pid="$1"
  local retries="${2:-20}"
  local attempt=0

  while [ "${attempt}" -lt "${retries}" ]; do
    if ! kill -0 "${pid}" >/dev/null 2>&1; then
      return 0
    fi
    sleep 0.1
    attempt=$((attempt + 1))
  done

  return 1
}

stop_service_tree() {
  local pid="$1"
  local label="$2"
  [ -z "${pid}" ] && return 0
  if ! kill -0 "${pid}" >/dev/null 2>&1; then
    return 0
  fi

  log "stopping ${label} (pid ${pid})"
  terminate_process_tree "${pid}"
  if ! wait_for_pid_exit "${pid}" 20; then
    log "force-killing ${label} (pid ${pid})"
    force_terminate_process_tree "${pid}"
  fi
}

parse_args() {
  while [ "$#" -gt 0 ]; do
    case "$1" in
      --no-bootstrap)
        RUN_BOOTSTRAP=0
        ;;
      --api-only)
        START_API=1
        START_WORKER=0
        ;;
      --worker-only)
        START_API=0
        START_WORKER=1
        ;;
      --no-css-watch)
        START_CSS_WATCH=0
        ;;
      --rust-watch)
        START_RUST_WATCH=1
        ;;
      --no-rust-watch)
        START_RUST_WATCH=0
        ;;
      --takeover-watchers)
        TAKEOVER_WATCHERS=1
        ;;
      --no-takeover-watchers)
        TAKEOVER_WATCHERS=0
        ;;
      --port)
        if [ "$#" -lt 2 ]; then
          echo "--port requires a value" >&2
          usage
          exit 1
        fi
        APP_PORT_OVERRIDE="$2"
        shift
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
}

load_runtime_env() {
  if [ ! -f "${DEV_ENV_FILE}" ]; then
    echo "missing development env file: ${DEV_ENV_FILE}" >&2
    exit 1
  fi

  set -a
  # shellcheck disable=SC1090
  source "${DEV_ENV_FILE}"
  if [ -f "${LOCAL_ENV_FILE}" ]; then
    # shellcheck disable=SC1090
    source "${LOCAL_ENV_FILE}"
  fi
  set +a
}

apply_port_override() {
  if [ -z "${APP_PORT_OVERRIDE}" ]; then
    return
  fi

  case "${APP_PORT_OVERRIDE}" in
    ''|*[!0-9]*)
      echo "invalid port: ${APP_PORT_OVERRIDE}" >&2
      exit 1
      ;;
  esac
  export APP_PORT="${APP_PORT_OVERRIDE}"
}

start_api() {
  log "starting bominal-api on ${APP_HOST:-0.0.0.0}:${APP_PORT:-8000}"
  (
    cd "${RUNTIME_DIR}"
    if [ "${START_RUST_WATCH}" = "1" ]; then
      exec cargo watch -x "run -p bominal-api --bin bominal-api"
    else
      exec cargo run -p bominal-api --bin bominal-api
    fi
  ) </dev/null &
  API_PID=$!
  log "bominal-api supervisor pid ${API_PID}"
}

start_worker() {
  log "starting bominal-worker"
  (
    cd "${RUNTIME_DIR}"
    if [ "${START_RUST_WATCH}" = "1" ]; then
      exec cargo watch -x "run -p bominal-worker"
    else
      exec cargo run -p bominal-worker
    fi
  ) </dev/null &
  WORKER_PID=$!
  log "bominal-worker supervisor pid ${WORKER_PID}"
}

start_css_watch() {
  log "starting frontend css watch"
  (
    cd "${RUNTIME_DIR}"
    exec npm --prefix frontend run watch:css
  ) </dev/null &
  CSS_WATCH_PID=$!
  log "css watch pid ${CSS_WATCH_PID}"
}

wait_for_exit() {
  while true; do
    if [ -n "${API_PID:-}" ] && ! kill -0 "${API_PID}" >/dev/null 2>&1; then
      wait "${API_PID}" >/dev/null 2>&1 || true
      return
    fi

    if [ -n "${WORKER_PID:-}" ] && ! kill -0 "${WORKER_PID}" >/dev/null 2>&1; then
      wait "${WORKER_PID}" >/dev/null 2>&1 || true
      return
    fi

    if [ -n "${CSS_WATCH_PID:-}" ] && ! kill -0 "${CSS_WATCH_PID}" >/dev/null 2>&1; then
      wait "${CSS_WATCH_PID}" >/dev/null 2>&1 || true
      return
    fi

    sleep 1
  done
}

main() {
  parse_args "$@"
  require_cmd cargo

  acquire_lock
  trap cleanup INT TERM EXIT

  if [ "${START_RUST_WATCH}" = "1" ]; then
    require_cargo_watch
  fi

  if [ "${START_API}" = "1" ] && [ "${START_CSS_WATCH}" = "1" ]; then
    require_cmd npm
  fi

  load_runtime_env
  apply_port_override
  preflight_cargo_watchers
  preflight_api_port

  if [ "${RUN_BOOTSTRAP}" = "1" ]; then
    if [ ! -x "${BOOTSTRAP_SCRIPT}" ]; then
      echo "missing executable bootstrap script: ${BOOTSTRAP_SCRIPT}" >&2
      exit 1
    fi

    log "running bootstrap-local (tests default off for dev loop)"
    BOMINAL_RUN_TESTS="${BOMINAL_RUN_TESTS:-0}" \
    BOMINAL_RUN_MIGRATIONS="${BOMINAL_RUN_MIGRATIONS:-1}" \
      "${BOOTSTRAP_SCRIPT}"
  fi

  load_runtime_env
  apply_port_override
  preflight_api_port

  if [ "${START_API}" = "1" ]; then
    start_api
  fi
  if [ "${START_WORKER}" = "1" ]; then
    start_worker
  fi
  if [ "${START_API}" = "1" ] && [ "${START_CSS_WATCH}" = "1" ]; then
    start_css_watch
  fi

  if [ "${START_API}" = "0" ] && [ "${START_WORKER}" = "0" ]; then
    echo "no services selected to start" >&2
    exit 1
  fi

  log "services running (Ctrl+C to stop)"
  log "health endpoints: http://127.0.0.1:${APP_PORT:-8000}/health and /ready"
  if [ "${START_API}" = "1" ] && [ "${START_WORKER}" = "1" ]; then
    log "note: concurrent cargo startup may briefly print file-lock waits; this is expected"
  fi
  if [ "${START_RUST_WATCH}" = "1" ]; then
    log "rust watch mode enabled (auto-restart on Rust source changes)"
  fi

  wait_for_exit
  log "one process exited; shutting down remaining services"
}

main "$@"
