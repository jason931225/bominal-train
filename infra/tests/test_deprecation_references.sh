#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
GUARD_SCRIPT="$ROOT_DIR/infra/scripts/deprecation_guard.py"
REGISTRY_PATH="$ROOT_DIR/docs/deprecations/registry.json"

assert_fails() {
  local msg="$1"
  shift
  if "$@" >/dev/null 2>&1; then
    echo "FAIL: expected failure - $msg" >&2
    exit 1
  fi
}

python3 "$GUARD_SCRIPT" validate \
  --root "$ROOT_DIR" \
  --registry "$REGISTRY_PATH" >/dev/null

python3 "$GUARD_SCRIPT" scan-active-references \
  --root "$ROOT_DIR" \
  --registry "$REGISTRY_PATH" >/dev/null

TMP_DIR="$(mktemp -d)"
trap 'rm -rf "$TMP_DIR"' EXIT
mkdir -p "$TMP_DIR/repo/docs/deprecations" "$TMP_DIR/repo/infra/scripts"

cat >"$TMP_DIR/repo/infra/scripts/local-wrapper.sh" <<'EOF'
#!/usr/bin/env bash
# stale reference to removed script:
echo "infra/scripts/old-deploy.sh"
EOF

cat >"$TMP_DIR/repo/docs/deprecations/registry.json" <<'EOF'
{
  "schema_version": 1,
  "generated_at": "2026-02-14",
  "deprecations": [
    {
      "id": "DEP-TEST-001",
      "surface": "script",
      "scope": "local",
      "artifact": "infra/scripts/old-deploy.sh",
      "replacement": "infra/scripts/deploy.sh",
      "owner": "Infra / Deployment",
      "status": "removed",
      "deprecated_on": "2026-01-01",
      "remove_after": "2026-01-15",
      "removed_on": "2026-02-10",
      "window_policy": "prod30_github14_local2",
      "local_release_cycles_required": 2,
      "local_release_cycles_completed": 2,
      "callers_scan_paths": [
        "infra/scripts"
      ],
      "notes": "Test fixture"
    }
  ]
}
EOF

assert_fails "removed artifact should fail active reference scan" \
  python3 "$GUARD_SCRIPT" scan-active-references \
    --root "$TMP_DIR/repo" \
    --registry "$TMP_DIR/repo/docs/deprecations/registry.json"

echo "OK: deprecation reference scan checks passed."
