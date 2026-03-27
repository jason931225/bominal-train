# Phase 1: Foundation - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md. This log preserves the alternatives considered.

**Date:** 2026-03-27
**Phase:** 01-foundation
**Areas discussed:** Active app crate strategy, foundation scope boundary, cargo-leptos integration path, shared UI source, CSS staging

---

## Active App Crate Strategy

| Option | Description | Selected |
|--------|-------------|----------|
| Keep `bominal-app` active, use `bominal-frontend` as donor only | Preserves the roadmap direction, keeps work on the crate already in the workspace, and mines the dormant SSR prototype for reusable code | ✓ |
| Revive `bominal-frontend` as the active app crate | Faster if the prototype were already clean, but it currently sits outside the workspace and conflicts with the chosen API architecture | |
| Run both crates in parallel | Creates split-brain ownership and doubles the migration surface | |

**Selection mode:** Auto-selected from codebase evidence
**Notes:** `crates/bominal-frontend` is not dead, but it is also not a clean execution target. It already contains a large Leptos prototype, yet it fails to parse today because the workspace is missing the shared Leptos dependencies it expects, and its API layer is built around direct server functions rather than the current `/api/` proxy decision.

---

## Foundation Scope Boundary

| Option | Description | Selected |
|--------|-------------|----------|
| Minimal compile-first shell | Convert `bominal-app` to SSR/hydrate feature gates, keep the app compiling, and defer full page parity to later phases | ✓ |
| Preserve current auth pages during Phase 1 | Tries to keep visible functionality immediately, but pulls Phase 4 work into the foundation pass | |

**Selection mode:** Auto-selected from roadmap boundary
**Notes:** The roadmap already reserves core infrastructure for Phase 2, shell/navigation for Phase 3, and auth pages for Phase 4. Trying to preserve the current CSR auth surface during Phase 1 would collapse those boundaries and make the foundation phase sprawl.

---

## cargo-leptos Integration Path

| Option | Description | Selected |
|--------|-------------|----------|
| Use root `Cargo.toml` workspace metadata and establish an early `ssr` server build path | Matches current cargo-leptos workspace docs and keeps Phase 1's build-orchestration goal honest | ✓ |
| Use a separate `Cargo-leptos.toml` file and defer real build verification | Based on stale assumptions from older draft plans; no longer matches current cargo-leptos guidance | |
| Delay `cargo leptos` setup until Phase 8 server integration | Pushes a Phase 1 risk into a later integration phase | |

**Selection mode:** Auto-selected from official docs + repo state
**Notes:** Current cargo-leptos workspace guidance uses `[[workspace.metadata.leptos]]` in the root manifest. The server does not need full Leptos route integration yet, but it does need a real `ssr` feature path so `cargo leptos build` targets the actual server crate instead of a future imaginary one.

---

## Shared UI Source

| Option | Description | Selected |
|--------|-------------|----------|
| Use `bominal-ui` as canonical source of truth | Keeps train aligned with the rest of the Bominal product ecosystem and avoids local token drift | ✓ |
| Keep SvelteKit CSS as the canonical source | Faster short-term, but locks train into product-local UI drift | |
| Blend both equally | Creates unclear ownership and parity conflicts | |

**Selection mode:** User decision
**Notes:** `../bominal-ui` already exists as a Rust-first shared UI crate with ecosystem CSS plus `train` and `auth` product skins. The migration plan should consume or derive from this crate, not treat `frontend/src/app.css` as the long-term source of truth.

---

## CSS Staging

| Option | Description | Selected |
|--------|-------------|----------|
| Carry build inputs forward from `bominal-ui`, defer parity verification | Materialize the shared UI CSS in the active app build path now, but save visual fidelity work for Phase 9 | ✓ |
| Defer all CSS until later | Keeps Phase 1 smaller, but leaves the build pipeline incomplete | |

**Selection mode:** Auto-selected from roadmap dependency flow
**Notes:** The style pipeline is a build dependency in Phase 1, even though visual verification belongs in Phase 9. The plan should wire canonical `bominal-ui` CSS inputs now without promising pixel-perfect parity yet.

---

## Claude's Discretion

- Exact manifest feature layout for `bominal-app` and `bominal-server`
- Whether dormant modules are gated out with `cfg` flags or replaced with temporary stubs
- Exact minimal `App` markup and CSS import structure

## Deferred Ideas

- Deleting or archiving `crates/bominal-frontend` entirely after its reusable code is absorbed and no active scripts refer to it
- Replacing Axum's SvelteKit `ServeDir(frontend/build)` fallback with `leptos_axum` routing in Phase 8
