#!/usr/bin/env bash
set -euo pipefail

SCRIPT_NAME="$(basename "$0")"

BASE_URL="${BASE_URL:-http://localhost:8000}"
EMAIL="${BENCH_EMAIL:-perf.bench@example.com}"
PASSWORD="${BENCH_PASSWORD:-SuperSecret123}"
DISPLAY_NAME="${BENCH_DISPLAY_NAME:-Perf Bench User}"
ITERATIONS="${BENCH_ITERATIONS:-30}"
ACTIVE_LIMIT="${BENCH_ACTIVE_LIMIT:-60}"
COMPLETED_LIMIT="${BENCH_COMPLETED_LIMIT:-80}"
REFRESH_COMPLETED="${BENCH_REFRESH_COMPLETED:-false}"
REGISTER_USER="true"

usage() {
  cat <<EOF
Usage: $SCRIPT_NAME [options]

Benchmark /api/train/tasks latency and print p50/p95 for active and completed lists.

Options:
  --base-url URL              API base URL (default: $BASE_URL)
  --email EMAIL               Login email used for benchmark (default: $EMAIL)
  --password PASSWORD         Login password (default: BENCH_PASSWORD or SuperSecret123)
  --display-name NAME         Display name for optional auto-register (default: $DISPLAY_NAME)
  --iterations N              Number of requests per status (default: $ITERATIONS)
  --active-limit N            limit for status=active query (default: $ACTIVE_LIMIT)
  --completed-limit N         limit for status=completed query (default: $COMPLETED_LIMIT)
  --refresh-completed BOOL    Use refresh_completed for completed query (default: $REFRESH_COMPLETED)
  --register-user BOOL        Register benchmark user before login (default: true)
  --help                      Show this help text

Environment overrides:
  BASE_URL, BENCH_EMAIL, BENCH_PASSWORD, BENCH_DISPLAY_NAME,
  BENCH_ITERATIONS, BENCH_ACTIVE_LIMIT, BENCH_COMPLETED_LIMIT, BENCH_REFRESH_COMPLETED
EOF
}

error() {
  echo "[ERROR] $*" >&2
}

info() {
  echo "[INFO] $*"
}

require_positive_integer() {
  local value="$1"
  local name="$2"
  if [[ ! "$value" =~ ^[1-9][0-9]*$ ]]; then
    error "$name must be a positive integer (got: $value)"
    exit 1
  fi
}

require_bool() {
  local value="$1"
  local name="$2"
  case "$value" in
    true|false) ;;
    *)
      error "$name must be true or false (got: $value)"
      exit 1
      ;;
  esac
}

percentile_from_file() {
  local file="$1"
  local p="$2"
  awk -v p="$p" '
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
  awk 'NR==1{min=$1} $1<min{min=$1} END{if (NR==0) printf "0.00"; else printf "%.2f", min}' "$file"
}

max_from_file() {
  local file="$1"
  awk 'NR==1{max=$1} $1>max{max=$1} END{if (NR==0) printf "0.00"; else printf "%.2f", max}' "$file"
}

request_ms() {
  local url="$1"
  local cookie_jar="$2"
  local http_code_file="$3"
  local response_file="$4"
  local timing

  timing="$(
    curl -sS \
      -o "$response_file" \
      -w '%{time_total} %{http_code}' \
      -b "$cookie_jar" \
      -c "$cookie_jar" \
      "$url"
  )"

  local seconds code
  seconds="${timing%% *}"
  code="${timing##* }"
  printf '%s' "$code" >"$http_code_file"
  awk -v sec="$seconds" 'BEGIN { printf "%.3f", sec * 1000 }'
}

register_if_needed() {
  local register="$1"
  if [[ "$register" != "true" ]]; then
    return 0
  fi

  curl -sS \
    -o /dev/null \
    -X POST "${BASE_URL}/api/auth/register" \
    -H 'Content-Type: application/json' \
    --data "{\"email\":\"${EMAIL}\",\"password\":\"${PASSWORD}\",\"display_name\":\"${DISPLAY_NAME}\"}" \
    || true
}

login_and_store_cookie() {
  local cookie_jar="$1"
  local response_file="$2"
  local code

  code="$(
    curl -sS \
      -o "$response_file" \
      -w '%{http_code}' \
      -c "$cookie_jar" \
      -X POST "${BASE_URL}/api/auth/login" \
      -H 'Content-Type: application/json' \
      --data "{\"email\":\"${EMAIL}\",\"password\":\"${PASSWORD}\",\"remember_me\":true}"
  )"
  if [[ "$code" != "200" ]]; then
    error "Login failed (HTTP $code). Response:"
    cat "$response_file" >&2
    exit 1
  fi
}

benchmark_status() {
  local status="$1"
  local limit="$2"
  local refresh_completed="$3"
  local iterations="$4"
  local cookie_jar="$5"
  local output_file="$6"
  local tmp_response="$7"
  local tmp_code="$8"
  local url="${BASE_URL}/api/train/tasks?status=${status}&limit=${limit}"

  if [[ "$status" == "completed" && "$refresh_completed" == "true" ]]; then
    url="${url}&refresh_completed=true"
  fi

  : >"$output_file"
  for _ in $(seq 1 "$iterations"); do
    local ms
    ms="$(request_ms "$url" "$cookie_jar" "$tmp_code" "$tmp_response")"
    if [[ "$(cat "$tmp_code")" != "200" ]]; then
      error "Request failed for status=${status} (HTTP $(cat "$tmp_code")). Response:"
      cat "$tmp_response" >&2
      exit 1
    fi
    echo "$ms" >>"$output_file"
  done
}

while [[ $# -gt 0 ]]; do
  case "$1" in
    --base-url)
      BASE_URL="$2"
      shift 2
      ;;
    --email)
      EMAIL="$2"
      shift 2
      ;;
    --password)
      PASSWORD="$2"
      shift 2
      ;;
    --display-name)
      DISPLAY_NAME="$2"
      shift 2
      ;;
    --iterations)
      ITERATIONS="$2"
      shift 2
      ;;
    --active-limit)
      ACTIVE_LIMIT="$2"
      shift 2
      ;;
    --completed-limit)
      COMPLETED_LIMIT="$2"
      shift 2
      ;;
    --refresh-completed)
      REFRESH_COMPLETED="$2"
      shift 2
      ;;
    --register-user)
      REGISTER_USER="$2"
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

require_positive_integer "$ITERATIONS" "--iterations"
require_positive_integer "$ACTIVE_LIMIT" "--active-limit"
require_positive_integer "$COMPLETED_LIMIT" "--completed-limit"
require_bool "$REFRESH_COMPLETED" "--refresh-completed"
require_bool "$REGISTER_USER" "--register-user"

tmp_dir="$(mktemp -d)"
trap 'rm -rf "$tmp_dir"' EXIT

cookie_jar="$tmp_dir/cookies.txt"
response_file="$tmp_dir/response.txt"
http_code_file="$tmp_dir/http_code.txt"
active_file="$tmp_dir/active_ms.txt"
completed_file="$tmp_dir/completed_ms.txt"

register_if_needed "$REGISTER_USER"
login_and_store_cookie "$cookie_jar" "$response_file"

info "Running ${ITERATIONS} requests for status=active (limit=${ACTIVE_LIMIT})"
benchmark_status "active" "$ACTIVE_LIMIT" "false" "$ITERATIONS" "$cookie_jar" "$active_file" "$response_file" "$http_code_file"
info "Running ${ITERATIONS} requests for status=completed (limit=${COMPLETED_LIMIT}, refresh_completed=${REFRESH_COMPLETED})"
benchmark_status "completed" "$COMPLETED_LIMIT" "$REFRESH_COMPLETED" "$ITERATIONS" "$cookie_jar" "$completed_file" "$response_file" "$http_code_file"

active_sorted="$tmp_dir/active_sorted.txt"
completed_sorted="$tmp_dir/completed_sorted.txt"
sort -n "$active_file" >"$active_sorted"
sort -n "$completed_file" >"$completed_sorted"

echo
echo "Train Task List Latency (ms)"
echo "base_url=${BASE_URL} iterations=${ITERATIONS}"
echo
printf "%-10s %8s %8s %8s %8s %8s\n" "status" "mean" "p50" "p95" "min" "max"
printf "%-10s %8s %8s %8s %8s %8s\n" \
  "active" \
  "$(mean_from_file "$active_file")" \
  "$(percentile_from_file "$active_sorted" 50)" \
  "$(percentile_from_file "$active_sorted" 95)" \
  "$(min_from_file "$active_file")" \
  "$(max_from_file "$active_file")"
printf "%-10s %8s %8s %8s %8s %8s\n" \
  "completed" \
  "$(mean_from_file "$completed_file")" \
  "$(percentile_from_file "$completed_sorted" 50)" \
  "$(percentile_from_file "$completed_sorted" 95)" \
  "$(min_from_file "$completed_file")" \
  "$(max_from_file "$completed_file")"
