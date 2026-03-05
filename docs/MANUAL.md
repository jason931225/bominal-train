# Bominal Manual

## Product Scope And Non-Negotiables

This manual is the canonical policy and operations source for **bominal**.

Mandatory invariants:
- Product name stays `bominal` in UI, docs, and config.
- `third_party/srtgo` and `third_party/catchtable` are read-only.
- Train-provider behavior must remain source-aligned with `third_party/srtgo/srtgo/srt.py` and `third_party/srtgo/srtgo/ktx.py`.
- Runtime baseline is Rust-first (`runtime/`), with no production dependency on retired Python runtime paths.
- Sensitive data must never be logged, serialized to queues, or persisted outside approved boundaries.

Authoritative code map:
- `runtime/crates/api`: API + SSR shell.
- `runtime/crates/worker`: background task runtime.
- `runtime/crates/shared`: shared contracts and integrations.
- `runtime/migrations`: SQL migrations.
- `runtime/frontend`: Tailwind build assets for runtime UI.
- `docs/handoff`: preserved external handoff reference set.

## Security And Data Handling Policy

Required controls:
- Secrets MUST be sourced from environment or managed secret stores. Plaintext secrets in docs, code, logs, and tickets are prohibited.
- Session cookies MUST be `HttpOnly`, `SameSite=Lax`, and `Secure` only in production.
- Payment and credential payloads MUST be encrypted at rest and redacted in logs.
- Provider requests MUST use TLS with certificate validation enabled.
- Internal APIs MUST require explicit service authentication; deny by default.

Prohibited patterns:
- Logging request/response bodies that can contain credentials or payment data.
- Storing raw PAN/CVV or equivalent cardholder fields in database, queue, or artifacts.
- Disabling TLS verification or broadening egress without explicit approval.

Evidence requirements:
- Security-sensitive changes include explicit test coverage for failure/deny paths.
- Every release candidate must include a secrets and log-redaction sanity check.

## Permissions And Approval Model

Risk classes:
- `READ`: inspection only.
- `WRITE`: repository changes.
- `DESTRUCTIVE`: irreversible deletes/resets/migrations.
- `SECURITY`: IAM/auth/secret/egress boundary changes.
- `PRODUCTION`: live-environment mutations.

Approval gates:
- `DESTRUCTIVE`, `SECURITY`, and `PRODUCTION` actions require explicit human approval.
- Break-glass production writes require:
  - owner approval,
  - time-bound window,
  - command list,
  - rollback plan,
  - incident/change reference.

Default behavior:
- Fail closed when permission scope is unclear.
- Prefer read-only validation before any mutating step.

## Engineering Quality Standard

Baseline quality rules:
- Every behavior change MUST include directly relevant tests.
- Critical-path changes (auth, payments, secrets, deployment) MUST include negative-path tests.
- No unresolved warnings in touched scope.
- Deprecated dependency/runtime warnings are actionable and must be triaged before release.

Coverage floors (minimum guardrails):
- Rust API + worker workspace line coverage: >= 80% for changed crates.
- Web test suite coverage: >= 75% lines/functions/branches/statements for changed areas.

Verification contract:
- Run formatting, compile, and tests before completion claims.
- Completion claims require command evidence, not assumptions.

## CI-CD Target State Policy

Platform is intentionally tool-agnostic. Any CI/CD implementation MUST satisfy these outcome gates.

Required pipeline stages:
1. Source integrity gate
- deterministic checkout
- lockfile integrity
- forbidden-secret scan

2. Build and static gate
- compile all runtime services
- type/lint checks for web/runtime assets

3. Test and quality gate
- unit + integration test execution
- coverage threshold enforcement
- critical-path negative test enforcement

4. Supply-chain and security gate
- dependency vulnerability scan
- base image provenance/signing checks

5. Release gate
- immutable artifacts with version metadata
- changelog and docs alignment verification

6. Deploy gate
- pre-deploy config validation
- migration safety validation
- rollout strategy enforcement (progressive/controlled)

7. Post-deploy gate
- health checks
- smoke checks
- rollback trigger evaluation

Change-scope routing policy:
- Docs-only deltas MUST bypass heavy CI/CD stages (build, runtime test, artifact publish, deploy).
- Docs-only means changes under `docs/**` or markdown-only file updates.
- Deploy workflows MUST NOT trigger from markdown-only changes even when they occur under otherwise deploy-scoped paths (for example `runtime/**` or `env/prod/**`).

Mandatory artifacts:
- build metadata,
- test and coverage reports,
- security scan report,
- deployment decision log,
- post-deploy verification summary.

## Deployment And Rollback Standard

Deployment policy:
- Use immutable artifacts.
- Prefer progressive rollout with health-gated advancement.
- Database migrations must be forward-safe and operationally reversible.

Minimum deployment checks:
- API liveness and readiness endpoints return expected states.
- Worker startup and queue connectivity validated.
- SSR/web asset availability confirmed.
- VM memory baseline is enforced before rollout:
  - `/swapfile` active at 2G,
  - `vm.swappiness=10`,
  - `vm.vfs_cache_pressure=50`.

Rollback policy:
- Immediate rollback on sustained health failure, data integrity risk, or security regression.
- Rollback path must be documented before rollout.
- Post-rollback incident note is mandatory for any production revert.

## Operations Runbook Core

Incident baseline:
- classify severity (`SEV1`-`SEV4`),
- assign incident owner,
- stabilize service first,
- then remediate root cause.

Triage sequence:
1. Confirm scope and user impact.
2. Validate health/readiness and upstream dependencies.
3. Check recent deploy/config deltas.
4. Execute the least-risk mitigation.
5. Capture timeline and decisions.

Operational expectations:
- Every production-impacting event produces a concise incident summary.
- Repeated incidents must generate a preventive action item.

## Documentation Governance

Single-source rule:
- `docs/MANUAL.md` is canonical policy + operations guidance.
- Other docs should reference this manual instead of redefining policy.

Change discipline:
- Docs-first for policy-sensitive work.
- Docs-last verification before completion.
- `CHANGELOG.md` must record notable docs/policy changes.

Pointer discipline:
- Keep `docs/README.md` as a compact pointer index.
- Keep `docs/START_HERE.md` as orientation entrypoint.
- Keep `docs/INTENT_ROUTING.md` as keyword router into manual sections.
- Keep `docs/PROD_ENV_CONTRACT.md` aligned with production env key contracts and secret-handling posture.

## Current Gap Register And Backfill Targets

Current repository state:
- Runtime source is present and buildable.
- Env templates are present under `env/**`.
- Baseline GitHub Actions workflows are present as `.github/workflows/ci.yml` and `.github/workflows/cd.yml`.
- VM deploy contract is fail-closed and environment-defined: CD requires protected deploy script-path variables, executes remote scripts over IAP SSH, runs SQL migrations before service restart, deploys immutable image digests, and triggers rollback when post-deploy health checks fail.
- Full infra validation suites and richer rollback automation are not yet fully restored.

Mandatory backfill targets:
1. CI quality pipeline implementation aligned to this manual's gates.
2. Deployment orchestration with preflight checks and rollback automation.
3. Infra validation suite for docs, security, deployment, and runtime contracts.
4. Production runbook command set with deterministic evidence capture.

Backfill completion criteria:
- All target-state pipeline gates are executable in-repo.
- Deployment + rollback paths are testable and documented.
- Manual and operational commands reflect actual committed infrastructure.
