Good. With your answers locked in, this is now a **refactor-and-hardening program**, not a discovery exercise.

Bominal already has the right high-level shape for that: a Rust monorepo with `api`, `worker`, `shared`, `migrations`, and a small `frontend` asset package; the API crate already depends on Leptos/Leptos Axum, and the repo already exposes SSR user pages plus admin/runtime/observability endpoints, including an SSE jobs stream.

The real problems are structural and operational, not “missing framework” problems: `web.rs` is about **2,397 lines**, `executor.rs` about **1,966 lines**, `admin_service.rs` about **1,094 lines**, `admin.rs` about **1,024 lines**, and `internal_auth.rs` about **696 lines**. That is where readability, velocity, and regression risk are concentrated today.

Your `dist` decision is also correct. Right now the repo tracks `runtime/frontend/dist`, the frontend package is only a small Tailwind/charts build package, the API serves assets from `FRONTEND_ASSETS_DIR`, and the current Dockerfile copies `frontend/dist` into the final image without building it internally. CD also prebuilds CSS before the image build. That is exactly the hybrid artifact policy that should be removed.

Given your constraints, my headline recommendation is:

**Keep the current Rust monorepo and runtime shape. Do not do a platform rewrite now.**
Refactor toward:

1. **Leptos component-based SSR**
2. **internal UI SDK**
3. **internal provider SDK**
4. **generated internal API client**
5. **artifact-clean CI/CD with mandatory branch protection**

## Final target state

### 1) Frontend target state

The API crate already has the Leptos dependencies needed for a real SSR component model, so stop treating the UI as giant route-local rendering blobs and standardize on Leptos components now.

**What to build**

- Create an internal workspace crate: `runtime/crates/ui`
- Move `ui-primitives` and `ui-patterns` into that crate as the official internal UI SDK
- Use it for:
  - layout shells
  - nav/header/sidebar
  - forms/inputs
  - tables
  - cards
  - badges/status pills
  - audit/event timelines
  - empty/loading/error states
  - auth/passkey flows
  - admin page shells

**What to remove**

- Route-specific HTML/string-building from `web.rs`
- Ad hoc inline page composition
- Any new UI logic inside `runtime/frontend` beyond assets

**Hard boundary**

- `runtime/frontend` becomes **asset-source only**:
  - Tailwind
  - chart bundle
  - static design tokens if needed

- All rendered app UI belongs in Leptos SSR components inside Rust.

### 2) Backend target state

The backend is already sensibly split across `api`, `worker`, and `shared`, and shared provider code already exists under `shared/providers/{srt,ktx}`. Keep that architecture, but make module boundaries real instead of nominal.

**What to build**

- An internal Rust **provider SDK** in `shared/providers`
- A versioned **API contract** and generated **internal TypeScript SDK**
- Smaller service/handler modules with strict layering

**What to remove**

- God-object service files
- Provider-specific branching scattered across API/worker code
- Direct UI route logic mixed with business logic

### 3) Tooling target state

The repo’s own manual calls for a stricter CI/CD posture than the current workflows enforce: the manual says changed Rust crates should reach **80% line coverage**, while CI falls back to **35%** unless a variable is set; dependency review is also conditional.

So the target state is:

- mandatory branch protection
- CI required before merge
- CD gated by CI success
- generated assets never tracked
- Docker builds its own frontend assets
- security/supply-chain checks mandatory, not opt-in

---

## Concrete refactor instructions

## Frontend

### A. Create the internal UI SDK

Create a new crate:

- `runtime/crates/ui`
  - `primitives/`
    - `button.rs`
    - `input.rs`
    - `select.rs`
    - `checkbox.rs`
    - `modal.rs`
    - `table.rs`
    - `badge.rs`
    - `icon.rs`

  - `patterns/`
    - `page_shell.rs`
    - `admin_shell.rs`
    - `dashboard_shell.rs`
    - `audit_timeline.rs`
    - `jobs_table.rs`
    - `observability_panel.rs`
    - `auth_forms.rs`

  - `theme/`
    - shared classes/tokens/helpers

  - `lib.rs`

**Rule:** new SSR UI code only lands here or in tiny route view wrappers.

### B. Break `web.rs` apart

`web.rs` is too large to survive another month of feature work safely.

Replace it with:

- `runtime/crates/api/src/web/mod.rs`
- `runtime/crates/api/src/web/auth.rs`
- `runtime/crates/api/src/web/dashboard.rs`
- `runtime/crates/api/src/web/jobs.rs`
- `runtime/crates/api/src/web/security.rs`
- `runtime/crates/api/src/web/admin/`
  - `mod.rs`
  - `maintenance.rs`
  - `users.rs`
  - `runtime.rs`
  - `observability.rs`
  - `security.rs`
  - `config.rs`
  - `audit.rs`

**Rule:** handlers in `http/*`, rendering in `web/*` or `ui`, business logic in `services/*`.

### C. Keep frontend tooling minimal

The only verified frontend scripts today are CSS/charts build scripts. Do **not** overreact by adding a large React/Vite/Next toolchain.

Add only what is justified:

- build validation
- Playwright smoke tests for critical flows
- optional tiny JS tests only if actual client-side JS grows

Do **not** add:

- a second frontend framework
- a SPA rewrite
- TypeScript app scaffolding unless you genuinely start building browser-side app logic

### D. Asset policy refactor

Your plan is right, with two additions.

**Implement**

1. Add `runtime/frontend/dist/` to `.gitignore`. The current `.gitignore` does not verify that policy yet.
2. Remove all tracked `runtime/frontend/dist/**` files.
3. Add `frontend/dist` to `runtime/.dockerignore` too, so Docker cannot accidentally depend on local generated assets. The runtime folder already has a `.dockerignore`.
4. Refactor `runtime/Dockerfile.api` into a multi-stage build:
   - `frontend-builder` from Node
   - `rust-builder` from Rust
   - final runtime image

5. In `frontend-builder`:
   - copy `frontend/package.json` and `package-lock.json`
   - run `npm ci`
   - copy `frontend/`
   - run `npm run build:css`
   - optionally `npm run build:charts` if you formalize that into release assets

6. In final image:
   - copy `dist` from `frontend-builder` to `/app/frontend/dist`
   - keep `FRONTEND_ASSETS_DIR=/app/frontend/dist`

**Improve your plan slightly**

- Also stop copying the entire `frontend/` folder into the Rust builder unless compile-time code actually needs it. The current Dockerfile copies it into the Rust build context even though the final image only needs the built `dist`.

### E. Frontend acceptance criteria

- No tracked files under `runtime/frontend/dist/**`
- `web.rs` removed or reduced to a thin module root
- `runtime/crates/ui` adopted by at least auth + dashboard + one admin section
- critical SSR pages smoke-tested
- no new UI route file over ~400–500 LOC without explicit exception

---

## Backend

## A. Split `admin_service.rs`

This should become a service family, not one file. It is currently over 1,000 LOC.

Refactor to:

- `services/admin/`
  - `capabilities.rs`
  - `incidents.rs`
  - `observability.rs`
  - `runtime.rs`
  - `security.rs`
  - `audit.rs`
  - `config.rs`
  - `mod.rs`

**Rule:** each file owns one domain area and its DTO mapping.

## B. Split `http/admin.rs`

Keep handlers thin and route-focused. It is also oversized today.

Refactor to:

- `http/admin/`
  - `mod.rs`
  - `capabilities.rs`
  - `incidents.rs`
  - `observability.rs`
  - `runtime.rs`
  - `security.rs`
  - `audit.rs`
  - `config.rs`

**Rule:** handlers call service methods and map results to HTTP only.

## C. Split `internal_auth.rs`

`internal_auth.rs` is large enough that auth policy changes will stay risky until it is separated.

Refactor to:

- `http/internal_auth/`
  - `guards.rs`
  - `middleware.rs`
  - `invites.rs`
  - `service_identity.rs`
  - `handlers.rs`
  - `mod.rs`

## D. Split `executor.rs`

The worker runtime already has `dlq`, `lease`, `retry`, and `executor` modules. That is the right direction; now finish the decomposition so `executor.rs` stops being the center of everything.

Refactor to:

- `worker/src/runtime/executor/`
  - `mod.rs`
  - `planner.rs`
  - `dispatcher.rs`
  - `state_machine.rs`
  - `result_mapper.rs`
  - `retry_policy.rs`
  - `rate_limit.rs`
  - `metrics.rs`
  - `provider_runner.rs`

**Key rule:** provider request cadence, retry/backoff, and state transitions must be testable independently.

## E. Build the internal provider SDK

Because provider code already lives under `shared/providers/{srt,ktx}`, that should become the single sanctioned provider boundary.

Create:

- `ProviderClient` trait
- typed request/response DTOs
- canonical `ProviderError`
- retryable/non-retryable classification
- redaction-safe logging wrapper
- provider capability flags
- idempotency key helper
- per-provider adapter implementation

**Rule:** API and worker code must not know provider-specific wire details.

## F. Build the internal API SDK

Because the SDK is internal-only, keep it simple:

- generate a versioned OpenAPI spec from the API crate
- generate an internal TypeScript client into `sdk/ts/internal`
- do not publish it publicly
- consume it by repo path or internal package source

**Rule:** the internal SDK is the only browser/admin consumer of typed API contracts.

## G. Backend performance work

Your first bottleneck will not be “2 users”; it will be **reservation task loops** and provider polling intensity. So optimize for **task concurrency, provider request cadence, and network chatter**, not homepage RPS.

Add:

- benchmark for provider request path
- benchmark for queue state transitions
- benchmark for retry/backoff
- load test for `/api/admin/runtime/jobs/stream`
- load test for `/api/admin/observability/timeseries`

The repo already has a meaningful API test base for health, auth, dashboard, admin, observability, and internal auth contracts, so expand from that instead of inventing a new testing culture from scratch.

---

## Utility & tooling

## A. Branch protection: make it mandatory

You confirmed CI is not currently enforced before merge/deploy. Fix that first.

For `main`:

- require pull requests
- require at least 1 review
- dismiss stale reviews
- require conversation resolution
- block direct pushes except owner emergency path if you want one
- require CI checks
- restrict force pushes
- require linear history if you want cleaner bisects

For CD:

- only deploy from protected `main` or signed version tags
- add environment approval for production

## B. Align CI with the repo manual

The manual says 80% Rust line coverage for changed crates; CI currently falls back to 35% unless configured otherwise. Close that gap.

Use a ratchet:

- immediately: 35 → 50
- next phase: 50 → 65
- final: 65 → 80

But keep **80** as the declared end state because that is already the documented policy.

## C. Remove optional security posture

Dependency review is currently conditional. Make it required. Attestation is not verified in CI either.

Add mandatory jobs for:

- dependency review
- `cargo-audit`
- `cargo-deny`
- `cargo-udeps`
- secret scanning if not already covered externally
- Docker build smoke for API image
- tracked-artifact guard

## D. Add the tracked-artifact guard

Add a CI step that fails if any `runtime/frontend/dist/**` file is tracked by git.

Use a hard fail such as:

- `git ls-files 'runtime/frontend/dist/**' | grep -q . && exit 1`

Also add a Docker smoke build so release jobs never rely on repo-committed artifacts.

## E. Simplify build ownership

Right now CD prebuilds CSS and then builds Docker, while the Dockerfile itself still expects source + tracked output. Move to one source of truth: **Docker builds release assets; CI validates them.**

That means:

- CI job: validate `npm ci && npm run build:css`
- Dockerfile: build release assets for image
- CD: stop assuming repo already contains `dist`

## F. Keep local DX strong, not fancy

Keep:

- `bootstrap-local.sh`
- `dev-up.sh`

Add:

- `justfile`
- pre-commit hooks
- one canonical `make verify` / `just verify`

The repo already has those bootstrap/dev scripts, so build on them rather than replacing them.

---

## Maintenance / operations

## A. Since the repo is going private, license is not a blocker

As of verification, the repo is public and GitHub reports no license on record. Since you plan to make it private, that is no longer a delivery blocker for the refactor itself. If it ever goes public again, add a license before reopening it.

## B. Keep the current runtime contract, not the current implementation mess

The current asset serving contract is coherent:

- `FRONTEND_ASSETS_DIR`
- default local path
- final image serving from `/app/frontend/dist`
- compose env using `/app/frontend/dist`

So do **not** change that contract. Clean up everything behind it.

## C. Add ADRs

Create ADRs for:

- SSR component model
- artifact policy (`dist` ephemeral only)
- provider SDK boundary
- internal API SDK generation
- deployment platform choice
- observability and retention policy

## D. Add runbooks

You already have admin/runtime/incident surfaces in the product; formalize the operator behavior around them. The current README already exposes incident and observability endpoints, so they should be supported by actual runbooks.

Add:

- deploy runbook
- rollback runbook
- provider degradation runbook
- queue backlog runbook
- auth incident runbook
- asset/build break runbook

---

## What not to do

Given the current verified codebase and your constraints, I would **not** do these right now:

- Do **not** rewrite the frontend as a SPA
- Do **not** add React/Vite/Next
- Do **not** split the runtime into microservices
- Do **not** move reservation execution into Supabase Edge Functions
- Do **not** replace your passkey/auth stack with Supabase Auth
- Do **not** publish a public SDK
- Do **not** add a lot of JS tooling when the verified frontend package is still mainly CSS/charts build infrastructure.

---

## Infra recommendation under your free-tier constraints

This is the one place where architecture discipline matters more than feature ambition.

The repo already has a long-running worker runtime shape (`dlq`, `lease`, `retry`, oversized `executor`), admin/runtime endpoints, and an SSE jobs stream. I infer from that shape that **Bominal is a better fit for long-lived container services than for edge-function execution of the core reservation loop**. Supabase Edge Functions on the free plan have limits like **256 MB memory**, **150 s wall clock**, and **2 s CPU time**, while Cloud Run’s free tier includes only **1 GiB/month outbound in North America**. Supabase’s free plan also has its own egress and Realtime quotas.

So my near-term recommendation is:

- **Do not move the worker to Supabase Edge Functions**
- **Do not do a platform migration before the refactor**
- Keep the existing Rust API + worker model
- Keep network usage lean:
  - immutable cache for `/assets`
  - gzip/brotli where appropriate
  - delta-only SSE payloads
  - strict provider polling only while tasks are active
  - bounded observability payloads

If you want a managed evolution path later, **Cloud Run containers** are a better target than Functions because the repo already has Dockerfiles for API and worker. But do that **after** the codebase is modularized and artifact-clean.

Evervault should stay **narrowly scoped** to sensitive relay/encryption boundaries, not become the backbone of your runtime. Evervault Relay is documented as available across plans and is intended for encrypted/decrypted third-party API traffic; your compose setup also already carries Evervault env hooks.

For GitHub Student benefits: use them opportunistically, but do not design the production architecture around promo-specific pack perks that can change. The student program exists, but the concrete bundle can evolve.

---

## Delivery plan

## Phase 1: stop the bleeding

1. branch protection
2. `dist` ephemeral-only policy
3. Docker multi-stage frontend build
4. CI tracked-artifact guard
5. dependency review mandatory
6. coverage ratchet started
7. create `runtime/crates/ui`
8. split `web.rs` first

## Phase 2: create real boundaries

1. split `admin_service.rs`
2. split `http/admin.rs`
3. split `internal_auth.rs`
4. split `executor.rs`
5. introduce internal provider SDK
6. move all shared SSR components into `ui`

## Phase 3: contracts and performance

1. generate internal API spec
2. generate internal TS SDK
3. benchmark worker/provider/queue paths
4. load-test SSE/runtime endpoints
5. add runbooks + ADRs
6. tighten coverage toward 80%

---

## Definition of done

You are done with the refactor when all of these are true:

- `runtime/frontend/dist/**` is never tracked
- Docker builds frontend release assets internally
- CD no longer depends on precommitted assets
- `web.rs` is gone as a giant monolith
- `executor.rs`, `admin_service.rs`, `admin.rs`, and `internal_auth.rs` are decomposed
- all new SSR UI uses the internal Leptos component SDK
- provider access goes through one internal provider SDK
- admin/browser consumers use one internal generated API SDK
- CI is required before merge
- coverage is on a path to the repo’s own documented standard
- no unnecessary framework/platform rewrite has been introduced

The next useful step is to convert this into a **PR-by-PR refactor backlog with exact file moves, acceptance criteria, and merge order**.
