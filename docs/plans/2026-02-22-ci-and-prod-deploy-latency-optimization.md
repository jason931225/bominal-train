# CI and Production Deploy Latency Optimization Plan

## Goal

Reduce end-to-end latency for web CI and production deployment by removing unnecessary work, tightening Docker cache behavior, and adding phase-level deploy timing evidence.

## Evidence Snapshot (2026-02-22)

### Web CI build path

1. Web image build runs even when no `web/**` files changed.
- `build-web` currently runs on workflow triggers that include `api/**`.
- Example observed: commit `4a086c5...` (API-only change) still executed web image build.

2. Largest single web-build sink is GHA cache export.
- In run `22275299849` (`Build Web Image` job `64436344666`):
- `#20 preparing build cache for export` took about `83.4s`.
- This is tied to `cache-to: type=gha,mode=max`.

3. `web/Dockerfile.prod` performs two dependency installs.
- `npm ci` in deps stage: about `58.5s`.
- `npm ci --omit=dev` in runner stage: about `12.6s`.
- Combined install cost: about `71s`.

4. Next production build is material but not the top sink.
- `npm run build` measured about `33.8s`.

5. Checkout depth overhead exists but is secondary.
- `actions/checkout@v4` with `fetch-depth: 0` is not the dominant delay.

### Production deployment path

Likely deployment slowdowns are concentrated in:
- registry image pulls (`docker compose pull`),
- sequential service replacement (`docker compose up -d --wait ...`),
- conservative healthcheck windows (`start_period`, `interval`, `retries`),
- gateway startup pre-check + migration (`check_duplicate_display_names.py` and `alembic upgrade head`),
- post-deploy smoke probe retry loop.

## Scope

In scope:
- CI path gating and build-cache tuning for web.
- Deploy script instrumentation to capture precise per-phase timing.
- Safe, low-risk Dockerfile optimization options for web image build.
- Root-cause evidence collection for live production deploy latency.

Out of scope (this plan):
- Direct merge of `feat-stage10-tasklist-tail-latency` (branch is not merge-safe as-is).
- Full extraction/implementation of Stage10 foundation items (key rotation, egress proxy hardening, internal identity rollout).

## Stage10 Exploration Coverage

This plan captures earlier Stage10 findings for traceability:

1. `api/app/core/crypto` key-rotation artifacts
- Provides a real `MASTER_KEY` rotation path with encrypted-secret rewrap by `kek_version`.
- Security-value foundation; requires controlled rollout and verification.

2. `infra/egress/train/Caddyfile` and `infra/egress/restaurant/Caddyfile`
- Controlled outbound proxy boundary for provider traffic.
- Useful for policy/compliance hardening but requires transport integration review.

3. `api/app/core/internal_identity.py`
- Short-lived internal service token primitives for gateway-to-internal trust.
- Foundation component; not fully wired in current main flow.

## Implementation Workstreams

### Workstream 1: Prevent unnecessary web builds

Files:
- `.github/workflows/build-push.yml`

Changes:
- Add change-detection gate (`paths-filter` or equivalent) and skip `build-web` when `web/**` and web build inputs are unchanged.
- Keep API image path unaffected for API-only commits.

Acceptance:
- API-only commit does not execute web image build/push job.
- Web-touching commit still executes web build/push job.

### Workstream 2: Reduce Docker cache export overhead

Files:
- `.github/workflows/build-push.yml`

Changes:
- Change web build cache policy from `cache-to: type=gha,mode=max` to lower-overhead mode (`mode=min`) or disable export where net-negative.
- Keep `cache-from` enabled.

Acceptance:
- Cache export phase time drops materially.
- Total web image job wall-clock decreases.

### Workstream 3: Remove duplicate dependency installation cost

Files:
- `web/Dockerfile.prod`

Changes (safe first):
- Preserve lockfile correctness.
- Prefer single install strategy for production runtime, avoiding duplicated `npm ci` where possible.
- Validate runtime behavior and image size tradeoff before finalizing.

Acceptance:
- Build time for dependency installation reduces versus current baseline.
- Built image still boots and serves production bundle correctly.

### Workstream 4: Optional Next.js build-path optimization

Files:
- `web/Dockerfile.prod`
- `web/next.config.*` (if needed)

Changes:
- Evaluate `output: "standalone"` and optimized copy strategy for runtime image.
- Keep this optional if gains are marginal compared with complexity.

Acceptance:
- Either measured improvement is captured and adopted, or explicit no-go rationale is documented.

### Workstream 5: Instrument deploy script and profile production rollout

Files:
- `infra/scripts/deploy.sh`
- `docs/DEPLOYMENT.md` (timing interpretation notes)

Changes:
- Add timestamped phase timing logs for:
- preflight,
- pull,
- rolling service updates,
- migration,
- health probe loop,
- smoke checks.
- Emit deterministic summary line with per-phase durations.

Acceptance:
- A production deploy run yields phase-by-phase timing numbers.
- Slowest phase is empirically identified from logs.

## Safety Constraints

- No destructive branch cleanup without merged-status and divergence verification.
- No production `docker compose down`.
- No direct edits to `/opt/bominal/deployments/*` tracking files.
- Preserve current rollback path and health-check gates.

## Verification Plan

### CI verification

1. API-only change simulation:
- Confirm web build job is skipped.

2. Web-only change simulation:
- Confirm web build job runs.

3. Compare baseline vs optimized web build:
- record total runtime,
- record cache export step duration,
- record dependency install step duration.

### Deployment verification

1. Execute deploy script on target host with timing instrumentation enabled.
2. Collect phase summary from deploy logs.
3. Confirm no change in health/rollback behavior.
4. Document actual top bottleneck phase and next action.

## Deliverables

1. Updated workflow with web-change gating and tuned cache policy.
2. Optimized `web/Dockerfile.prod` (if verified safe and beneficial).
3. Timed deploy script logs and bottleneck report.
4. Updated documentation pointers and deployment notes.

## Success Criteria

- Web CI avoids web builds on API-only commits.
- Web CI median runtime improves with measured evidence.
- Production deploy latency is decomposed into measured phases (not estimates).
- Follow-up optimization decisions are backed by timing data and risk assessment.
