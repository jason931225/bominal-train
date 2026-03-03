#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
# shellcheck source=scripts/prod/_env_lib.sh
source "${SCRIPT_DIR}/_env_lib.sh"

require_cmd docker
require_var BOMINAL_DATABASE_URL
require_var BOMINAL_MIGRATIONS_DIR

if [ ! -d "${BOMINAL_MIGRATIONS_DIR}" ]; then
  fail "migrations directory not found: ${BOMINAL_MIGRATIONS_DIR}"
fi

normalize_database_url() {
  local value="$1"
  value="${value/postgresql+asyncpg:\/\//postgresql://}"
  value="${value/postgresql+psycopg:\/\//postgresql://}"
  value="${value/postgresql+psycopg2:\/\//postgresql://}"
  value="$(printf '%s' "${value}" \
    | sed -E \
      -e 's/([?&])ssl=true/\1sslmode=require/g' \
      -e 's/([?&])ssl=false/\1sslmode=disable/g' \
      -e 's/([?&])ssl=/\1sslmode=/g')"
  printf '%s' "${value}"
}

database_url="$(normalize_database_url "${BOMINAL_DATABASE_URL}")"
use_native_psql=0
if command -v psql >/dev/null 2>&1; then
  use_native_psql=1
fi

if [ "${use_native_psql}" = "0" ] && [ "$(uname -s)" != "Linux" ]; then
  fail "psql is required on non-Linux hosts (docker --network host fallback is Linux-only)"
fi

psql_exec_sql() {
  local sql="$1"
  if [ "${use_native_psql}" = "1" ]; then
    psql "${database_url}" --no-psqlrc --set ON_ERROR_STOP=1 --tuples-only --no-align --quiet -c "${sql}"
  else
    docker run --rm --network host postgres:16-alpine \
      psql "${database_url}" --no-psqlrc --set ON_ERROR_STOP=1 --tuples-only --no-align --quiet -c "${sql}"
  fi
}

psql_exec_file() {
  local migration_file="$1"
  if [ "${use_native_psql}" = "1" ]; then
    psql "${database_url}" --no-psqlrc --set ON_ERROR_STOP=1 --file "${migration_file}"
  else
    docker run --rm --network host \
      -v "${BOMINAL_MIGRATIONS_DIR}:/migrations:ro" \
      postgres:16-alpine \
      psql "${database_url}" --no-psqlrc --set ON_ERROR_STOP=1 --file "/migrations/$(basename "${migration_file}")"
  fi
}

mapfile -t migration_files < <(find "${BOMINAL_MIGRATIONS_DIR}" -maxdepth 1 -type f -name '*.sql' | sort)
if [ "${#migration_files[@]}" -eq 0 ]; then
  fail "no migration files found in ${BOMINAL_MIGRATIONS_DIR}"
fi

psql_exec_sql "create table if not exists runtime_schema_migrations (migration_name text primary key, applied_at timestamptz not null default now());" >/dev/null

for migration_file in "${migration_files[@]}"; do
  migration_name="$(basename "${migration_file}")"
  escaped_migration_name="$(printf '%s' "${migration_name}" | sed "s/'/''/g")"
  already_applied="$(
    psql_exec_sql "select 1 from runtime_schema_migrations where migration_name = '${escaped_migration_name}' limit 1;" \
      | tr -d '[:space:]'
  )"
  if [ "${already_applied}" = "1" ]; then
    log "skip migration (already applied): ${migration_name}"
    continue
  fi

  log "apply migration: ${migration_name}"
  psql_exec_file "${migration_file}"
  psql_exec_sql "insert into runtime_schema_migrations (migration_name) values ('${escaped_migration_name}') on conflict (migration_name) do nothing;" >/dev/null
done

log "migrations completed"
