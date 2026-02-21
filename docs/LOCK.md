# Dynamic Lock Ledger

Use this file to claim dynamic edit scopes during multi-session execution.

Rules:
- Acquire minimal lock scope before writing code.
- Commit lock acquisition separately.
- Re-check this file before writing and before each commit.
- Release lock after stage completion.
- Commit lock release separately.
- Template/example entries must never use live statuses.

## Current Entries

### LOCK-2026-02-21-API-SPLIT-LATENCY-HARDENING
- status: ACTIVE
- owner_session: Codex Session
- scope:
  - `api/**`
  - `infra/**`
  - `docs/**`
  - `README.md`
  - `CHANGELOG.md`
- reason: Implement gateway/domain API split, session/auth/provider latency hardening, and naming-consistent infra/docs updates with verification
- created_at_utc: 2026-02-21T21:45:00Z
- released_at_utc:

### LOCK-2026-02-18-EMAIL-WORKER-LEGACY-PAYLOAD
- status: RELEASED
- owner_session: Codex Session
- scope:
  - `api/**`
  - `docs/**`
  - `CHANGELOG.md`
- reason: Diagnose and fix legacy email payload validation failures in deliver_email_job with regression coverage and docs sync
- created_at_utc: 2026-02-18T14:10:11Z
- released_at_utc: 2026-02-18T14:15:30Z

### LOCK-2026-02-18-KTX-WAIT-RESERVE-FIX
- status: RELEASED
- owner_session: Codex Session
- scope:
  - `api/**`
  - `docs/**`
  - `CHANGELOG.md`
- reason: Diagnose and implement KTX wait-reserve candidate selection hotfix with regression coverage and docs/changelog sync
- created_at_utc: 2026-02-18T13:53:07Z
- released_at_utc: 2026-02-18T14:02:21Z

### LOCK-2026-02-17-RESY-REFRESH-LOGOUT-STAGE3
- status: RELEASED
- owner_session: Codex Session
- scope:
  - `api/**`
  - `docs/**`
  - `infra/env/prod/api.env.example`
  - `CHANGELOG.md`
- reason: Implement Resy auth.refresh and logout contract paths with tests/config/docs synchronization
- created_at_utc: 2026-02-17T19:17:41Z
- released_at_utc: 2026-02-17T19:20:54Z

### LOCK-2026-02-17-RESY-STAGE2-ADAPTER
- status: RELEASED
- owner_session: Codex Session
- scope:
  - `api/**`
  - `docs/**`
  - `infra/env/prod/api.env.example`
  - `CHANGELOG.md`
- reason: Implement Resy stage-2 adapter operations (profile/search/create/cancel) with tests and docs sync
- created_at_utc: 2026-02-17T19:07:21Z
- released_at_utc: 2026-02-17T19:15:29Z

### LOCK-2026-02-17-RESY-THIRDPARTY-CROSSCHECK
- status: RELEASED
- owner_session: Codex Session
- scope:
  - `docs/**`
  - `CHANGELOG.md`
- reason: Cross-check `third_party/resy` endpoint/payload references and synchronize canonical provider docs/plan status
- created_at_utc: 2026-02-17T18:58:22Z
- released_at_utc: 2026-02-17T19:05:36Z

### LOCK-2026-02-17-RESY-AUTH-STAGE1
- status: RELEASED
- owner_session: Codex Session
- scope:
  - `api/**`
  - `docs/**`
  - `infra/env/prod/api.env.example`
  - `CHANGELOG.md`
- reason: Resy auth.start/auth.complete stage-1 implementation with config/tests/docs updates
- created_at_utc: 2026-02-17T18:03:07Z
- released_at_utc: 2026-02-17T18:06:52Z

### LOCK-2026-02-17-OPENTABLE-OTP-RESPONSE-FREEZE
- status: RELEASED
- owner_session: Codex Session
- scope:
  - `api/**`
  - `docs/**`
  - `CHANGELOG.md`
- reason: OpenTable OTP success/error response schema freeze with adapter normalization and docs/test updates
- created_at_utc: 2026-02-17T17:59:50Z
- released_at_utc: 2026-02-17T18:02:29Z

### LOCK-2026-02-17-OPENTABLE-CONFIRMATION-ENRICHMENT
- status: RELEASED
- owner_session: Codex Session
- scope:
  - `api/**`
  - `docs/**`
  - `infra/env/prod/api.env.example`
  - `CHANGELOG.md`
- reason: OpenTable create enrichment + safe policy mapping tests/docs follow-up (excluding deferred policy-checkbox doc section)
- created_at_utc: 2026-02-17T17:49:12Z
- released_at_utc: 2026-02-17T17:54:09Z

### LOCK-2026-02-14-STAGE8
- status: RELEASED
- owner_session: Codex Session
- scope:
  - `docs/**`
  - `infra/tests/**`
  - `.github/workflows/infra-tests.yml`
  - `CHANGELOG.md`
- reason: Stage 8 program closure and archival hygiene implementation
- created_at_utc: 2026-02-14T20:09:47Z
- released_at_utc: 2026-02-14T20:17:48Z

### LOCK-2026-02-14-STAGE9
- status: RELEASED
- owner_session: Codex Session
- scope:
  - `api/**`
  - `web/**`
  - `infra/**`
  - `docs/**`
  - `CHANGELOG.md`
- reason: Stage 9 backend-first then frontend performance optimization implementation
- created_at_utc: 2026-02-14T20:23:23Z
- released_at_utc: 2026-02-14T20:54:52Z

### LOCK-2026-02-14-STAGE9B
- status: RELEASED
- owner_session: Codex Session
- scope:
  - `api/**`
  - `web/**`
  - `infra/**`
  - `docs/**`
  - `CHANGELOG.md`
- reason: Stage 9 follow-up implementation (migration verify, benchmarks, reset workflow, frontend verification, push/review)
- created_at_utc: 2026-02-14T21:03:19Z
- released_at_utc: 2026-02-15T03:43:25Z

### LOCK-2026-02-14-STAGE9C
- status: RELEASED
- owner_session: Codex Session
- scope:
  - `.github/workflows/infra-tests.yml`
  - `docs/**`
  - `CHANGELOG.md`
- reason: Stage 9 CI follow-up to add new shell script validations to infra workflow
- created_at_utc: 2026-02-15T03:48:05Z
- released_at_utc: 2026-02-15T03:49:26Z

### LOCK-2026-02-15-STAGE10
- status: RELEASED
- owner_session: Codex Session
- scope:
  - `api/**`
  - `infra/**`
  - `docs/**`
  - `CHANGELOG.md`
- reason: Stage 10 backend task-list tail-latency optimization and benchmark rerun
- created_at_utc: 2026-02-15T04:01:19Z
- released_at_utc: 2026-02-15T04:09:48Z

### LOCK-2026-02-15-STAGE11
- status: RELEASED
- owner_session: Codex Session
- scope:
  - `web/**`
  - `docs/**`
  - `CHANGELOG.md`
- reason: Stage 11 frontend task-dashboard polling and render performance optimization
- created_at_utc: 2026-02-15T04:10:50Z
- released_at_utc: 2026-02-15T04:12:48Z

### LOCK-2026-02-15-STAGE12
- status: RELEASED
- owner_session: Codex Session
- scope:
  - `api/**`
  - `web/**`
  - `infra/**`
  - `.github/workflows/infra-tests.yml`
  - `docs/**`
  - `CHANGELOG.md`
- reason: Stage 12 comprehensive perf hardening (frontend tests + benchmark gates + CI/docs)
- created_at_utc: 2026-02-15T04:24:19Z
- released_at_utc: 2026-02-15T04:40:39Z

### LOCK-2026-02-17-RESTAURANT-FOUNDATIONS
- status: RELEASED
- owner_session: Codex Session
- scope:
  - `api/**`
  - `web/**`
  - `docs/**`
  - `infra/tests/**`
  - `CHANGELOG.md`
- reason: Restaurant provider contract documentation, adapter readiness, and module implementation foundations
- created_at_utc: 2026-02-17T15:14:39Z
- released_at_utc: 2026-02-17T15:26:12Z

### LOCK-2026-02-17-OPENTABLE-ADAPTER-STAGE1
- status: RELEASED
- owner_session: Codex Session
- scope:
  - `api/**`
  - `docs/**`
  - `CHANGELOG.md`
- reason: OpenTable adapter stage 1 implementation (OTP/auth/search/create/cancel contract paths)
- created_at_utc: 2026-02-17T15:28:36Z
- released_at_utc: 2026-02-17T15:34:32Z

### LOCK-2026-02-17-OPENTABLE-CONTRACT-FREEZE
- status: RELEASED
- owner_session: Codex Session
- scope:
  - `api/**`
  - `docs/**`
  - `infra/env/prod/api.env.example`
  - `CHANGELOG.md`
- reason: OpenTable search/create contract freeze (operation names and variable schema)
- created_at_utc: 2026-02-17T15:56:14Z
- released_at_utc: 2026-02-17T15:58:57Z

### LOCK-2026-02-17-OPENTABLE-SEARCH-HASH
- status: RELEASED
- owner_session: Codex Session
- scope:
  - `api/**`
  - `docs/**`
  - `infra/env/prod/api.env.example`
  - `CHANGELOG.md`
- reason: Wire observed OpenTable RestaurantsAvailability + BookDetailsStandardSlotLock contracts and update adapter/docs
- created_at_utc: 2026-02-17T16:47:45Z
- released_at_utc: 2026-02-17T17:04:41Z

## Template (Non-live Example)

### LOCK-EXAMPLE-001
- status: EXAMPLE_ACTIVE
- owner_session: Session X
- scope:
  - `path/**`
- reason: <task/stage reason>
- created_at_utc: <YYYY-MM-DDTHH:MM:SSZ>
- released_at_utc:
