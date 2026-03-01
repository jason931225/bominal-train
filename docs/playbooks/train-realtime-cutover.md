# Train Realtime Cutover

## Objective

Cut over train UI live updates to Supabase Realtime as the primary transport, keep SSE as automatic fallback during rollout, and register SSE deprecation with a standard 30-day production window.

## Scope

- In scope:
  - Frontend transport manager behavior in `web/lib/train/task-events.ts`.
  - Phased rollout via web env flags.
  - SSE deprecation registration and docs/changelog updates.
  - Verification of dashboard, task detail, and top-nav attention update paths.
- Out of scope:
  - Removing SSE route in the same change.
  - Backend task-state semantics or worker retry behavior changes.
  - DB schema/publication changes for `task_realtime_events`.

## Preconditions

- Required accounts/roles:
  - Repo write access.
  - Deployment access for web env changes.
- Required services/tools:
  - `npm` for web tests/lint/build.
  - Standard docs/deprecation policy test scripts.
- Required environment state:
  - `task_realtime_events` projection and publication already deployed.
  - SSE endpoint `/api/train/tasks/events` still available during transition.

## Inputs

### Dependency-derived inputs

- Existing realtime projection table + publication from prior migrations.
- Existing consumer contract of `subscribeTrainTaskEvents(listener)`.
- Existing train live-update component tests.

### Non-dependency inputs

- Rollout flags in `infra/env/*/web.env`.
- Rollout phase target (`10`, `50`, `100` percent).
- Deprecation dates:
  - `deprecated_on=2026-03-01`
  - `remove_after=2026-03-31`

## Deterministic Procedure

1. Set transport flags in web env:
   - `NEXT_PUBLIC_TRAIN_EVENTS_REALTIME_ENABLED`
   - `NEXT_PUBLIC_TRAIN_EVENTS_REALTIME_CANARY_PERCENT`
   - `NEXT_PUBLIC_TRAIN_EVENTS_REALTIME_RETRY_SECONDS`
2. Implement/verify transport manager behavior:
   - realtime primary on eligible sessions.
   - automatic fallback to SSE on realtime subscribe/token/config/channel failures.
   - periodic realtime retry while on SSE fallback.
3. Keep consumer API unchanged:
   - `subscribeTrainTaskEvents(listener)` remains the only consumer interface.
4. Register deprecation in `docs/deprecations/registry.json`:
   - add production deprecation entry for SSE endpoint string `/api/train/tasks/events`.
5. Update docs and pointers:
   - architecture + deployment docs for realtime-primary + fallback behavior.
   - add this playbook to pointer indexes.
6. Run verification commands:
   - `npm --prefix web run test -- --run lib/train/task-events.test.ts components/train/__tests__/train-dashboard.polling.test.tsx components/train/__tests__/train-task-detail.test.tsx components/__tests__/top-nav-task-attention.test.tsx`
   - `npm --prefix web run lint`
   - `npm --prefix web run build`
   - `bash infra/tests/test_deprecation_policy.sh`
   - `bash infra/tests/test_deprecation_references.sh`
   - `bash infra/tests/test_docs_pointers.sh`
   - `bash infra/tests/test_docs_consistency.sh`
   - `bash infra/tests/test_intent_routing.sh`
   - `bash infra/tests/test_changelog.sh`

## Verification Checkpoints

- Checkpoint A:
  - Expected signal: eligible users connect via Supabase Realtime and receive task-state/ticket-status updates.
  - Failure signal: no realtime subscription or stale UI updates.
- Checkpoint B:
  - Expected signal: realtime failures activate SSE automatically and recover back to realtime after retry.
  - Failure signal: update stream stalls or manual reload required.
- Checkpoint C:
  - Expected signal: docs/deprecation policy tests pass with new registry entry and pointers.
  - Failure signal: deprecation guard/pointer/consistency checks fail.

## Failure Modes and Recovery

- Failure mode: realtime channel errors for canary users.
  - Detection: missing updates without active SSE request in browser.
  - Recovery: set `NEXT_PUBLIC_TRAIN_EVENTS_REALTIME_ENABLED=false` or `NEXT_PUBLIC_TRAIN_EVENTS_REALTIME_CANARY_PERCENT=0`, redeploy web.
- Failure mode: SSE fallback not activating.
  - Detection: no `/api/train/tasks/events` request and no updates after realtime failure.
  - Recovery: rollback to previous web release and re-run transport tests.
- Failure mode: deprecation policy gate failures.
  - Detection: `test_deprecation_policy.sh` or `test_deprecation_references.sh` failure.
  - Recovery: fix registry fields/status/dates or update caller scan paths, then re-run checks.

## Security and Redaction

- Never persist:
  - Supabase access tokens.
  - Provider secrets or payment data in event payloads/logs.
- Redaction requirements:
  - Keep task-event payloads limited to safe task metadata and status deltas.
- Safe artifacts:
  - Test logs, docs updates, and deprecation registry metadata.

## Artifacts and Pointers

- `web/lib/train/task-events.ts`
- `web/lib/train/task-events.test.ts`
- `infra/env/dev/web.env`
- `infra/env/prod/web.env.example`
- `docs/deprecations/registry.json`
- `docs/humans/engineering/ARCHITECTURE.md`
- `docs/humans/operations/DEPLOYMENT.md`

## Change History

- [0000000] Added initial train realtime cutover playbook with phased rollout, fallback behavior, and deprecation gate procedure.
