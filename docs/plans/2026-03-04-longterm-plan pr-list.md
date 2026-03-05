Here is the **PR-by-PR refactor backlog and merge order**.

I’m preserving four verified contracts while the code moves around: `third_party/**` stays read-only, the API keeps serving `/assets` from `FRONTEND_ASSETS_DIR`, the final container still serves assets from `/app/frontend/dist`, and the UI standardizes on the **existing Leptos SSR stack** already present in the workspace instead of adding a second frontend framework. The biggest hotspots today are `web.rs` (~2.4k lines), `executor.rs` (~2.0k), `admin_service.rs` (~1.1k), `http/admin.rs` (~1.0k), `internal_auth.rs` (~696), `http/auth.rs` (~568), and `provider_jobs_service.rs` (~534). There is already a usable API test base for admin, auth, dashboard, health, and internal API auth contracts, so those should be the guardrails for every refactor PR.

Before code PRs, do the repo-setting work first: protect `main`, require PR review, require CI before merge, and put production deploy behind environment approval. That matters because CD currently triggers directly on pushes to `main`, and it still prebuilds frontend CSS before the Docker image build.

## Non-PR setup: lock the repo before refactoring

* Protect `main`.
* Require CI checks for merge.
* Require at least 1 review.
* Freeze direct edits to these files while their split PRs are open: `web.rs`, `http/auth.rs`, `http/admin.rs`, `internal_auth.rs`, `admin_service.rs`, `provider_jobs_service.rs`, `worker/src/runtime/executor.rs`.
* Keep `third_party/**` untouched throughout the program.

## PR-01 — Frontend artifact policy + CI repair

**Why:** `runtime/frontend/dist` is still tracked in the repo, and CI currently runs `check:css:budget` even though `runtime/frontend/package.json` does not define that script. The tracked generated files currently include `tailwind.css` and `lightweight-charts.standalone.production.js`.

**Touch**

* `.gitignore`
* delete tracked `runtime/frontend/dist/**`
* `runtime/frontend/package.json`
* `runtime/frontend/scripts/check-css-budget.mjs`
* `.github/workflows/ci.yml`

**Do**

* Add `runtime/frontend/dist/` to `.gitignore`.
* Delete all tracked generated artifacts under `runtime/frontend/dist/**`.
* Add a real `check:css:budget` script.
* Add a CI step that fails if any file under `runtime/frontend/dist/**` is tracked by git.
* Keep the runtime asset serving contract unchanged.

**Done when**

* No tracked files remain under `runtime/frontend/dist/**`.
* `npm --prefix runtime/frontend run build:css` and `npm --prefix runtime/frontend run check:css:budget` both pass in CI.
* CI fails if someone reintroduces tracked generated artifacts.

## PR-02 — Make Docker the source of truth for frontend assets

**Why:** `runtime/.dockerignore` already excludes `frontend/dist/`, which is correct, but `runtime/Dockerfile.api` still copies `frontend/dist` into the final image without building it internally, and CD still prebuilds CSS before the Docker image build. That is the brittle hybrid contract you want to eliminate.

**Touch**

* `runtime/Dockerfile.api`
* `.github/workflows/cd.yml`
* `.github/workflows/ci.yml`
* `runtime/frontend/package.json`
* optional: `runtime/frontend/scripts/copy-static.mjs`
* optional: `runtime/frontend/static/`

**Do**

* Convert `runtime/Dockerfile.api` to a multi-stage build:

  * `frontend-builder`
  * `rust-builder`
  * final runtime image
* In `frontend-builder`, run the frontend build inside Docker.
* In the final image, still copy assets to `/app/frontend/dist`.
* Remove the CD assumption that repo-committed assets exist before image build.
* Add a Docker smoke build in CI.

**Done when**

* A clean checkout with no tracked `dist` still produces a working API image.
* Final image still serves assets from `/app/frontend/dist`.
* CD no longer depends on repo-committed frontend artifacts.

## PR-03 — Create the internal UI crate and SSR conventions

**Why:** the workspace already has Leptos dependencies, and the API crate is already SSR-capable. This is the right time to turn the `ui-patterns` and `ui-primitives` you mentioned into the official internal UI SDK.

**Touch**

* `runtime/Cargo.toml`
* `runtime/crates/ui/Cargo.toml`
* `runtime/crates/ui/src/lib.rs`
* `runtime/crates/ui/src/primitives/*`
* `runtime/crates/ui/src/patterns/*`
* `runtime/crates/ui/src/theme.rs`
* `runtime/crates/api/Cargo.toml`
* `docs/adr/0001-ssr-component-model.md`
* `docs/adr/0002-ui-sdk-boundary.md`

**Do**

* Add `crates/ui` to the workspace.
* Move or recreate `ui-primitives` and `ui-patterns` inside that crate.
* Establish a rule: new SSR UI lands in `crates/ui` plus tiny route wrappers only.

**Done when**

* `bominal-ui` compiles and is imported by the API crate.
* The crate exposes at least:

  * page shell
  * admin shell
  * dashboard shell
  * form field
  * table
  * card
  * status badge
  * passkey dialog
* No visible route behavior changes yet.

## PR-04 — Split `web.rs` without changing routes

**Why:** `web.rs` is the single biggest API-side UI hotspot, and it currently contains the user dashboard/security markup plus large inline scripts. I did not find admin route ownership in that file, so the first split should focus on user-facing pages only.

**Touch**

* rename `runtime/crates/api/src/web.rs` → `runtime/crates/api/src/web/mod.rs`
* `runtime/crates/api/src/web/layout.rs`
* `runtime/crates/api/src/web/home.rs`
* `runtime/crates/api/src/web/auth.rs`
* `runtime/crates/api/src/web/dashboard/overview.rs`
* `runtime/crates/api/src/web/dashboard/jobs.rs`
* `runtime/crates/api/src/web/dashboard/job_detail.rs`
* `runtime/crates/api/src/web/dashboard/security.rs`

**Do**

* Move render functions only.
* Keep symbol re-exports in `web/mod.rs` so route handlers do not all have to change at once.
* Preserve route paths and current behavior.

**Done when**

* `web.rs` monolith is gone.
* No new UI file exceeds ~600 LOC.
* Existing route tests still pass.

## PR-05 — Split `http/auth.rs` and connect auth pages to components

**Why:** `http/auth.rs` is already large enough to deserve its own module tree, and auth pages already have dedicated tests.

**Touch**

* rename `runtime/crates/api/src/http/auth.rs` → `runtime/crates/api/src/http/auth/mod.rs`
* `runtime/crates/api/src/http/auth/pages.rs`
* `runtime/crates/api/src/http/auth/passkeys.rs`
* `runtime/crates/api/src/http/auth/sessions.rs`
* `runtime/crates/api/src/http/auth/callbacks.rs`
* `runtime/crates/api/src/http/mod.rs`
* `runtime/crates/api/src/web/home.rs`
* `runtime/crates/api/src/web/auth.rs`
* `runtime/crates/ui/src/patterns/auth/*`

**Do**

* Separate page routes from passkey/session/callback handling.
* Render auth/home pages through the new UI crate rather than route-local HTML blobs.

**Done when**

* `auth_page_test.rs` stays green.
* Auth pages render through `crates/ui`.
* `http/auth` becomes a thin route tree instead of one large file.

## PR-06 — Componentize dashboard, jobs, and security pages

**Why:** the current route map includes `/dashboard`, `/dashboard/jobs`, `/dashboard/jobs/{job_id}`, and `/dashboard/security`, and those views are currently concentrated in the old `web.rs` area.

**Touch**

* `runtime/crates/ui/src/patterns/dashboard_shell.rs`
* `runtime/crates/ui/src/patterns/jobs_table.rs`
* `runtime/crates/ui/src/patterns/job_detail.rs`
* `runtime/crates/ui/src/patterns/security_panel.rs`
* `runtime/crates/api/src/web/dashboard/*.rs`
* optional: `runtime/frontend/static/dashboard.js`
* optional: `runtime/frontend/static/passkey.js`

**Do**

* Move dashboard shell, jobs table, job detail, and security/passkey UI into components.
* If any client-side JS must remain, move it to tiny static assets rather than long inline `<script>` blocks.

**Done when**

* `dashboard_routes_test.rs` stays green.
* No large inline script blocks remain in SSR page code.
* Dashboard rendering is assembled from reusable components, not page-local strings.

## PR-07 — Split `admin_service.rs` by domain

**Why:** `admin_service.rs` is a major maintainability hotspot.

**Touch**

* rename `runtime/crates/api/src/services/admin_service.rs` → `runtime/crates/api/src/services/admin/mod.rs`
* `runtime/crates/api/src/services/admin/capabilities.rs`
* `runtime/crates/api/src/services/admin/incidents.rs`
* `runtime/crates/api/src/services/admin/maintenance.rs`
* `runtime/crates/api/src/services/admin/users.rs`
* `runtime/crates/api/src/services/admin/runtime.rs`
* `runtime/crates/api/src/services/admin/observability.rs`
* `runtime/crates/api/src/services/admin/security.rs`
* `runtime/crates/api/src/services/admin/config.rs`
* `runtime/crates/api/src/services/admin/audit.rs`
* optional: `runtime/crates/api/src/services/admin/types.rs`
* `runtime/crates/api/src/services/mod.rs`

**Do**

* Split service logic by domain.
* Move DTO mapping and service-specific helpers into the relevant submodule.

**Done when**

* No admin service file exceeds ~400–500 LOC.
* Admin business logic is grouped by domain instead of one file.
* Existing admin tests remain green.

## PR-08 — Split `http/admin.rs` into pages and API handlers

**Why:** `http/admin.rs` is too large, and the repo already documents both admin UI pages and admin API surfaces.

**Touch**

* rename `runtime/crates/api/src/http/admin.rs` → `runtime/crates/api/src/http/admin/mod.rs`
* `runtime/crates/api/src/http/admin/pages/mod.rs`
* `runtime/crates/api/src/http/admin/pages/maintenance.rs`
* `runtime/crates/api/src/http/admin/pages/users.rs`
* `runtime/crates/api/src/http/admin/pages/runtime.rs`
* `runtime/crates/api/src/http/admin/pages/observability.rs`
* `runtime/crates/api/src/http/admin/pages/security.rs`
* `runtime/crates/api/src/http/admin/pages/config.rs`
* `runtime/crates/api/src/http/admin/pages/audit.rs`
* `runtime/crates/api/src/http/admin/api/mod.rs`
* `runtime/crates/api/src/http/admin/api/capabilities.rs`
* `runtime/crates/api/src/http/admin/api/incidents.rs`
* `runtime/crates/api/src/http/admin/api/observability.rs`
* `runtime/crates/api/src/http/admin/api/runtime.rs`
* `runtime/crates/api/src/http/mod.rs`

**Do**

* Keep handlers thin: parse request, call service, map response.
* Separate admin page routes from admin API routes.

**Done when**

* `admin_routes_test.rs`, `admin_observability_test.rs`, and `admin_audit_log_test.rs` all stay green.
* Route paths stay unchanged.
* Business logic lives in services, not handlers.

## PR-09 — Split internal auth and fold invites into the same module tree

**Why:** `internal_auth.rs` is another hotspot, and there is already a separate invites file plus a contract test for internal API auth.

**Touch**

* rename `runtime/crates/api/src/http/internal_auth.rs` → `runtime/crates/api/src/http/internal_auth/mod.rs`
* fold `runtime/crates/api/src/http/internal_auth_invites.rs` into:

  * `runtime/crates/api/src/http/internal_auth/guards.rs`
  * `runtime/crates/api/src/http/internal_auth/middleware.rs`
  * `runtime/crates/api/src/http/internal_auth/invites.rs`
  * `runtime/crates/api/src/http/internal_auth/service_identity.rs`
  * `runtime/crates/api/src/http/internal_auth/handlers.rs`
* `runtime/crates/api/src/http/mod.rs`

**Do**

* Co-locate internal auth guards, middleware, invite handling, and handler entrypoints.
* Make service identity and invite logic explicit.

**Done when**

* `internal_api_auth_contract_test.rs` stays green.
* `internal_auth.rs` and `internal_auth_invites.rs` are gone as top-level files.
* Internal auth responsibilities are separated cleanly.

## PR-10 — Normalize `shared/providers` into a real internal provider SDK

**Why:** the current generic provider abstraction leaks `srt::*` types through the shared trait surface. Since provider code is already split into `srt` and `ktx`, that is the natural seam for canonical contracts. Also, `third_party/**` is explicitly read-only, so the SDK boundary has to live in your own shared crate.

**Touch**

* `runtime/crates/shared/src/providers/mod.rs`
* `runtime/crates/shared/src/providers/contract.rs`
* `runtime/crates/shared/src/providers/model.rs`
* `runtime/crates/shared/src/providers/error.rs`
* `runtime/crates/shared/src/providers/capabilities.rs`
* `runtime/crates/shared/src/providers/redaction.rs`
* `runtime/crates/shared/src/providers/retry.rs`
* adapt `runtime/crates/shared/src/providers/srt/*`
* adapt `runtime/crates/shared/src/providers/ktx/*`

**Do**

* Create canonical provider request/response DTOs.
* Create a canonical `ProviderError`.
* Keep provider-specific wire models inside the SRT/KTX adapters only.

**Done when**

* API and worker public signatures no longer depend on `shared::providers::srt::*`.
* Provider-specific types do not leak past adapter boundaries.
* Retryability and redaction rules are centralized.

## PR-11 — Split `provider_jobs_service.rs` after provider normalization

**Why:** `provider_jobs_service.rs` is already a moderate hotspot, and it should get smaller once provider DTO mapping moves into the provider SDK.

**Touch**

* rename `runtime/crates/api/src/services/provider_jobs_service.rs` → `runtime/crates/api/src/services/provider_jobs/mod.rs`
* `runtime/crates/api/src/services/provider_jobs/commands.rs`
* `runtime/crates/api/src/services/provider_jobs/queries.rs`
* `runtime/crates/api/src/services/provider_jobs/mapping.rs`
* `runtime/crates/api/src/services/provider_jobs/state.rs`
* `runtime/crates/api/src/services/mod.rs`

**Do**

* Separate command, query, state, and mapping responsibilities.
* Consume canonical provider SDK types only.

**Done when**

* `provider_jobs_service.rs` monolith is gone.
* No provider-specific wire DTO mapping remains in API service code.

## PR-12 — Split worker `executor.rs` into testable runtime modules

**Why:** the worker runtime already has `dlq`, `lease`, and `retry` modules, but `executor.rs` is still the center of gravity and is nearly 2,000 lines long.

**Touch**

* rename `runtime/crates/worker/src/runtime/executor.rs` → `runtime/crates/worker/src/runtime/executor/mod.rs`
* `runtime/crates/worker/src/runtime/executor/planner.rs`
* `runtime/crates/worker/src/runtime/executor/dispatcher.rs`
* `runtime/crates/worker/src/runtime/executor/state_machine.rs`
* `runtime/crates/worker/src/runtime/executor/provider_runner.rs`
* `runtime/crates/worker/src/runtime/executor/result_mapper.rs`
* `runtime/crates/worker/src/runtime/executor/retry_policy.rs`
* `runtime/crates/worker/src/runtime/executor/rate_limit.rs`
* `runtime/crates/worker/src/runtime/executor/metrics.rs`
* `runtime/crates/worker/src/runtime/mod.rs`

**Do**

* Make state transitions, retry classification, and provider cadence independently testable.
* Put the per-task request cadence rule in `rate_limit.rs` so the 1 req/sec/task policy is explicit and configurable.

**Done when**

* The executor monolith is gone.
* Unit tests cover state transitions, retry decisions, and rate limiting.
* No runtime file exceeds ~500 LOC.

## PR-13 — Add versioned internal API schema + generated internal SDK

**Touch**

* `runtime/crates/api/src/openapi.rs`
* handler/DTO annotations across API modules
* `sdk/openapi/bominal-internal.v1.json`
* `sdk/ts/internal/*`
* `scripts/generate-internal-sdk.*`
* `.github/workflows/ci.yml`

**Do**

* Add one schema generation path for the API.
* Commit a versioned internal schema.
* Generate an internal SDK for future internal tooling and typed consumers.
* Add CI freshness checks for generated artifacts.

**Done when**

* Schema diffs are visible in PRs.
* Generated internal client output is reproducible.
* This remains **internal-only** and is not published externally.

## PR-14 — Align CI with the manual, add security checks, and baseline performance

**Why:** the manual says changed Rust crates should hit 80% line coverage, while CI currently falls back to 35%, and dependency review is still conditional. The repo also already exposes runtime SSE and observability endpoints that deserve explicit performance baselines.

**Touch**

* `.github/workflows/ci.yml`
* optional: `.github/workflows/perf.yml`
* `runtime/crates/api/benches/*`
* `runtime/crates/worker/benches/*`
* `perf/k6/*` or equivalent

**Do**

* Make dependency review mandatory.
* Add `cargo audit`, `cargo deny`, and `cargo udeps`.
* Raise the Rust coverage gate to 50 now, then ratchet to 65, then 80.
* Add Docker smoke build as a required check.
* Capture baseline performance for:

  * admin jobs SSE
  * observability timeseries
  * queue/runtime state transitions
  * provider request path

**Done when**

* CI no longer contradicts the manual.
* Security and supply-chain checks are mandatory.
* You have repeatable baseline perf numbers, not guesses.

## PR-15 — Update docs, ADRs, and runbooks to match reality

**Why:** `docs/MANUAL.md` is the canonical policy/ops document, so it needs to match the new artifact policy and module boundaries.

**Touch**

* `docs/MANUAL.md`
* `docs/adr/0001-ssr-component-model.md`
* `docs/adr/0002-frontend-artifact-policy.md`
* `docs/adr/0003-provider-sdk-boundary.md`
* `docs/adr/0004-internal-api-schema.md`
* `docs/runbooks/deploy.md`
* `docs/runbooks/rollback.md`
* `docs/runbooks/provider-degradation.md`
* `docs/runbooks/queue-backlog.md`

**Do**

* Remove all references to committed `dist`.
* Document the multi-stage Docker build.
* Document the UI crate boundary and provider SDK boundary.
* Document the protected-branch deploy flow.

**Done when**

* The manual matches the workflows and Dockerfiles.
* New team members can build, test, and deploy without tribal knowledge.

## Merge order and parallelization

Use this sequence:

1. **Repo settings**
2. **PR-01**
3. **PR-02**
4. **PR-03**

After PR-03, split into lanes:

* **UI lane:** PR-04 → PR-05 → PR-06
* **Admin/auth lane:** PR-07 → PR-08 → PR-09
* **Provider/runtime lane:** PR-10 → PR-11 → PR-12

Then:

* **PR-13** after PR-08/09/10 are stable
* **PR-14** can start after PR-02, but make it fully required near the end
* **PR-15** last

Do **not** run PR-04/05/06 in parallel, and do **not** run PR-07/08/09 in parallel. Those lanes touch the same hot files and will create needless conflicts.

## Hard rules during the refactor

* No edits under `third_party/**`.
* No route path changes unless a PR explicitly declares one.
* No env var renames around asset serving. The `/assets` + `FRONTEND_ASSETS_DIR` contract stays.
* No platform rewrite during this program.
* No new frontend framework.
* No more tracked generated assets.

Start with **PR-01** and **PR-02** only after branch protection is on. That keeps the rest of the refactor from piling on top of a broken artifact/CI contract.
