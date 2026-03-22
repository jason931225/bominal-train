# Bominal: Claude Agentic Workflow Guide

## Project Overview
**Bominal** is a Rust-based SaaS application for train reservation management, featuring real-time booking, payment processing with encryption, and secure passkey authentication. It combines server-side rendering with client-side hydration for optimal performance.

## Architecture
- **Framework**: Axum 0.8 SSR backend + Leptos 0.8 hydration frontend
- **7 Crates**:
  - `bominal-server`: Axum routes, SSR rendering, WebAuthn endpoints
  - `bominal-frontend`: Leptos components, WASM targets, reactive state
  - `bominal-db`: SQLx migrations (3 total), connection pooling
  - `bominal-domain`: Core types (User, Reservation, Payment, PasskeyCredential)
  - `bominal-email`: Resend API integration for transactional emails
  - `bominal-provider`: Train provider integrations, HTTP client abstractions
  - `bominal-service`: Business logic, cache patterns, event handlers

## Build & Development
```bash
# Full release build (opt-level=3, thin LTO)
cargo build --release

# Leptos SSR + frontend compilation (includes WASM + esbuild TS interop)
cargo leptos build

# Format all workspace code
cargo fmt --all -- --check

# Lint with strict warnings-as-errors
cargo clippy --workspace --all-targets -- -D warnings

# Run all tests
cargo test --workspace

# Test specific crate
cargo test -p bominal-server
```

## Database
- **PostgreSQL 16** with SQLx compile-time checking (sqlx::query!())
- **Migrations**: `crates/bominal-db/migrations/` (3 files)
  - `20260311000001_initial_schema.sql` - Users, reservations, payments
  - `20260312000001_add_passkey_and_expiry_fields.sql` - WebAuthn columns
  - `20260312000002_passkey_challenge_state.sql` - Authentication state
- **Connection URL**: `DATABASE_URL=postgres://bominal:bominal@localhost:5432/bominal`
- **Run migrations**: SQLx auto-runs on app startup via `sqlx::query!()` compile checks

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
- **Evervault**: Payment card tokenization (3 required env vars: `EV_APP_ID`, `EV_API_KEY`, `EV_SRT_DOMAIN`, `EV_KTX_DOMAIN`)
- **AES-256-GCM**: Provider API credentials encrypted at rest with `ENCRYPTION_KEY` (64 hex chars)
- **Argon2**: Password hashing for fallback auth

### Integration Testing
- Mock provider responses in `bominal-provider/tests/`
- Test isolation: use `#[tokio::test]` with transaction rollback
- Database fixtures: `bominal-db::test::setup_pool()`

## Deployment
- **Docker**: Multi-stage build (Rust 1.85 builder → Alpine runtime)
  - Dependency layer cached separately from source
  - Node.js 22 in builder for esbuild TS compilation
  - Tailwind CSS v4 standalone binary (arch-aware)
  - Binary: `/app/target/release/bominal-server`

- **Systemd**: Unit file in `deployment/bominal.service`
  - User: `bominal`, socket: `/var/run/bominal.sock`
  - Restart on failure, hardened with read-only filesystem

- **Caddy**: Reverse proxy in `deployment/Caddyfile`
  - TLS termination, compression (gzip + brotli)
  - Rewrite `/assets/*` to versioned paths

## Environment
**.env file required** (reference: `.env.example`)
- `PORT`, `BOMINAL_ENV`, `APP_BASE_URL`
- `DATABASE_URL` (PostgreSQL 16)
- `VALKEY_URL` (Redis-compatible)
- `ENCRYPTION_KEY` (AES-256-GCM, 64 hex chars)
- `RESEND_API_KEY`, `EMAIL_FROM` (email delivery)
- `EV_*` vars (Evervault card encryption)

## Dependencies
- **Runtime**: Tokio (async), Axum (routing), Tower (middleware), SQLx (database)
- **Frontend**: Leptos (SSR/hydration), leptos_router, leptos_meta
- **Crypto**: Argon2 (password hashing), AES-256-GCM, WebAuthn
- **Observability**: Tracing, Prometheus metrics, JSON structured logging

## Common Tasks
- **Add migration**: `sqlx migrate add -r <name>` in `crates/bominal-db/`
- **Check compilation**: `cargo check --workspace` (fast)
- **WASM build**: `cargo build --profile wasm-release --target wasm32-unknown-unknown`
- **Docker build**: `docker build -t bominal:latest .`
