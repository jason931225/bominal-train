# Bominal: Claude Agentic Workflow Guide

## Project Overview
**Bominal** is a train reservation management SaaS with real-time booking, payment processing with Evervault encryption, and secure passkey authentication. The product now ships as a Leptos 0.8 SSR application served directly by the Axum backend.

## Architecture
- **Server**: `crates/bominal-server` owns Axum routing, middleware, WebAuthn endpoints, SSE, and Leptos SSR integration.
- **App**: `crates/bominal-app` owns the Leptos route table, SSR shell, hydrated islands, typed `/api` proxy calls, app state, and browser interop.
- **Service layer**: `crates/bominal-service` holds business logic shared by REST handlers and the Leptos app/server layer.
- **Shared design system**: `bominal-ui` is the canonical UI source and is pulled in as a sibling crate.
- **Data/infrastructure crates**:
  - `crates/bominal-db`: SQLx migrations and connection pooling
  - `crates/bominal-domain`: shared DTOs, i18n strings, task/provider types
  - `crates/bominal-email`: Resend integration

## Build & Development
```bash
# Local build entry point
./dev-build.sh

# Direct cargo-leptos workflows
cargo leptos build
cargo leptos serve
cargo leptos watch

# Release build
./deployment/build.sh

# Fast compile checks
cargo check -p bominal-app --features ssr
cargo check -p bominal-app --target wasm32-unknown-unknown --features hydrate
cargo check -p bominal-server --features ssr

# Tests
cargo test -p bominal-app --lib
cargo test -p bominal-server

# Formatting and linting
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
```

### Local Dev Workflow
1. Run `cargo leptos serve` for the integrated Leptos + Axum dev loop, or `./dev-build.sh` for a one-shot build.
2. Provide `.env` values before starting the server.
3. Open `http://localhost:3000`.
4. Use `cargo leptos watch` when iterating on the app shell/pages and `cargo check` when you only need compile feedback.

### Build Prerequisites
- Rust stable with `wasm32-unknown-unknown`
- `cargo-leptos`
- Tailwind CSS v4 standalone binary available on `PATH`
- No JavaScript package-manager dependency in the active build pipeline

## Database
- PostgreSQL 16 with SQLx compile-time checked queries
- Migrations live in `crates/bominal-db/migrations/`
- Local default URL: `DATABASE_URL=postgres://bominal:bominal@localhost:5432/bominal`

## Key Patterns

### Leptos SSR + Islands
- `crates/bominal-app/src/lib.rs` owns the route table and SSR shell.
- `crates/bominal-server/src/routes.rs` mounts Leptos SSR for non-`/api` routes and serves static assets from `target/site`.
- Interactive surfaces stay in hydrated islands; server rendering remains the default path.

### Typed API Proxy
- The app calls existing backend endpoints through typed Leptos server functions in `crates/bominal-app/src/api.rs`.
- `/api` remains the backend contract; migration did not move business logic into ad hoc frontend-only code.

### WebAuthn + Browser Interop
- Passkey flows live in `crates/bominal-app/src/api/passkey.rs` and `crates/bominal-app/src/browser.rs`.
- Browser-only helpers are centralized in `crates/bominal-app/assets/interop.js`.

### Real-Time Updates
- SSE remains exposed at `/api/tasks/events`.
- Home/tasks pages consume the existing task event stream through shared app state helpers.

### Security
- Evervault handles payment-card encryption (`EV_*` environment variables).
- Provider credentials remain encrypted at rest with AES-256-GCM using `ENCRYPTION_KEY`.
- Password fallback auth continues to use Argon2.

## API Routes
All application APIs remain under `/api/`:
- **Auth**: register, login, current-user, passkey ceremony endpoints
- **Search**: train search and station suggestion endpoints
- **Reservations**: reservation CRUD, payment, refund, cancel flows
- **Tasks**: CRUD plus SSE events
- **Cards**: payment-card management
- **Providers**: train-provider credential management

## Deployment
- **Docker**: multi-stage Rust build using `cargo leptos build --release --precompress`
  - Build with the sibling UI repo provided as a named BuildKit context:
    `docker buildx build --build-context bominal_ui=../bominal-ui -f Dockerfile -t bominal-train:latest .`
  - Runtime serves `/app/bominal-server` and assets under `/app/target/site`
- **Systemd**: `deployment/bominal.service`
- **Caddy**: `deployment/Caddyfile`

## Environment
Reference `.env.example` for the required variables:
- `PORT`, `BOMINAL_ENV`, `APP_BASE_URL`
- `DATABASE_URL`
- `VALKEY_URL`
- `ENCRYPTION_KEY`
- `RESEND_API_KEY`, `EMAIL_FROM`
- `EV_*`

## Dependencies
- **Backend/runtime**: Tokio, Axum, Tower, SQLx
- **App/runtime**: Leptos, leptos_router, leptos_meta, leptos_axum
- **HTTP clients**: `wreq` for provider integrations, `reqwest` for general HTTP/email
- **Observability**: tracing + Prometheus metrics

## Gotchas
- Session cookies only use `Secure` when `APP_BASE_URL` is `https://...`; keep that off for `http://localhost`.
- `bominal-ui` is a sibling path dependency, so Docker builds must pass the named `bominal_ui` context.
- `wreq`/BoringSSL plus `webauthn-rs`/OpenSSL can still trigger macOS linker issues; if you see `OPENSSL_sk_*` failures, ensure Homebrew OpenSSL 3 is installed.
- The active CSS entry point is `crates/bominal-app/style/app.css`; do not reintroduce parallel frontend build assets.

## Common Tasks
- Add a migration: `sqlx migrate add -r <name>` in `crates/bominal-db/`
- Fast workspace compile sweep: `cargo check --workspace`
- Full local app build: `./dev-build.sh`
- Integrated dev server: `cargo leptos serve`
- Docker build: `docker buildx build --build-context bominal_ui=../bominal-ui -f Dockerfile -t bominal-train:latest .`
