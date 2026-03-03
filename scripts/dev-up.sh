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
START_RUST_WATCH=0
APP_PORT_OVERRIDE=""

usage() {
  cat <<EOF
Usage: ./scripts/dev-up.sh [options]

Options:
  --no-bootstrap   Skip local dependency/bootstrap prep.
  --api-only       Start only bominal-api.
  --worker-only    Start only bominal-worker.
  --no-css-watch   Do not run Tailwind CSS watch loop.
  --rust-watch     Auto-restart Rust services on source changes (requires cargo-watch).
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

cleanup() {
  local code=$?

  if [ -n "${API_PID:-}" ] && kill -0 "${API_PID}" >/dev/null 2>&1; then
    log "stopping bominal-api (pid ${API_PID})"
    kill "${API_PID}" >/dev/null 2>&1 || true
  fi

  if [ -n "${WORKER_PID:-}" ] && kill -0 "${WORKER_PID}" >/dev/null 2>&1; then
    log "stopping bominal-worker (pid ${WORKER_PID})"
    kill "${WORKER_PID}" >/dev/null 2>&1 || true
  fi

  if [ -n "${CSS_WATCH_PID:-}" ] && kill -0 "${CSS_WATCH_PID}" >/dev/null 2>&1; then
    log "stopping css watch (pid ${CSS_WATCH_PID})"
    kill "${CSS_WATCH_PID}" >/dev/null 2>&1 || true
  fi

  wait >/dev/null 2>&1 || true
  exit "${code}"
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

start_api() {
  log "starting bominal-api on ${APP_HOST:-0.0.0.0}:${APP_PORT:-8000}"
  (
    cd "${RUNTIME_DIR}"
    if [ "${START_RUST_WATCH}" = "1" ]; then
      cargo watch -x "run -p bominal-api"
    else
      cargo run -p bominal-api
    fi
  ) &
  API_PID=$!
}

start_worker() {
  log "starting bominal-worker"
  (
    cd "${RUNTIME_DIR}"
    if [ "${START_RUST_WATCH}" = "1" ]; then
      cargo watch -x "run -p bominal-worker"
    else
      cargo run -p bominal-worker
    fi
  ) &
  WORKER_PID=$!
}

start_css_watch() {
  log "starting frontend css watch"
  (
    cd "${RUNTIME_DIR}"
    npm --prefix frontend run watch:css
  ) &
  CSS_WATCH_PID=$!
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

  if [ "${START_RUST_WATCH}" = "1" ]; then
    require_cargo_watch
  fi

  if [ "${START_API}" = "1" ] && [ "${START_CSS_WATCH}" = "1" ]; then
    require_cmd npm
  fi

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

  if [ -n "${APP_PORT_OVERRIDE}" ]; then
    case "${APP_PORT_OVERRIDE}" in
      ''|*[!0-9]*)
        echo "invalid port: ${APP_PORT_OVERRIDE}" >&2
        exit 1
        ;;
    esac
    export APP_PORT="${APP_PORT_OVERRIDE}"
  fi

  trap cleanup INT TERM EXIT

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
  if [ "${START_RUST_WATCH}" = "1" ]; then
    log "rust watch mode enabled (auto-restart on Rust source changes)"
  fi

  wait_for_exit
  log "one process exited; shutting down remaining services"
}

main "$@"
