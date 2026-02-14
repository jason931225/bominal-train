#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
source "$ROOT_DIR/infra/scripts/lib/env_utils.sh"

TMP_DIR="$(mktemp -d)"
trap 'rm -rf "$TMP_DIR"' EXIT

assert_eq() {
  local actual="$1"
  local expected="$2"
  local msg="$3"
  if [[ "$actual" != "$expected" ]]; then
    echo "FAIL: $msg (actual='$actual' expected='$expected')" >&2
    exit 1
  fi
}

test_env_key_value_parses_trimmed_values() {
  local file="$TMP_DIR/sample.env"
  cat >"$file" <<'EOF'
# comment
FOO=bar
SPACED = value with space
QUOTED="quoted value"
EMPTY=
EOF

  assert_eq "$(env_key_value "$file" "FOO")" "bar" "FOO parsing"
  assert_eq "$(env_key_value "$file" "SPACED")" "value with space" "SPACED parsing"
  assert_eq "$(env_key_value "$file" "QUOTED")" "quoted value" "QUOTED parsing"
  assert_eq "$(env_key_value "$file" "MISSING")" "" "MISSING parsing"
}

test_copy_env_from_examples_creates_targets() {
  local env_dir="$TMP_DIR/env"
  mkdir -p "$env_dir"
  echo "A=1" >"$env_dir/api.env.example"
  echo "B=2" >"$env_dir/web.env.example"

  copy_env_from_examples "$env_dir" >/dev/null
  [[ -f "$env_dir/api.env" ]] || { echo "FAIL: api.env not created" >&2; exit 1; }
  [[ -f "$env_dir/web.env" ]] || { echo "FAIL: web.env not created" >&2; exit 1; }
}

test_placeholder_detection_fails_on_change_me() {
  local file="$TMP_DIR/prod.env"
  echo "INTERNAL_API_KEY=CHANGE_ME_VALUE" >"$file"
  if require_no_env_placeholders "$file" >/dev/null 2>&1; then
    echo "FAIL: placeholder should fail validation" >&2
    exit 1
  fi
}

test_env_key_required_nonempty() {
  local file="$TMP_DIR/required.env"
  cat >"$file" <<'EOF'
INTERNAL_API_KEY=abc123
MASTER_KEY=secret
EOF

  require_env_key_nonempty "$file" "INTERNAL_API_KEY"
  require_env_key_nonempty "$file" "MASTER_KEY"
  if require_env_key_nonempty "$file" "MISSING_KEY" >/dev/null 2>&1; then
    echo "FAIL: missing key should fail validation" >&2
    exit 1
  fi
}

test_resolve_compose_file_prefers_primary_and_falls_back() {
  local repo_dir="$TMP_DIR/repo"
  mkdir -p "$repo_dir/infra"
  touch "$repo_dir/infra/docker-compose.yml"

  assert_eq "$(resolve_compose_file "$repo_dir")" "$repo_dir/infra/docker-compose.yml" "fallback compose file resolution"

  touch "$repo_dir/infra/docker-compose.prod.yml"
  assert_eq "$(resolve_compose_file "$repo_dir")" "$repo_dir/infra/docker-compose.prod.yml" "primary compose file resolution"
}

test_env_key_value_parses_trimmed_values
test_copy_env_from_examples_creates_targets
test_placeholder_detection_fails_on_change_me
test_env_key_required_nonempty
test_resolve_compose_file_prefers_primary_and_falls_back

echo "OK: env_utils tests passed."
