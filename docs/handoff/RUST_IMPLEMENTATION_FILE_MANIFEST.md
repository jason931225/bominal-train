# Rust Implementation File Manifest

Purpose: current inventory of files used by the active Rust runtime path.

Rules:
- Paths are repo-relative and must exist in the current tree.
- `status` tracks current lifecycle:
  - `active`: required by the current runtime path.
  - `supporting`: optional but operationally relevant.
- Legacy or removed paths must not be listed.

| path | status | notes |
|---|---|---|
| docs/plans/2026-03-02-runtime-test-backfill-srt-parity.md | supporting | current runtime test/backfill execution plan |
| docs/handoff/RUST_IMPLEMENTATION_FILE_MANIFEST.md | active | this manifest |
| runtime/Cargo.toml | active | Rust workspace root |
| runtime/Cargo.lock | active | reproducible dependency lock |
| runtime/README.md | active | runtime workspace usage |
| runtime/crates/api/Cargo.toml | active | API crate manifest |
| runtime/crates/api/src/main.rs | active | API entrypoint and routes |
| runtime/crates/api/src/http/mod.rs | active | HTTP route composition |
| runtime/crates/api/src/services/mod.rs | active | API service wiring |
| runtime/crates/api/tests/internal_api_auth_contract_test.rs | supporting | API auth contract coverage |
| runtime/crates/shared/Cargo.toml | active | shared crate manifest |
| runtime/crates/shared/src/lib.rs | active | shared exports |
| runtime/crates/shared/src/config.rs | active | runtime configuration contracts |
| runtime/crates/shared/src/queue.rs | active | runtime queue contract |
| runtime/crates/shared/src/providers/mod.rs | active | provider modules |
| runtime/crates/shared/src/repo/mod.rs | active | persistence modules |
| runtime/crates/worker/Cargo.toml | active | worker crate manifest |
| runtime/crates/worker/src/main.rs | active | worker entrypoint |
| runtime/crates/worker/src/runtime/mod.rs | active | worker runtime module wiring |
| runtime/frontend/package.json | active | frontend build scripts |
| runtime/frontend/tailwind.config.js | active | Tailwind config |
| runtime/frontend/styles/tailwind.css | active | CSS source |
| runtime/frontend/dist/tailwind.css | active | built CSS artifact served by runtime API |
| runtime/migrations/202603010001_bootstrap.sql | active | baseline schema migration |
| runtime/migrations/202603010002_supabase_auth_user_sync.sql | active | auth sync migration |
| runtime/migrations/202603020001_runtime_jobs.sql | active | runtime jobs migration |
| runtime/migrations/202603030001_provider_runtime_v2.sql | active | provider runtime v2 migration |
| runtime/.dockerignore | active | runtime Docker build ignores |
| runtime/Dockerfile.api | active | API image build |
| runtime/Dockerfile.worker | active | worker image build |
| .github/workflows/ci.yml | active | CI workflow |
| .github/workflows/cd.yml | active | CD workflow |
