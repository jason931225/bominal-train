#!/usr/bin/env bash
set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
PROD_UP_SCRIPT="${REPO_ROOT}/scripts/prod-up.sh"

LAST_STATUS=0
LAST_OUTPUT_FILE=""

fail() {
  printf 'test failure: %s\n' "$1" >&2
  exit 1
}

assert_status() {
  local expected="$1"
  local label="$2"
  if [ "${LAST_STATUS}" -ne "${expected}" ]; then
    printf 'expected status %s, got %s (%s)\n' "${expected}" "${LAST_STATUS}" "${label}" >&2
    if [ -n "${LAST_OUTPUT_FILE}" ] && [ -f "${LAST_OUTPUT_FILE}" ]; then
      printf -- '--- output (%s) ---\n' "${label}" >&2
      cat "${LAST_OUTPUT_FILE}" >&2
      printf -- '--- end output ---\n' >&2
    fi
    exit 1
  fi
}

assert_contains() {
  local file_path="$1"
  local needle="$2"
  local label="$3"
  if ! grep -qF -- "${needle}" "${file_path}"; then
    printf 'missing expected content (%s): %s\n' "${label}" "${needle}" >&2
    printf -- '--- file: %s ---\n' "${file_path}" >&2
    cat "${file_path}" >&2
    printf -- '--- end file ---\n' >&2
    exit 1
  fi
}

new_case() {
  local case_dir
  case_dir="$(mktemp -d "${TMPDIR:-/tmp}/prod-up-test.XXXXXX")"
  mkdir -p "${case_dir}/bin"

  cat > "${case_dir}/runtime.env" <<'EOF'
BOMINAL_API_IMAGE=ghcr.io/example/bominal-api@sha256:aaa
BOMINAL_WORKER_IMAGE=ghcr.io/example/bominal-worker@sha256:bbb
DATABASE_URL=postgresql://example
EOF

  cat > "${case_dir}/compose.prod.yml" <<'EOF'
services:
  api:
    image: example
  worker:
    image: example
  redis:
    image: redis:7
EOF

  cat > "${case_dir}/vm-secrets.env" <<'EOF'
BOMINAL_DATABASE_URL=postgresql://example
EOF

  cat > "${case_dir}/bin/docker" <<'EOF'
#!/usr/bin/env bash
set -euo pipefail
printf '%s\n' "$*" >> "${MOCK_DOCKER_LOG}"

if [ "${1:-}" != "compose" ]; then
  exit 0
fi

shift
subcommand=""
for arg in "$@"; do
  case "${arg}" in
    ps|start|logs|up|pull)
      subcommand="${arg}"
      ;;
  esac
done

case "${subcommand}" in
  ps)
    if printf '%s\n' "$*" | grep -q -- '--services'; then
      case "${MOCK_PS_MODE:-full}" in
        full)
          printf 'redis\napi\nworker\n'
          ;;
        missing_api)
          printf 'redis\nworker\n'
          ;;
        missing_all)
          printf '\n'
          ;;
      esac
    else
      printf 'NAME STATUS\n'
    fi
    ;;
esac
EOF
  chmod +x "${case_dir}/bin/docker"

  cat > "${case_dir}/mock-health.sh" <<'EOF'
#!/usr/bin/env bash
set -euo pipefail
printf 'healthcheck\n' >> "${MOCK_HEALTH_LOG}"
if [ "${MOCK_HEALTH_FAIL:-0}" = "1" ]; then
  exit 1
fi
EOF
  chmod +x "${case_dir}/mock-health.sh"

  cat > "${case_dir}/mock-deploy.sh" <<'EOF'
#!/usr/bin/env bash
set -euo pipefail
printf 'deploy\n' >> "${MOCK_DEPLOY_LOG}"
if [ "${MOCK_DEPLOY_FAIL:-0}" = "1" ]; then
  exit 1
fi
EOF
  chmod +x "${case_dir}/mock-deploy.sh"

  cat > "${case_dir}/mock-rollback.sh" <<'EOF'
#!/usr/bin/env bash
set -euo pipefail
printf 'rollback\n' >> "${MOCK_ROLLBACK_LOG}"
if [ "${MOCK_ROLLBACK_FAIL:-0}" = "1" ]; then
  exit 1
fi
EOF
  chmod +x "${case_dir}/mock-rollback.sh"

  printf '%s' "${case_dir}"
}

run_case() {
  local case_dir="$1"
  shift

  LAST_OUTPUT_FILE="${case_dir}/command.out"

  set +e
  PATH="${case_dir}/bin:${PATH}" \
    MOCK_DOCKER_LOG="${case_dir}/docker.log" \
    MOCK_HEALTH_LOG="${case_dir}/health.log" \
    MOCK_DEPLOY_LOG="${case_dir}/deploy.log" \
    MOCK_ROLLBACK_LOG="${case_dir}/rollback.log" \
    MOCK_PS_MODE="${MOCK_PS_MODE:-full}" \
    MOCK_HEALTH_FAIL="${MOCK_HEALTH_FAIL:-0}" \
    MOCK_DEPLOY_FAIL="${MOCK_DEPLOY_FAIL:-0}" \
    MOCK_ROLLBACK_FAIL="${MOCK_ROLLBACK_FAIL:-0}" \
    BOMINAL_PROD_DEPLOY_SCRIPT_PATH="${case_dir}/mock-deploy.sh" \
    BOMINAL_PROD_HEALTHCHECK_SCRIPT_PATH="${case_dir}/mock-health.sh" \
    BOMINAL_PROD_ROLLBACK_SCRIPT_PATH="${case_dir}/mock-rollback.sh" \
    "${PROD_UP_SCRIPT}" "$@" >"${LAST_OUTPUT_FILE}" 2>&1
  LAST_STATUS=$?
  set -e
}

run_help_smoke() {
  LAST_OUTPUT_FILE="$(mktemp "${TMPDIR:-/tmp}/prod-up-help.XXXXXX")"
  set +e
  "${PROD_UP_SCRIPT}" help >"${LAST_OUTPUT_FILE}" 2>&1
  LAST_STATUS=$?
  set -e
}

main() {
  [ -x "${PROD_UP_SCRIPT}" ] || fail "missing executable script: ${PROD_UP_SCRIPT}"

  run_help_smoke
  assert_status 0 "help"
  assert_contains "${LAST_OUTPUT_FILE}" "Usage: ./scripts/prod-up.sh" "help usage"

  run_help_smoke
  run_case "$(new_case)" deploy
  assert_status 2 "deploy requires --yes"

  run_case "$(new_case)" rollback
  assert_status 2 "rollback requires --yes"

  local case_start_missing
  case_start_missing="$(new_case)"
  MOCK_PS_MODE="missing_api" run_case "${case_start_missing}" start \
    --runtime-env "${case_start_missing}/runtime.env" \
    --compose-file "${case_start_missing}/compose.prod.yml" \
    --vm-secret-env "${case_start_missing}/vm-secrets.env"
  assert_status 3 "start missing containers"
  assert_contains "${LAST_OUTPUT_FILE}" "runtime containers missing" "start guidance"

  local case_start_ok
  case_start_ok="$(new_case)"
  run_case "${case_start_ok}" start \
    --runtime-env "${case_start_ok}/runtime.env" \
    --compose-file "${case_start_ok}/compose.prod.yml" \
    --vm-secret-env "${case_start_ok}/vm-secrets.env"
  assert_status 0 "start success"
  assert_contains "${case_start_ok}/docker.log" "start redis api worker" "start command"
  assert_contains "${case_start_ok}/health.log" "healthcheck" "health invoked"

  local case_deploy_ok
  case_deploy_ok="$(new_case)"
  run_case "${case_deploy_ok}" deploy --yes \
    --runtime-env "${case_deploy_ok}/runtime.env" \
    --compose-file "${case_deploy_ok}/compose.prod.yml" \
    --vm-secret-env "${case_deploy_ok}/vm-secrets.env"
  assert_status 0 "deploy success"
  assert_contains "${case_deploy_ok}/deploy.log" "deploy" "deploy invoked"
  assert_contains "${case_deploy_ok}/health.log" "healthcheck" "deploy health invoked"

  local case_deploy_health_fail
  case_deploy_health_fail="$(new_case)"
  MOCK_HEALTH_FAIL="1" run_case "${case_deploy_health_fail}" deploy --yes \
    --runtime-env "${case_deploy_health_fail}/runtime.env" \
    --compose-file "${case_deploy_health_fail}/compose.prod.yml" \
    --vm-secret-env "${case_deploy_health_fail}/vm-secrets.env"
  assert_status 4 "deploy health failure exit code"
  assert_contains "${case_deploy_health_fail}/rollback.log" "rollback" "rollback after deploy health failure"

  local case_status
  case_status="$(new_case)"
  run_case "${case_status}" status \
    --runtime-env "${case_status}/runtime.env" \
    --compose-file "${case_status}/compose.prod.yml"
  assert_status 0 "status success"
  assert_contains "${LAST_OUTPUT_FILE}" "api_image=ghcr.io/example/bominal-api@sha256:aaa" "status api image"
  assert_contains "${LAST_OUTPUT_FILE}" "worker_image=ghcr.io/example/bominal-worker@sha256:bbb" "status worker image"

  local case_logs_default
  case_logs_default="$(new_case)"
  run_case "${case_logs_default}" logs \
    --runtime-env "${case_logs_default}/runtime.env" \
    --compose-file "${case_logs_default}/compose.prod.yml"
  assert_status 0 "logs default success"
  assert_contains "${case_logs_default}/docker.log" "logs api worker" "logs default services"

  local case_project_name
  case_project_name="$(new_case)"
  run_case "${case_project_name}" status \
    --project-name "bominal-prod" \
    --runtime-env "${case_project_name}/runtime.env" \
    --compose-file "${case_project_name}/compose.prod.yml"
  assert_status 0 "project name forwarding"
  assert_contains "${case_project_name}/docker.log" "--project-name bominal-prod" "project name in compose args"

  printf 'prod-up tests passed\n'
}

main "$@"
