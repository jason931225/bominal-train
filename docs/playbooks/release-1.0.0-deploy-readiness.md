# 1.0.0 Release and Deployment Readiness

## Objective

Provide a deterministic, repeatable workflow to confirm **bominal** is ready for a `1.0.0` release and production deployment, with clear go/no-go checkpoints and rollback-ready evidence.

## Scope

- In scope:
  - release-readiness verification for code/docs/version state
  - production deploy preflight and post-deploy verification commands
  - SemVer mapping steps for promoting from pre-1.0 mode to `1.0.0`
  - full big-bang audit matrix across `current` and clean `origin/main` lanes
  - auth and train/task functional verification in dual runtime matrix (`EDGE_TASK_NOTIFY_ENABLED=false/true`)
  - payment boundary verification via Evervault contracts without real payment attempts
- Out of scope:
  - authoring large feature changes
  - emergency incident response outside standard deployment flow
  - production break-glass actions without explicit approval context

## Preconditions

- Required accounts/roles:
  - repo write access for branch + PR workflow
  - production VM operator access for `sudo -u bominal` deploy command
- Required services/tools:
  - `git`, `bash`, `python3`, and Docker/Compose on deployment host
  - CI pipelines for infra quality gates and image builds
- Required environment state:
  - production env files are present and placeholder-free
  - deployment host has healthy baseline resources and network egress

## Inputs

### Dependency-derived inputs

- target release commit SHA (usually `HEAD` of approved branch)
- current version baseline and resolved version output from `infra/scripts/version_guard.py`
- current production deployment status from `infra/scripts/deploy.sh --status`

### Non-dependency inputs

- approval to perform release promotion (when moving to `1.0.0`)
- approved release notes/changelog content
- operational window for deployment and smoke verification

## Deterministic Procedure

1. Validate documentation/control-plane baselines.
   - `bash infra/tests/test_docs_pointers.sh`
   - `bash infra/tests/test_intent_routing.sh`
   - `bash infra/tests/test_docs_consistency.sh`
   - `bash infra/tests/test_changelog.sh`
2. Validate release mapping state before promotion.
   - `python3 infra/scripts/version_guard.py validate`
   - `python3 infra/scripts/version_guard.py resolve --commit HEAD`
3. Verify deployment-preflight contracts before any production mutation.
   - `bash infra/tests/test_predeploy_check.sh`
   - `bash infra/tests/test_deploy_preflight.sh`
4. Prepare `1.0.0` SemVer parity update.
   - add a new `1.0.0` entry in `docs/releases/version-map.json` mapped to the release commit with `"bump": "major"`
   - rerun `python3 infra/scripts/version_guard.py validate`
5. Execute canonical production deployment.
   - `sudo -u bominal /opt/bominal/repo/infra/scripts/deploy.sh <release-commit-sha>`
6. Run post-deploy health verification.
   - `sudo -u bominal /opt/bominal/repo/infra/scripts/deploy.sh --status`
   - verify API health endpoint and web endpoint from the deployment host/network perimeter
7. Complete release evidence and traceability.
   - ensure `CHANGELOG.md` contains release-ready notable entries
   - tag/release using your approved release process after deployment verification is green

## Renewed Big-Bang Audit (Mandatory Before 1.0.0 Release)

Run this matrix before final go/no-go:

1. Execute the policy/runtime gate set in both lanes (`current` and clean `origin/main`):
   - `bash infra/tests/test_docs_no_duplicate_security_sections.sh`
   - `bash infra/tests/test_policy_runtime_parity.sh`
   - `bash infra/tests/test_secret_residency_contract.sh`
   - `bash infra/tests/test_payment_boundary_regressions.sh`
   - `bash infra/tests/test_sync_edge_secrets_from_gsm.sh`
   - `bash infra/tests/test_free_tier_status_report.sh`
   - `bash infra/tests/test_ensure_uv_api_venv.sh`
2. Run host Python bootstrap/parity:
   - `bash infra/scripts/ensure-uv-api-venv.sh`
3. Run auth functional suite:
   - `cd api && uv run --python .venv/bin/python -m pytest -q tests/test_auth_flow.py tests/test_auth_route_units.py tests/test_auth_service_units.py tests/test_supabase_auth.py tests/test_supabase_auth_service.py`
4. Run train/task suites:
   - `cd api && uv run --python .venv/bin/python -m pytest -q tests/test_train_tasks.py tests/test_train_queue.py tests/test_train_worker_provider_flow_units.py tests/test_train_worker_remaining_units.py tests/test_train_router_and_rate_limiter_units.py tests/test_rate_limit_units.py tests/test_worker_runtime_units.py`
   - `cd api && EDGE_TASK_NOTIFY_ENABLED=false uv run --python .venv/bin/python -m pytest -q tests/test_train_worker_provider_flow_units.py tests/test_train_worker_remaining_units.py`
   - `cd api && EDGE_TASK_NOTIFY_ENABLED=true SUPABASE_URL=https://example.supabase.co SUPABASE_SERVICE_ROLE_KEY=test-key SUPABASE_EDGE_TIMEOUT_SECONDS=5 EMAIL_FROM_ADDRESS=no-reply@example.com uv run --python .venv/bin/python -m pytest -q tests/test_train_worker_provider_flow_units.py tests/test_train_worker_remaining_units.py`
5. Run payment-boundary verification (no payment execution):
   - `cd api && uv run --python .venv/bin/python -m pytest -q tests/test_wallet.py tests/test_system_payment_service_units.py tests/test_train_service_payment_units.py`
   - `npm ci --prefix web`
   - `npm --prefix web run test -- --run components/wallet/__tests__/payment-settings-panel.test.tsx`
6. Run train web smoke:
   - `npm --prefix web run test -- --run components/train/__tests__/train-dashboard.actions.test.tsx components/train/__tests__/train-dashboard.polling.test.tsx`
7. Capture per-gate pass/fail evidence and classify failures:
   - `doc drift` vs `runtime drift` vs `environment drift`.
8. Apply blocker logic:
   - payment fail => `NO-GO`
   - payment pass and all mandatory gates pass => `GO` (track non-blockers separately)

### Payment Guardrail for Audit Execution

- Real payment attempts are prohibited during this audit.
- Payment verification must remain mock/contract based:
  - Evervault relay/function wiring checks
  - wallet schema boundary checks (CVV/plaintext fallback rejection)
  - log-boundary/redaction checks
- Release packet must explicitly state that no real payment attempt occurred.

## Verification Checkpoints

- Checkpoint A: documentation and changelog validators pass.
  - Expected signal: all docs validators exit `0` and no pointer/routing/changelog failures.
  - Failure signal: missing pointer registration, stale routing entries, or changelog format failures.
- Checkpoint B: version guard validates pre-release and post-`1.0.0` mapping edits.
  - Expected signal: `version_guard.py validate` exits `0` and resolve output matches expected release version.
  - Failure signal: invalid bump transition or duplicate/missing commit-version mapping.
- Checkpoint C: deploy + smoke verification succeed.
  - Expected signal: deploy script completes successfully; `--status` and health checks report healthy services.
  - Failure signal: preflight block, smoke-check failure, or unhealthy core services.

## Failure Modes and Recovery

- Failure mode: version mapping rejected during `1.0.0` promotion.
  - Detection: `version_guard.py validate` non-zero with bump/mapping errors.
  - Recovery: fix `docs/releases/version-map.json`, revalidate, and do not deploy until green.
- Failure mode: deployment preflight blocks rollout.
  - Detection: `infra/scripts/deploy.sh` exits before mutation and reports threshold/env/deprecation gate failure.
  - Recovery: resolve reported preflight issue (env placeholders, resources, deprecation policy) and rerun.
- Failure mode: smoke verification fails after rollout attempt.
  - Detection: deploy output indicates smoke-check failure and/or rollback path trigger.
  - Recovery: follow rollback procedures in `docs/humans/operations/DEPLOYMENT.md` and capture incident evidence.

## Security and Redaction

- Never persist:
  - plaintext secrets, token values, cardholder data, or full provider payloads containing sensitive fields.
- Redaction requirements:
  - redact env secrets and authorization headers in logs/screenshots.
  - keep deploy evidence to status, health, and commit/version metadata.
- Safe artifacts:
  - validator outputs, commit SHAs, version resolver output, and health-check summaries.

## Artifacts and Pointers

- `docs/releases/README.md`
- `docs/releases/version-map.json`
- `docs/humans/operations/DEPLOYMENT.md`
- `docs/governance/CHANGE_MANAGEMENT.md`
- `infra/scripts/deploy.sh`
- `infra/scripts/version_guard.py`
- `infra/scripts/predeploy-check.sh`

## Change History

- [0000000] Initial playbook for deterministic `1.0.0` release readiness and production deployment execution.
- [0000000] Added renewed big-bang release audit matrix and payment/no-charge verification guardrails aligned with secret residency, free-tier, and infra quality gate policies.
