#!/usr/bin/env bash
set -euo pipefail

usage() {
  printf 'Usage: %s <owner>/<repo>\n' "${0##*/}" >&2
}

fail() {
  printf 'error: %s\n' "$1" >&2
  exit 1
}

require_command() {
  local command_name="$1"
  if ! command -v "${command_name}" >/dev/null 2>&1; then
    fail "required command not found: ${command_name}"
  fi
}

validate_repo_arg() {
  local repo="$1"
  if [[ ! "${repo}" =~ ^[^/]+/[^/]+$ ]]; then
    fail "repository must be in <owner>/<repo> format"
  fi
}

query_branch_protection_bool() {
  local repo="$1"
  local jq_expression="$2"
  local output

  if ! output="$(
    gh api \
      -H "Accept: application/vnd.github+json" \
      --jq "${jq_expression}" \
      "repos/${repo}/branches/main/protection"
  )"; then
    fail "failed to query branch protection for ${repo}"
  fi

  case "${output}" in
    true | false)
      printf '%s\n' "${output}"
      ;;
    *)
      fail "unexpected response while checking ${repo}: ${output}"
      ;;
  esac
}

main() {
  if [ "$#" -ne 1 ]; then
    usage
    exit 2
  fi

  local repo="$1"
  validate_repo_arg "${repo}"

  require_command gh

  if ! gh auth status >/dev/null 2>&1; then
    fail "GitHub CLI is not authenticated. Run: gh auth login"
  fi

  local has_required_pr_reviews
  has_required_pr_reviews="$(
    query_branch_protection_bool "${repo}" \
      ".required_pull_request_reviews != null and ((.required_pull_request_reviews.required_approving_review_count // 0) >= 1)"
  )"
  if [ "${has_required_pr_reviews}" != "true" ]; then
    fail "missing required pull request reviews (required_approving_review_count >= 1) on main branch for ${repo}"
  fi

  local has_strict_status_checks
  has_strict_status_checks="$(
    query_branch_protection_bool "${repo}" \
      ".required_status_checks != null and .required_status_checks.strict == true and ((((.required_status_checks.contexts // []) | length) > 0) or (((.required_status_checks.checks // []) | length) > 0))"
  )"
  if [ "${has_strict_status_checks}" != "true" ]; then
    fail "missing strict status checks with at least one required check/context on main branch for ${repo}"
  fi
}

main "$@"
