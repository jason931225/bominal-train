#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
GUARD_SCRIPT="$ROOT_DIR/infra/scripts/deprecation_guard.py"

TMP_DIR="$(mktemp -d)"
trap 'rm -rf "$TMP_DIR"' EXIT

mkdir -p "$TMP_DIR/repo/docs/deprecations" "$TMP_DIR/repo/infra/scripts" "$TMP_DIR/repo/.github/workflows"

assert_fails() {
  local msg="$1"
  shift
  if "$@" >/dev/null 2>&1; then
    echo "FAIL: expected failure - $msg" >&2
    exit 1
  fi
}

write_valid_registry() {
  cat >"$TMP_DIR/repo/docs/deprecations/registry.json" <<'EOF'
{
  "schema_version": 1,
  "generated_at": "2026-02-14",
  "deprecations": [
    {
      "id": "DEP-VALID-PROD",
      "surface": "runtime",
      "scope": "production",
      "artifact": "infra/docker-compose.deploy.yml.deprecated",
      "replacement": "infra/docker-compose.prod.yml",
      "owner": "Infra / Deployment",
      "status": "removed",
      "deprecated_on": "2026-01-01",
      "remove_after": "2026-02-01",
      "removed_on": "2026-02-10",
      "removal_commit": "5039127",
      "window_policy": "prod30_github14_local2",
      "callers_scan_paths": [
        "infra/scripts",
        ".github/workflows"
      ],
      "notes": "Test fixture"
    },
    {
      "id": "DEP-VALID-GHA",
      "surface": "ci",
      "scope": "github",
      "artifact": ".github/workflows/deploy-legacy.yml",
      "replacement": ".github/workflows/deploy.yml",
      "owner": "Infra / CI",
      "status": "deprecated",
      "deprecated_on": "2026-02-01",
      "remove_after": "2026-02-20",
      "window_policy": "prod30_github14_local2",
      "callers_scan_paths": [
        ".github/workflows"
      ],
      "notes": "Test fixture"
    },
    {
      "id": "DEP-VALID-LOCAL",
      "surface": "script",
      "scope": "local",
      "artifact": "infra/scripts/local-legacy.sh",
      "replacement": "infra/scripts/local-check.sh",
      "owner": "Infra / Tooling",
      "status": "deprecated",
      "deprecated_on": "2026-02-10",
      "remove_after": "2026-02-28",
      "window_policy": "prod30_github14_local2",
      "local_release_cycles_required": 2,
      "local_release_cycles_completed": 1,
      "callers_scan_paths": [
        "infra/scripts"
      ],
      "notes": "Test fixture"
    }
  ]
}
EOF
}

write_valid_registry
python3 "$GUARD_SCRIPT" validate \
  --root "$TMP_DIR/repo" \
  --registry "$TMP_DIR/repo/docs/deprecations/registry.json" >/dev/null

# Missing required field should fail.
cat >"$TMP_DIR/repo/docs/deprecations/registry.json" <<'EOF'
{
  "schema_version": 1,
  "generated_at": "2026-02-14",
  "deprecations": [
    {
      "id": "DEP-BAD-001",
      "surface": "runtime",
      "scope": "production",
      "replacement": "infra/docker-compose.prod.yml",
      "owner": "Infra / Deployment",
      "status": "deprecated",
      "deprecated_on": "2026-02-01",
      "remove_after": "2026-03-10",
      "window_policy": "prod30_github14_local2",
      "callers_scan_paths": [
        "infra/scripts"
      ],
      "notes": "Missing artifact"
    }
  ]
}
EOF
assert_fails "missing artifact field" \
  python3 "$GUARD_SCRIPT" validate \
    --root "$TMP_DIR/repo" \
    --registry "$TMP_DIR/repo/docs/deprecations/registry.json"

# Production window shorter than 30 days should fail.
cat >"$TMP_DIR/repo/docs/deprecations/registry.json" <<'EOF'
{
  "schema_version": 1,
  "generated_at": "2026-02-14",
  "deprecations": [
    {
      "id": "DEP-BAD-002",
      "surface": "runtime",
      "scope": "production",
      "artifact": "infra/scripts/deploy-old.sh",
      "replacement": "infra/scripts/deploy.sh",
      "owner": "Infra / Deployment",
      "status": "deprecated",
      "deprecated_on": "2026-02-01",
      "remove_after": "2026-02-20",
      "window_policy": "prod30_github14_local2",
      "callers_scan_paths": [
        "infra/scripts"
      ],
      "notes": "Too short for production"
    }
  ]
}
EOF
assert_fails "production window < 30 days" \
  python3 "$GUARD_SCRIPT" validate \
    --root "$TMP_DIR/repo" \
    --registry "$TMP_DIR/repo/docs/deprecations/registry.json"

# Local scope requires at least two release cycles tracked.
cat >"$TMP_DIR/repo/docs/deprecations/registry.json" <<'EOF'
{
  "schema_version": 1,
  "generated_at": "2026-02-14",
  "deprecations": [
    {
      "id": "DEP-BAD-003",
      "surface": "script",
      "scope": "local",
      "artifact": "infra/scripts/local-legacy.sh",
      "replacement": "infra/scripts/local-check.sh",
      "owner": "Infra / Tooling",
      "status": "deprecated",
      "deprecated_on": "2026-02-10",
      "remove_after": "2026-02-28",
      "window_policy": "prod30_github14_local2",
      "local_release_cycles_required": 1,
      "local_release_cycles_completed": 0,
      "callers_scan_paths": [
        "infra/scripts"
      ],
      "notes": "Requires at least two cycles"
    }
  ]
}
EOF
assert_fails "local release cycle minimum not enforced" \
  python3 "$GUARD_SCRIPT" validate \
    --root "$TMP_DIR/repo" \
    --registry "$TMP_DIR/repo/docs/deprecations/registry.json"

echo "OK: deprecation policy validation checks passed."
