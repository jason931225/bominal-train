# AGENTS.md

Guidance for AI/code agents working in this repository.

## Mission

Build and operate **bominal**, a modular product platform with:

- `web/` (Next.js App Router + TypeScript + Tailwind)
- `api/` (FastAPI + Postgres + Alembic)
- `worker` (arq background jobs)
- `redis` (queue + train-provider rate limiter)
- `third_party/srtgo` (read-only train provider reference)

## Non-negotiables

1. Preserve product name as `bominal` in UI, config, and docs.
2. Treat `third_party/srtgo` as read-only. Never patch or reformat it.
3. Keep provider integrations source-aligned with `srtgo/srt.py` and `srtgo/ktx.py`.
4. Never log sensitive data (passwords, tokens, card data, CVV, full provider payloads with secrets).
5. Keep session auth cookie behavior:
   - httpOnly
   - SameSite=Lax
   - Secure only in production
6. For multi-session execution, use dynamic lock protocol:
   - `docs/LOCK.md` for active scopes
   - `docs/REQUEST.md` for cross-scope edits
   - follow `docs/EXECUTION_PROTOCOL.md`
7. Maintain a commit-based `CHANGELOG.md` using Keep a Changelog structure:
   - every behavior/ops/doc-visible change must be recorded in `## Unreleased`
   - each changelog line must include commit SHA reference (short SHA is acceptable)
   - do not merge without changelog update unless explicitly approved
8. Follow `docs/DOCUMENTATION_WORKFLOW.md` for standardized documentation operations.
9. For complex/repeatable tasks, use or create a playbook under `docs/playbooks/`.
10. Follow `docs/PERMISSIONS.md` for approval boundaries and scope-control requirements.
11. Follow `docs/GUARDRAILS.md` as hard constraints; guardrails override permissions.
12. Use `docs/INTENT_ROUTING.md` before broad searches to reduce token usage.
13. Treat `third_party/**` docs as reference only, never canonical policy.
14. Follow `docs/DEPRECATION_WORKFLOW.md` and `docs/deprecations/registry.json` for deprecation/removal changes.

## First files to read

1. `docs/README.md`
2. `docs/EXECUTION_PROTOCOL.md`
3. `docs/ARCHITECTURE.md`
4. `docs/CONTRIBUTING.md`
5. `docs/DEPLOYMENT.md` (for production deploys and rollbacks)
6. `docs/RUNBOOK.md`
7. `README.md`
8. `CHANGELOG.md`
9. `docs/DOCUMENTATION_WORKFLOW.md`
10. `docs/PERMISSIONS.md`
11. `docs/GUARDRAILS.md`
12. `docs/INTENT_ROUTING.md`
13. `docs/DEPRECATION_WORKFLOW.md`

## Docs-First Gate (Mandatory)

Before any planning, coding, or deploy action:
1. Read all files listed in **First files to read**.
2. Confirm current task requirements against `docs/EXECUTION_PROTOCOL.md`.
3. If any instruction conflicts, follow the stricter rule and document the decision in task notes.

Do not start implementation before this gate is completed.

## Docs-Last Gate (Mandatory)

Before claiming completion, opening PR, or merging:
1. Re-read relevant docs touched by the task.
2. Update docs to match implemented behavior.
3. Verify doc consistency with `docs/EXECUTION_PROTOCOL.md` and active plan requirements.
4. Update `CHANGELOG.md` with commit-based entries for all notable changes in scope.

Do not mark work complete until this gate is completed.

## Docs Exception Policy

For doc-related tasks, follow AGENTS/doc protocol strictly.
If any required behavior is ambiguous, conflicting, or needs an exception:
1. Stop immediately.
2. Ask for clarification/permission.
3. Resume only after explicit direction.

## Pointer Convention (Mandatory)

Use `docs/README.md` as the canonical pointer library.
Any new canonical doc/plan used for implementation must be added there using the required pointer format before completion.

## Repo map

- `web/app/` page routes
- `web/components/` UI and feature components
- `web/lib/` shared client/server helpers and design tokens
- `api/app/http/routes/` HTTP routes
- `api/app/modules/train/` train domain (router, service, worker, providers)
- `api/app/core/crypto/` envelope encryption and redaction
- `api/app/db/` SQLAlchemy models/session
- `api/alembic/versions/` DB migrations
- `infra/` compose files, env files, scripts

## Local workflow

```bash
git submodule update --init --recursive
docker compose -f infra/docker compose.yml up --build
```

Use these for focused verification:

```bash
docker compose -f infra/docker compose.yml exec api pytest -q
docker compose -f infra/docker compose.yml exec web npx tsc --noEmit
```

## Change strategy

1. Keep API handlers thin; heavy train logic belongs in service/worker.
2. Prefer additive migrations; do not mutate old migrations.
3. Reuse `web/lib/ui.ts` classes for consistent UI.
4. Keep train times in KST in user-facing UI.
5. Use explicit safe metadata fields (`*_safe`) when storing provider response details.

## Definition of done for feature work

- Build compiles (web typecheck, backend imports cleanly).
- Relevant backend tests pass.
- Docker compose stack starts cleanly.
- No broken auth/session flow regressions.
- No unresolved placeholders in production env templates.
- Docs updated in `docs/` when behavior or operations change.

## Production Deployment (IMPORTANT)

Use the current canonical script flow for production:

```bash
# Deploy using the current production deploy script
sudo -u bominal /opt/bominal/repo/infra/scripts/deploy.sh
```

Remote example:

```bash
gcloud compute ssh bominal-deploy --zone=us-central1-a --tunnel-through-iap \
  --command="cd /opt/bominal/repo && sudo -u bominal infra/scripts/deploy.sh"
```

Requirements:
- `deploy.sh` must handle running-container detection.
- `deploy.sh` must run resource/swap preflight checks.
- Health checks and rollback path must be enforced by script.

**Never**:
- Use `docker compose down` in production (causes downtime)
- Modify `/opt/bominal/deployments/*` version tracking files
- Skip health check verification after deploy

See `docs/DEPLOYMENT.md` for full procedures.
