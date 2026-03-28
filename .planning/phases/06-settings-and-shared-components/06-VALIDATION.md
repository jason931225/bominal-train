---
phase: 6
slug: settings-and-shared-components
status: ready
nyquist_compliant: true
wave_0_complete: true
created: 2026-03-27
---

# Phase 6 — Validation Strategy

## Automated Verification Contract

| Scope | Command | Result |
|-------|---------|--------|
| App unit surface | `cargo test -p bominal-app --lib` | Required |
| App SSR build | `cargo check -p bominal-app --features ssr` | Required |
| App hydrate build | `cargo check -p bominal-app --target wasm32-unknown-unknown --features hydrate` | Required |

## Artifact Checks

| Artifact | Expectation |
|----------|-------------|
| [crates/bominal-app/src/pages/settings.rs](/Users/jasonlee/projects/bominal-train/.claude/worktrees/gsd-autonomous-phase4/crates/bominal-app/src/pages/settings.rs) | provider management, card management, appearance toggles, and logout in one protected page |
| [crates/bominal-app/src/components/glass_panel.rs](/Users/jasonlee/projects/bominal-train/.claude/worktrees/gsd-autonomous-phase4/crates/bominal-app/src/components/glass_panel.rs) | reusable SSR glass container surface |
| [crates/bominal-app/src/components/icon.rs](/Users/jasonlee/projects/bominal-train/.claude/worktrees/gsd-autonomous-phase4/crates/bominal-app/src/components/icon.rs) | normalized shared icon glyphs used across settings and cards |
| [crates/bominal-app/src/components/skeleton.rs](/Users/jasonlee/projects/bominal-train/.claude/worktrees/gsd-autonomous-phase4/crates/bominal-app/src/components/skeleton.rs) | loading placeholder components for SSR/hydrate states |
| [crates/bominal-app/src/components/card_brand.rs](/Users/jasonlee/projects/bominal-train/.claude/worktrees/gsd-autonomous-phase4/crates/bominal-app/src/components/card_brand.rs) | card-brand detection and formatted display helpers |
| [crates/bominal-app/src/components/bottom_sheet.rs](/Users/jasonlee/projects/bominal-train/.claude/worktrees/gsd-autonomous-phase4/crates/bominal-app/src/components/bottom_sheet.rs) | reusable interactive bottom-sheet shell |
| [crates/bominal-app/src/components/selection_prompt.rs](/Users/jasonlee/projects/bominal-train/.claude/worktrees/gsd-autonomous-phase4/crates/bominal-app/src/components/selection_prompt.rs) | interactive selection affordance used by the search flow |
| [crates/bominal-app/src/components/ticket_card.rs](/Users/jasonlee/projects/bominal-train/.claude/worktrees/gsd-autonomous-phase4/crates/bominal-app/src/components/ticket_card.rs) | shared train-result card surface |
| [crates/bominal-app/src/components/task_card.rs](/Users/jasonlee/projects/bominal-train/.claude/worktrees/gsd-autonomous-phase4/crates/bominal-app/src/components/task_card.rs) | shared task card surface with status display |
| [crates/bominal-app/assets/interop.js](/Users/jasonlee/projects/bominal-train/.claude/worktrees/gsd-autonomous-phase4/crates/bominal-app/assets/interop.js) | browser-side Evervault bridge required by the new add-card form |
| [crates/bominal-app/style/app.css](/Users/jasonlee/projects/bominal-train/.claude/worktrees/gsd-autonomous-phase4/crates/bominal-app/style/app.css) | shared Phase 6 glass, settings, theme, card, and sheet styles |

## Manual Verification

Optional browser validation is still recommended for appearance toggles, bottom-sheet behavior, and the encrypted card-add flow, but compile and unit verification are sufficient for this migration phase because the page logic stays on the existing backend/API contract and the Evervault runtime is isolated behind a small browser shim.
