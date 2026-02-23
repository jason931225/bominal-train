#!/usr/bin/env bash
# -----------------------------------------------------------------------------
# local-check.sh — Start stack, wait for health, run tests/typecheck
# -----------------------------------------------------------------------------
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"
source "$SCRIPT_DIR/lib/env_utils.sh"
COMPOSE_FILE="infra/docker-compose.yml"

DOWN_AFTER=false
for arg in "$@"; do
  if [[ "$arg" == "--down" ]]; then
    DOWN_AFTER=true
  fi
done

cd "$REPO_ROOT"

command -v docker >/dev/null 2>&1 || { echo "Error: docker is required"; exit 1; }
command -v curl >/dev/null 2>&1 || { echo "Error: curl is required"; exit 1; }
detect_compose_cmd

APP_VERSION="$(resolve_dev_app_version "$REPO_ROOT")"
BUILD_VERSION="$(resolve_dev_build_version "$REPO_ROOT")"
export APP_VERSION BUILD_VERSION

print_logs() {
  echo ""
  echo "=== Recent docker compose logs (api/worker/web/mailpit) ==="
  "${COMPOSE_CMD[@]}" -f "$COMPOSE_FILE" logs --tail=200 api worker web mailpit 2>/dev/null || true
  echo ""
  echo "Hint: run:"
  echo "  ${COMPOSE_CMD[*]} -f $COMPOSE_FILE ps"
  echo "  ${COMPOSE_CMD[*]} -f $COMPOSE_FILE logs -f api worker web mailpit"
}

cleanup() {
  if [[ "$DOWN_AFTER" == "true" ]]; then
    echo ""
    echo "→ Stopping stack (--down)..."
    "${COMPOSE_CMD[@]}" -f "$COMPOSE_FILE" down --remove-orphans >/dev/null 2>&1 || true
  fi
}
trap cleanup EXIT

if [[ ! -f "infra/env/dev/api.env" ]]; then
  echo "Error: Environment files not found. Run ./infra/scripts/local-setup.sh first."
  exit 1
fi

echo "=== bominal local check ==="
echo "→ Using APP_VERSION=$APP_VERSION BUILD_VERSION=$BUILD_VERSION"
echo "→ Starting Docker Compose services..."
"${COMPOSE_CMD[@]}" -f "$COMPOSE_FILE" up -d --build --remove-orphans

echo "→ Waiting for API health (http://localhost:8000/health)..."
api_body=""
for _ in $(seq 1 60); do
  api_body="$(curl -fsS http://localhost:8000/health 2>/dev/null || true)"
  if [[ -n "$api_body" ]] \
    && echo "$api_body" | grep -Eq '"db"[[:space:]]*:[[:space:]]*true' \
    && echo "$api_body" | grep -Eq '"redis"[[:space:]]*:[[:space:]]*true'; then
    echo "  OK"
    break
  fi
  sleep 2
done
if [[ -z "$api_body" ]] \
  || ! echo "$api_body" | grep -Eq '"db"[[:space:]]*:[[:space:]]*true' \
  || ! echo "$api_body" | grep -Eq '"redis"[[:space:]]*:[[:space:]]*true'; then
  echo "  FAILED: API /health did not report db=true and redis=true"
  echo "  Body: ${api_body:-<empty>}"
  print_logs
  exit 1
fi

echo "→ Waiting for web (http://localhost:3000)..."
web_code=""
for _ in $(seq 1 60); do
  web_code="$(curl -sS -o /dev/null -w '%{http_code}' http://localhost:3000 2>/dev/null || true)"
  if [[ "$web_code" == "200" || "$web_code" == "302" || "$web_code" == "307" || "$web_code" == "308" ]]; then
    echo "  OK (HTTP $web_code)"
    break
  fi
  sleep 2
done
if [[ "$web_code" != "200" && "$web_code" != "302" && "$web_code" != "307" && "$web_code" != "308" ]]; then
  echo "  FAILED: web did not respond with 200/3xx (last HTTP ${web_code:-<empty>})"
  print_logs
  exit 1
fi

echo "→ Waiting for Mailpit UI (http://localhost:8025)..."
mailpit_code=""
for _ in $(seq 1 30); do
  mailpit_code="$(curl -sS -o /dev/null -w '%{http_code}' http://localhost:8025 2>/dev/null || true)"
  if [[ "$mailpit_code" == "200" || "$mailpit_code" == "302" || "$mailpit_code" == "307" || "$mailpit_code" == "308" ]]; then
    echo "  OK (HTTP $mailpit_code)"
    break
  fi
  sleep 1
done
if [[ "$mailpit_code" != "200" && "$mailpit_code" != "302" && "$mailpit_code" != "307" && "$mailpit_code" != "308" ]]; then
  echo "  FAILED: Mailpit did not respond with 200/3xx (last HTTP ${mailpit_code:-<empty>})"
  print_logs
  exit 1
fi

check_worker_service() {
  local service="$1"
  local expected_settings="$2"

  echo "→ Checking worker service: $service..."
  for _ in $(seq 1 30); do
    local container_id
    container_id="$("${COMPOSE_CMD[@]}" -f "$COMPOSE_FILE" ps -q "$service" 2>/dev/null || true)"
    if [[ -n "$container_id" ]]; then
      local running_state health_state
      running_state="$(docker inspect -f '{{.State.Running}}' "$container_id" 2>/dev/null || true)"
      health_state="$(docker inspect -f '{{if .State.Health}}{{.State.Health.Status}}{{else}}none{{end}}' "$container_id" 2>/dev/null || true)"
      if [[ "$running_state" == "true" ]] && [[ "$health_state" == "healthy" || "$health_state" == "none" ]]; then
        if "${COMPOSE_CMD[@]}" -f "$COMPOSE_FILE" exec -T "$service" python -c "import os,sys; needle=sys.argv[1]; found=any(needle in open(f'/proc/{p}/cmdline','rb').read().decode('utf-8','ignore') for p in os.listdir('/proc') if p.isdigit() and os.path.exists(f'/proc/{p}/cmdline')); raise SystemExit(0 if found else 1)" "$expected_settings" >/dev/null 2>&1; then
          echo "  OK"
          return 0
        fi
      fi
    fi
    sleep 1
  done

  echo "  FAILED: $service is not healthy with expected process '$expected_settings'"
  print_logs
  exit 1
}

check_worker_service "worker" "app.worker.WorkerSettings"

echo "→ Running backend tests (api: pytest -q)..."
if ! "${COMPOSE_CMD[@]}" -f "$COMPOSE_FILE" exec -T api pytest -q; then
  echo "  FAILED: backend tests"
  print_logs
  exit 1
fi

echo "→ Running web typecheck (web: npx tsc --noEmit)..."
if ! "${COMPOSE_CMD[@]}" -f "$COMPOSE_FILE" exec -T web npx tsc --noEmit; then
  echo "  FAILED: web typecheck"
  print_logs
  exit 1
fi

echo ""
echo "=== Local check PASSED ==="
