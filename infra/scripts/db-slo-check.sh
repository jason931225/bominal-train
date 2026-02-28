#!/usr/bin/env bash
set -euo pipefail

SCRIPT_NAME="$(basename "$0")"

SAMPLES_FILE=""
DATABASE_URL=""
CONNECT_ITERATIONS=20
CONNECT_TIMEOUT_SECONDS=3
CONNECT_P95_MAX_MS=""
API_LOG_FILE=""
AUTH_TIMEOUT_MAX=""
AUTH_TIMEOUT_PATTERN='TimeoutError|QueryCanceledError'

usage() {
  cat <<EOF
Usage: $SCRIPT_NAME [options]

Evaluate DB-path SLO thresholds from connect-latency samples and API timeout/error logs.

Options:
  --samples-file PATH          File with one connect latency (ms) value per line.
  --database-url URL           Postgres URL to probe when --samples-file is not provided.
  --connect-iterations N       Number of TCP connect probes for URL mode (default: $CONNECT_ITERATIONS).
  --connect-timeout-seconds N  Socket connect timeout for URL mode (default: $CONNECT_TIMEOUT_SECONDS).
  --connect-p95-max-ms N       Fail if connect p95 exceeds this threshold.
  --api-log-file PATH          API log file for timeout/error counting.
  --auth-timeout-max N         Fail if auth timeout/error count exceeds this threshold.
  --auth-timeout-pattern REGEX Regex counted from api log file (default: $AUTH_TIMEOUT_PATTERN).
  --help                       Show this help text.

Notes:
  - Supply either --samples-file or --database-url.
  - For deploy gating, use --database-url + --api-log-file.
EOF
}

error() {
  echo "[ERROR] $*" >&2
}

info() {
  echo "[INFO] $*"
}

require_file() {
  local path="$1"
  local name="$2"
  if [[ ! -f "$path" ]]; then
    error "$name not found: $path"
    exit 1
  fi
}

require_non_negative_integer() {
  local value="$1"
  local name="$2"
  if [[ ! "$value" =~ ^[0-9]+$ ]]; then
    error "$name must be a non-negative integer (got: ${value:-<empty>})"
    exit 1
  fi
}

require_positive_number() {
  local value="$1"
  local name="$2"
  if ! awk -v v="$value" 'BEGIN { exit !(v + 0 > 0) }'; then
    error "$name must be > 0 (got: ${value:-<empty>})"
    exit 1
  fi
}

percentile_from_file() {
  local file="$1"
  local percentile="$2"
  awk -v p="$percentile" '
    {
      values[++n] = $1
    }
    END {
      if (n == 0) {
        printf "0.00"
        exit
      }
      rank = (p / 100.0) * n
      idx = int(rank)
      if (rank > idx) {
        idx++
      }
      if (idx < 1) {
        idx = 1
      } else if (idx > n) {
        idx = n
      }
      printf "%.2f", values[idx]
    }
  ' "$file"
}

mean_from_file() {
  local file="$1"
  awk '
    {
      sum += $1
      count++
    }
    END {
      if (count == 0) {
        printf "0.00"
      } else {
        printf "%.2f", sum / count
      }
    }
  ' "$file"
}

min_from_file() {
  local file="$1"
  awk 'NR == 1 { min = $1 } $1 < min { min = $1 } END { if (NR == 0) printf "0.00"; else printf "%.2f", min }' "$file"
}

max_from_file() {
  local file="$1"
  awk 'NR == 1 { max = $1 } $1 > max { max = $1 } END { if (NR == 0) printf "0.00"; else printf "%.2f", max }' "$file"
}

benchmark_connect_samples() {
  local database_url="$1"
  local iterations="$2"
  local connect_timeout="$3"
  local output_file="$4"
  python3 - "$database_url" "$iterations" "$connect_timeout" <<'PY' >"$output_file"
import socket
import sys
import time
from urllib.parse import urlsplit

url = sys.argv[1]
iterations = int(sys.argv[2])
timeout = float(sys.argv[3])
parsed = urlsplit(url)
host = parsed.hostname
port = parsed.port or 5432
if not host:
    raise SystemExit("database URL missing host")

for _ in range(iterations):
    started = time.perf_counter()
    sock = socket.create_connection((host, port), timeout=timeout)
    sock.close()
    elapsed_ms = (time.perf_counter() - started) * 1000.0
    print(f"{elapsed_ms:.3f}")
PY
}

while [[ $# -gt 0 ]]; do
  case "$1" in
    --samples-file)
      SAMPLES_FILE="$2"
      shift 2
      ;;
    --database-url)
      DATABASE_URL="$2"
      shift 2
      ;;
    --connect-iterations)
      CONNECT_ITERATIONS="$2"
      shift 2
      ;;
    --connect-timeout-seconds)
      CONNECT_TIMEOUT_SECONDS="$2"
      shift 2
      ;;
    --connect-p95-max-ms)
      CONNECT_P95_MAX_MS="$2"
      shift 2
      ;;
    --api-log-file)
      API_LOG_FILE="$2"
      shift 2
      ;;
    --auth-timeout-max)
      AUTH_TIMEOUT_MAX="$2"
      shift 2
      ;;
    --auth-timeout-pattern)
      AUTH_TIMEOUT_PATTERN="$2"
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

if [[ -z "$SAMPLES_FILE" && -z "$DATABASE_URL" ]]; then
  error "Provide either --samples-file or --database-url."
  exit 1
fi

require_non_negative_integer "$CONNECT_ITERATIONS" "--connect-iterations"
if [[ "$CONNECT_ITERATIONS" -lt 1 ]]; then
  error "--connect-iterations must be >= 1"
  exit 1
fi
require_positive_number "$CONNECT_TIMEOUT_SECONDS" "--connect-timeout-seconds"

if [[ -n "$CONNECT_P95_MAX_MS" ]]; then
  require_positive_number "$CONNECT_P95_MAX_MS" "--connect-p95-max-ms"
fi
if [[ -n "$AUTH_TIMEOUT_MAX" ]]; then
  require_non_negative_integer "$AUTH_TIMEOUT_MAX" "--auth-timeout-max"
  if [[ -z "$API_LOG_FILE" ]]; then
    error "--api-log-file is required when --auth-timeout-max is set."
    exit 1
  fi
fi
if [[ -n "$API_LOG_FILE" ]]; then
  require_file "$API_LOG_FILE" "--api-log-file"
fi

tmp_dir="$(mktemp -d)"
trap 'rm -rf "$tmp_dir"' EXIT
samples_tmp="$tmp_dir/connect-samples.txt"
sorted_tmp="$tmp_dir/connect-samples-sorted.txt"

if [[ -n "$SAMPLES_FILE" ]]; then
  require_file "$SAMPLES_FILE" "--samples-file"
  awk 'NF > 0 { print $1 }' "$SAMPLES_FILE" >"$samples_tmp"
else
  info "Running connect benchmark ($CONNECT_ITERATIONS iterations)"
  benchmark_connect_samples "$DATABASE_URL" "$CONNECT_ITERATIONS" "$CONNECT_TIMEOUT_SECONDS" "$samples_tmp"
fi

if [[ ! -s "$samples_tmp" ]]; then
  error "No connect samples were collected."
  exit 1
fi

sort -n "$samples_tmp" >"$sorted_tmp"

connect_p50="$(percentile_from_file "$sorted_tmp" 50)"
connect_p95="$(percentile_from_file "$sorted_tmp" 95)"
connect_mean="$(mean_from_file "$samples_tmp")"
connect_min="$(min_from_file "$sorted_tmp")"
connect_max="$(max_from_file "$sorted_tmp")"

timeout_count=0
if [[ -n "$API_LOG_FILE" ]]; then
  timeout_count="$(grep -E -c "$AUTH_TIMEOUT_PATTERN" "$API_LOG_FILE" || true)"
fi

echo "[INFO] DB connect latency (ms): min=$connect_min mean=$connect_mean p50=$connect_p50 p95=$connect_p95 max=$connect_max"
if [[ -n "$API_LOG_FILE" ]]; then
  echo "[INFO] Auth timeout/error matches: $timeout_count (pattern: $AUTH_TIMEOUT_PATTERN)"
fi

if [[ -n "$CONNECT_P95_MAX_MS" ]]; then
  if ! awk -v p95="$connect_p95" -v max="$CONNECT_P95_MAX_MS" 'BEGIN { exit !(p95 <= max) }'; then
    error "DB connect p95 exceeded threshold ($connect_p95 > $CONNECT_P95_MAX_MS ms)."
    exit 1
  fi
fi

if [[ -n "$AUTH_TIMEOUT_MAX" ]]; then
  if [[ "$timeout_count" -gt "$AUTH_TIMEOUT_MAX" ]]; then
    error "Auth timeout/error count exceeded threshold ($timeout_count > $AUTH_TIMEOUT_MAX)."
    exit 1
  fi
fi

echo "[INFO] DB SLO checks passed."
