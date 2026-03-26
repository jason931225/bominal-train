# Bominal: Claude Agentic Workflow Guide

## Project Overview
**Bominal** is a train reservation management SaaS with real-time booking, payment processing with encryption, and secure passkey authentication. The frontend is a SvelteKit SPA served by an Axum REST API backend.

## Architecture
- **Backend**: Axum 0.8 REST API (Rust)
- **Frontend**: SvelteKit 2.16 + Svelte 5 + Tailwind CSS 4 (TypeScript) — static SPA via `adapter-static`
- **6 Rust Crates** (workspace members):
  - `bominal-server`: Axum routes, SPA serving, WebAuthn endpoints
  - `bominal-db`: SQLx migrations, connection pooling
  - `bominal-domain`: Core types (User, Reservation, Payment, PasskeyCredential)
  - `bominal-email`: Resend API integration for transactional emails
  - `bominal-service`: Business logic, cache patterns, event handlers, provider integrations (SRT/KTX)
- **SvelteKit Frontend** (`frontend/`):
  - `src/routes/` — Pages: auth, home, search, tasks, reservations, settings
  - `src/lib/api/` — Typed fetch client (`get<T>`, `post<T>`, etc.) with cookie auth
  - `src/lib/stores/` — Svelte 5 runes (`$state`, `$derived`) for auth, SSE, theme
  - `src/lib/components/` — Glass morphism UI components (GlassPanel, TicketCard, etc.)
  - `src/lib/i18n/` — Korean (default), English, Japanese
  - `src/lib/types/` — TypeScript interfaces mirroring Rust DTOs

## Build & Development
```bash
# Full build (SvelteKit frontend + Rust server)
./dev-build.sh

# Release build for deployment
./deployment/build.sh

# Server-only rebuild (skip frontend if unchanged)
cargo build --release

# Frontend-only rebuild
cd frontend && npm run build

# Frontend dev server (with API proxy to :3000)
cd frontend && npm run dev

# Format all workspace code
cargo fmt --all -- --check

# Lint with strict warnings-as-errors
cargo clippy --workspace --all-targets -- -D warnings

# Run all tests
cargo test --workspace

# Test specific crate
cargo test -p bominal-server
```

### Local Dev Workflow
1. Run `./dev-build.sh` (builds SvelteKit to `frontend/build/`, compiles Rust server)
2. Start server: `./target/release/bominal-server` (requires `.env`)
3. Open `http://localhost:3000` (or whatever `PORT` is set to)
4. If only Rust code changed: `cargo build --release` (skip frontend rebuild)
5. If only frontend changed: `cd frontend && npm run build` (skip Rust rebuild)
6. For frontend hot-reload: `cd frontend && npm run dev` (Vite proxies `/api/*` to `:3000`)

### Build Prerequisites
- `npm` (for SvelteKit/Vite build)
- Rust toolchain (stable)
- No WASM target needed — frontend is pure TypeScript/Svelte

## Database
- **PostgreSQL 16** with SQLx compile-time checking (sqlx::query!())
- **Migrations**: `crates/bominal-db/migrations/`
  - `20260311000001_initial_schema.sql` - Users, reservations, payments
  - `20260312000001_add_passkey_and_expiry_fields.sql` - WebAuthn columns
  - `20260312000002_passkey_challenge_state.sql` - Authentication state
  - `20260324000001_add_provider_to_task_targets.sql` - Provider field on task targets
- **Connection URL**: `DATABASE_URL=postgres://bominal:bominal@localhost:5432/bominal`

## Key Patterns

### WebAuthn Passkey Authentication
- `webauthn-rs` 0.5 with conditional UI support
- Challenge generation in `bominal-server` endpoints
- Credential storage: `passcode_credentials` table with `public_key` blob + `counter` tracking
- Challenge state: temporary state in DB, expires via cron job

### Valkey Cache (Redis-compatible)
- Connection: `VALKEY_URL=redis://127.0.0.1:6379`
- Session store pattern: `session:{user_id}` → JSON
- Rate limiting: `ratelimit:{endpoint}:{ip}` → counter
- Invalidation on logout/password change

### Encryption & Security
- **Evervault**: Payment card tokenization (`EV_APP_ID`, `EV_API_KEY`, `EV_SRT_DOMAIN`, `EV_KTX_DOMAIN`)
- **AES-256-GCM**: Provider API credentials encrypted at rest with `ENCRYPTION_KEY` (64 hex chars)
- **Argon2**: Password hashing for fallback auth

### Real-Time Updates
- **SSE**: `/api/tasks/events` endpoint for real-time task status updates
- Frontend connects via `EventSource` with pub/sub callback pattern
- Auto-reconnection handled by browser

### Integration Testing
- Mock provider responses in `bominal-service/src/providers/` tests
- Test isolation: use `#[tokio::test]` with transaction rollback
- Database fixtures: `bominal-db::test::setup_pool()`

## API Routes
All under `/api/` prefix:
- **Auth**: `/auth/register`, `/auth/login`, `/auth/me`, `/auth/passkey/*` (rate-limited 20 req/min)
- **Search**: `/search`, `/stations/{provider}`, `/stations/{provider}/suggest` (30 req/min)
- **Reservations**: CRUD + cancel/pay/refund (payments: 10 req/min)
- **Tasks**: CRUD + SSE events at `/tasks/events`
- **Cards**: Payment card management
- **Providers**: Train provider credential management

## Deployment
- **Docker**: Multi-stage build (Rust 1.85 + Node.js 22 builder → Debian Bookworm-slim runtime)
  - SvelteKit built to `frontend/build/` in builder stage
  - Binary: `/app/target/release/bominal-server`

- **Systemd**: Unit file in `deployment/bominal.service`
  - User: `bominal`, socket: `/var/run/bominal.sock`
  - Restart on failure, hardened with read-only filesystem

- **Caddy**: Reverse proxy in `deployment/Caddyfile`
  - TLS termination, compression (gzip + brotli)

## Environment
**.env file required** (reference: `.env.example`)
- `PORT`, `BOMINAL_ENV`, `APP_BASE_URL`
- `DATABASE_URL` (PostgreSQL 16)
- `VALKEY_URL` (Redis-compatible)
- `ENCRYPTION_KEY` (AES-256-GCM, 64 hex chars)
- `RESEND_API_KEY`, `EMAIL_FROM` (email delivery)
- `EV_*` vars (Evervault card encryption)

## Dependencies
- **Backend**: Tokio (async), Axum (routing), Tower (middleware), SQLx (database)
- **HTTP Clients**: wreq (SRT/KTX providers, TLS fingerprint emulation via BoringSSL), reqwest (email/general)
- **Frontend**: SvelteKit 2.16, Svelte 5, Tailwind CSS 4, Vite 6
- **Crypto**: Argon2 (password hashing), AES-256-GCM, WebAuthn
- **Observability**: Tracing, Prometheus metrics, JSON structured logging

## Key Conventions
- **REST API** routes are under `/api/` (Axum handlers in `bominal-server`)
- **SPA serving**: `frontend/build/` served via `ServeDir` with `index.html` fallback for client-side routing
- **Frontend API client**: `frontend/src/lib/api/client.ts` — typed fetch wrapper with cookie auth
- **i18n**: Korean default, translations in `frontend/src/lib/i18n/`
- **Svelte 5 runes**: Use `$state`, `$derived` for reactive state (not legacy stores)

## Gotchas
- **Cookie `Secure` flag**: Session cookies set `; Secure` only when `APP_BASE_URL` starts with `https://`. On `http://localhost`, omitting this is required or the browser silently drops the cookie and auth breaks.
- **Vite dev proxy**: `frontend/vite.config.ts` proxies `/api/*` and `/health` to `http://localhost:3000`. The Rust server must be running for frontend dev mode to work.
- **Legacy `bominal-frontend` crate**: Still on disk at `crates/bominal-frontend/` but removed from workspace. Dead code — do not use.
- **adapter-static**: SvelteKit builds to static files. No SSR at runtime — all rendering is client-side.
- **OpenSSL/BoringSSL dual-link**: `wreq` uses BoringSSL (boring-sys2), `webauthn-rs` uses OpenSSL. The `bominal-server/build.rs` resolves the symbol conflict on macOS via `-force_load` of the system OpenSSL dylib. On Linux, the system OpenSSL 3 resolves naturally. If you see `OPENSSL_sk_*` linker errors, ensure Homebrew OpenSSL 3 is installed (`brew install openssl@3`).

## Common Tasks
- **Add migration**: `sqlx migrate add -r <name>` in `crates/bominal-db/`
- **Check compilation**: `cargo check --workspace` (fast)
- **Full build**: `./dev-build.sh`
- **Frontend dev**: `cd frontend && npm run dev`
- **Docker build**: `docker build -t bominal:latest .`
