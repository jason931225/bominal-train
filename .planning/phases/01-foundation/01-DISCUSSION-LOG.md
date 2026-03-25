# Phase 1: Foundation Fixes - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-03-24
**Phase:** 01-foundation
**Areas discussed:** API parameter fix

---

## API Parameter Fix

| Option | Description | Selected |
|--------|-------------|----------|
| Fix frontend (Recommended) | Change frontend to send `q` — matches existing backend contract, zero backend change | ✓ |
| Fix backend | Change backend SuggestQuery field from `q` to `query` — more descriptive but modifies API contract | |
| You decide | Claude picks whichever is cleanest | |

**User's choice:** Fix frontend
**Notes:** User chose the recommended option — keep backend stable, fix the caller.

---

## Search Button Logic

No discussion needed — straightforward addition of `!date` to the disabled condition. Success criteria from SRCH-06 is unambiguous.

---

## Claude's Discretion

- Variable naming cosmetics in `suggestStations()`
- Inline comment on disabled logic

## Deferred Ideas

None
