# Agent Documentation Overlay

This file defines agent-specific documentation workflow constraints.

Canonical policy is `docs/governance/DOCUMENTATION_POLICY.md`.

## Agent Rules

- Docs-first: route intent and open canonical docs before edits.
- In-flight: update docs in the same change as behavior/ops changes.
- Docs-last: re-read changed docs and ensure pointer/changelog parity.
- Prefer links to canonical policy over duplicated policy text.

## Required Validation (Docs Work)

- `bash infra/tests/test_docs_pointers.sh`
- `bash infra/tests/test_intent_routing.sh`
- `bash infra/tests/test_docs_consistency.sh`
- `bash infra/tests/test_changelog.sh`
