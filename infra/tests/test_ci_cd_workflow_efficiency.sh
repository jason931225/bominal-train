#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
CI_INFRA_WORKFLOW="$ROOT_DIR/.github/workflows/ci-infra-quality-gates.yml"
CI_BUILD_WORKFLOW="$ROOT_DIR/.github/workflows/ci-build-publish-images.yml"
CD_WORKFLOW="$ROOT_DIR/.github/workflows/cd-deploy-production.yml"

matches_pattern() {
  local pattern="$1"
  local file="$2"
  if command -v rg >/dev/null 2>&1; then
    rg -n -- "$pattern" "$file" >/dev/null
    return $?
  fi
  grep -En -- "$pattern" "$file" >/dev/null
}

if ! matches_pattern 'echo "changed_api=\$changed_api" >> "\$GITHUB_OUTPUT"' "$CI_INFRA_WORKFLOW"; then
  echo "FAIL: ci-infra-quality-gates must emit changed_api output." >&2
  exit 1
fi

if ! matches_pattern 'echo "changed_web=\$changed_web" >> "\$GITHUB_OUTPUT"' "$CI_INFRA_WORKFLOW"; then
  echo "FAIL: ci-infra-quality-gates must emit changed_web output." >&2
  exit 1
fi

if ! matches_pattern 'actions/setup-node@v4' "$CI_INFRA_WORKFLOW" || ! matches_pattern "if: steps.change_scope.outputs.changed_api_or_web == 'true'" "$CI_INFRA_WORKFLOW"; then
  echo "FAIL: ci-infra-quality-gates must gate Node setup on API/web change scope." >&2
  exit 1
fi

if ! matches_pattern "name: Install web test dependencies" "$CI_INFRA_WORKFLOW" || ! matches_pattern "if: steps.change_scope.outputs.changed_api_or_web == 'true'" "$CI_INFRA_WORKFLOW"; then
  echo "FAIL: ci-infra-quality-gates must gate web dependency install on API/web change scope." >&2
  exit 1
fi

if ! matches_pattern "name: Dependency vulnerability scan \(Web production deps\)" "$CI_INFRA_WORKFLOW" || ! matches_pattern "if: steps.change_scope.outputs.changed_api_or_web == 'true'" "$CI_INFRA_WORKFLOW"; then
  echo "FAIL: ci-infra-quality-gates must gate web audit on API/web change scope." >&2
  exit 1
fi

if ! matches_pattern "name: Dependency vulnerability scan \(Python\)" "$CI_INFRA_WORKFLOW" || ! matches_pattern "if: steps.change_scope.outputs.changed_api == 'true'" "$CI_INFRA_WORKFLOW"; then
  echo "FAIL: ci-infra-quality-gates must gate Python dependency audit on API changes." >&2
  exit 1
fi

if ! matches_pattern '^\s*workflow_dispatch:\s*$' "$CI_BUILD_WORKFLOW"; then
  echo "FAIL: ci-build-publish-images must retain manual workflow_dispatch trigger." >&2
  exit 1
fi

if ! matches_pattern '^\s*build_api:\s*$' "$CI_BUILD_WORKFLOW" || ! matches_pattern '^\s*build_web:\s*$' "$CI_BUILD_WORKFLOW"; then
  echo "FAIL: ci-build-publish-images must define build_api/build_web manual inputs." >&2
  exit 1
fi

if ! matches_pattern 'github.event.inputs.build_api' "$CI_BUILD_WORKFLOW"; then
  echo "FAIL: Build API Image job must honor manual build_api input." >&2
  exit 1
fi

if ! matches_pattern 'github.event.inputs.build_web' "$CI_BUILD_WORKFLOW"; then
  echo "FAIL: Build Web Image job must honor manual build_web input." >&2
  exit 1
fi

if matches_pattern 'workflow_run:' "$CD_WORKFLOW"; then
  echo "FAIL: cd-deploy-production must not auto-trigger on workflow_run." >&2
  exit 1
fi

if ! matches_pattern '^\s*release:\s*$' "$CD_WORKFLOW"; then
  echo "FAIL: cd-deploy-production must auto-trigger on release events." >&2
  exit 1
fi

if matches_pattern 'test_deprecation_policy\.sh|test_deprecation_references\.sh|Validate deprecation gates' "$CD_WORKFLOW"; then
  echo "FAIL: cd-deploy-production must not duplicate deprecation policy checks already enforced in CI." >&2
  exit 1
fi

echo "PASS: CI/CD workflow efficiency policies validated."
