#!/usr/bin/env bash
set -euo pipefail

SCRIPT_NAME="$(basename "$0")"
ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"

SUBCOMMAND="${1:-}"
if [[ -n "$SUBCOMMAND" ]]; then
  shift
fi

COMPOSE_FILE="${COMPOSE_FILE:-$ROOT_DIR/infra/docker-compose.prod.yml}"
API_SERVICE="${API_SERVICE:-api}"
API_BASE_URL="${API_BASE_URL:-http://127.0.0.1:8000}"
AUTH_PATH="${AUTH_PATH:-/api/auth/me}"
ITERATIONS="${ITERATIONS:-30}"
LOG_WINDOW_MINUTES="${LOG_WINDOW_MINUTES:-30}"
OUTPUT_DIR="${OUTPUT_DIR:-$ROOT_DIR/artifacts/db-deep-dive-$(date -u +%Y%m%dT%H%M%SZ)}"

POOLER_URL=""
DIRECT_URL=""

usage() {
  cat <<EOF
Usage: $SCRIPT_NAME <baseline|ab> [options]

Commands:
  baseline   Collect DB/auth-path latency baseline from the running API container.
  ab         Compare pooler vs direct DB URLs from the same API container runtime.

Common options:
  --compose-file PATH         Docker compose file (default: $COMPOSE_FILE)
  --api-service NAME          API service name in compose (default: $API_SERVICE)
  --iterations N              Sample count for probes (default: $ITERATIONS)
  --output-dir PATH           Output directory for JSON artifacts (default: timestamped artifacts dir)

Baseline options:
  --api-base-url URL          API base URL for endpoint timing (default: $API_BASE_URL)
  --auth-path PATH            Auth endpoint path to probe (default: $AUTH_PATH)
  --log-window-minutes N      Docker log window for error counts (default: $LOG_WINDOW_MINUTES)

AB options:
  --pooler-url URL            Pooler Postgres URL (required for ab)
  --direct-url URL            Direct Postgres URL (required for ab)

Examples:
  $SCRIPT_NAME baseline --iterations 30
  $SCRIPT_NAME ab --pooler-url "postgresql+asyncpg://..." --direct-url "postgresql+asyncpg://..."
EOF
}

error() {
  echo "[ERROR] $*" >&2
}

info() {
  echo "[INFO] $*"
}

require_integer_ge_one() {
  local value="$1"
  local name="$2"
  if [[ ! "$value" =~ ^[0-9]+$ ]] || [[ "$value" -lt 1 ]]; then
    error "$name must be an integer >= 1 (got: ${value:-<empty>})"
    exit 1
  fi
}

require_cmd() {
  local command_name="$1"
  if ! command -v "$command_name" >/dev/null 2>&1; then
    error "Missing required command: $command_name"
    exit 1
  fi
}

docker_compose() {
  docker compose -f "$COMPOSE_FILE" "$@"
}

normalize_db_url_for_asyncpg() {
  local db_url="$1"
  python3 - "$db_url" <<'PY'
import sys
from urllib.parse import parse_qsl, urlencode, urlsplit, urlunsplit

url = (sys.argv[1] or "").strip()
if not url:
    print("")
    raise SystemExit(0)

try:
    parsed = urlsplit(url)
except Exception:
    print(url)
    raise SystemExit(0)

scheme = (parsed.scheme or "").lower()
if scheme == "postgresql+asyncpg":
    parsed = parsed._replace(scheme="postgresql")

query_pairs = parse_qsl(parsed.query, keep_blank_values=True)
has_ssl = any(key.lower() == "ssl" for key, _ in query_pairs)
sslmode_values = [value for key, value in query_pairs if key.lower() == "sslmode"]
filtered_pairs = [(key, value) for key, value in query_pairs if key.lower() != "sslmode"]
if sslmode_values and not has_ssl:
    filtered_pairs.append(("ssl", sslmode_values[-1]))

rebuilt_query = urlencode(filtered_pairs, doseq=True)
print(urlunsplit((parsed.scheme, parsed.netloc, parsed.path, rebuilt_query, parsed.fragment)))
PY
}

benchmark_db_url_in_container() {
  local db_url="$1"
  local iterations="$2"
  local normalized_db_url
  normalized_db_url="$(normalize_db_url_for_asyncpg "$db_url")"
  docker_compose exec -T "$API_SERVICE" python - "$normalized_db_url" "$iterations" <<'PY'
import asyncio
import json
import socket
import statistics
import sys
import time
from urllib.parse import urlsplit

import asyncpg

db_url = sys.argv[1]
iterations = int(sys.argv[2])

def pct(values, p):
    if not values:
        return 0.0
    ordered = sorted(values)
    rank = int((p / 100.0) * len(ordered))
    if (p / 100.0) * len(ordered) > rank:
        rank += 1
    rank = max(1, min(rank, len(ordered)))
    return float(ordered[rank - 1])

async def main():
    parsed = urlsplit(db_url)
    host = parsed.hostname
    port = parsed.port or 5432
    if not host:
        raise RuntimeError("database URL missing host")

    connect_ms = []
    query_fresh_ms = []
    query_reuse_ms = []
    errors = []

    for _ in range(iterations):
        t0 = time.perf_counter()
        sock = socket.create_connection((host, port), timeout=3)
        sock.close()
        connect_ms.append((time.perf_counter() - t0) * 1000.0)

    for _ in range(iterations):
        t0 = time.perf_counter()
        conn = await asyncpg.connect(dsn=db_url)
        try:
            await conn.execute("SELECT 1")
        finally:
            await conn.close()
        query_fresh_ms.append((time.perf_counter() - t0) * 1000.0)

    conn = await asyncpg.connect(dsn=db_url)
    try:
        for _ in range(iterations):
            t0 = time.perf_counter()
            await conn.execute("SELECT 1")
            query_reuse_ms.append((time.perf_counter() - t0) * 1000.0)
    finally:
        await conn.close()

    def summarize(values):
        return {
            "min": round(min(values), 2) if values else 0.0,
            "mean": round(statistics.fmean(values), 2) if values else 0.0,
            "p50": round(pct(values, 50), 2),
            "p95": round(pct(values, 95), 2),
            "max": round(max(values), 2) if values else 0.0,
        }

    out = {
        "host": host,
        "port": port,
        "iterations": iterations,
        "connect_ms": summarize(connect_ms),
        "query_fresh_ms": summarize(query_fresh_ms),
        "query_reuse_ms": summarize(query_reuse_ms),
        "errors": errors,
    }
    print(json.dumps(out))

try:
    asyncio.run(main())
except Exception as exc:
    print(json.dumps({"error": str(exc)}))
    raise
PY
}

collect_endpoint_latencies() {
  local url="$1"
  local iterations="$2"
  local output_file="$3"
  : >"$output_file"
  for _ in $(seq 1 "$iterations"); do
    local timing
    timing="$(curl -sS -o /dev/null -w '%{time_total} %{http_code}' "$url")" || timing="0 000"
    local seconds http_code
    seconds="${timing%% *}"
    http_code="${timing##* }"
    awk -v sec="$seconds" -v code="$http_code" 'BEGIN { printf "%.3f %s\n", sec * 1000.0, code }' >>"$output_file"
  done
}

summarize_endpoint_samples() {
  local input_file="$1"
  python3 - "$input_file" <<'PY'
import json
import statistics
import sys

path = sys.argv[1]
latencies = []
codes = {}
with open(path, "r", encoding="utf-8") as handle:
    for line in handle:
        line = line.strip()
        if not line:
            continue
        ms_str, code = line.split()
        latencies.append(float(ms_str))
        codes[code] = codes.get(code, 0) + 1

def pct(values, p):
    if not values:
        return 0.0
    ordered = sorted(values)
    rank = int((p / 100.0) * len(ordered))
    if (p / 100.0) * len(ordered) > rank:
        rank += 1
    rank = max(1, min(rank, len(ordered)))
    return float(ordered[rank - 1])

payload = {
    "count": len(latencies),
    "min": round(min(latencies), 2) if latencies else 0.0,
    "mean": round(statistics.fmean(latencies), 2) if latencies else 0.0,
    "p50": round(pct(latencies, 50), 2),
    "p95": round(pct(latencies, 95), 2),
    "max": round(max(latencies), 2) if latencies else 0.0,
    "http_codes": codes,
}
print(json.dumps(payload))
PY
}

resolve_active_database_url() {
  docker_compose exec -T "$API_SERVICE" python - <<'PY'
import os

target = (os.getenv("DATABASE_URL_TARGET") or "pooler").strip().lower()
pooler = (os.getenv("DATABASE_URL") or "").strip()
direct = (os.getenv("DATABASE_URL_DIRECT") or "").strip()

if target == "direct" and direct:
    print(direct)
else:
    print(pooler)
PY
}

run_baseline() {
  require_cmd docker
  require_cmd python3
  require_cmd curl
  require_integer_ge_one "$ITERATIONS" "--iterations"
  require_integer_ge_one "$LOG_WINDOW_MINUTES" "--log-window-minutes"

  mkdir -p "$OUTPUT_DIR"
  local baseline_json="$OUTPUT_DIR/baseline-summary.json"
  local db_json="$OUTPUT_DIR/db-active.json"
  local health_samples="$OUTPUT_DIR/health-ready.samples"
  local auth_samples="$OUTPUT_DIR/auth-me.samples"
  local api_logs="$OUTPUT_DIR/api.logs"

  info "Collecting baseline into $OUTPUT_DIR"

  local active_url
  active_url="$(resolve_active_database_url)"
  if [[ -z "$active_url" ]]; then
    error "Unable to resolve active DATABASE_URL from API container."
    exit 1
  fi

  benchmark_db_url_in_container "$active_url" "$ITERATIONS" >"$db_json"
  if grep -q '"error"' "$db_json"; then
    error "DB benchmark failed: $(cat "$db_json")"
    exit 1
  fi

  collect_endpoint_latencies "$API_BASE_URL/health/ready" "$ITERATIONS" "$health_samples"
  collect_endpoint_latencies "$API_BASE_URL$AUTH_PATH" "$ITERATIONS" "$auth_samples"

  docker_compose logs --since "${LOG_WINDOW_MINUTES}m" "$API_SERVICE" >"$api_logs" 2>&1 || true
  local timeout_count query_canceled_count auth_me_5xx auth_me_4xx
  timeout_count="$(grep -c 'TimeoutError' "$api_logs" || true)"
  query_canceled_count="$(grep -c 'QueryCanceledError' "$api_logs" || true)"
  auth_me_5xx="$(grep -E -c '/api/auth/me.*status=5[0-9]{2}' "$api_logs" || true)"
  auth_me_4xx="$(grep -E -c '/api/auth/me.*status=4[0-9]{2}' "$api_logs" || true)"

  local load_snapshot
  load_snapshot="$(uptime || true)"

  local health_summary auth_summary
  health_summary="$(summarize_endpoint_samples "$health_samples")"
  auth_summary="$(summarize_endpoint_samples "$auth_samples")"

  python3 - "$db_json" "$health_summary" "$auth_summary" "$timeout_count" "$query_canceled_count" "$auth_me_5xx" "$auth_me_4xx" "$load_snapshot" "$baseline_json" <<'PY'
import json
import sys
from datetime import datetime, timezone

db_json_path = sys.argv[1]
health_summary = json.loads(sys.argv[2])
auth_summary = json.loads(sys.argv[3])
timeout_count = int(sys.argv[4])
query_canceled_count = int(sys.argv[5])
auth_me_5xx = int(sys.argv[6])
auth_me_4xx = int(sys.argv[7])
load_snapshot = sys.argv[8]
out_path = sys.argv[9]

with open(db_json_path, "r", encoding="utf-8") as handle:
    db_metrics = json.load(handle)

payload = {
    "captured_at_utc": datetime.now(timezone.utc).isoformat(),
    "host_load_snapshot": load_snapshot,
    "database_metrics": db_metrics,
    "endpoint_metrics": {
        "health_ready": health_summary,
        "auth_me": auth_summary,
    },
    "log_counts": {
        "TimeoutError": timeout_count,
        "QueryCanceledError": query_canceled_count,
        "auth_me_5xx": auth_me_5xx,
        "auth_me_4xx": auth_me_4xx,
    },
}

with open(out_path, "w", encoding="utf-8") as handle:
    json.dump(payload, handle, indent=2)

print(json.dumps(payload, indent=2))
PY

  info "Baseline summary written: $baseline_json"
}

run_ab() {
  require_cmd docker
  require_cmd python3
  require_integer_ge_one "$ITERATIONS" "--iterations"

  if [[ -z "$POOLER_URL" || -z "$DIRECT_URL" ]]; then
    error "ab requires --pooler-url and --direct-url."
    exit 1
  fi

  mkdir -p "$OUTPUT_DIR"
  local pooler_json="$OUTPUT_DIR/ab-pooler.json"
  local direct_json="$OUTPUT_DIR/ab-direct.json"
  local decision_json="$OUTPUT_DIR/ab-decision.json"

  info "Running A/B benchmark (pooler vs direct)"
  benchmark_db_url_in_container "$POOLER_URL" "$ITERATIONS" >"$pooler_json"
  benchmark_db_url_in_container "$DIRECT_URL" "$ITERATIONS" >"$direct_json"

  python3 - "$pooler_json" "$direct_json" "$decision_json" <<'PY'
import json
import sys
from datetime import datetime, timezone

pooler_path, direct_path, decision_path = sys.argv[1:4]

with open(pooler_path, "r", encoding="utf-8") as handle:
    pooler = json.load(handle)
with open(direct_path, "r", encoding="utf-8") as handle:
    direct = json.load(handle)

pooler_combined_p95 = float(pooler["connect_ms"]["p95"]) + float(pooler["query_fresh_ms"]["p95"])
direct_combined_p95 = float(direct["connect_ms"]["p95"]) + float(direct["query_fresh_ms"]["p95"])

improvement_pct = 0.0
if pooler_combined_p95 > 0:
    improvement_pct = ((pooler_combined_p95 - direct_combined_p95) / pooler_combined_p95) * 100.0

direct_stable = not bool(direct.get("errors"))
should_switch_to_direct = direct_stable and improvement_pct >= 40.0

payload = {
    "captured_at_utc": datetime.now(timezone.utc).isoformat(),
    "pooler_combined_p95_ms": round(pooler_combined_p95, 2),
    "direct_combined_p95_ms": round(direct_combined_p95, 2),
    "improvement_pct": round(improvement_pct, 2),
    "direct_stable": direct_stable,
    "decision_rule": "switch when direct improves connect+query p95 by >= 40% and remains stable",
    "should_switch_to_direct": should_switch_to_direct,
}

with open(decision_path, "w", encoding="utf-8") as handle:
    json.dump(payload, handle, indent=2)

print(json.dumps(payload, indent=2))
PY

  info "A/B decision summary written: $decision_json"
}

while [[ $# -gt 0 ]]; do
  case "$1" in
    --compose-file)
      COMPOSE_FILE="$2"
      shift 2
      ;;
    --api-service)
      API_SERVICE="$2"
      shift 2
      ;;
    --api-base-url)
      API_BASE_URL="$2"
      shift 2
      ;;
    --auth-path)
      AUTH_PATH="$2"
      shift 2
      ;;
    --iterations)
      ITERATIONS="$2"
      shift 2
      ;;
    --log-window-minutes)
      LOG_WINDOW_MINUTES="$2"
      shift 2
      ;;
    --output-dir)
      OUTPUT_DIR="$2"
      shift 2
      ;;
    --pooler-url)
      POOLER_URL="$2"
      shift 2
      ;;
    --direct-url)
      DIRECT_URL="$2"
      shift 2
      ;;
    --help|-h)
      usage
      exit 0
      ;;
    *)
      error "Unknown argument: $1"
      usage
      exit 1
      ;;
  esac
done

case "$SUBCOMMAND" in
  baseline)
    run_baseline
    ;;
  ab)
    run_ab
    ;;
  ""|-h|--help|help)
    usage
    ;;
  *)
    error "Unknown subcommand: $SUBCOMMAND"
    usage
    exit 1
    ;;
esac
