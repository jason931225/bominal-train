#!/usr/bin/env bash
# -----------------------------------------------------------------------------
# local-check.sh — Start stack, wait for health, run tests/typecheck
# -----------------------------------------------------------------------------
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"
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

# Detect docker compose command (v2 plugin preferred)
if docker compose version >/dev/null 2>&1; then
  COMPOSE_CMD=(docker compose)
elif command -v docker-compose >/dev/null 2>&1; then
  COMPOSE_CMD=(docker-compose)
else
  echo "Error: docker compose (v2) or docker-compose (v1) is required"
  exit 1
fi

print_logs() {
  echo ""
  echo "=== Recent docker compose logs (api/worker/worker-restaurant/web) ==="
  "${COMPOSE_CMD[@]}" -f "$COMPOSE_FILE" logs --tail=200 api worker worker-restaurant web 2>/dev/null || true
  echo ""
  echo "Hint: run:"
  echo "  ${COMPOSE_CMD[*]} -f $COMPOSE_FILE ps"
  echo "  ${COMPOSE_CMD[*]} -f $COMPOSE_FILE logs -f api worker worker-restaurant web"
}

cleanup() {
  if [[ "$DOWN_AFTER" == "true" ]]; then
    echo ""
    echo "→ Stopping stack (--down)..."
    "${COMPOSE_CMD[@]}" -f "$COMPOSE_FILE" down >/dev/null 2>&1 || true
  fi
}
trap cleanup EXIT

if [[ ! -f "infra/env/dev/api.env" ]]; then
  echo "Error: Environment files not found. Run ./infra/scripts/local-setup.sh first."
  exit 1
fi

echo "=== bominal local check ==="
echo "→ Starting Docker Compose services..."
"${COMPOSE_CMD[@]}" -f "$COMPOSE_FILE" up -d --build

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
