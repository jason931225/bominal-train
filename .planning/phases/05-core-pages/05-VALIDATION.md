---
phase: 5
slug: core-pages
status: ready
nyquist_compliant: true
wave_0_complete: true
created: 2026-03-27
---

# Phase 5 — Validation Strategy

## Automated Verification Contract

| Scope | Command | Result |
|-------|---------|--------|
| App unit surface | `cargo test -p bominal-app --lib` | Required |
| App SSR build | `cargo check -p bominal-app --features ssr` | Required |
| App hydrate build | `cargo check -p bominal-app --target wasm32-unknown-unknown --features hydrate` | Required |

## Artifact Checks

| Artifact | Expectation |
|----------|-------------|
| [crates/bominal-app/src/pages/home.rs](/Users/jasonlee/projects/bominal-train/.claude/worktrees/gsd-autonomous-phase4/crates/bominal-app/src/pages/home.rs) | home dashboard with active-task summary and refresh affordance |
| [crates/bominal-app/src/pages/search.rs](/Users/jasonlee/projects/bominal-train/.claude/worktrees/gsd-autonomous-phase4/crates/bominal-app/src/pages/search.rs) | provider toggle, station autocomplete, time/date controls, results, and task creation |
| [crates/bominal-app/src/pages/tasks.rs](/Users/jasonlee/projects/bominal-train/.claude/worktrees/gsd-autonomous-phase4/crates/bominal-app/src/pages/tasks.rs) | active/completed tabs, SSE refresh, and swipe-reveal cancel |
| [crates/bominal-app/src/pages/reservations.rs](/Users/jasonlee/projects/bominal-train/.claude/worktrees/gsd-autonomous-phase4/crates/bominal-app/src/pages/reservations.rs) | provider-filtered reservations with ticket expansion and pay/cancel/refund actions |
| [crates/bominal-app/src/components/sse_reload.rs](/Users/jasonlee/projects/bominal-train/.claude/worktrees/gsd-autonomous-phase4/crates/bominal-app/src/components/sse_reload.rs) | browser-side SSE bridge for live task refresh |
| [crates/bominal-app/style/app.css](/Users/jasonlee/projects/bominal-train/.claude/worktrees/gsd-autonomous-phase4/crates/bominal-app/style/app.css) | shared Phase 5 card, field, tab, status, and swipe styles |

## Manual Verification

Optional browser validation is still recommended for touch-oriented affordances like the pull-to-refresh hint and swipe-reveal cancel pattern, but compile/unit validation is sufficient for this migration phase because the pages only orchestrate existing backend endpoints.
