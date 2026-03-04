# Bominal Docs Index

Canonical documentation:
- `docs/MANUAL.md` - single source of policy, quality, operations, and target-state CI/CD controls.
- `docs/START_HERE.md` - entrypoint for humans and agents.
- `docs/INTENT_ROUTING.md` - keyword router into manual sections.
- `docs/GUARDRAIL.MD` - explicit immutable boundary: `third_party/**` is read-only.
- `docs/PROD_ENV_CONTRACT.md` - key-by-key production env classification (`required` / `optional` / `secret-manager-only` / `must-be-false-in-prod` / `public-safe`).

Preserved reference set:
- `docs/handoff/**` - unchanged external handoff package and research references.

Repository-level governance entrypoints:
- `AGENTS.md` - mandatory constraints for automated contributors.
- `CHANGELOG.md` - commit-based change log.

Active implementation plans:
- `docs/plans/2026-03-02-runtime-test-backfill-srt-parity.md` - runtime test backfill and SRT parity rewrite plan.
- `docs/plans/2026-03-03-frontend-admin-ux-claude-prompt.md` - Claude prompt package for mobile/desktop frontend and admin maintenance UX design.
- `docs/plans/2026-03-04-http3-edge-maybe-todo.md` - deferred edge HTTP/3 evaluation backlog item.

Operational playbooks:
- `docs/playbooks/RUST_PRODUCTION_CUTOVER.md` - hard-cutover + rollback checklist for Rust production deploy.

Note:
- Legacy split docs trees (governance/humans/agents/playbooks/plans/etc.) were intentionally removed from canonical usage.
