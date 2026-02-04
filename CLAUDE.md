# CLAUDE.md

AI agent behavioral rules and quick reference for the **bominal** project.

> For detailed architecture, read `docs/ARCHITECTURE.md`. For operations, see `docs/RUNBOOK.md`.

---

## Behavioral Rules (Always Enforced)

- Do what has been asked; nothing more, nothing less
- NEVER create files unless absolutely necessary for achieving the goal
- ALWAYS prefer editing an existing file to creating a new one
- NEVER proactively create documentation files (*.md) or README files unless explicitly requested
- ALWAYS read a file before editing it
- NEVER commit secrets, credentials, or .env files
- NEVER log sensitive data (passwords, tokens, card data, CVV, full provider payloads)
- Preserve product name as **bominal** (lowercase) in UI, config, and docs
- Treat `third_party/srtgo/` as **read-only** — never patch or reformat it

---

## Concurrency Guidelines

When performing multiple independent operations, batch them in a single message:

- **Batch file reads/edits** — read all needed files in parallel, apply all edits together
- **Batch terminal commands** — chain with `&&` when appropriate
- **Batch search operations** — use grep alternation `word1|word2|word3` instead of separate searches

---

## File Organization

| Directory | Purpose |
|-----------|---------|
| `api/app/http/routes/` | HTTP route handlers (thin — validate, parse, delegate) |
| `api/app/modules/train/` | Train domain: router, service, worker, providers, schemas |
| `api/app/core/` | Config, security, rate limiting, logging, crypto |
| `api/app/db/` | SQLAlchemy models and session factory |
| `api/app/services/` | Cross-cutting services (auth, email, wallet) |
| `api/alembic/versions/` | Database migrations (additive only) |
| `api/tests/` | pytest tests |
| `web/app/` | Next.js App Router pages |
| `web/components/` | React components (UI, train, wallet, admin) |
| `web/lib/` | Shared helpers: `ui.ts`, `types.ts`, `theme.ts`, `kst.ts` |
| `infra/scripts/` | Deployment and setup scripts |
| `infra/env/` | Environment files (dev/, prod/) |
| `docs/` | Project documentation |

---

## Key Patterns

### Backend (FastAPI + Python)

| Pattern | Rule |
|---------|------|
| **Thin handlers** | Route handlers validate/parse only; business logic in `service.py` / `worker.py` |
| **Provider interface** | `TrainProvider` Protocol in `api/app/modules/train/providers/` |
| **Envelope encryption** | Random DEK per record, wrapped with `MASTER_KEY` — see `api/app/core/crypto/` |
| **Safe metadata** | Use `*_safe` fields for logs/stored metadata (redacted) |
| **Async everywhere** | AsyncSession, async Redis, async HTTP clients |
| **Session auth** | httpOnly cookies, SameSite=Lax, Secure in production only |
| **Argon2id passwords** | OWASP-recommended parameters in `api/app/core/security.py` |

### Frontend (Next.js + TypeScript)

| Pattern | Rule |
|---------|------|
| **UI tokens** | Reuse classes from `web/lib/ui.ts` (e.g., `UI_BUTTON`, `UI_CARD`) |
| **Types** | Keep `web/lib/types.ts` in sync with API response shapes |
| **KST times** | Display train times in Korea Standard Time using `web/lib/kst.ts` |
| **Zod validation** | Client-side form validation with Zod schemas |
| **Server auth** | Use `web/lib/server-auth.ts` for Server Component auth checks |

### Migrations

- Naming: `YYYYMMDD_NNNN_description.py`
- Additive-only — do not mutate old migrations
- Include `upgrade()` and `downgrade()` functions
- Seed data (roles, etc.) goes in migrations

---

## Quick Commands

```bash
# Start local dev (all services with hot reload)
docker compose -f infra/docker-compose.yml up --build

# Run backend tests
docker compose -f infra/docker-compose.yml exec api pytest -q

# Run frontend typecheck
docker compose -f infra/docker-compose.yml exec web npx tsc --noEmit

# Health check
curl -sS http://localhost:8000/health

# Monitor system status (one-time snapshot)
docker compose -f infra/docker-compose.yml exec api python -m app.monitor

# Monitor with live refresh
docker compose -f infra/docker-compose.yml exec api python -m app.monitor --watch

# Create migration
(cd api && alembic revision -m "describe_change")

# Zero-downtime production deploy (on VM)
sudo -u bominal /opt/bominal/repo/infra/scripts/deploy-zero-downtime.sh

# Production monitor (on VM)
/opt/bominal/repo/infra/scripts/bominal-monitor --watch

# Rollback production
sudo -u bominal /opt/bominal/repo/infra/scripts/deploy-zero-downtime.sh --rollback
```

---

## Task Routing

| Task Type | Files Likely Affected |
|-----------|----------------------|
| **Bug fix** | Specific module file, possibly test file |
| **New API endpoint** | `api/app/http/routes/` or `api/app/modules/*/router.py`, schemas, tests |
| **New feature** | Multiple: router, service, worker, schemas, web components |
| **Refactor** | Existing files only — no new files unless extracting module |
| **Security fix** | `api/app/core/security.py`, `api/app/core/crypto/`, possibly middleware |
| **UI change** | `web/components/`, possibly `web/lib/ui.ts` for new tokens |
| **DB schema change** | New migration in `api/alembic/versions/`, model in `api/app/db/models.py` |
| **Train provider** | `api/app/modules/train/providers/`, keep aligned with `third_party/srtgo/` |

---

## Task Complexity Detection

**Dig deeper** when task involves:
- Multiple files (3+)
- New feature implementation
- API changes with frontend updates
- Security-related changes
- Database schema changes
- Train provider integration

**Quick fix** for:
- Single file edits
- Simple bug fixes (1-2 lines)
- Documentation updates
- Configuration changes
- Typo corrections

---

## Security-Critical Areas

⚠️ **Extra caution required** — these areas handle sensitive data:

| Area | Location | Notes |
|------|----------|-------|
| **Credentials encryption** | `api/app/core/crypto/` | Envelope encryption for SRT/KTX credentials |
| **Password hashing** | `api/app/core/security.py` | Argon2id — do not change parameters without review |
| **Session tokens** | `api/app/core/security.py` | SHA-256 hashed before storage |
| **Payment cards** | `api/app/services/wallet_service.py` | CVV only in Redis with TTL, never in Postgres |
| **Redaction** | `api/app/core/crypto/redaction.py` | Always use for log output |
| **Cookie auth** | `api/app/http/deps.py` | Preserve httpOnly, SameSite, Secure flags |
| **MASTER_KEY** | Environment variable | Required for all encryption — rotation requires re-encryption |

**Never**:
- Log passwords, tokens, card numbers, CVV, or raw provider credentials
- Store CVV in database (Redis cache only with short TTL)
- Disable httpOnly on session cookies
- Commit `.env` files or hardcode secrets

---

## Provider Integration Rules

Train provider code in `api/app/modules/train/providers/` must stay **source-aligned** with:
- `third_party/srtgo/srtgo/srt.py` (SRT provider reference)
- `third_party/srtgo/srtgo/ktx.py` (KTX provider reference)

When updating provider integration:
1. Read the reference implementation in `third_party/srtgo/` first
2. Mirror the API call patterns and error handling
3. Use `*_safe` fields for any metadata stored or logged
4. Test with mock provider before real provider

---

## Definition of Done

Before marking a task complete:

- [ ] Build compiles (`npx tsc --noEmit` for web, imports work for api)
- [ ] Relevant backend tests pass (`pytest -q`)
- [ ] Docker compose stack starts cleanly
- [ ] No auth/session flow regressions
- [ ] No unresolved placeholders in production env templates
- [ ] Docs updated in `docs/` if behavior or operations changed

---

## Environment Quick Reference

| Service | Dev URL | Notes |
|---------|---------|-------|
| Web | http://localhost:3000 | Next.js with hot reload |
| API | http://localhost:8000 | FastAPI with auto-reload |
| API Docs | http://localhost:8000/api/docs | Admin-only (requires login) |
| Mailpit | http://localhost:8025 | Dev email UI |
| Postgres | localhost:5432 | Database |
| Redis | localhost:6379 | Queue, cache, rate limiting |

---

## First Files to Read

When starting work on this project:

1. `docs/README.md` — Project overview
2. `docs/ARCHITECTURE.md` — System design
3. `docs/CONTRIBUTING.md` — Development workflow
4. `AGENTS.md` — Detailed AI agent guidance
5. `api/app/core/config.py` — All configuration options

---

Remember: **Read before edit, batch operations, preserve security semantics.**
