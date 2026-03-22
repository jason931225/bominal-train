# Enable WASM Hydration — Fix All Broken Frontend Interactivity

## Context

The Leptos frontend is deployed SSR-only — no WASM bundle is built or served. This means **every `on:click`, `on:input`, `on:change` handler is dead**. Only `ActionForm` (plain HTML POST) and `<a>` links work. The entire search → select trains → create task flow is non-functional, along with modals, toggles, tab switches, and passkey ceremonies.

Rewriting 50+ handlers in vanilla JS via `interop.ts` is infeasible and fights the framework. The correct fix is to **enable Leptos WASM hydration** so the browser takes over reactivity after SSR renders the initial HTML.

Additionally, the station search uses a static `<select>` dropdown despite a sophisticated 8-strategy fuzzy matching algorithm (`station_search.rs`) already wired to `/api/stations/{provider}/suggest`. This should be connected.

---

## Phase 1 — Enable WASM Hydration Build

### 1a. Workspace Cargo.toml — add `hydrate` feature

**File**: `Cargo.toml` (root)

```toml
# Before
leptos = { version = "0.8", features = ["ssr"] }
leptos_router = { version = "0.8", features = ["ssr"] }

# After
leptos = { version = "0.8", features = ["ssr", "hydrate"] }
leptos_router = { version = "0.8", features = ["ssr", "hydrate"] }
```

### 1b. Frontend Cargo.toml — add `cdylib` crate type

**File**: `crates/bominal-frontend/Cargo.toml`

Add after `[dependencies]` section:

```toml
[lib]
crate-type = ["rlib", "cdylib"]
```

(`rlib` for the server dep, `cdylib` for WASM target)

### 1c. Create WASM entry point

**New file**: `crates/bominal-frontend/src/hydrate.rs`

```rust
use wasm_bindgen::prelude::wasm_bindgen;

#[wasm_bindgen(start)]
pub fn hydrate() {
    use crate::app::App;
    leptos::prelude::hydrate_body(App);
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

Or use `wasm-bindgen`'s `--target web` output which produces a JS loader.

---

## Phase 2 — Build Pipeline

### 2a. Dockerfile — add WASM build step

**File**: `Dockerfile`

After the Tailwind CSS compilation step (line 67-68), add:

```dockerfile
# Build WASM hydration bundle
RUN cargo build --profile wasm-release --target wasm32-unknown-unknown -p bominal-frontend --lib

# Install wasm-bindgen-cli for JS bindings generation
RUN cargo install wasm-bindgen-cli --version 0.2.*

# Generate JS/WASM bindings
RUN wasm-bindgen \
    target/wasm32-unknown-unknown/wasm-release/bominal_frontend.wasm \
    --out-dir /app/pkg \
    --target web \
    --no-typescript
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

## Phase 3 — Fix Compilation Issues

When building for both `wasm32` and native targets, some dependencies won't compile for WASM (e.g., `leptos_axum`, `bominal-service`, `axum`). These are server-only.

### 3a. Gate server-only deps behind `cfg`

**File**: `crates/bominal-frontend/Cargo.toml`

```toml
[target.'cfg(not(target_arch = "wasm32"))'.dependencies]
leptos_axum.workspace = true
axum.workspace = true
bominal-service = { path = "../bominal-service" }
```

### 3b. Gate server-only code

**File**: `crates/bominal-frontend/src/lib.rs`

```rust
#[cfg(not(target_arch = "wasm32"))]
pub mod api;   // server functions only compile on server

#[cfg(target_arch = "wasm32")]
mod hydrate;
```

**File**: `crates/bominal-frontend/src/app.rs` — gate `api` imports:
```rust
#[cfg(not(target_arch = "wasm32"))]
use crate::api;
```

The `#[server]` macro already handles this — server function bodies are stripped on WASM. But the imports of server-only types (DbPool, EncryptionKey, etc.) must be gated.

---

## Phase 4 — Remove Dead Interop Workarounds

Once WASM hydration works, remove the SSR workarounds we added:

**File**: `crates/bominal-frontend/ts/interop.ts`
- Remove `__doPasskeyRegister()`, `__doPasskeyLogin()`, and the DOMContentLoaded auto-wiring (these are now handled by WASM)

**File**: `crates/bominal-frontend/src/pages/auth/add_passkey_page.rs`
- Restore `on:click` handler with WASM passkey ceremony (now works because WASM hydrates)
- Change `<a href="/home">` skip link back to button with `on:click` if preferred

**File**: `crates/bominal-frontend/src/pages/auth/login_page.rs`
- Restore `on:click` passkey login handler (WASM now active)

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
| `Cargo.toml` | Add `hydrate` feature to leptos/leptos_router |
| `crates/bominal-frontend/Cargo.toml` | Add `cdylib` crate type, gate server deps |
| `crates/bominal-frontend/src/lib.rs` | Gate `api` module, add `hydrate` module |
| `crates/bominal-frontend/src/hydrate.rs` | **New** — WASM entry point |
| `crates/bominal-frontend/src/app.rs` | Add WASM `<script>` tag to shell |
| `crates/bominal-frontend/src/api/*.rs` | Gate server-only imports behind `cfg` |
| `crates/bominal-server/src/routes.rs` | Serve `/pkg/*`, update CSP |
| `Dockerfile` | Add WASM build + wasm-bindgen step |
| `crates/bominal-frontend/ts/interop.ts` | Remove passkey workarounds |
| `crates/bominal-frontend/src/pages/auth/add_passkey_page.rs` | Restore WASM handlers |
| `crates/bominal-frontend/src/pages/auth/login_page.rs` | Restore WASM handlers |

---

## Verification

1. `cargo check --workspace` — passes (native)
2. `cargo build --target wasm32-unknown-unknown -p bominal-frontend --lib` — produces WASM
3. `cargo test -p bominal-domain -p bominal-frontend -p bominal-server` — all pass
4. `cargo clippy --workspace` — zero warnings
5. `docker compose up -d --build app` — container starts
6. Browser: open `/auth/login` → password toggle works, passkey button works
7. Browser: open `/search` → date picker opens, passengers adjust, trains selectable
8. Browser: open `/settings` → theme toggles, card form works
9. Browser: open DevTools console → no CSP violations, WASM loads
