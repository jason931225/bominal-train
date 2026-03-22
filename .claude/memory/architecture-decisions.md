# Bominal: Architecture Decisions

## Framework Choices

### Axum over Actix-web
- **Why Axum**: Tower ecosystem composability, better async trait support, lighter dependencies
- **Justification**: Leptos officially supports Leptos_axum; Tower middleware stack is cleaner than Actix guards
- **Trade-offs**: Smaller ecosystem than Actix; more manual route organization needed

### Leptos 0.8 over Yew
- **Why Leptos**: Native SSR support with hydration, fine-grained reactivity (not VDOM), Axum integration
- **Justification**: Reduces client-side initialization latency; compiles to smaller WASM (fine-grained reactivity doesn't ship virtual DOM)
- **Trade-offs**: Newer framework; smaller community; steeper learning curve for component patterns

### PostgreSQL 16 over MySQL/SQLite
- **Why PostgreSQL**: Advanced features (JSONB, ARRAY, window functions), strong ACID guarantees, UUID types
- **Justification**: Support for complex passkey credential storage (JSONB); JSON for audit logs; UUID v4/v7 native
- **Trade-offs**: Higher resource overhead than SQLite (acceptable for SaaS); replication complexity vs managed services

### SQLx over ORM (Diesel/Tokio-Postgres)
- **Why SQLx**: Compile-time SQL checking, no macro overhead, zero-cost abstractions
- **Justification**: Catch SQL errors at compile time; type safety without reflection; migrations are plain SQL (version control friendly)
- **Trade-offs**: Less scaffolding; requires explicit query writing; migrations require database connectivity at compile time

## Caching & State

### Valkey (Redis-compatible) over In-Memory Cache
- **Why Valkey**: Single-source of truth across horizontally scaled instances
- **Justification**: Passkey challenges must be shared across replicas; session invalidation is instant
- **Trade-offs**: Extra network I/O; requires operational deployment; single point of failure (mitigate with Sentinel/Cluster)

### Valkey Patterns Chosen
- **Session storage**: `session:{user_id}` → JSON serialized session (10min TTL)
- **Rate limiting**: `ratelimit:{endpoint}:{ip}` → counter with EX=60s (sliding window)
- **Passkey challenge**: `challenge:{user_id}:{challenge_bytes}` → challenge state + WASM public key (5min TTL)
- **Non-authoritative**: Eviction doesn't fail requests; stale challenges re-authenticated

## Authentication & Security

### WebAuthn Passkeys over Password + 2FA
- **Why WebAuthn**: Phishing-resistant, FIDO2 standard, better UX (biometric/hardware)
- **Justification**: Train booking is high-value; password reuse risk unacceptable; passkeys have native OS support
- **Trade-offs**: Fallback password auth needed for legacy devices; requires client library; challenge state overhead

### Evervault over In-House Card Encryption
- **Why Evervault**: PCI-DSS compliance outsourced, tokenization (cards never touch our servers), HWNE (hardened, isolated)
- **Justification**: Payment card regulations are complex; Evervault handles attestation + compliance audit
- **Trade-offs**: Vendor lock-in; additional API calls; 3 environment variables to manage

### AES-256-GCM for Provider Credentials
- **Why AES-GCM**: Authenticated encryption (integrity + confidentiality), NIST-approved, hardware-accelerated
- **Justification**: Provider API keys stored at rest; GCM nonce prevents replay attacks
- **Trade-offs**: Requires 32-byte key; nonce generation must be cryptographically secure (using rand crate)

## Crate Organization

### 7-Crate Split Philosophy: Explicit Boundaries Over Monolith
1. **bominal-server**: HTTP endpoints, SSR rendering, request/response
2. **bominal-frontend**: Leptos components, reactive state, WASM
3. **bominal-db**: Database layer only (migrations, connection, sqlx wrappers)
4. **bominal-domain**: Domain types (User, Reservation, Payment), serialization
5. **bominal-email**: Email service abstraction (Resend client, templates)
6. **bominal-provider**: Third-party integrations (train APIs, HTTP client)
7. **bominal-service**: Business logic (booking flow, validation, cache patterns)

**Rationale**: Force clear dependency graph; prevent circular imports; enable independent testing of business logic from HTTP layer

## Deployment Architecture

### Docker Multi-Stage Build Philosophy
1. **Builder stage**: Rust 1.85 + Node.js 22 (esbuild), full dependencies
2. **Runtime stage**: Alpine, binary only, minimal attack surface
3. **Layering**: Copy Cargo manifests first → cache deps → copy source (avoid full rebuild on code changes)

### Systemd Service Unit
- **Socket**: `/var/run/bominal.sock` (Caddy reverse proxy connects here)
- **Hardening**: `ReadOnlyPaths`, `NoNewPrivileges`, `PrivateTmp`, `ProtectSystem=strict`
- **Restart**: on-failure (connection resets trigger restart)

### Caddy Reverse Proxy
- **Functions**: TLS termination, compression (gzip + brotli), static asset versioning
- **Why not Nginx**: TOML config more readable; automatic HTTPS cert management; dynamic plugin system

## Performance & Reliability

### Rust Edition 2024
- **Why**: Optimized allocator (`tokio-console` compatible), union syntax, dyn Sized
- **MSRV**: 1.85 (latest stable at project inception)

### Profile Optimization Strategy
- **dev**: opt-level=1 (local fast compile) + opt-level=3 for deps (tests run faster)
- **release**: opt-level=3 + thin LTO + codegen-units=1 (production binary size optimized)
- **wasm-release**: opt-level=s + strip debuginfo (minimize WASM size for hydration payload)

### Metrics & Observability
- **Tracing**: structured logging to stdout (JSON via tracing-subscriber for log aggregation)
- **Prometheus**: metrics exported on `/metrics` endpoint (CPU, requests, latency percentiles)
- **No database query profiling in production** (SQLx compile-time checks catch N+1 at compile time)
