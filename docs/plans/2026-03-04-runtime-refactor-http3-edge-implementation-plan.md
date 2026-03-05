# Runtime Refactor + Edge HTTP/3 Readiness Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Execute the runtime refactor/hardening backlog in `runtime/` while preserving existing production contracts, and keep HTTP/3 rollout strictly edge-only behind measurable canary promotion criteria.

**Architecture:** Use a phase-gated, contract-first program. First lock repository and artifact policies, then decompose API/worker hotspots into smaller modules with behavior-preserving tests, then normalize provider and internal API boundaries, then enforce CI/security/perf gates, and finally align docs/runbooks. Treat HTTP/3 as a separate edge experiment only after backlog stabilization.

**Tech Stack:** Rust workspace (`bominal-api`, `bominal-worker`, `bominal-shared`, `bominal-ui-primitives`, `bominal-ui-patterns`), axum + Leptos SSR, sqlx/redis, Node/Tailwind assets, Docker, GitHub Actions.

---

## Brainstorming Outcome

### Approach A (Strictly Sequential PRs)
Complete PR-01 through PR-15 in one lane.

- Pros: lowest merge-conflict risk.
- Cons: longest lead time; bottlenecks on single owner.

### Approach B (Phase-Gated + Controlled Parallel Lanes) **Recommended**
Do shared prerequisites first, then run independent lanes with explicit file-ownership boundaries.

- Pros: faster delivery while keeping conflict risk manageable.
- Cons: requires stronger coordination and PR discipline.

### Approach C (Big-Bang Refactor Branch)
Implement all module splits and contracts in one long-lived branch.

- Pros: fewer intermediate migrations.
- Cons: highest regression risk; difficult review; high rollback cost.

**Recommendation:** Use Approach B. It matches the existing PR list intent and keeps each change reviewable while preserving production safety.

## Guardrails (Do Not Violate)

- Never modify `third_party/srtgo/**` or `third_party/catchtable/**`.
- Keep product name exactly `bominal`.
- Keep route paths stable unless a task explicitly declares a path change.
- Preserve `/assets` + `FRONTEND_ASSETS_DIR` serving contract.
- Preserve session cookie policy (`HttpOnly`, `SameSite=Lax`, `Secure` only in production).
- Never log or persist secrets, credentials, PAN/CVV, or raw sensitive payloads.

## Program Order

1. Task 1-3: repo/asset/container baseline
2. Task 4-7: UI/web/auth/dashboard lane
3. Task 8-10: admin/internal-auth lane
4. Task 11-13: provider/worker lane
5. Task 14-16: API schema, CI/security/perf, docs alignment
6. Task 17: deferred HTTP/3 activation criteria path

Do not run Task 5/6/7 in parallel with each other.  
Do not run Task 8/9/10 in parallel with each other.

### Task 1: Lock Repository Policy and Capture Baseline Evidence

**Files:**
- Create: `scripts/tests/test_repo_branch_policy.sh`
- Create: `docs/plans/2026-03-04-refactor-baseline-evidence.md`

**Step 1: Write the failing test**

```bash
#!/usr/bin/env bash
set -euo pipefail

repo="${1:?owner/repo required}"
json="$(gh api "repos/${repo}/branches/main/protection")"
printf '%s' "${json}" | rg -q '"required_pull_request_reviews"' || {
  echo "missing required pull request reviews"
  exit 1
}
printf '%s' "${json}" | rg -q '"strict":true' || {
  echo "missing strict status checks"
  exit 1
}
```

**Step 2: Run test to verify it fails**

Run: `bash scripts/tests/test_repo_branch_policy.sh <owner>/<repo>`  
Expected: FAIL if branch protections are incomplete.

**Step 3: Write minimal implementation**

```bash
gh api --method PUT repos/<owner>/<repo>/branches/main/protection \
  -f required_status_checks.strict=true \
  -f enforce_admins=true \
  -f required_pull_request_reviews.dismiss_stale_reviews=true \
  -f required_pull_request_reviews.required_approving_review_count=1
```

**Step 4: Run test to verify it passes**

Run: `bash scripts/tests/test_repo_branch_policy.sh <owner>/<repo>`  
Expected: PASS with no output.

**Step 5: Commit**

```bash
git add scripts/tests/test_repo_branch_policy.sh docs/plans/2026-03-04-refactor-baseline-evidence.md
git commit -m "chore(repo): enforce branch protection baseline for refactor program"
```

### Task 2: Frontend Artifact Policy Enforcement (Generated-Only Dist)

**Files:**
- Modify: `.gitignore`
- Modify: `.github/workflows/ci.yml`
- Modify: `runtime/frontend/package.json`
- Modify: `runtime/frontend/scripts/check-css-budget.mjs`
- Test: `scripts/tests/test_frontend_dist_untracked.sh`

**Step 1: Write the failing test**

```bash
#!/usr/bin/env bash
set -euo pipefail
tracked="$(git ls-files 'runtime/frontend/dist/**')"
if [ -n "${tracked}" ]; then
  echo "tracked dist files detected"
  printf '%s\n' "${tracked}"
  exit 1
fi
```

**Step 2: Run test to verify it fails**

Run: `bash scripts/tests/test_frontend_dist_untracked.sh`  
Expected: FAIL if any generated `dist` file is tracked.

**Step 3: Write minimal implementation**

```bash
echo 'runtime/frontend/dist/' >> .gitignore
git rm -r --cached runtime/frontend/dist || true
npm --prefix runtime/frontend run build:css
npm --prefix runtime/frontend run check:css:budget
```

**Step 4: Run test to verify it passes**

Run: `bash scripts/tests/test_frontend_dist_untracked.sh`  
Expected: PASS.

**Step 5: Commit**

```bash
git add .gitignore .github/workflows/ci.yml runtime/frontend/package.json runtime/frontend/scripts/check-css-budget.mjs scripts/tests/test_frontend_dist_untracked.sh
git commit -m "build(frontend): enforce generated-only dist artifact policy"
```

### Task 3: Docker-First Frontend Build Contract

**Files:**
- Modify: `runtime/Dockerfile.api`
- Modify: `.github/workflows/cd.yml`
- Modify: `.github/workflows/ci.yml`
- Test: `scripts/tests/test_runtime_api_image_smoke.sh`

**Step 1: Write the failing test**

```bash
#!/usr/bin/env bash
set -euo pipefail
rm -rf runtime/frontend/dist
DOCKER_BUILDKIT=1 docker build -f runtime/Dockerfile.api -t bominal/api:smoke runtime
```

**Step 2: Run test to verify it fails**

Run: `bash scripts/tests/test_runtime_api_image_smoke.sh`  
Expected: FAIL if image still depends on prebuilt local `runtime/frontend/dist`.

**Step 3: Write minimal implementation**

```dockerfile
FROM node:22-bookworm AS frontend-builder
WORKDIR /app/frontend
COPY frontend/package.json frontend/package-lock.json ./
RUN npm ci
COPY frontend ./
RUN npm run build:css
```

**Step 4: Run test to verify it passes**

Run: `bash scripts/tests/test_runtime_api_image_smoke.sh`  
Expected: PASS and built image serves `/app/frontend/dist` assets.

**Step 5: Commit**

```bash
git add runtime/Dockerfile.api .github/workflows/cd.yml .github/workflows/ci.yml scripts/tests/test_runtime_api_image_smoke.sh
git commit -m "build(api): make docker frontend build the only release asset source"
```

### Task 4: Formalize Internal UI SDK Boundary

**Files:**
- Modify: `runtime/Cargo.toml`
- Create: `runtime/crates/ui/Cargo.toml`
- Create: `runtime/crates/ui/src/lib.rs`
- Modify: `runtime/crates/api/Cargo.toml`
- Test: `runtime/crates/ui/src/lib.rs` (unit test module)

**Step 1: Write the failing test**

```rust
#[test]
fn re_exports_patterns_and_primitives() {
    let _ = bominal_ui::ThemeMode::Light;
    let _ = bominal_ui::DashboardSection::Home;
}
```

**Step 2: Run test to verify it fails**

Run: `cd runtime && cargo test -p bominal-ui re_exports_patterns_and_primitives -- --nocapture`  
Expected: FAIL before `bominal-ui` crate exists.

**Step 3: Write minimal implementation**

```rust
pub use bominal_ui_primitives::*;
pub use bominal_ui_patterns::*;
```

**Step 4: Run test to verify it passes**

Run: `cd runtime && cargo test -p bominal-ui -- --nocapture`  
Expected: PASS.

**Step 5: Commit**

```bash
git add runtime/Cargo.toml runtime/crates/ui/Cargo.toml runtime/crates/ui/src/lib.rs runtime/crates/api/Cargo.toml
git commit -m "refactor(ui): add bominal-ui facade crate boundary"
```

### Task 5: Split `web.rs` into Route-Preserving Modules

**Files:**
- Modify: `runtime/crates/api/src/web.rs` -> `runtime/crates/api/src/web/mod.rs`
- Create: `runtime/crates/api/src/web/layout.rs`
- Create: `runtime/crates/api/src/web/home.rs`
- Create: `runtime/crates/api/src/web/auth.rs`
- Create: `runtime/crates/api/src/web/dashboard/overview.rs`
- Create: `runtime/crates/api/src/web/dashboard/jobs.rs`
- Create: `runtime/crates/api/src/web/dashboard/job_detail.rs`
- Create: `runtime/crates/api/src/web/dashboard/security.rs`
- Test: `runtime/crates/api/tests/dashboard_routes_test.rs`
- Test: `runtime/crates/api/tests/auth_page_test.rs`

**Step 1: Write the failing test**

```rust
#[tokio::test]
async fn dashboard_routes_keep_existing_paths_after_web_split() {
    // request /dashboard and /dashboard/jobs
    // assert status == 200 and known page marker is present
}
```

**Step 2: Run test to verify it fails**

Run: `cd runtime && cargo test -p bominal-api --test dashboard_routes_test -- --nocapture`  
Expected: FAIL once module extraction starts without re-export glue.

**Step 3: Write minimal implementation**

```rust
// runtime/crates/api/src/web/mod.rs
pub mod auth;
pub mod dashboard;
pub mod home;
pub mod layout;

pub use auth::*;
pub use dashboard::*;
pub use home::*;
pub use layout::*;
```

**Step 4: Run test to verify it passes**

Run: `cd runtime && cargo test -p bominal-api --test dashboard_routes_test --test auth_page_test -- --nocapture`  
Expected: PASS with unchanged route paths and page markers.

**Step 5: Commit**

```bash
git add runtime/crates/api/src/web runtime/crates/api/tests/dashboard_routes_test.rs runtime/crates/api/tests/auth_page_test.rs
git commit -m "refactor(api): split web module without route behavior changes"
```

### Task 6: Split `http/auth.rs` and Wire Auth Views to UI Components

**Files:**
- Modify: `runtime/crates/api/src/http/auth.rs` -> `runtime/crates/api/src/http/auth/mod.rs`
- Create: `runtime/crates/api/src/http/auth/pages.rs`
- Create: `runtime/crates/api/src/http/auth/passkeys.rs`
- Create: `runtime/crates/api/src/http/auth/sessions.rs`
- Create: `runtime/crates/api/src/http/auth/callbacks.rs`
- Modify: `runtime/crates/api/src/http/mod.rs`
- Modify: `runtime/crates/api/src/web/auth.rs`
- Test: `runtime/crates/api/tests/auth_page_test.rs`

**Step 1: Write the failing test**

```rust
#[tokio::test]
async fn auth_landing_still_exposes_passkey_and_password_ctas() {
    // GET /
    // assert contains "Authenticate with passkey"
    // assert contains "Sign in with email/password"
}
```

**Step 2: Run test to verify it fails**

Run: `cd runtime && cargo test -p bominal-api --test auth_page_test -- --nocapture`  
Expected: FAIL during auth handler split before pages are reconnected.

**Step 3: Write minimal implementation**

```rust
// runtime/crates/api/src/http/auth/mod.rs
pub mod callbacks;
pub mod pages;
pub mod passkeys;
pub mod sessions;
```

**Step 4: Run test to verify it passes**

Run: `cd runtime && cargo test -p bominal-api --test auth_page_test -- --nocapture`  
Expected: PASS with unchanged auth-page behavior.

**Step 5: Commit**

```bash
git add runtime/crates/api/src/http/auth runtime/crates/api/src/http/mod.rs runtime/crates/api/src/web/auth.rs runtime/crates/api/tests/auth_page_test.rs
git commit -m "refactor(api): decompose auth handlers and preserve auth landing UX"
```

### Task 7: Componentize Dashboard, Jobs, and Security Views

**Files:**
- Modify: `runtime/crates/ui_patterns/src/lib.rs`
- Create: `runtime/crates/ui_patterns/src/dashboard_shell.rs`
- Create: `runtime/crates/ui_patterns/src/jobs_table.rs`
- Create: `runtime/crates/ui_patterns/src/job_detail.rs`
- Create: `runtime/crates/ui_patterns/src/security_panel.rs`
- Modify: `runtime/crates/api/src/web/dashboard/*.rs`
- Optional Create: `runtime/frontend/assets/js/dashboard/entry.js`
- Test: `runtime/crates/api/tests/dashboard_routes_test.rs`

**Step 1: Write the failing test**

```rust
#[tokio::test]
async fn dashboard_jobs_and_security_pages_render_component_markers() {
    // assert dashboard pages include stable data-testid markers from ui_patterns
}
```

**Step 2: Run test to verify it fails**

Run: `cd runtime && cargo test -p bominal-api --test dashboard_routes_test -- --nocapture`  
Expected: FAIL before component markers are introduced.

**Step 3: Write minimal implementation**

```rust
pub fn render_jobs_table(rows: &[JobRow]) -> String {
    format!(r#"<section data-testid=\"jobs-table\">{}</section>"#, rows.len())
}
```

**Step 4: Run test to verify it passes**

Run: `cd runtime && cargo test -p bominal-api --test dashboard_routes_test -- --nocapture`  
Expected: PASS and no large inline script blocks remain in SSR pages.

**Step 5: Commit**

```bash
git add runtime/crates/ui_patterns/src runtime/crates/api/src/web/dashboard runtime/crates/api/tests/dashboard_routes_test.rs runtime/frontend/assets/js/dashboard/entry.js
git commit -m "refactor(ui): move dashboard/jobs/security rendering into reusable patterns"
```

### Task 8: Split `admin_service.rs` by Domain

**Files:**
- Modify: `runtime/crates/api/src/services/admin_service.rs` -> `runtime/crates/api/src/services/admin/mod.rs`
- Create: `runtime/crates/api/src/services/admin/capabilities.rs`
- Create: `runtime/crates/api/src/services/admin/incidents.rs`
- Create: `runtime/crates/api/src/services/admin/maintenance.rs`
- Create: `runtime/crates/api/src/services/admin/users.rs`
- Create: `runtime/crates/api/src/services/admin/runtime.rs`
- Create: `runtime/crates/api/src/services/admin/observability.rs`
- Create: `runtime/crates/api/src/services/admin/security.rs`
- Create: `runtime/crates/api/src/services/admin/config.rs`
- Create: `runtime/crates/api/src/services/admin/audit.rs`
- Modify: `runtime/crates/api/src/services/mod.rs`
- Test: `runtime/crates/api/tests/admin_routes_test.rs`
- Test: `runtime/crates/api/tests/admin_observability_test.rs`
- Test: `runtime/crates/api/tests/admin_audit_log_test.rs`

**Step 1: Write the failing test**

```rust
#[tokio::test]
async fn admin_incident_update_path_uses_domain_service_module() {
    // exercise /api/admin/incidents/{id}/status and assert existing envelope contract
}
```

**Step 2: Run test to verify it fails**

Run: `cd runtime && cargo test -p bominal-api --test admin_routes_test --test admin_observability_test --test admin_audit_log_test -- --nocapture`  
Expected: FAIL while module boundaries are being introduced.

**Step 3: Write minimal implementation**

```rust
// runtime/crates/api/src/services/admin/mod.rs
pub mod audit;
pub mod capabilities;
pub mod config;
pub mod incidents;
pub mod maintenance;
pub mod observability;
pub mod runtime;
pub mod security;
pub mod users;
```

**Step 4: Run test to verify it passes**

Run: `cd runtime && cargo test -p bominal-api --test admin_routes_test --test admin_observability_test --test admin_audit_log_test -- --nocapture`  
Expected: PASS with no admin route regressions.

**Step 5: Commit**

```bash
git add runtime/crates/api/src/services/admin runtime/crates/api/src/services/mod.rs runtime/crates/api/tests/admin_routes_test.rs runtime/crates/api/tests/admin_observability_test.rs runtime/crates/api/tests/admin_audit_log_test.rs
git commit -m "refactor(api): split admin service monolith by domain"
```

### Task 9: Split `http/admin.rs` into Pages and API Handler Trees

**Files:**
- Modify: `runtime/crates/api/src/http/admin.rs` -> `runtime/crates/api/src/http/admin/mod.rs`
- Create: `runtime/crates/api/src/http/admin/pages/mod.rs`
- Create: `runtime/crates/api/src/http/admin/pages/maintenance.rs`
- Create: `runtime/crates/api/src/http/admin/pages/users.rs`
- Create: `runtime/crates/api/src/http/admin/pages/runtime.rs`
- Create: `runtime/crates/api/src/http/admin/pages/observability.rs`
- Create: `runtime/crates/api/src/http/admin/pages/security.rs`
- Create: `runtime/crates/api/src/http/admin/pages/config.rs`
- Create: `runtime/crates/api/src/http/admin/pages/audit.rs`
- Create: `runtime/crates/api/src/http/admin/api/mod.rs`
- Create: `runtime/crates/api/src/http/admin/api/capabilities.rs`
- Create: `runtime/crates/api/src/http/admin/api/incidents.rs`
- Create: `runtime/crates/api/src/http/admin/api/observability.rs`
- Create: `runtime/crates/api/src/http/admin/api/runtime.rs`
- Modify: `runtime/crates/api/src/http/mod.rs`
- Test: `runtime/crates/api/tests/admin_routes_test.rs`
- Test: `runtime/crates/api/tests/admin_observability_test.rs`
- Test: `runtime/crates/api/tests/admin_audit_log_test.rs`

**Step 1: Write the failing test**

```rust
#[tokio::test]
async fn admin_route_paths_remain_stable_after_handler_split() {
    // verify /admin/* and /api/admin/* paths keep status + shape
}
```

**Step 2: Run test to verify it fails**

Run: `cd runtime && cargo test -p bominal-api --test admin_routes_test --test admin_observability_test --test admin_audit_log_test -- --nocapture`  
Expected: FAIL before `http/admin/mod.rs` exports are complete.

**Step 3: Write minimal implementation**

```rust
// runtime/crates/api/src/http/admin/mod.rs
pub mod api;
pub mod pages;
```

**Step 4: Run test to verify it passes**

Run: `cd runtime && cargo test -p bominal-api --test admin_routes_test --test admin_observability_test --test admin_audit_log_test -- --nocapture`  
Expected: PASS and handlers remain thin.

**Step 5: Commit**

```bash
git add runtime/crates/api/src/http/admin runtime/crates/api/src/http/mod.rs runtime/crates/api/tests/admin_routes_test.rs runtime/crates/api/tests/admin_observability_test.rs runtime/crates/api/tests/admin_audit_log_test.rs
git commit -m "refactor(api): split admin http handlers into page and api modules"
```

### Task 10: Split Internal Auth and Fold Invite Logic Into One Tree

**Files:**
- Modify: `runtime/crates/api/src/http/internal_auth.rs` -> `runtime/crates/api/src/http/internal_auth/mod.rs`
- Move: `runtime/crates/api/src/http/internal_auth_invites.rs` -> `runtime/crates/api/src/http/internal_auth/invites.rs`
- Create: `runtime/crates/api/src/http/internal_auth/guards.rs`
- Create: `runtime/crates/api/src/http/internal_auth/middleware.rs`
- Create: `runtime/crates/api/src/http/internal_auth/service_identity.rs`
- Create: `runtime/crates/api/src/http/internal_auth/handlers.rs`
- Modify: `runtime/crates/api/src/http/mod.rs`
- Test: `runtime/crates/api/tests/internal_api_auth_contract_test.rs`

**Step 1: Write the failing test**

```rust
#[tokio::test]
async fn internal_api_auth_contract_is_unchanged_after_split() {
    // missing token => 401, malformed token => 401, valid token reaches handler
}
```

**Step 2: Run test to verify it fails**

Run: `cd runtime && cargo test -p bominal-api --test internal_api_auth_contract_test -- --nocapture`  
Expected: FAIL while module extraction is incomplete.

**Step 3: Write minimal implementation**

```rust
// runtime/crates/api/src/http/internal_auth/mod.rs
pub mod guards;
pub mod handlers;
pub mod invites;
pub mod middleware;
pub mod service_identity;
```

**Step 4: Run test to verify it passes**

Run: `cd runtime && cargo test -p bominal-api --test internal_api_auth_contract_test -- --nocapture`  
Expected: PASS with fail-closed auth behavior preserved.

**Step 5: Commit**

```bash
git add runtime/crates/api/src/http/internal_auth runtime/crates/api/src/http/mod.rs runtime/crates/api/tests/internal_api_auth_contract_test.rs
git commit -m "refactor(api): unify internal auth and invite handling modules"
```

### Task 11: Normalize `shared/providers` Into Canonical Internal Provider SDK

**Files:**
- Modify: `runtime/crates/shared/src/providers/mod.rs`
- Create: `runtime/crates/shared/src/providers/contract.rs`
- Create: `runtime/crates/shared/src/providers/model.rs`
- Create: `runtime/crates/shared/src/providers/error.rs`
- Create: `runtime/crates/shared/src/providers/capabilities.rs`
- Create: `runtime/crates/shared/src/providers/redaction.rs`
- Create: `runtime/crates/shared/src/providers/retry.rs`
- Modify: `runtime/crates/shared/src/providers/srt/mod.rs`
- Modify: `runtime/crates/shared/src/providers/ktx/mod.rs`
- Test: `runtime/crates/shared/src/providers/mod.rs` (unit tests)

**Step 1: Write the failing test**

```rust
#[test]
fn provider_contract_does_not_expose_srt_wire_types() {
    // compile-time assertion: shared::providers::ProviderClient uses canonical DTOs only
}
```

**Step 2: Run test to verify it fails**

Run: `cd runtime && cargo test -p bominal-shared provider_contract_does_not_expose_srt_wire_types -- --nocapture`  
Expected: FAIL before contract extraction.

**Step 3: Write minimal implementation**

```rust
pub trait ProviderClient {
    fn search(&mut self, req: ProviderSearchRequest) -> Result<ProviderSearchResponse, ProviderError>;
}
```

**Step 4: Run test to verify it passes**

Run: `cd runtime && cargo test -p bominal-shared providers:: -- --nocapture`  
Expected: PASS and adapter boundaries remain provider-specific internally.

**Step 5: Commit**

```bash
git add runtime/crates/shared/src/providers
git commit -m "refactor(shared): introduce canonical provider sdk contracts"
```

### Task 12: Split `provider_jobs_service.rs` After Provider SDK Normalization

**Files:**
- Modify: `runtime/crates/api/src/services/provider_jobs_service.rs` -> `runtime/crates/api/src/services/provider_jobs/mod.rs`
- Create: `runtime/crates/api/src/services/provider_jobs/commands.rs`
- Create: `runtime/crates/api/src/services/provider_jobs/queries.rs`
- Create: `runtime/crates/api/src/services/provider_jobs/mapping.rs`
- Create: `runtime/crates/api/src/services/provider_jobs/state.rs`
- Modify: `runtime/crates/api/src/services/mod.rs`
- Test: `runtime/crates/api/tests/dashboard_routes_test.rs` (jobs surfaces)

**Step 1: Write the failing test**

```rust
#[tokio::test]
async fn provider_jobs_paths_use_canonical_provider_models_only() {
    // compile/runtime check that API service layer no longer consumes srt::wire DTOs
}
```

**Step 2: Run test to verify it fails**

Run: `cd runtime && cargo test -p bominal-api provider_jobs_paths_use_canonical_provider_models_only -- --nocapture`  
Expected: FAIL before provider_jobs split/mapping extraction.

**Step 3: Write minimal implementation**

```rust
// runtime/crates/api/src/services/provider_jobs/mod.rs
pub mod commands;
pub mod mapping;
pub mod queries;
pub mod state;
```

**Step 4: Run test to verify it passes**

Run: `cd runtime && cargo test -p bominal-api --test dashboard_routes_test -- --nocapture`  
Expected: PASS with unchanged route behavior.

**Step 5: Commit**

```bash
git add runtime/crates/api/src/services/provider_jobs runtime/crates/api/src/services/mod.rs runtime/crates/api/tests/dashboard_routes_test.rs
git commit -m "refactor(api): split provider jobs service and consume canonical provider sdk"
```

### Task 13: Split Worker `executor.rs` Into Testable Runtime Modules

**Files:**
- Modify: `runtime/crates/worker/src/runtime/executor.rs` -> `runtime/crates/worker/src/runtime/executor/mod.rs`
- Create: `runtime/crates/worker/src/runtime/executor/planner.rs`
- Create: `runtime/crates/worker/src/runtime/executor/dispatcher.rs`
- Create: `runtime/crates/worker/src/runtime/executor/state_machine.rs`
- Create: `runtime/crates/worker/src/runtime/executor/provider_runner.rs`
- Create: `runtime/crates/worker/src/runtime/executor/result_mapper.rs`
- Create: `runtime/crates/worker/src/runtime/executor/retry_policy.rs`
- Create: `runtime/crates/worker/src/runtime/executor/rate_limit.rs`
- Create: `runtime/crates/worker/src/runtime/executor/metrics.rs`
- Modify: `runtime/crates/worker/src/runtime/mod.rs`
- Test: `runtime/crates/worker/src/runtime/executor/*.rs` (unit tests)

**Step 1: Write the failing test**

```rust
#[test]
fn task_rate_limit_enforces_one_request_per_second_by_default() {
    // assert RateLimitPolicy::default().next_delay(attempt=2) == 1s
}
```

**Step 2: Run test to verify it fails**

Run: `cd runtime && cargo test -p bominal-worker task_rate_limit_enforces_one_request_per_second_by_default -- --nocapture`  
Expected: FAIL before `rate_limit.rs` exists.

**Step 3: Write minimal implementation**

```rust
pub struct RateLimitPolicy {
    pub per_task_qps: u32,
}

impl Default for RateLimitPolicy {
    fn default() -> Self { Self { per_task_qps: 1 } }
}
```

**Step 4: Run test to verify it passes**

Run: `cd runtime && cargo test -p bominal-worker executor:: -- --nocapture`  
Expected: PASS with modular state/retry/rate-limit coverage.

**Step 5: Commit**

```bash
git add runtime/crates/worker/src/runtime/executor runtime/crates/worker/src/runtime/mod.rs
git commit -m "refactor(worker): split executor monolith into testable runtime modules"
```

### Task 14: Add Versioned Internal API Schema + Generated Internal SDK

**Files:**
- Modify: `runtime/crates/api/src/openapi.rs`
- Modify: `runtime/crates/api/src/http/**/*.rs` (schema annotations)
- Create: `sdk/openapi/bominal-internal.v1.json`
- Create: `sdk/ts/internal/*`
- Create: `scripts/generate-internal-sdk.sh`
- Modify: `.github/workflows/ci.yml`

**Step 1: Write the failing test**

```bash
#!/usr/bin/env bash
set -euo pipefail
./scripts/generate-internal-sdk.sh
if ! git diff --quiet -- sdk/openapi sdk/ts/internal; then
  echo "generated sdk/openapi artifacts are stale"
  exit 1
fi
```

**Step 2: Run test to verify it fails**

Run: `bash scripts/generate-internal-sdk.sh && git status --short sdk/openapi sdk/ts/internal`  
Expected: FAIL/dirty output before generator and committed artifacts are aligned.

**Step 3: Write minimal implementation**

```bash
cd runtime
cargo run -p bominal-api --bin bominal-api -- --export-openapi ../sdk/openapi/bominal-internal.v1.json
```

**Step 4: Run test to verify it passes**

Run: `bash scripts/generate-internal-sdk.sh && git diff --exit-code -- sdk/openapi sdk/ts/internal`  
Expected: PASS with no diffs.

**Step 5: Commit**

```bash
git add runtime/crates/api/src/openapi.rs sdk/openapi/bominal-internal.v1.json sdk/ts/internal scripts/generate-internal-sdk.sh .github/workflows/ci.yml
git commit -m "feat(api): add versioned internal schema and generated sdk freshness gate"
```

### Task 15: Align CI With Manual Gates + Security/Perf Baselines

**Files:**
- Modify: `.github/workflows/ci.yml`
- Create: `.github/workflows/perf.yml`
- Create: `runtime/crates/api/benches/admin_sse.rs`
- Create: `runtime/crates/api/benches/observability_timeseries.rs`
- Create: `runtime/crates/worker/benches/runtime_state_transitions.rs`
- Create: `perf/k6/admin_jobs_stream.js`
- Create: `perf/k6/observability_timeseries.js`

**Step 1: Write the failing test**

```yaml
# CI assertions:
# - dependency review required on PRs
# - coverage floor not below 50
# - docker smoke build required
# - cargo audit/cargo deny/cargo udeps required
```

**Step 2: Run test to verify it fails**

Run: `act pull_request -W .github/workflows/ci.yml` (or push draft PR and inspect checks)  
Expected: FAIL where required gates are currently optional or missing.

**Step 3: Write minimal implementation**

```yaml
- name: Security audit
  run: cargo audit
- name: Deny policy
  run: cargo deny check
- name: Unused deps
  run: cargo udeps --all-targets
```

**Step 4: Run test to verify it passes**

Run: `cd runtime && cargo test --workspace --locked && docker build -f Dockerfile.api -t bominal/api:ci-smoke .`  
Expected: PASS locally and in CI with required checks green.

**Step 5: Commit**

```bash
git add .github/workflows/ci.yml .github/workflows/perf.yml runtime/crates/api/benches runtime/crates/worker/benches perf/k6
git commit -m "ci: enforce manual-aligned quality security and perf gates"
```

### Task 16: Documentation/ADR/Runbook Alignment

**Files:**
- Modify: `docs/MANUAL.md`
- Create: `docs/adr/0001-ssr-component-model.md`
- Create: `docs/adr/0002-frontend-artifact-policy.md`
- Create: `docs/adr/0003-provider-sdk-boundary.md`
- Create: `docs/adr/0004-internal-api-schema.md`
- Create: `docs/playbooks/DEPLOY_RUNTIME.md`
- Create: `docs/playbooks/ROLLBACK_RUNTIME.md`
- Create: `docs/playbooks/PROVIDER_DEGRADATION.md`
- Create: `docs/playbooks/QUEUE_BACKLOG.md`
- Create: `scripts/tests/test_docs_contracts.sh`
- Modify: `docs/README.md`
- Modify: `CHANGELOG.md`

**Step 1: Write the failing test**

```bash
#!/usr/bin/env bash
set -euo pipefail

for required in \
  docs/MANUAL.md \
  docs/README.md \
  docs/INTENT_ROUTING.md \
  docs/PROD_ENV_CONTRACT.md; do
  [ -f "${required}" ] || { echo "missing ${required}"; exit 1; }
done

rg -q "docs/MANUAL.md" docs/README.md || {
  echo "docs/README.md must point to docs/MANUAL.md"
  exit 1
}

rg -q "## Unreleased" CHANGELOG.md || {
  echo "CHANGELOG.md must contain ## Unreleased"
  exit 1
}
```

**Step 2: Run test to verify it fails**

Run commands above.  
Expected: FAIL until docs pointers/intent/changelog reflect new boundaries.

**Step 3: Write minimal implementation**

```markdown
## Documentation Governance
- frontend build artifacts are generated-only
- docker image is source of truth for `/app/frontend/dist`
- provider SDK boundary is canonical in `runtime/crates/shared/src/providers/*`
```

**Step 4: Run test to verify it passes**

Run commands above again.  
Expected: PASS.

**Step 5: Commit**

```bash
git add docs/MANUAL.md docs/adr docs/playbooks scripts/tests/test_docs_contracts.sh docs/README.md CHANGELOG.md
git commit -m "docs: align manual adrs and runbooks with refactor target state"
```

### Task 17: HTTP/3 Edge Canary (Deferred Activation Only)

**Files:**
- Modify: `docs/plans/2026-03-04-http3-edge-maybe-todo.md`
- Create: `docs/playbooks/HTTP3_EDGE_CANARY.md`
- Create: `perf/http3/baseline-metrics.md`

**Step 1: Write the failing test**

```markdown
Promotion criteria must be measurable:
- p95 latency improvement >= 5% OR
- protocol/connection failure reduction >= 20%
- no error-rate regression > 0.1 percentage points
- no auth/session regression signals
```

**Step 2: Run test to verify it fails**

Run: manual checklist review against observability snapshots for 7 baseline days.  
Expected: FAIL (remain deferred) if baseline evidence is missing.

**Step 3: Write minimal implementation**

```markdown
Canary rollout:
1. Collect 7-day baseline.
2. Enable edge HTTP/3 for canary slice with HTTP/2/1.1 fallback.
3. Compare KPI deltas, then promote or rollback.
```

**Step 4: Run test to verify it passes**

Run: execute canary checklist and attach metric evidence.  
Expected: PASS only when all promotion gates are satisfied.

**Step 5: Commit**

```bash
git add docs/plans/2026-03-04-http3-edge-maybe-todo.md docs/playbooks/HTTP3_EDGE_CANARY.md perf/http3/baseline-metrics.md
git commit -m "docs(ops): codify deferred http3 edge canary gating and evidence"
```

## Final Verification Gate (Run Before Any Merge)

```bash
./scripts/bootstrap-local.sh
cd runtime && cargo fmt --all && cargo check --workspace --locked && cargo test --workspace --locked
npm --prefix runtime/frontend run build:css
npm --prefix runtime/frontend run check:css:budget
docker build -f runtime/Dockerfile.api -t bominal/api:final-smoke runtime
bash scripts/tests/test_docs_contracts.sh
```

Expected: all commands pass, no tracked `runtime/frontend/dist/**`, no guardrail violations, and no changes under `third_party/**`.
