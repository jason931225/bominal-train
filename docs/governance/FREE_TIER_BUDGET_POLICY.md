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

## Canonical Cloud Scheduler Baseline (Production)

Canonical production schedule set (project `bominal`, location `us-central1`, timezone `Etc/UTC`):

| Job ID | Schedule (cron) | Target type | Target |
|---|---|---|---|
| `bominal-daily-drift-check` | `0 9 * * *` | Pub/Sub | `projects/bominal/topics/bominal-ops-triggers` (`{"action":"drift_check","source":"cloud_scheduler"}`) |
| `bominal-daily-budget-snapshot` | `30 9 * * *` | Pub/Sub | `projects/bominal/topics/bominal-ops-triggers` (`{"action":"budget_snapshot","source":"cloud_scheduler"}`) |
| `bominal-hourly-smoke` | `0 * * * *` | HTTP GET | `https://www.bominal.com/health` |

Policy requirements:

1. This set is the only approved baseline under current free-tier constraints.
2. Any added or replacement Scheduler job requires change-management review and runbook update in the same change.
3. Weekly evidence must include job list output proving the baseline remains enabled and drift-free.

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
