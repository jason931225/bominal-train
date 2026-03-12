# Frontend Rewrite + Feature Completion

> **Supersedes**: Master plan (`fizzy-roaming-sparkle.md`) sections 4.1 (theme system), 4.5 (token system), and Phase 5.1 (build pipeline's separate `lightningcss` step). The 4-route WebAuthn pattern here supersedes the master plan's 3-route pattern.

## Context

The Leptos SSR frontend is functional but visually broken and structurally divergent from the React prototype. Pages render with correct data but wrong layout, raw Rust code leaks into the search page, interactive components were built but never wired into pages, and the card encryption pipeline uses AES-256-GCM instead of Evervault. WebAuthn (passkeys) is planned but unimplemented.

This spec covers a layered rewrite: theme foundation first, then page-by-page structural fixes, then missing features (WebAuthn, Evervault JS SDK, auto-pay).

## Design Decisions

### Theme System

**Kill all existing theme/palette variants.** The current CSS has 5 theme/palette combinations (1 light default + 1 dark default + 3 dark palettes: transit-slate, night-teal, warm-platform) plus colorblind overrides. Remove all of them.

Replace with two themes + light/dark mode:

1. **Rosewood Dusk** (default) — warm rose-tinted neutrals, nostalgic
2. **Clear Sky** — soft blue, airy, similar to current Vite prototype but muted

Each theme has light + dark variants. User picks theme + mode. No palette sub-variants.

**Persistence**: `localStorage` with two keys:
- `bominal-theme`: `rosewood` | `clear-sky`
- `bominal-mode`: `light` | `dark`

**CSS architecture**: Theme applied via `html[data-theme][data-mode]` attributes. Four combinations total:
- `data-theme="rosewood" data-mode="light"`
- `data-theme="rosewood" data-mode="dark"`
- `data-theme="clear-sky" data-mode="light"`
- `data-theme="clear-sky" data-mode="dark"`

### Color Tokens

Token names adopt the existing `--color-` prefix convention used throughout all components (e.g., `--color-bg-primary`, `--color-text-primary`). This avoids updating every class reference in every component. The `--glass-*` tokens retain their existing naming (no `--color-` prefix) since they are consumed by `.glass-panel` / `.glass-card` CSS utility classes, not by individual component class references. The mapping below shows the semantic token name and its value per theme combo.

**New tokens**: Several tokens below (e.g., `--color-brand-text`, `--color-bg-primary-end`, `--color-status-*-bg`) are new additions not present in the current `main.css`. They will be adopted during Layer 2 page rewrites. Existing components will continue working with the tokens they already reference.

#### Rosewood Dusk — Light
| Token | Value |
|-------|-------|
| `--color-bg-primary` | `#fdf9f6` |
| `--color-bg-primary-end` | `#f8f2ee` (gradient end) |
| `--color-bg-elevated` | `#fff` |
| `--color-bg-sunken` | `rgba(248,242,238,0.8)` |
| `--color-text-primary` | `#3a2e28` |
| `--color-text-secondary` | `#5a4a42` |
| `--color-text-tertiary` | `#8a6f62` |
| `--color-text-disabled` | `#b8a89a` |
| `--color-brand-primary` | `rgba(176,120,104,0.12)` (tinted glass) |
| `--color-brand-text` | `#8a6050` |
| `--color-brand-border` | `rgba(176,120,104,0.18)` |
| `--color-border-default` | `rgba(176,120,104,0.1)` |
| `--color-border-subtle` | `rgba(176,120,104,0.06)` |
| `--color-border-focus` | `rgba(176,120,104,0.3)` |
| `--color-interactive-hover` | `rgba(176,120,104,0.06)` |
| `--glass-panel-bg` | `rgba(253,249,246,0.72)` |
| `--glass-card-bg` | `rgba(253,249,246,0.65)` |
| `--glass-border` | `rgba(255,255,255,0.55)` |
| `--glass-shadow` | `0 16px 36px rgba(138,111,98,0.06)` |
| `--glass-card-shadow` | `0 12px 28px rgba(138,111,98,0.05)` |
| `--color-status-success` | `#5a7a62` |
| `--color-status-success-bg` | `rgba(122,154,130,0.15)` |
| `--color-status-error` | `#a85a4a` |
| `--color-status-error-bg` | `rgba(168,90,74,0.1)` |
| `--color-status-warning` | `#9a7a3a` |
| `--color-status-warning-bg` | `rgba(154,122,58,0.1)` |
| `--color-mesh-1` | `rgba(176,120,104,0.12)` |
| `--color-mesh-2` | `rgba(122,154,130,0.1)` |
| `--color-mesh-3` | `rgba(138,122,154,0.08)` |

#### Rosewood Dusk — Dark (Softer)
| Token | Value |
|-------|-------|
| `--color-bg-primary` | `#1e1a17` |
| `--color-bg-primary-end` | `#262220` |
| `--color-bg-elevated` | `#2a2622` |
| `--color-bg-sunken` | `rgba(30,26,23,0.6)` |
| `--color-text-primary` | `#d8ccc4` |
| `--color-text-secondary` | `#c8b8ac` |
| `--color-text-tertiary` | `#9a8c80` |
| `--color-text-disabled` | `#6a6058` |
| `--color-brand-primary` | `rgba(196,138,122,0.12)` |
| `--color-brand-text` | `#c49a8c` |
| `--color-brand-border` | `rgba(196,138,122,0.15)` |
| `--color-border-default` | `rgba(160,136,120,0.08)` |
| `--color-border-subtle` | `rgba(160,136,120,0.05)` |
| `--color-border-focus` | `rgba(196,138,122,0.25)` |
| `--color-interactive-hover` | `rgba(196,138,122,0.08)` |
| `--glass-panel-bg` | `rgba(38,34,32,0.75)` |
| `--glass-card-bg` | `rgba(38,34,32,0.65)` |
| `--glass-border` | `rgba(160,136,120,0.1)` |
| `--glass-shadow` | `0 16px 36px rgba(0,0,0,0.2)` |
| `--glass-card-shadow` | `0 12px 28px rgba(0,0,0,0.15)` |
| `--color-status-success` | `#a4c8ac` |
| `--color-status-success-bg` | `rgba(138,170,146,0.15)` |
| `--color-status-error` | `#d4736a` |
| `--color-status-error-bg` | `rgba(200,110,90,0.12)` |
| `--color-status-warning` | `#d4aa6a` |
| `--color-status-warning-bg` | `rgba(200,160,90,0.12)` |
| `--color-mesh-1` | `rgba(196,138,122,0.06)` |
| `--color-mesh-2` | `rgba(138,170,146,0.04)` |
| `--color-mesh-3` | `rgba(154,138,170,0.04)` |

#### Clear Sky — Light
| Token | Value |
|-------|-------|
| `--color-bg-primary` | `#f6f9fd` |
| `--color-bg-primary-end` | `#edf3fa` |
| `--color-bg-elevated` | `#fff` |
| `--color-bg-sunken` | `rgba(237,243,250,0.8)` |
| `--color-text-primary` | `#1e2e42` |
| `--color-text-secondary` | `#3a4a5e` |
| `--color-text-tertiary` | `#5a7ab0` |
| `--color-text-disabled` | `#a0b0c8` |
| `--color-brand-primary` | `rgba(70,110,180,0.1)` |
| `--color-brand-text` | `#4a6eaa` |
| `--color-brand-border` | `rgba(70,110,180,0.12)` |
| `--color-border-default` | `rgba(70,110,180,0.08)` |
| `--color-border-subtle` | `rgba(70,110,180,0.05)` |
| `--color-border-focus` | `rgba(70,110,180,0.25)` |
| `--color-interactive-hover` | `rgba(70,110,180,0.06)` |
| `--glass-panel-bg` | `rgba(246,249,253,0.72)` |
| `--glass-card-bg` | `rgba(246,249,253,0.65)` |
| `--glass-border` | `rgba(255,255,255,0.55)` |
| `--glass-shadow` | `0 16px 36px rgba(60,90,140,0.06)` |
| `--glass-card-shadow` | `0 12px 28px rgba(60,90,140,0.05)` |
| `--color-status-success` | `#3a8a60` |
| `--color-status-success-bg` | `rgba(60,150,110,0.1)` |
| `--color-status-error` | `#a85040` |
| `--color-status-error-bg` | `rgba(190,80,60,0.1)` |
| `--color-status-warning` | `#9a7a30` |
| `--color-status-warning-bg` | `rgba(160,130,50,0.1)` |
| `--color-mesh-1` | `rgba(80,120,200,0.08)` |
| `--color-mesh-2` | `rgba(120,100,200,0.05)` |
| `--color-mesh-3` | `rgba(60,150,110,0.04)` |

#### Clear Sky — Dark
| Token | Value |
|-------|-------|
| `--color-bg-primary` | `#10161e` |
| `--color-bg-primary-end` | `#182030` |
| `--color-bg-elevated` | `#1e2838` |
| `--color-bg-sunken` | `rgba(16,22,30,0.6)` |
| `--color-text-primary` | `#c0d0e4` |
| `--color-text-secondary` | `#a0b4cc` |
| `--color-text-tertiary` | `#6a8aba` |
| `--color-text-disabled` | `#4a5a70` |
| `--color-brand-primary` | `rgba(80,120,190,0.1)` |
| `--color-brand-text` | `#7aa0d0` |
| `--color-brand-border` | `rgba(80,120,190,0.08)` |
| `--color-border-default` | `rgba(80,110,160,0.06)` |
| `--color-border-subtle` | `rgba(80,110,160,0.04)` |
| `--color-border-focus` | `rgba(80,120,190,0.2)` |
| `--color-interactive-hover` | `rgba(80,120,190,0.08)` |
| `--glass-panel-bg` | `rgba(24,32,48,0.75)` |
| `--glass-card-bg` | `rgba(24,32,48,0.65)` |
| `--glass-border` | `rgba(80,110,160,0.08)` |
| `--glass-shadow` | `0 16px 36px rgba(0,0,0,0.2)` |
| `--glass-card-shadow` | `0 12px 28px rgba(0,0,0,0.15)` |
| `--color-status-success` | `#80c0a0` |
| `--color-status-success-bg` | `rgba(80,170,120,0.12)` |
| `--color-status-error` | `#d08070` |
| `--color-status-error-bg` | `rgba(200,100,80,0.12)` |
| `--color-status-warning` | `#d0aa60` |
| `--color-status-warning-bg` | `rgba(200,160,80,0.12)` |
| `--color-mesh-1` | `rgba(70,110,180,0.05)` |
| `--color-mesh-2` | `rgba(120,100,200,0.03)` |
| `--color-mesh-3` | `rgba(60,150,110,0.03)` |

### Button Style

All primary action buttons use **tinted glass** style:
- Background: `var(--color-brand-primary)` (translucent)
- Text: `var(--color-brand-text)`
- Border: `1px solid var(--color-brand-border)`
- Hover: increase opacity by ~50%
- Active: `scale(0.97)`

No solid-fill buttons anywhere. Destructive actions use `--color-status-error` tint instead.

### Build Tooling

**Tailwind CSS v4 with Lightning CSS** (built into `@tailwindcss/cli`):
```makefile
css:
    npx --yes @tailwindcss/cli \
      -i crates/bominal-frontend/style/main.css \
      -o crates/bominal-frontend/style/output.css \
      --content 'crates/bominal-frontend/src/**/*.rs'
```

Tailwind v4 uses Lightning CSS internally for transforms, vendor prefixes, and CSS nesting. No separate `lightningcss-cli` step is needed — this supersedes the master plan's Phase 5.1 build pipeline.

### JS Interop Strategy

The master plan mandates "TypeScript only (never plain JS) — compiled via esbuild." The theme init script in `app.rs` is an existing exception (inline `<script>` for FOUC prevention). For WebAuthn and Evervault interop, use a **TypeScript source file** compiled via esbuild:

- Source: `crates/bominal-frontend/ts/interop.ts`
- Output: `crates/bominal-frontend/ts/interop.js` (committed, rebuilt on change)
- Contents: WebAuthn ceremony wrappers + Evervault encrypt wrapper
- Loaded in `app.rs` shell via `<script src="/interop.js">`

The theme init script remains inline (must run before paint to prevent FOUC).

---

## Layer 1: CSS/Theme Foundation

### 1.1 Rewrite `main.css`

Replace the entire theme variable system. Remove:
- All 5 theme/palette combinations (light default, dark default, dark transit-slate, dark night-teal, dark warm-platform)
- All `data-palette` selectors
- All colorblind mode overrides (`html[data-colorblind='true']`)

Add:
- `html[data-theme="rosewood"][data-mode="light"]` — Rosewood Dusk light tokens
- `html[data-theme="rosewood"][data-mode="dark"]` — Rosewood Dusk dark tokens
- `html[data-theme="clear-sky"][data-mode="light"]` — Clear Sky light tokens
- `html[data-theme="clear-sky"][data-mode="dark"]` — Clear Sky dark tokens

Semantic token names use existing `--color-` prefix convention so components don't need per-theme logic updates.

**Semantic note on `--color-text-primary`**: In the current CSS, `--color-text-primary` maps to `--theme-text-strong` (the darkest/strongest text). In the new system, `--color-text-primary` remains the strongest body text color for each theme (e.g., `#3a2e28` in Rosewood Light). There is no separate `--color-text-strong` — `--color-text-primary` serves that role. `--color-text-secondary` is for regular supporting text.

### 1.2 Update theme init script in `app.rs`

Replace 3-key system (`bominal-theme`, `bominal-palette`, `bominal-colorblind`) with 2-key system:
- `bominal-theme`: `rosewood` | `clear-sky`
- `bominal-mode`: `light` | `dark`

Set `data-theme` and `data-mode` on `<html>`. Remove `data-palette` and `data-colorblind` attributes.

### 1.3 Update settings page theme UI

Replace:
- Dark mode toggle + palette dropdown + colorblind toggle

With:
- Theme picker: two cards (Rosewood Dusk / Clear Sky) with color swatches
- Mode toggle: Light / Dark switch

### 1.4 Rebuild Tailwind output

Run `make css` to regenerate `output.css` with new tokens.

**Files**: `main.css`, `app.rs`, `settings_view.rs`

---

## Layer 2: Page-by-Page Rewrite

Match each Leptos page to the React prototype's structure, wiring in the existing components.

### 2.1 Fix search page bug

**File**: `search_panel.rs`

The passenger counter area renders raw Rust closure code as visible text (seen in browser as `= 9 on : click = move | _ | set_adult_count.update(|c| *c = (*c + 1).min(9)) >+`). Investigate the exact broken `view!` macro syntax in the passenger counter section (around lines 220-230) and fix it so the `+` button renders as a proper button element. This may be a Leptos 0.8 syntax issue with event handlers inside the view macro.

### 2.2 Auth page

Match React structure:
- Centered glass card with icon (train/fingerprint)
- "Welcome back" / "Create account" headings
- Input fields with left icons (mail, lock)
- Password visibility toggle
- "Forgot password?" link
- Tinted glass primary button
- "Sign up" / "Sign in" toggle link at bottom
- Floating blur blobs in background

### 2.3 Home page

Match React structure:
- Bominal wordmark header
- Hero glass panel: title, description, two action cards (Start A Search, Open Tasks) in 2-column grid
- Each card: icon + title + description
- Bottom nav with active state pill

### 2.4 Search page

Wire existing components into search_panel.rs:
- Station text inputs (not dropdowns) with location icon + swap button
- `DatePicker` component (already built) instead of `<input type="date">`
- `PassengerSelector` component (already built) instead of broken inline counter
- `TimeSlider` component (already built) instead of `<input type="time">`
- Auto-Pay / Notify / Auto-Retry toggle chips
- Tinted glass search button

After search results load:
- `TicketCard` component for each train result
- `SelectionPrompt` floating bar when items selected
- `ReviewModal` for confirmation before task creation
- `SortableList` for reordering selected trains

### 2.5 Tasks page

Match React structure:
- "Reservation Tasks" heading with back button
- "Active Tasks" / "Completed Tasks" sections (not tabs)
- Rich task cards using departure → route → arrival visualization
- Status badges (Running/Idle/Confirmed/Failed/Cancelled/Awaiting Payment)
- Action buttons per card: Notify, Retry, Pause/Resume, View Details
- "Pay Fare" prominent button for awaiting-payment tasks
- Expandable "Click to view N more schedules" section

### 2.6 Settings page

Match React structure:
- Section navigation with colored icon badges + descriptions + chevrons
- Sections: Accessibility, Security, Payment, Notifications
- Each section drills down (show/hide via signal, not separate routes)
- Accessibility: theme picker + mode toggle
- Security: email display, password change, passkey management (wired in Layer 3)
- Payment: saved cards list, add card form with Evervault (wired in Layer 3)

### 2.7 Bottom nav active state

Add active state indicator: pill background on current tab, accent color on icon + label.

**Files**: All pages in `src/pages/`, `src/components/bottom_nav.rs`

---

## Layer 3: Missing Features

### 3.1 WebAuthn / Passkeys

**Dependency**: Add `webauthn-rs = { version = "0.5", features = ["danger-allow-state-serialisation"] }` to workspace.

**Database migration**: The existing `passkey_credentials` table needs additional columns:
- `aaguid BYTEA` — authenticator attestation GUID (for display icons)
- `transports TEXT[]` — transport hints (USB/NFC/BLE/internal) for login
- `label TEXT NOT NULL DEFAULT 'My Passkey'` — user-provided name for Settings UI

**Backend** (`bominal-server`):
- New file: `src/passkey.rs`
- `POST /api/auth/passkey/register/start` — begin registration ceremony (returns challenge + options)
- `POST /api/auth/passkey/register/finish` — complete registration (stores credential)
- `POST /api/auth/passkey/login/start` — begin authentication (returns challenge)
- `POST /api/auth/passkey/login/finish` — complete authentication (returns session)
- This 4-route start/finish pattern is the correct WebAuthn ceremony flow and supersedes the master plan's 3-route pattern.

**Frontend** (`bominal-frontend`):
- TypeScript interop (`ts/interop.ts`) wraps `navigator.credentials.create()` / `.get()`
- Leptos signals bridge JS ↔ WASM for ceremony state via `wasm_bindgen`
- Auth page: "Sign in with Passkey" button, registration prompt after signup
- Settings Security section: list passkeys with labels, add/remove

### 3.2 Evervault JS SDK — Kill AES-256 for Cards

**Problem**: `crates/bominal-frontend/src/api/cards.rs:67-74` (the Leptos server function `add_card`) encrypts card fields with AES-256-GCM server-side. This defeats the purpose — the server sees plaintext card data during SSR. Note: the Axum REST handler at `crates/bominal-server/src/cards.rs` already validates `ev:` prefixes (lines 157-193) — only the Leptos server function path still uses AES-256. The Leptos server function path should be removed entirely; all card operations should go through the Axum REST API `/api/cards` endpoint, which already has the correct Evervault validation.

**Note on AES-256 scope**: AES-256-GCM encryption (`bominal_domain::crypto::encryption`) is **only being removed for payment card data**. It remains in use for provider credentials (`provider_credentials.encrypted_password`), which are non-PCI business secrets. The `EncryptionKey` type and encryption module stay in the codebase.

**Solution**:
1. Load Evervault JS SDK in `app.rs` shell: `<script src="https://js.evervault.com/v2"></script>`
2. Initialize with `ev_app_id` — the server config (`config.rs`) already reads `EV_APP_ID` from env and passes it to `EvervaultConfig`. The remaining task is rendering `ev_app_id` into the HTML shell (e.g., `<meta name="ev-app-id" content="...">`) so the JS SDK can read it.
3. TypeScript interop (`ts/interop.ts`) wraps `window.evervault.encrypt()` for card form
4. Card form submits `ev:` prefixed tokens to `POST /api/cards`
5. Delete the AES-256 encryption path in `api/cards.rs` (remove `encrypt()` calls on card fields)
6. **Add `ev:` prefix validation** to the server `add_card` handler (`cards.rs`): reject any card field that doesn't start with `ev:` to ensure plaintext card data is never stored

**Existing card migration**: Since this is greenfield (no production users), no migration of existing AES-encrypted cards is needed. Any test cards in the DB can be re-added via the new Evervault flow.

**Card expiry format**: Input accepts MMYY (user-facing). Stored as MMYY in DB, encrypted via Evervault. Since the stored value is an opaque `ev:` token, the runner cannot perform format conversion. SRT expects YYMM and KTX expects MMYY. This discrepancy must be handled at the Evervault Relay egress layer via a transform function, or by storing two separately-encrypted expiry values (one MMYY, one YYMM) at card-add time. Decision deferred to implementation — flag as P1 during Layer 3.

**Frontend card form** (`settings_view.rs`):
- Card number, password (first 2 digits), birthday (YYMMDD), expiry (MMYY) inputs
- On submit: call Evervault encrypt via TypeScript interop
- Send encrypted `ev:` tokens to `POST /api/cards`
- Brand detection (already built in `card_brand.rs`) stays pure Rust

### 3.3 Auto-Pay Pipeline

The runner already fetches encrypted card data and routes through Evervault Relay. Verify end-to-end:
1. Task with `auto_pay=true` reaches `AwaitingPayment`
2. Runner fetches card from DB (`ev:` encrypted fields)
3. Provider client sends payment through Relay proxy
4. Relay decrypts `ev:` values in-flight
5. Payment reaches SRT/KTX with plaintext card data

**Safety rule** (from srtgo.py): NEVER auto-pay standby/waiting reservations. Add check in runner before payment.

### 3.4 Expose Evervault App ID to Frontend

The server config already reads `EV_APP_ID` and stores it in `EvervaultConfig`. The task is rendering it into the HTML shell for client-side JS:
- Add `ev_app_id: String` to the Leptos context (alongside existing `EncryptionKey`, `EmailClient`, etc.)
- Render `<meta name="ev-app-id" content="{ev_app_id}">` in `app.rs` shell
- TypeScript interop reads this meta tag to initialize the Evervault SDK

**Files**: `api/cards.rs`, `app.rs`, `ts/interop.ts` (new), `settings_view.rs`, new `passkey.rs`, `runner.rs`, `routes.rs`, `cards.rs` (server handler)

---

## Files Changed

| File | Layer | Action |
|------|-------|--------|
| `style/main.css` | 1 | Full theme rewrite |
| `src/app.rs` | 1, 3 | Theme init, Evervault meta tag, interop.js script |
| `src/pages/auth_page.rs` | 2 | Layout rewrite + passkey button |
| `src/pages/home_view.rs` | 2 | Layout rewrite |
| `src/pages/search_panel.rs` | 2 | Bug fix + wire components |
| `src/pages/tasks_view.rs` | 2 | Layout rewrite with rich cards |
| `src/pages/settings_view.rs` | 2, 3 | Layout rewrite + Evervault form |
| `src/components/bottom_nav.rs` | 2 | Active state indicator |
| `src/api/cards.rs` | 3 | Remove Leptos server function `add_card`; all card ops go through Axum `/api/cards` |
| `ts/interop.ts` | 3 | New: WebAuthn + Evervault JS wrappers |
| `crates/bominal-server/src/passkey.rs` | 3 | New: WebAuthn handlers |
| `crates/bominal-server/src/cards.rs` | 3 | Add `ev:` prefix validation |
| `crates/bominal-server/src/routes.rs` | 3 | Add passkey routes |
| `crates/bominal-server/src/runner.rs` | 3 | Standby payment guard (already has guard; verify) |
| `Cargo.toml` (workspace) | 3 | Add webauthn-rs |
| DB migration | 3 | Add aaguid, transports, label to passkey_credentials |

## Verification

1. `make css` produces output.css with all 4 theme combos
2. Theme switching works in browser (Rosewood ↔ Clear Sky, Light ↔ Dark)
3. No raw Rust code visible on any page
4. All interactive components render and function in search flow
5. Card form encrypts via Evervault JS SDK (no AES-256 path)
6. Server rejects card submissions without `ev:` prefix
7. WebAuthn registration + login ceremony works
8. Auto-pay flows through Evervault Relay without plaintext exposure
9. Standby reservations are never auto-paid
10. `cargo check` and `cargo clippy` clean
11. All existing tests pass + new tests for passkey + card endpoints
