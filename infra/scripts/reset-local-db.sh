#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/../.." && pwd)"

# shellcheck source=infra/scripts/lib/env_utils.sh
source "${SCRIPT_DIR}/lib/env_utils.sh"

SCRIPT_NAME="$(basename "$0")"

COMPOSE_FILE="${REPO_ROOT}/infra/docker-compose.yml"
FRESH_SCHEMA="false"
PRESERVE_SIGNIN="true"
ALLOW_NON_DEV="false"
CONFIRMED="false"

usage() {
  cat <<EOF
Usage: $SCRIPT_NAME [options]

Reset local DB state for performance testing, with optional fresh-schema rebuild.

Default behavior:
  - preserves users/sign-in data in place
  - truncates runtime/high-churn tables (sessions, tasks, attempts, artifacts, secrets, tokens)

Options:
  --compose-file PATH     Compose file (default: $COMPOSE_FILE)
  --fresh-schema          Drop/recreate public schema, run migrations, and optionally restore sign-in credentials
  --preserve-signin       Preserve sign-in credentials (default)
  --no-preserve-signin    Do not preserve sign-in credentials
  --allow-non-dev         Allow use with non-dev compose files (blocked by default)
  --yes                   Required confirmation flag
  --help                  Show this help text

Examples:
  $SCRIPT_NAME --yes
  $SCRIPT_NAME --fresh-schema --yes
  $SCRIPT_NAME --fresh-schema --no-preserve-signin --yes
EOF
}

while [[ $# -gt 0 ]]; do
  case "$1" in
    --compose-file)
      COMPOSE_FILE="$2"
      shift 2
      ;;
    --fresh-schema)
      FRESH_SCHEMA="true"
      shift
      ;;
    --preserve-signin)
      PRESERVE_SIGNIN="true"
      shift
      ;;
    --no-preserve-signin)
      PRESERVE_SIGNIN="false"
      shift
      ;;
    --allow-non-dev)
      ALLOW_NON_DEV="true"
      shift
      ;;
    --yes)
      CONFIRMED="true"
      shift
      ;;
    --help|-h)
      usage
      exit 0
      ;;
    *)
      log_error "Unknown argument: $1"
      usage
      exit 1
      ;;
  esac
done

if [[ "$CONFIRMED" != "true" ]]; then
  log_error "--yes is required. This operation is destructive."
  exit 1
fi

if [[ ! -f "$COMPOSE_FILE" ]]; then
  log_error "Compose file not found: $COMPOSE_FILE"
  exit 1
fi

if [[ "$ALLOW_NON_DEV" != "true" ]] && [[ "$COMPOSE_FILE" == *"prod"* ]]; then
  log_error "Refusing to run against a production compose file without --allow-non-dev."
  exit 1
fi

detect_compose_cmd

if ! compose_service_is_running "$COMPOSE_FILE" postgres; then
  log_info "Starting postgres service..."
  "${COMPOSE_CMD[@]}" -f "$COMPOSE_FILE" up -d postgres
fi

API_SERVICE="$(first_running_compose_service "$COMPOSE_FILE" api || true)"
if [[ -z "$API_SERVICE" ]]; then
  API_SERVICE="api"
fi

if [[ "$FRESH_SCHEMA" == "true" ]] && ! compose_service_is_running "$COMPOSE_FILE" "$API_SERVICE"; then
  log_info "Starting ${API_SERVICE} service (needed for alembic migrations)..."
  "${COMPOSE_CMD[@]}" -f "$COMPOSE_FILE" up -d "$API_SERVICE"
fi

tmp_dir="$(mktemp -d)"
trap 'rm -rf "$tmp_dir"' EXIT
snapshot_csv="${tmp_dir}/signin_snapshot.csv"

snapshot_signin_credentials() {
  if [[ "$PRESERVE_SIGNIN" != "true" ]]; then
    return 0
  fi
  log_info "Snapshotting users.email/display_name/password_hash..."
  "${COMPOSE_CMD[@]}" -f "$COMPOSE_FILE" exec -T postgres \
    psql -v ON_ERROR_STOP=1 -U bominal -d bominal \
    -c "COPY (SELECT email, COALESCE(display_name, ''), password_hash FROM users ORDER BY created_at) TO STDOUT WITH CSV" \
    >"$snapshot_csv"
  log_info "Snapshot rows: $(wc -l <"$snapshot_csv" | tr -d ' ')"
}

truncate_runtime_tables() {
  log_info "Truncating runtime/high-churn tables..."
  if [[ "$PRESERVE_SIGNIN" == "true" ]]; then
    "${COMPOSE_CMD[@]}" -f "$COMPOSE_FILE" exec -T postgres \
      psql -v ON_ERROR_STOP=1 -U bominal -d bominal <<'SQL'
BEGIN;
TRUNCATE TABLE
  sessions,
  verification_tokens,
  password_reset_tokens,
  task_attempts,
  artifacts,
  tasks,
  secrets
RESTART IDENTITY CASCADE;
COMMIT;
SQL
  else
    "${COMPOSE_CMD[@]}" -f "$COMPOSE_FILE" exec -T postgres \
      psql -v ON_ERROR_STOP=1 -U bominal -d bominal <<'SQL'
BEGIN;
TRUNCATE TABLE
  sessions,
  verification_tokens,
  password_reset_tokens,
  task_attempts,
  artifacts,
  tasks,
  secrets,
  users
RESTART IDENTITY CASCADE;
COMMIT;
SQL
  fi
}

fresh_schema_reset() {
  log_warn "Dropping and recreating public schema..."
  "${COMPOSE_CMD[@]}" -f "$COMPOSE_FILE" exec -T postgres \
    psql -v ON_ERROR_STOP=1 -U bominal -d bominal <<'SQL'
DROP SCHEMA public CASCADE;
CREATE SCHEMA public;
GRANT ALL ON SCHEMA public TO bominal;
GRANT ALL ON SCHEMA public TO public;
SQL

  log_info "Running alembic upgrade head..."
  "${COMPOSE_CMD[@]}" -f "$COMPOSE_FILE" exec -T "$API_SERVICE" alembic upgrade head

  if [[ "$PRESERVE_SIGNIN" == "true" ]] && [[ -s "$snapshot_csv" ]]; then
    log_info "Restoring preserved sign-in credentials..."
    "${COMPOSE_CMD[@]}" -f "$COMPOSE_FILE" cp "$snapshot_csv" postgres:/tmp/bominal_signin_snapshot.csv
    "${COMPOSE_CMD[@]}" -f "$COMPOSE_FILE" exec -T postgres \
      psql -v ON_ERROR_STOP=1 -U bominal -d bominal <<'SQL'
CREATE TEMP TABLE signin_restore (
  email text NOT NULL,
  display_name text NOT NULL,
  password_hash text NOT NULL
);
\copy signin_restore(email, display_name, password_hash) FROM '/tmp/bominal_signin_snapshot.csv' WITH CSV;

INSERT INTO users (id, email, password_hash, display_name, role_id, created_at, updated_at)
SELECT
  gen_random_uuid(),
  email,
  password_hash,
  NULLIF(display_name, ''),
  2,
  now(),
  now()
FROM signin_restore;
SQL
  fi
}

snapshot_signin_credentials

if [[ "$FRESH_SCHEMA" == "true" ]]; then
  fresh_schema_reset
else
  truncate_runtime_tables
fi

log_info "DB reset complete."
if [[ "$PRESERVE_SIGNIN" == "true" ]]; then
  log_info "Sign-in credentials were preserved when available."
else
  log_warn "Sign-in credentials were not preserved."
fi
