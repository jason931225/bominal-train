# 2026-02-27 Full Audit — 1.0.0 Big-Bang Release Readiness (Renewed)

## Scope

This artifact records execution of the renewed 1.0.0 full release audit plan with:

- governance/policy runtime parity gates
- functional verification for auth and train/task processes
- payment-path verification through Evervault relay/function boundaries only
- no real payment attempt

Canonical references:

- `docs/governance/SECRETS_RESIDENCY_POLICY.md`
- `docs/governance/FREE_TIER_BUDGET_POLICY.md`
- `.github/workflows/ci-infra-quality-gates.yml`
- `.github/workflows/cd-deploy-production.yml`
- `docs/humans/operations/DEPLOYMENT.md`
- `docs/humans/operations/RUNBOOK.md`

## Execution Context

- Repository: `bominal`
- Audit date: `2026-02-27`
- Audit evidence root: `artifacts/release-audit-20260227T221136Z/`
- Lanes executed:
  - `current` (initial run)
  - `current-remediated` (post-remediation rerun)
  - `origin-main` (clean baseline worktree)

## Lane Outcomes

| Lane | Pass | Fail | Notes |
|---|---:|---:|---|
| `current` | 7 | 9 | Functional failures caused by host cache permission drift (`~/.cache/uv`, `~/.npm`). |
| `current-remediated` | 16 | 0 | All gates passed after environment remediation. |
| `origin-main` | 16 | 0 | Full baseline parity with remediated lane. |

## Remediation Ledger

### R1 — Host cache permission drift (Environment Drift)

- Symptom:
  - `uv` cache initialization failed at `~/.cache/uv`.
  - `npm ci` failed at `~/.npm/_cacache`.
- Impact:
  - functional checks failed despite policy checks passing.
- Action:
  - rerun with writable cache overrides:
    - `UV_CACHE_DIR=/Users/jasonlee/bominal/.cache/uv` (workspace lane)
    - `NPM_CONFIG_CACHE=/Users/jasonlee/bominal/.cache/npm` (workspace lane)
    - `UV_CACHE_DIR=/tmp/bominal-uv-cache` (origin-main lane)
    - `NPM_CONFIG_CACHE=/tmp/bominal-npm-cache` (origin-main lane)
- Result:
  - all previously failing functional gates turned green.
- Classification:
  - environment drift (not code/runtime contract regression).

## Gate Matrix (Final)

| Gate ID | Check | `current-remediated` | `origin-main` |
|---|---|---|---|
| `G01` | `test_docs_no_duplicate_security_sections.sh` | PASS | PASS |
| `G02` | `test_policy_runtime_parity.sh` | PASS | PASS |
| `G03` | `test_secret_residency_contract.sh` | PASS | PASS |
| `G04` | `test_payment_boundary_regressions.sh` | PASS | PASS |
| `G05` | `test_sync_edge_secrets_from_gsm.sh` | PASS | PASS |
| `G06` | `test_free_tier_status_report.sh` | PASS | PASS |
| `G07` | `test_ensure_uv_api_venv.sh` | PASS | PASS |
| `F00` | `ensure-uv-api-venv.sh` bootstrap | PASS | PASS |
| `F-AUTH-01` | auth/session test suite | PASS | PASS |
| `F-TRAIN-01` | train/task functional suite | PASS | PASS |
| `F-TRAIN-EDGE-OFF` | edge notify off matrix | PASS | PASS |
| `F-TRAIN-EDGE-ON` | edge notify on matrix | PASS | PASS |
| `F-PAY-API-01` | payment API boundary suite | PASS | PASS |
| `F-PAY-WEB-DEPS` | web deps install for payment/train UI tests | PASS | PASS |
| `F-PAY-WEB-01` | wallet Evervault UI test | PASS | PASS |
| `F-TRAIN-WEB-01` | train dashboard UI tests | PASS | PASS |

## Payment Hard-Blocker Assessment

Payment verification was performed without attempting a real payment:

- boundary regression gate passed (`G04`)
- wallet/payment API suite passed (`F-PAY-API-01`)
- web Evervault wallet payload suite passed (`F-PAY-WEB-01`)
- Evervault plaintext/CVV rejection contract remained enforced via policy/runtime and wallet-schema checks

No charge execution, card submission to live payment provider, or real payment mutation was performed.

## Go/No-Go Decision

- **Payment blocker status:** PASS
- **Governance + functional matrix status:** PASS in both final lanes
- **Decision:** **GO** for 1.0.0 release readiness under current policy contracts

## Evidence Pointers

- Initial lane matrix: `artifacts/release-audit-20260227T221136Z/current/matrix.tsv`
- Remediated lane matrix: `artifacts/release-audit-20260227T221136Z/current-remediated/matrix.tsv`
- Baseline lane matrix: `artifacts/release-audit-20260227T221136Z/origin-main/matrix.tsv`
- Per-gate logs:
  - `artifacts/release-audit-20260227T221136Z/current/logs/`
  - `artifacts/release-audit-20260227T221136Z/current-remediated/logs/`
  - `artifacts/release-audit-20260227T221136Z/origin-main/logs/`

## Follow-Up Actions

1. Keep weekly `free_tier_status_report.sh` evidence collection active per policy.
2. Keep GSM active secret versions under `<=6` per family.
3. Keep edge secret sync step before any rollout with `EDGE_TASK_NOTIFY_ENABLED=true`.
