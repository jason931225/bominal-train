# Free-Tier Budget Policy

## Purpose

Protect reliability and cost safety by enforcing explicit hosted-usage thresholds for Supabase and GCP free-tier services.

## Scope

- Supabase project usage (database/storage/functions/egress)
- Google Cloud usage for Always Free services used by bominal
- Google Secret Manager active secret-version counts
- Cloud Scheduler job-count budget for operations automation

## Thresholds

- Warning threshold: `70%` of observed free-tier budget
- Hard alert threshold: `85%` of observed free-tier budget
- Breach response: reduce non-essential workloads, disable optional offload flags, and file incident follow-up

## Free-Tier Operating Limits

- Cloud Scheduler planned jobs must remain `<= 3` for current operations profile.
- Secret Manager active versions should remain `<= 6` per secret family unless a temporary incident rotation is active.

## Evidence and Cadence

Weekly evidence capture is required:

1. Supabase usage dashboard snapshot with check date.
2. GCP billing usage snapshot with check date.
3. GSM active-version summary by secret.
4. Cloud Scheduler job list with count.

## Required Tooling

- `infra/scripts/free_tier_status_report.sh` for weekly Markdown evidence output.
- Runbook procedures must link to live pricing/usage sources and include the exact check date.

## Guardrails

- Do not hardcode stale quota numbers in human-facing runbooks; cite source URL + date checked.
- Offload changes that increase baseline VM or hosted spend must include a net-gain rationale before rollout.
- If free-tier usage exceeds hard alert threshold, defer non-critical migrations/offloads until stabilized.
