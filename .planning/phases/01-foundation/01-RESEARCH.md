# Phase 1: Foundation - Research

**Researched:** 2026-03-27
**Domain:** Leptos 0.8 islands foundation, cargo-leptos workspace setup, repo convergence strategy
**Confidence:** HIGH

## Summary

Phase 1 is not a greenfield migration. The repo currently has four frontend tracks or sources in play:

1. `crates/bominal-app` is the active workspace app crate and compiles today, but only in CSR mode.
2. `crates/bominal-frontend` is a much richer Leptos prototype with SSR/hydrate-era structure, but it sits outside the workspace, does not parse against the current root manifest, and assumes a direct server-function architecture the roadmap no longer wants.
3. `frontend/` is still the production SvelteKit app that Axum serves from `frontend/build`.
4. `../bominal-ui` is the canonical shared UI crate with ecosystem CSS plus `train` and `auth` product skins, and it compiles successfully with Leptos enabled.

The correct foundation move is to converge on **one** active app crate and **one** canonical UI source: keep `bominal-app` as the execution target, treat `bominal-frontend` as donor code, and treat `bominal-ui` as the design-system source of truth. Official Leptos docs confirm that islands mode requires enabling the `islands` feature on `leptos` and switching hydration from `hydrate_body(...)` to `hydrate_islands()`. Current cargo-leptos workspace docs also confirm that multi-package projects should be declared in the root manifest under `[[workspace.metadata.leptos]]`, not in a separate `Cargo-leptos.toml` file.

**Primary recommendation:** Use Phase 1 to establish the build foundation only: feature-gated `bominal-app`, a real `ssr` server build path, root cargo-leptos workspace metadata, and `bominal-ui`-backed CSS/build inputs. Do not try to keep the current auth flows alive during this phase, and do not blindly delete `crates/bominal-frontend` before the team has mined its useful code and retired its build references.

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions
- **D-01:** `crates/bominal-app` remains the active migration target. `crates/bominal-frontend` is donor/reference only.
- **D-02:** Phase 1 is compile-first. Current CSR pages do not need to stay working in this phase.
- **D-03:** cargo-leptos workspace config lives in root `Cargo.toml` via `[[workspace.metadata.leptos]]`.
- **D-04:** `bominal-server` gets a real `ssr` build path now, but Axum route handoff to `leptos_axum` waits until Phase 8.
- **D-05:** `bominal-ui` is the canonical UI source of truth.
- **D-06:** Shared UI style inputs move into the active app build path now, while visual parity stays deferred.
- **D-07:** The active app architecture remains proxy-to-`/api/`, not direct server functions.

### Claude's Discretion
- Exact feature-gating/stub strategy for the current CSR modules
- Exact manifest layout and dependency split
- Minimal shell and style file layout

### Deferred Ideas (OUT OF SCOPE)
- Deleting `crates/bominal-frontend` outright
- Replacing the SvelteKit static fallback in Axum
- Removing Node.js from Docker/dev scripts
- Visual parity verification
</user_constraints>

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|------------------|
| FND-01 | Retire `bominal-frontend` as the active crate and clean workspace/build ownership | Codebase scan shows it is useful donor code, but not a viable active target in its current state |
| FND-02 | Rewrite `bominal-app` for SSR/hydrate with islands architecture | Leptos islands docs require `features = ["islands"]` and `hydrate_islands()` |
| FND-03 | Add Leptos workspace dependencies | `crates/bominal-frontend` already expects workspace-scoped Leptos deps; current root manifest is missing them |
| FND-04 | Configure cargo-leptos build system | Current cargo-leptos workspace docs use `[[workspace.metadata.leptos]]` in root `Cargo.toml`; shared UI CSS should come from `bominal-ui` |
| FND-05 | Verify dual compilation | `wasm32-unknown-unknown` and `cargo-leptos` are already installed locally, so Phase 1 can verify the real build path |
</phase_requirements>

## Current Repo State

### What Already Works
- `cargo check -p bominal-app` succeeds today.
- `cargo-leptos 0.3.5` is installed.
- `wasm32-unknown-unknown` is installed.
- `tailwindcss` is available locally.
- `cargo check --manifest-path ../bominal-ui/Cargo.toml --features 'leptos train auth'` succeeds.

### What Is Still Wrong for SSR Migration
- `crates/bominal-app/Cargo.toml` still uses `leptos = { version = "0.8", features = ["csr"] }` and `gloo-net`.
- `crates/bominal-app/src/lib.rs` still uses `#[wasm_bindgen(start)]` and `mount_to_body(App)`, which is the wrong entrypoint for islands mode.
- `crates/bominal-app/src/i18n.rs` reads `window.navigator.language()`, which is not SSR-safe.
- `crates/bominal-server/src/routes.rs` still serves SvelteKit assets from `frontend/build`.
- Root `Cargo.toml` does not define any Leptos workspace dependencies or cargo-leptos workspace metadata.

### Why `crates/bominal-frontend` Changes the Discussion
- It contains a real Leptos route/component surface, not an empty dead crate.
- It already models SSR/hydrate-era modules (`app.rs`, `hydrate.rs`, page/components tree).
- It currently fails before compilation because it inherits `leptos*` workspace dependencies that do not exist in the root manifest.
- Its API layer is written around direct server functions, which conflicts with the current roadmap decision to proxy through existing `/api/` endpoints.

**Inference from the codebase:** `crates/bominal-frontend` should be mined as a donor, not revived as-is.

### Why `bominal-ui` Changes the Discussion
- It is already the canonical shared UI crate for Bominal products.
- It exposes ecosystem CSS plus product skins for both `train` and `auth`, which matches this app's main surface split.
- It compiles today with `leptos`, `train`, and `auth` features enabled.
- Its token names (`--lg-*`, provider colors, auth container classes) align with the existing train frontend's design vocabulary.

**Inference from the codebase:** the migration should consume or deterministically derive from `bominal-ui`, not treat `frontend/src/app.css` as the new design-system owner.

## Official Leptos Guidance

### Islands Mode
- The Leptos book's islands guide says islands mode is activated by adding the `islands` feature to the `leptos` dependency and replacing `hydrate_body(...)` with `leptos::mount::hydrate_islands()`.
- The same guide says the server shell should later render hydration scripts with `islands=true`.

**Planning consequence:** the current donor crate's `hydrate_body(App)` path is a useful prototype, but it is not the final islands-ready entrypoint.

### cargo-leptos Workspace Setup
- Current cargo-leptos docs say multi-package workspaces should define Leptos projects in the root manifest under `[[workspace.metadata.leptos]]`.
- The docs also say cargo-leptos builds the lib package with `--no-default-features --features=hydrate` and the server binary with `--no-default-features --features=ssr`.

**Planning consequence:** the root manifest needs workspace metadata and the server crate must expose an `ssr` feature path early enough for the toolchain to target it cleanly.

## Shared UI Guidance

- `bominal-ui` is a standalone Rust crate with features for `leptos`, `train`, and `auth`.
- `src/css/mod.rs` exports `ECOSYSTEM_CSS` and product-specific CSS constants plus `full_css(product)`.
- `src/css/train.rs` and `src/css/auth.rs` map directly onto this product's route/layout split.
- `src/leptos_components/mod.rs` currently exports a shared `Icon` component surface.

**Planning consequence:** Phase 1 should make `bominal-app` depend on `bominal-ui` and treat app-local CSS as a deterministic derivative or bridge from that crate, not as a hand-maintained parallel system.

## Recommended Architecture

### Pattern 1: Single Active App Crate
Use `crates/bominal-app` as the only active app crate in the workspace. Pull reusable routes, components, and CSS from `crates/bominal-frontend` as source material, but do not keep both crates as first-class app implementations.

Why this is the right move:
- `bominal-app` already belongs to the active workspace and is the lowest-friction place to land the migration.
- The roadmap and project docs already assume it is the destination crate.
- The donor crate's architecture needs adaptation anyway, so reviving it intact only delays the convergence work.

### Pattern 2: Compile-First Phase Boundary
Phase 1 should intentionally reduce scope to "the build graph is correct" rather than "pages still work."

Practical consequence:
- Replace the current `bominal-app` entrypoint with a minimal `App`.
- Gate or stub browser-only modules that would otherwise block SSR/hydrate feature compilation.
- Do not port auth flows, stores, or route trees yet.

### Pattern 3: Root cargo-leptos Metadata
Define the multi-package Leptos project in the root manifest:

```toml
[[workspace.metadata.leptos]]
name = "bominal-train"
bin-package = "bominal-server"
lib-package = "bominal-app"
bin-features = ["ssr"]
bin-default-features = false
lib-features = ["hydrate"]
lib-default-features = false
```

Then add the style/asset/output parameters there as Phase 1 build inputs.

### Pattern 4: Canonical CSS From `bominal-ui`
Move the active app's CSS build input under `crates/bominal-app`, but make it a deterministic bridge from `bominal-ui` rather than a fresh source of truth copied from SvelteKit.

Practical consequence:
- Add `bominal-ui` as a dependency for the active app crate.
- Use `bominal-ui` ecosystem + `train` + `auth` CSS as the canonical styling payload.
- Keep the old SvelteKit CSS only as a compatibility diff/reference while parity work proceeds.

## Anti-Patterns to Avoid

- **Do not use a separate `Cargo-leptos.toml` file.** Current workspace docs point to root `Cargo.toml` metadata instead.
- **Do not revive `crates/bominal-frontend` unchanged.** Its direct server-function architecture conflicts with the current plan, and it is not even parseable against today's root manifest.
- **Do not treat `frontend/src/app.css` as the new source of truth.** The shared crate `bominal-ui` now owns the canonical UI direction.
- **Do not try to preserve current auth page behavior in Phase 1.** That turns foundation work into a disguised Phase 4.
- **Do not delete `crates/bominal-frontend` immediately.** `Makefile` and human migration work still need it as reference material.
- **Do not defer the server `ssr` feature path until Phase 8.** That would hide a build integration problem behind later route work.

## Common Pitfalls

### Pitfall 1: Wrong cargo-leptos Config Location
**What goes wrong:** Planning around `Cargo-leptos.toml` or package-local metadata in a multi-package workspace.
**Why it happens:** Older draft plans and examples often assume single-package setups.
**How to avoid:** Use `[[workspace.metadata.leptos]]` in the root manifest for this repo.

### Pitfall 2: Browser APIs Leaking Into SSR
**What goes wrong:** `window`, `navigator`, `gloo-net`, or `wasm_bindgen(start)` code remains in the default app path and breaks `--features ssr`.
**How to avoid:** Gate browser-only code or stub it out in Phase 1. Keep the minimal shell SSR-safe.

### Pitfall 3: Treating Donor Code as Architecture
**What goes wrong:** The existence of `crates/bominal-frontend` tempts the migration to inherit its direct server-function API surface.
**How to avoid:** Treat it as a catalog of reusable UI and route work, not as the architecture source of truth.

### Pitfall 4: Deleting the Donor Crate Too Early
**What goes wrong:** The repo loses the best available Leptos page/component reference while build scripts and humans still depend on it.
**How to avoid:** Retire it from active ownership in Phase 1, then delete/archive it during a later cleanup phase once the migration has absorbed the needed code.

### Pitfall 5: Shared UI in Name Only
**What goes wrong:** The app says it uses `bominal-ui` but still hand-maintains a diverging local CSS system.
**How to avoid:** Make `bominal-ui` explicit in manifests, canonical refs, and build verification. Treat local stylesheets as generated/bridged artifacts, not the owner.

## Validation Architecture

### Build Checks
| Check | Purpose |
|-------|---------|
| `cargo check -p bominal-app --features ssr` | Confirms the app crate compiles on the server side |
| `cargo check -p bominal-app --target wasm32-unknown-unknown --features hydrate` | Confirms the app crate compiles for the WASM client |
| `cargo check -p bominal-server --features ssr` | Confirms the server crate exposes a valid cargo-leptos build path |
| `cargo leptos build` | Confirms the integrated workspace build path is real, not just individually checkable |
| `cargo check --manifest-path ../bominal-ui/Cargo.toml --features 'leptos train auth'` | Confirms the shared UI crate is available and compilable for the intended product skins |

### Evidence Checks
| Check | Purpose |
|-------|---------|
| `cargo check --manifest-path crates/bominal-frontend/Cargo.toml --features ssr` currently fails | Confirms the donor crate is not a drop-in active target today |
| `rg "frontend/build|npm|bominal-frontend"` in build scripts | Identifies later cleanup work that should not be hidden inside Phase 1 |

## Environment Availability

- `cargo-leptos 0.3.5` available locally
- `wasm32-unknown-unknown` target installed
- `tailwindcss` binary installed
- `bominal-ui` verified locally with `leptos train auth` features
- Existing `frontend/package.json` still defines the SvelteKit build toolchain, so Phase 1 should avoid pretending Node is already gone

## Sources

### Primary repo evidence
- `Cargo.toml`
- `crates/bominal-app/Cargo.toml`
- `crates/bominal-app/src/lib.rs`
- `crates/bominal-app/src/api.rs`
- `crates/bominal-app/src/i18n.rs`
- `crates/bominal-server/Cargo.toml`
- `crates/bominal-server/src/routes.rs`
- `crates/bominal-frontend/Cargo.toml`
- `crates/bominal-frontend/src/app.rs`
- `crates/bominal-frontend/src/hydrate.rs`
- `crates/bominal-frontend/src/api/mod.rs`
- `../bominal-ui/Cargo.toml`
- `../bominal-ui/.planning/PROJECT.md`
- `../bominal-ui/.planning/ROADMAP.md`
- `../bominal-ui/src/lib.rs`
- `../bominal-ui/src/css/mod.rs`
- `../bominal-ui/src/css/ecosystem.rs`
- `../bominal-ui/src/css/train.rs`
- `../bominal-ui/src/css/auth.rs`
- `frontend/src/app.css`
- `frontend/package.json`
- `dev-build.sh`
- `Dockerfile`
- `Makefile`

### Primary external docs
- Leptos islands guide: https://book.leptos.dev/islands.html
- cargo-leptos workspace setup: https://github.com/leptos-rs/cargo-leptos#workspace-setup

## Metadata

**Confidence breakdown:**
- Repo-state assessment: HIGH
- cargo-leptos config recommendation: HIGH
- donor-crate salvage recommendation: HIGH
- shared UI dependency direction: HIGH
- exact manifest dependency list: MEDIUM-HIGH (verify during execution with `cargo check`)

**Research date:** 2026-03-27
**Valid until:** Re-check if the project upgrades beyond Leptos 0.8 / cargo-leptos 0.3.x
