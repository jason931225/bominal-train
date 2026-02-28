# Production DB Deep Dive and Gate

## Objective

Diagnose and remediate production auth-path slowness driven by DB connectivity/query latency, then enforce deploy-time regression gates for DB connect p95 and auth timeout/error counts.

## Scope

- In scope:
  - Baseline capture for DB connect/query latency, `/api/auth/me` latency, and timeout/error log counts.
  - A/B comparison between Supabase pooler and direct DB endpoints.
  - Runtime DB pool/timeout tuning and health probe semantics (`/health/live`, `/health/ready`).
  - Optional deploy-time DB SLO gates via `infra/scripts/deploy.sh`.
- Out of scope:
  - Data model changes/migrations (unless a separate approved migration plan exists).
  - Provider/business-logic rewrites unrelated to DB-path latency.

## Preconditions

- Required accounts/roles:
  - Production VM shell access as operator (`bominal`) with permission to run canonical deploy scripts.
- Required services/tools:
  - Docker + Docker Compose
  - `python3`, `curl`
  - Running `api` container
- Required environment state:
  - `infra/env/prod/api.env` exists and passes `infra/scripts/predeploy-check.sh`.
  - Production writes are only performed in approved deploy windows.

## Inputs

### Dependency-derived inputs

- Existing production `DATABASE_URL` / optional `DATABASE_URL_DIRECT`.
- Current deploy smoke settings (`SMOKE_MAX_ATTEMPTS`, `SMOKE_RETRY_DELAY_SECONDS`).
- Recent API logs from `docker compose logs api`.

### Non-dependency inputs

- Baseline and A/B sample count (default 30).
- SLO thresholds:
  - DB connect p95 max (ms)
  - auth timeout/error count max in log window.

## Deterministic Procedure

1. Capture baseline:
  - `infra/scripts/db-deep-dive.sh baseline --iterations 30 --log-window-minutes 30`
  - Artifact: `artifacts/db-deep-dive-*/baseline-summary.json`.
2. Run pooler/direct A/B if direct endpoint candidate exists:
  - `infra/scripts/db-deep-dive.sh ab --iterations 30 --pooler-url "<pooler-url>" --direct-url "<direct-url>"`
  - Artifact: `artifacts/db-deep-dive-*/ab-decision.json`.
3. Apply DB runtime tuning in env/config (pool size, overflow, connect/query/statement timeouts).
4. Re-run baseline and compare p95 + timeout/error rates vs step 1.
5. Enable optional deploy gate:
  - set `DB_SLO_GATE_ENABLED=true`
  - set thresholds (`DB_SLO_CONNECT_P95_MAX_MS`, `DB_SLO_AUTH_TIMEOUT_MAX`)
  - run canonical deploy and ensure DB SLO check passes.

## Verification Checkpoints

- Checkpoint A: Baseline artifact integrity
  - Expected signal: `baseline-summary.json` contains `database_metrics`, `endpoint_metrics`, and `log_counts`.
  - Failure signal: missing file, missing keys, or benchmark error payload.
- Checkpoint B: A/B decision determinism
  - Expected signal: decision artifact includes combined p95 values and `should_switch_to_direct`.
  - Failure signal: missing decision artifact or ambiguous/no benchmark output.
- Checkpoint C: Deploy gate enforcement
  - Expected signal: deploy output includes `DB SLO checks passed`.
  - Failure signal: deploy fails with DB connect p95/auth-timeout threshold breach.

## Failure Modes and Recovery

- Failure mode: DB benchmark fails to connect/execute.
  - Detection: `db-deep-dive.sh` emits benchmark error JSON and exits non-zero.
  - Recovery: validate DB URL/credentials/network path, then rerun baseline.
- Failure mode: Direct endpoint not stable.
  - Detection: A/B artifact reports `direct_stable=false`.
  - Recovery: keep `DATABASE_URL_TARGET=pooler`; continue pool/client timeout tuning.
- Failure mode: Deploy gate blocks release unexpectedly.
  - Detection: deploy exits during optional DB SLO check stage.
  - Recovery: inspect gate output + logs, adjust thresholds only with operator approval and documented rationale.

## Security and Redaction

- Never persist:
  - raw DB credentials, secrets, session tokens, or full provider payloads.
- Redaction requirements:
  - do not print full DB URLs in artifacts/logs; host/port-only summaries are allowed.
- Safe artifacts:
  - aggregate latency metrics, status-code counts, timeout/error counts, and decision summaries.

## Artifacts and Pointers

- Scripts:
  - `infra/scripts/db-deep-dive.sh`
  - `infra/scripts/db-slo-check.sh`
  - `infra/scripts/deploy.sh`
- Validation:
  - `infra/tests/test_db_slo_check.sh`
  - `infra/tests/test_predeploy_check.sh`

## Change History

- [0000000] Added DB deep-dive + deploy-gate playbook for production auth-path latency incidents.
