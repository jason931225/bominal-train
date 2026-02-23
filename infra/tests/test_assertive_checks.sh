#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
PY_CHECK="$ROOT_DIR/infra/scripts/check_assertive_tests.py"
WEB_CHECK="$ROOT_DIR/infra/scripts/check_assertive_tests_web.mjs"

TMP_DIR="$(mktemp -d)"
trap 'rm -rf "$TMP_DIR"' EXIT

mkdir -p "$TMP_DIR/api/tests" "$TMP_DIR/web/components/__tests__"

cat >"$TMP_DIR/api/tests/test_pass.py" <<'EOF'
def test_ok():
    value = 1 + 1
    assert value == 2
EOF
python3 "$PY_CHECK" "$TMP_DIR/api/tests" >/dev/null

cat >"$TMP_DIR/api/tests/test_fail.py" <<'EOF'
def test_missing_assert():
    x = 1 + 1
    _ = x
EOF
if python3 "$PY_CHECK" "$TMP_DIR/api/tests" >/dev/null 2>&1; then
  echo "ERROR: python assertive check should fail on non-assertive test." >&2
  exit 1
fi
rm -f "$TMP_DIR/api/tests/test_fail.py"

cat >"$TMP_DIR/api/tests/test_vacuous.py" <<'EOF'
def test_vacuous():
    assert True
EOF
if python3 "$PY_CHECK" "$TMP_DIR/api/tests" >/dev/null 2>&1; then
  echo "ERROR: python assertive check should fail on vacuous assertions." >&2
  exit 1
fi
rm -f "$TMP_DIR/api/tests/test_vacuous.py"

cat >"$TMP_DIR/web/components/__tests__/pass.test.ts" <<'EOF'
import { it, expect } from "vitest";
it("ok", () => {
  expect(1 + 1).toBe(2);
});
EOF
node "$WEB_CHECK" "$TMP_DIR/web" >/dev/null

mkdir -p "$TMP_DIR/web/node_modules/some_pkg"
cat >"$TMP_DIR/web/node_modules/some_pkg/ignored.test.ts" <<'EOF'
import { it } from "vitest";
it("ignored because dependency path", () => {
  const x = 1 + 1;
  void x;
});
EOF
node "$WEB_CHECK" "$TMP_DIR/web" >/dev/null

cat >"$TMP_DIR/web/components/__tests__/fail.test.ts" <<'EOF'
import { it } from "vitest";
it("missing expect", () => {
  const x = 1 + 1;
  void x;
});
EOF
if node "$WEB_CHECK" "$TMP_DIR/web" >/dev/null 2>&1; then
  echo "ERROR: web assertive check should fail on non-assertive test." >&2
  exit 1
fi
rm -f "$TMP_DIR/web/components/__tests__/fail.test.ts"

cat >"$TMP_DIR/web/components/__tests__/vacuous.test.ts" <<'EOF'
import { it, expect } from "vitest";
it("vacuous expect", () => {
  expect(true).toBe(true);
});
EOF
if node "$WEB_CHECK" "$TMP_DIR/web" >/dev/null 2>&1; then
  echo "ERROR: web assertive check should fail on vacuous assertions." >&2
  exit 1
fi

echo "OK: assertive-check script tests passed."
