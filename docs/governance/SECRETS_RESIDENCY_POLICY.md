# Secrets Residency Policy

## Purpose

Define authoritative secret residency for bominal so deploy/runtime behavior is deterministic, auditable, and aligned with PCI/CDE boundary minimization.

## Authority

- Google Secret Manager (GSM) is the authoritative source for high-trust runtime secrets.
- Supabase Edge secrets are a scoped runtime copy for Edge Functions only.
- Production `infra/env/prod/api.env` must not contain plaintext values for GSM-authoritative keys.

## Residency Matrix

### GSM authoritative secrets

- `MASTER_KEY` via `GSM_MASTER_KEY_*`
- `EVERVAULT_APP_ID` via `EVERVAULT_APP_ID_SECRET_ID/_VERSION`
- `EVERVAULT_API_KEY` via `EVERVAULT_API_KEY_SECRET_ID/_VERSION`
- `INTERNAL_API_KEY` via `INTERNAL_API_KEY_SECRET_ID/_VERSION`
- `RESEND_API_KEY` via `RESEND_API_KEY_SECRET_ID/_VERSION`
- `SUPABASE_MANAGEMENT_API_TOKEN` via `SUPABASE_MANAGEMENT_API_TOKEN_SECRET_ID/_VERSION`

### Supabase Edge runtime secrets

- `RESEND_API_KEY` (synced copy from GSM for `task-notify`)
- `EMAIL_FROM_ADDRESS`
- `EMAIL_FROM_NAME` (optional)

### Bootstrap/env-only secrets

These remain in `infra/env/prod/api.env` because they bootstrap runtime access and cannot depend on later secret resolution:

- `SUPABASE_SERVICE_ROLE_KEY`
- `SUPABASE_AUTH_API_KEY` (or approved fallback behavior)
- `DATABASE_URL`
- `SYNC_DATABASE_URL`

## Production Contracts

1. Secret source ambiguity is forbidden.
- `INTERNAL_API_KEY` and `INTERNAL_API_KEY_SECRET_ID` cannot both be set.
- `RESEND_API_KEY`, `RESEND_API_KEY_SECRET_ID`, and `RESEND_API_KEY_VAULT_NAME` cannot be mixed.

2. GSM secret versions must be pinned.
- `latest` is not allowed for production secret references.

3. Edge notify contract.
- If `EDGE_TASK_NOTIFY_ENABLED=true`, Edge secret material for `RESEND_API_KEY` and `EMAIL_FROM_ADDRESS` must be present before rollout.

4. Resend source contract.
- For `EMAIL_PROVIDER=resend`, prefer GSM-backed `RESEND_API_KEY_SECRET_ID`.
- `RESEND_API_KEY_VAULT_NAME` is allowed only when `EDGE_TASK_NOTIFY_ENABLED=true` and `SUPABASE_VAULT_ENABLED=true`.

## Rotation and Version Limits (Free-Tier Aware)

- Target cap: active GSM secret versions per secret family `<= 6`.
- Rotate one secret family at a time to prevent temporary version spikes.
- Default cadence:
  - `MASTER_KEY`: quarterly
  - `EVERVAULT_*`, `INTERNAL_API_KEY`, `RESEND_API_KEY`, `SUPABASE_MANAGEMENT_API_TOKEN`: semiannual
- Incident response rotations may override cadence but must still prune old versions after verification.

## Operational Sequence

1. Rotate/update GSM secret value and pin target version in `infra/env/prod/api.env`.
2. Run `infra/scripts/predeploy-check.sh` and policy/runtime parity tests.
3. Sync Edge delivery secrets from GSM using `infra/scripts/sync-edge-secrets-from-gsm.sh`.
4. Deploy with `infra/scripts/deploy.sh`.
5. Capture evidence in runbook weekly reports.

## Enforcement

- `infra/tests/test_policy_runtime_parity.sh`
- `infra/tests/test_secret_residency_contract.sh`
- `infra/scripts/predeploy-check.sh`

All are blocking gates for production deployment and CI quality workflows.
