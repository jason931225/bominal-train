# Enable WASM Hydration — Fix All Broken Frontend Interactivity

## Context

The Leptos frontend is deployed SSR-only — no WASM bundle is built or served. This means **every `on:click`, `on:input`, `on:change` handler is dead**. Only `ActionForm` (plain HTML POST) and `<a>` links work. The entire search → select trains → create task flow is non-functional, along with modals, toggles, tab switches, and passkey ceremonies.

Rewriting 50+ handlers in vanilla JS via `interop.ts` is infeasible and fights the framework. The correct fix is to **enable Leptos WASM hydration** so the browser takes over reactivity after SSR renders the initial HTML.

Additionally, the station search uses a static `<select>` dropdown despite a sophisticated 8-strategy fuzzy matching algorithm (`station_search.rs`) already wired to `/api/stations/{provider}/suggest`. This should be connected.

---

## Phase 0 — Move Shared Types to `bominal-domain`

Several DTO types (`CardInfo`, `TrainInfo`, `StationInfo`, `ProviderInfo`, `ReservationInfo`,
`CreateTaskInput`, `UpdateTaskInput`) were defined in `bominal-service` and re-exported by the
frontend. Because `bominal-service` depends on server-only crates (`sqlx`, `bominal-db`, etc.),
these types would not be available when building the frontend for WASM.

**Fix**: Move these pure serde structs to `bominal-domain/src/dto.rs` (which compiles for WASM).
Both `bominal-service` and `bominal-frontend` re-export from `bominal_domain::dto`.

### 0a. New file: `crates/bominal-domain/src/dto.rs`

Contains all shared DTO types with only `serde`, `chrono`, `uuid` dependencies.

### 0b. Update `bominal-domain/src/lib.rs`

```rust
pub mod dto;
```

### 0c. Update `bominal-service` files

Each service file changes from defining the struct locally to:
```rust
pub use bominal_domain::dto::CardInfo;
// etc.
```

### 0d. Update `bominal-frontend/src/api/*.rs` re-exports

```rust
pub use bominal_domain::dto::CardInfo;      // was: bominal_service::cards::CardInfo
pub use bominal_domain::dto::{StationInfo, TrainInfo};  // was: bominal_service::search::*
// etc.
```

---

## Phase 1 — Enable WASM Hydration Build

### 1a. Cargo feature flags (CRITICAL: `ssr` and `hydrate` are mutually exclusive)

In Leptos 0.8, `ssr` and `hydrate` cannot both be active in the same compilation target.
Use Cargo features on the frontend crate to conditionally activate one or the other.

**File**: `Cargo.toml` (root workspace) — **remove** features from leptos/leptos_router:

```toml
# Before
leptos = { version = "0.8", features = ["ssr"] }
leptos_router = { version = "0.8", features = ["ssr"] }

# After — features are set per-crate, not at workspace level
leptos = { version = "0.8" }
leptos_router = { version = "0.8" }
```

**File**: `crates/bominal-frontend/Cargo.toml` — add features and optional deps:

```toml
[lib]
crate-type = ["rlib", "cdylib"]

[features]
ssr = ["leptos/ssr", "leptos_router/ssr", "dep:leptos_axum", "dep:axum", "dep:bominal-service"]
hydrate = ["leptos/hydrate", "leptos_router/hydrate"]

[dependencies]
bominal-domain = { path = "../bominal-domain", default-features = false }
bominal-service = { path = "../bominal-service", optional = true }
leptos_axum = { workspace = true, optional = true }
axum = { workspace = true, optional = true }
# ... rest unchanged
```

**File**: `crates/bominal-server/Cargo.toml` — activate `ssr` feature:

```toml
bominal-frontend = { path = "../bominal-frontend", features = ["ssr"] }
leptos = { workspace = true, features = ["ssr"] }
```

### 1b. Feature-gate crypto in `bominal-domain` for WASM compatibility

`bominal-domain` depends on `argon2` and `aes-gcm` (crypto). The frontend only uses `i18n`,
`station_search`, `task`, and `auth` types — not crypto. Gate crypto deps:

**File**: `crates/bominal-domain/Cargo.toml`

```toml
[features]
default = ["crypto"]
crypto = ["dep:argon2", "dep:aes-gcm", "dep:base64", "dep:rand"]

[dependencies]
argon2 = { workspace = true, optional = true }
aes-gcm = { workspace = true, optional = true }
base64 = { workspace = true, optional = true }
rand = { workspace = true, optional = true }
```

**File**: `crates/bominal-domain/src/lib.rs`

```rust
#[cfg(feature = "crypto")]
pub mod crypto;
```

### 1c. Create WASM entry point

**New file**: `crates/bominal-frontend/src/hydrate.rs`

```rust
use wasm_bindgen::prelude::wasm_bindgen;

#[wasm_bindgen(start)]
pub fn hydrate() {
    use crate::app::App;
    leptos::mount::hydrate_body(App);
}
```

Export from lib.rs:
```rust
#[cfg(target_arch = "wasm32")]
mod hydrate;
```

### 1d. HTML shell — add WASM script tag

**File**: `crates/bominal-frontend/src/app.rs` (shell function)

Add after the `interop.js` script tag:
```html
<script type="module">
    import init from '/pkg/bominal_frontend.js';
    init();
</script>
```

---

## Phase 2 — Build Pipeline

### 2a. Dockerfile — add WASM build step

**File**: `Dockerfile`

After the Tailwind CSS compilation step, add:

```dockerfile
# Build WASM hydration bundle
RUN cargo build --profile wasm-release --target wasm32-unknown-unknown \
    -p bominal-frontend --lib --features hydrate --no-default-features

# Install wasm-bindgen-cli (version MUST match wasm-bindgen crate exactly)
RUN cargo install wasm-bindgen-cli --version $(grep -A1 'name = "wasm-bindgen"' Cargo.lock | grep version | head -1 | cut -d'"' -f2)

# Generate JS/WASM bindings
RUN wasm-bindgen \
    target/wasm32-unknown-unknown/wasm-release/bominal_frontend.wasm \
    --out-dir /app/pkg \
    --target web \
    --no-typescript

# Optimize WASM bundle size
RUN apt-get update && apt-get install -y --no-install-recommends binaryen && rm -rf /var/lib/apt/lists/*
RUN wasm-opt -Os /app/pkg/bominal_frontend_bg.wasm -o /app/pkg/bominal_frontend_bg.wasm
```

In the runtime stage:
```dockerfile
COPY --from=builder /app/pkg /app/pkg
```

### 2b. Serve WASM assets

**File**: `crates/bominal-server/src/routes.rs`

Add static route for WASM bundle (after interop.js route):
```rust
.nest_service("/pkg", tower_http::services::ServeDir::new("pkg"))
```

### 2c. CSP update

**File**: `crates/bominal-server/src/routes.rs`

Add `'wasm-unsafe-eval'` to `script-src` in CSP header to allow WASM execution:
```
script-src 'self' 'unsafe-inline' 'wasm-unsafe-eval' https://js.evervault.com
```

---

## Phase 3 — Gate Server-Only Code

The `#[server]` macro strips function bodies on WASM and replaces them with HTTP-calling stubs.
So server functions **remain available** on WASM. Do NOT gate the `api` module itself.

Instead, gate only the non-`#[server]` helper functions and their server-only imports:

### 3a. Gate helper functions in API files

**File**: `crates/bominal-frontend/src/api/auth.rs`

```rust
#[cfg(feature = "ssr")]
pub(crate) fn extract_session_id() -> Option<String> { ... }

#[cfg(feature = "ssr")]
async fn create_session_and_set_cookie(...) { ... }
```

**File**: `crates/bominal-frontend/src/api/tasks.rs`

```rust
#[cfg(feature = "ssr")]
pub(crate) async fn require_auth() -> Result<(bominal_service::DbPool, Uuid), ServerFnError> { ... }
```

---

## Phase 4 — Remove Dead Interop Workarounds

Once WASM hydration works, remove the SSR workarounds we added:

**File**: `crates/bominal-frontend/ts/interop.ts`

Remove (ceremony orchestration — now handled by WASM):
- `__doPasskeyRegister()` function
- `__doPasskeyLogin()` function
- `DOMContentLoaded` auto-wiring listener

Keep (still called by WASM via wasm-bindgen):
- `__startPasskeyRegistration()` — WebAuthn browser API wrapper
- `__startPasskeyLogin()` — WebAuthn browser API wrapper
- `__submitCard()` — Evervault card encryption bridge
- `__evEncrypt()` — Evervault encryption
- `__startViewTransition()` — View transitions helper
- `toBase64url()` — Base64url utility

**File**: `crates/bominal-frontend/src/pages/auth/add_passkey_page.rs`
- Restore `on:click` handler calling `crate::api::passkey::do_passkey_register()` via WASM
- Replace `id="btn-passkey-register"` button (was auto-wired) with reactive `on:click`
- Add error signal for displaying passkey registration errors

**File**: `crates/bominal-frontend/src/pages/auth/login_page.rs`
- Restore `on:click` passkey login handler calling `crate::api::passkey::do_passkey_login()`
- Replace `id="btn-passkey-login"` button with reactive `on:click`

---

## Phase 5 — Station Autocomplete (stretch goal)

Replace the static `<select>` station picker with a searchable autocomplete that calls the existing suggest API.

**Backend**: Already done — `GET /api/stations/{provider}/suggest?q=...` in `search.rs:117-172`

**Frontend change**: `crates/bominal-frontend/src/pages/search_panel.rs`

Replace the `<select>` dropdown (lines 629-669) with a text input + dropdown that:
1. Calls `/api/stations/{provider}/suggest?q={input}` on each keystroke (debounced)
2. Renders ranked suggestions with match source
3. Applies autocorrect on submit

This can be implemented as a new component `StationInput` or as a JS function in `interop.ts` that enhances a plain `<input>`.

---

## Files Modified

| File | Changes |
|------|---------|
| `Cargo.toml` | Remove features from leptos/leptos_router workspace deps |
| `crates/bominal-domain/Cargo.toml` | Feature-gate crypto deps |
| `crates/bominal-domain/src/lib.rs` | Gate `crypto` module, add `dto` module |
| `crates/bominal-domain/src/dto.rs` | **New** — shared DTO types |
| `crates/bominal-frontend/Cargo.toml` | Add `[features]` (ssr/hydrate), `cdylib`, optional deps |
| `crates/bominal-frontend/src/lib.rs` | Add `hydrate` module (cfg-gated) |
| `crates/bominal-frontend/src/hydrate.rs` | **New** — WASM entry point |
| `crates/bominal-frontend/src/app.rs` | Add WASM `<script>` tag to shell |
| `crates/bominal-frontend/src/api/auth.rs` | Update type re-export, gate helpers with `#[cfg(feature = "ssr")]` |
| `crates/bominal-frontend/src/api/tasks.rs` | Update type re-exports, gate `require_auth` |
| `crates/bominal-frontend/src/api/{cards,search,providers,reservations}.rs` | Update type re-exports to `bominal_domain::dto` |
| `crates/bominal-server/Cargo.toml` | Activate `bominal-frontend/ssr`, add `leptos/ssr` |
| `crates/bominal-server/src/routes.rs` | Serve `/pkg/*`, update CSP with `wasm-unsafe-eval` |
| `crates/bominal-service/src/{cards,search,providers,reservations,tasks}.rs` | Re-export types from `bominal_domain::dto` |
| `Dockerfile` | WASM build step, wasm-bindgen (pinned), wasm-opt, copy pkg |
| `crates/bominal-frontend/ts/interop.ts` | Remove passkey ceremony orchestration (keep WebAuthn wrappers) |
| `crates/bominal-frontend/src/pages/auth/add_passkey_page.rs` | Restore WASM `on:click` handler |
| `crates/bominal-frontend/src/pages/auth/login_page.rs` | Restore WASM `on:click` handler |

---

## Verification

1. `cargo check -p bominal-frontend --features ssr` — native SSR compiles
2. `cargo check -p bominal-frontend --features hydrate --target wasm32-unknown-unknown` — WASM compiles
3. `cargo check --workspace` — full workspace passes
4. `cargo test -p bominal-domain -p bominal-server` — all pass
5. `cargo clippy --workspace` — zero warnings
6. `docker compose up -d --build app` — container starts
7. Browser: open `/auth/login` → password toggle works, passkey button works
8. Browser: open `/search` → date picker opens, passengers adjust, trains selectable
9. Browser: open `/settings` → theme toggles, card form works
10. Browser: open DevTools console → no CSP violations, WASM loads
