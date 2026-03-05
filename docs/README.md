# Bominal Docs Index

Canonical documentation:
- `docs/MANUAL.md` - single source of policy, quality, operations, and target-state CI/CD controls.
- `docs/MANUAL.md#github-project-management-policy` - canonical GitHub governance contract (labels, issue/PR policy, project tracking, wiki linkage, milestones/tags, and branch protection baseline).
- `docs/MANUAL.md#project-tracking` - canonical three-board operating model (`bominal Workstreams`, `bominal Review`, `bominal Agent Command`) and automation expectations.
- `docs/START_HERE.md` - entrypoint for humans and agents.
- `docs/INTENT_ROUTING.md` - keyword router into manual sections.
- `docs/GUARDRAIL.MD` - explicit immutable boundary: `third_party/**` is read-only.
- `docs/PROD_ENV_CONTRACT.md` - key-by-key production env classification (`required` / `optional` / `secret-manager-only` / `must-be-false-in-prod` / `public-safe`).

Preserved reference set:
- `docs/handoff/**` - unchanged external handoff package and research references.

Repository-level governance entrypoints:
- `AGENTS.md` - mandatory constraints for automated contributors.
- `CHANGELOG.md` - commit-based change log.
- GitHub Wiki - active operational/onboarding/project coordination pages that must reference canonical policy anchors in `docs/MANUAL.md`.

Active implementation plans:
- `docs/plans/2026-03-02-runtime-test-backfill-srt-parity.md` - runtime test backfill and SRT parity rewrite plan.
- `docs/plans/2026-03-03-frontend-admin-ux-claude-prompt.md` - Claude prompt package for mobile/desktop frontend and admin maintenance UX design.
- `docs/plans/2026-03-04-http3-edge-maybe-todo.md` - deferred edge HTTP/3 evaluation backlog item.

Operational playbooks:
- `docs/playbooks/RUST_PRODUCTION_CUTOVER.md` - hard-cutover + rollback checklist for Rust production deploy.
- `docs/playbooks/GITHUB_PROJECT_AUTOMATION.md` - board topology, branch-promotion (`dev -> staging -> main`) automation policy, orchestrator issue contract, secondary-review disposition, and tested GH CLI/MCP command set for agents.
- `docs/playbooks/GITHUB_PROJECT_OPERATIONS.md` - tested PAT/MCP project-board operations runbook, automation validation flow, and secondary-review command policy (`@codex review`, risk-based `@copilot review`).

Note:
- Legacy split docs trees (governance/humans/agents/playbooks/plans/etc.) were intentionally removed from canonical usage.
