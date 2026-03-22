# Bominal: Debugging Patterns & Common Issues

## Database & Migration Errors

### SQLx Migration Failure: "Unexpected number of rows"
**Symptom**: Compile fails with `sqlx::Error: expected 1 row, got N`
**Root Cause**: Migration query in `crates/bominal-db/migrations/` returns multiple rows or zero rows
**Solution**:
```sql
-- Wrong: SELECT returns multiple rows in migration
INSERT INTO users ... RETURNING * ;

-- Right: Wrap in transaction or use RETURNING clause only for affected rows
INSERT INTO users ... RETURNING id ;
```
**Prevention**: Test migrations against dev database before deploy
```bash
sqlx migrate run --database-url=$DATABASE_URL --path crates/bominal-db/migrations
```

### "Postgres SQLX requires DATABASE_URL for compile-time verification"
**Symptom**: Compilation fails even though query is valid
**Root Cause**: `.sqlx` cache outdated or database offline during build
**Solution**:
```bash
# Regenerate offline cache
cargo sqlx prepare -- cargo test

# Or skip compile-time checking for CI
SQLX_OFFLINE=true cargo build --release
```
**Prevention**: Commit `.sqlx/` folder to git; use `SQLX_OFFLINE=true` in CI pipelines

### Passkey Table Schema Mismatch
**Symptom**: `column "counter" does not exist` runtime error
**Root Cause**: Running against old schema; migration 20260312000001 not applied
**Solution**:
```bash
# Check which migrations have run
psql -U bominal bominal -c "SELECT * FROM _sqlx_migrations;"

# Apply pending migrations
sqlx migrate run --database-url=$DATABASE_URL --path crates/bominal-db/migrations
```
**Prevention**: Run migrations on startup in `bominal-server/src/main.rs`:
```rust
sqlx::migrate!("crates/bominal-db/migrations")
    .run(&pool)
    .await?;
```

---

## Leptos SSR & Hydration Errors

### "Hydration mismatch: expected text node, got element"
**Symptom**: Browser console warns hydration mismatch; interactive features don't work
**Root Cause**:
- Server rendered different HTML than client (e.g., conditional rendering based on time, randomness)
- Component uses `window::location` in render phase (not available on server)
- WASM hasn't loaded yet (client-side JavaScript runs before hydration completes)

**Solution**:
```rust
// Wrong: Random value differs between server & client
let id = uuid::Uuid::new_v4().to_string();

// Right: Use effect hook or prop passed from server
#[component]
fn Widget(id: String) -> impl IntoView {
    view! { <div id={id}> /* guaranteed same on server & client */ </div> }
}
```

For server-only logic:
```rust
// Use #[cfg(feature = "ssr")] or check leptos::is_server()
#[cfg(feature = "ssr")]
async fn get_user_from_session(req: &HttpRequest) -> User { /* ... */ }
```

**Prevention**:
- Use `create_effect` for client-only initialization
- Pass time/randomness from server as props
- Test with `cargo leptos build && cargo leptos serve`

### WASM Hydration Binary Size Explosion
**Symptom**: `index.wasm` is 5+ MB; hydration slow in production
**Root Cause**:
- Debug symbols included in WASM binary
- Leptos components not code-split or lazy-loaded
- Heavy dependencies in frontend crate (e.g., full chrono instead of parsing)

**Solution**:
```toml
# In Cargo.toml [profile.wasm-release]
[profile.wasm-release]
inherits = "release"
opt-level = "s"  # Optimize for size
strip = "debuginfo"  # Remove debug symbols
lto = true  # Link-time optimization
```

```bash
# Build with size optimization
cargo build --profile wasm-release --target wasm32-unknown-unknown

# Post-process with wasm-opt (if available)
wasm-opt -Os --output output.wasm input.wasm
```

**Prevention**: Monitor WASM size in CI; set size budget (e.g., max 300KB gzipped)

### "Expected `Fn() -> impl IntoView`, found `impl IntoView`"
**Symptom**: Leptos component prop expects closure, you're passing direct value
**Root Cause**: Using `<Component prop=value/>` instead of `<Component prop=move || value/>`
**Solution**:
```rust
// Wrong: Direct reactive value
let children = get_children();  // type: Vec<User>
view! { <UserList children=children/> }

// Right: Wrap in signal or closure
let children = create_rw_signal(get_children());
view! { <UserList children=move || children.get()/> }
```

---

## Docker Build Errors

### "Copy failed: file not found" in Docker
**Symptom**: Docker build fails at `COPY crates/bominal-server/Cargo.toml`
**Root Cause**: Running `docker build` from wrong directory or cargo manifest path typo
**Solution**:
```bash
# Build from project root
cd /path/to/bominal && docker build -t bominal:latest .

# Verify Dockerfile has correct paths
cat Dockerfile | grep "COPY crates/"
```

### Docker Build Cache Not Working (Full Rebuild Every Time)
**Symptom**: Docker rebuilds dependencies every push; layer 8 (copy source) invalidates cache
**Root Cause**: Cargo.lock changes or source files copied before lock file
**Solution**:
```dockerfile
# Right order: lock file first (stable), source later (volatile)
COPY Cargo.lock ./
COPY Cargo.toml ./
RUN cargo build --release  # Cache hit if lock file unchanged
COPY crates/ crates/       # This layer can miss cache
RUN touch crates -r src && cargo build --release  # Recompile only
```

### "esbuild: executable not found"
**Symptom**: Docker build fails: "esbuild not found in PATH"
**Root Cause**: Node.js or npm not installed in builder stage
**Solution** (in Dockerfile):
```dockerfile
FROM rust:1.85-bookworm AS builder
RUN curl -fsSL https://deb.nodesource.com/setup_22.x | bash - \
    && apt-get install -y --no-install-recommends nodejs \
    && npm install -g esbuild
```

### Tailwind CSS Binary Architecture Mismatch
**Symptom**: Docker build on ARM Mac fails with "Tailwind: exec format error"
**Root Cause**: Downloaded x86_64 binary on ARM machine
**Solution** (in Dockerfile):
```dockerfile
# Detect architecture at build time
RUN ARCH=$(uname -m) && \
    TAILWIND_ARCH=$([ "$ARCH" = "aarch64" ] && echo "arm64" || echo "x64") && \
    npm install -g @tailwindcss/cli@4
```

---

## Valkey Cache Issues

### "Connection refused" on `VALKEY_URL`
**Symptom**: Runtime error `redis://127.0.0.1:6379` unreachable
**Root Cause**: Valkey service not running or wrong port
**Solution**:
```bash
# Start Valkey in Docker
docker run -d -p 6379:6379 valkey/valkey:latest

# Verify connection
redis-cli -h 127.0.0.1 -p 6379 ping  # Should return "PONG"
```

### Passkey Challenge Expired Before User Submits
**Symptom**: "Invalid challenge" error after 5+ minutes idle
**Root Cause**: Valkey key TTL (5 minutes) expired; challenge state lost
**Solution**:
```rust
// In bominal-service/src/auth.rs
let challenge_ttl = Duration::from_secs(300);  // 5 minutes
valkey.set_ex(challenge_key, challenge_json, 300).await?;

// Client-side: warn if form idle > 4 minutes
create_effect(move || {
    if challenge_age() > Duration::from_secs(240) {
        show_warning("Challenge expiring soon. Click 'Refresh' to re-authenticate.");
    }
});
```

---

## WebAuthn & Passkey Specific

### "Credential not found in database"
**Symptom**: User logged in yesterday, today they can't authenticate
**Root Cause**:
- `passcode_credentials` table not synced
- Column `counter` missing (pre-migration data)
- UPSERT conflict in credential storage

**Solution**:
```rust
// Check migration status
SELECT version, success FROM _sqlx_migrations WHERE description LIKE '%passkey%';

// Manual fix: ensure counter column exists and has default
ALTER TABLE passcode_credentials ADD COLUMN IF NOT EXISTS counter BIGINT NOT NULL DEFAULT 0;
```

### "Challenge state not found" on SignInResponse
**Symptom**: WebAuthn verification fails with no challenge in cache
**Root Cause**: User took too long; Valkey evicted challenge; or wrong challenge ID
**Solution**:
```rust
// In bominal-server/src/routes/auth.rs
let challenge = valkey.get(&challenge_id)
    .await
    .ok_or(AuthError::ChallengeExpired)?;  // Clear error message
```

**Prevention**:
- Log challenge creation with timestamp
- Monitor Valkey eviction rate
- Implement client-side timeout warning at 4 minutes

---

## Testing Patterns

### Integration Test: Passkey Flow
```rust
#[tokio::test]
async fn test_passkey_signup_flow() {
    let pool = setup_test_pool().await;
    let valkey = setup_test_valkey().await;

    // 1. Create credential on server
    let challenge = generate_challenge();
    valkey.set_ex("challenge:test_user", challenge_json, 300).await.unwrap();

    // 2. Verify credential matches schema
    let cred = valkey.get("challenge:test_user").await.unwrap();
    assert!(cred.contains("challenge"));

    // 3. Cleanup (transaction rollback)
    pool.execute("ROLLBACK").await.unwrap();
}
```

### Unit Test: Encryption
```rust
#[test]
fn test_provider_credential_encryption() {
    let key = decrypt_key("0000...0000");
    let plaintext = "sk_live_abc123";
    let (ciphertext, nonce) = encrypt_aes_256_gcm(&plaintext, &key).unwrap();
    let decrypted = decrypt_aes_256_gcm(&ciphertext, &nonce, &key).unwrap();
    assert_eq!(decrypted, plaintext);
}
```
