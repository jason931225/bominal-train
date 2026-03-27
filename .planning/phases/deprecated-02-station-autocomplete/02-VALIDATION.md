---
phase: 2
slug: station-autocomplete
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-03-25
---

# Phase 2 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | vitest (frontend unit/component tests) + manual browser verification |
| **Config file** | `frontend/vite.config.ts` (vitest config inline or separate `vitest.config.ts`) |
| **Quick run command** | `cd frontend && npx vitest run --reporter=verbose` |
| **Full suite command** | `cd frontend && npx vitest run --coverage` |
| **Estimated runtime** | ~10 seconds |

---

## Sampling Rate

- **After every task commit:** Run `cd frontend && npx vitest run --reporter=verbose`
- **After every plan wave:** Run `cd frontend && npx vitest run --coverage`
- **Before `/gsd:verify-work`:** Full suite must be green
- **Max feedback latency:** 15 seconds

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|-----------|-------------------|-------------|--------|
| TBD | TBD | TBD | SRCH-01 | unit + integration | `npx vitest run StationInput` | ❌ W0 | ⬜ pending |
| TBD | TBD | TBD | SRCH-02 | unit | `npx vitest run StationInput --grep IME` | ❌ W0 | ⬜ pending |
| TBD | TBD | TBD | SRCH-03 | unit | `npx vitest run StationInput --grep keyboard` | ❌ W0 | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

- [ ] `frontend/src/lib/components/__tests__/StationInput.test.ts` — stubs for SRCH-01, SRCH-02, SRCH-03
- [ ] vitest + @testing-library/svelte installed as devDependencies
- [ ] `frontend/vitest.config.ts` or vitest block in `vite.config.ts`

*Wave 0 must complete before any component implementation begins.*

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| IME composition with real Korean keyboard | SRCH-02 | jsdom cannot simulate real IME events | Type Korean characters in the input on Chrome/Firefox, verify no premature API calls during composition |
| Visual glass morphism dropdown rendering | N/A (UI-SPEC) | CSS glass effects not testable in jsdom | Open dropdown in light and dark mode, verify glass panel backdrop-filter and transparency |
| Focus/blur timing with mousedown | SRCH-03 | Browser event ordering not reproducible in jsdom | Click a dropdown option, verify selection commits before blur hides dropdown |

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify or Wave 0 dependencies
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all MISSING references
- [ ] No watch-mode flags
- [ ] Feedback latency < 15s
- [ ] `nyquist_compliant: true` set in frontmatter

**Approval:** pending
