# Documentation Policy

## Goals

- keep docs discoverable and accurate
- prevent policy/procedure drift
- keep updates tied to behavior changes

## Workflow

### Docs-first
- route intent via `docs/INTENT_ROUTING.md`
- use `docs/README.md` pointer library

### In-flight
- update docs in the same change as behavior/ops changes
- keep edits scoped and additive where possible

### Docs-last
- re-read changed docs
- ensure pointers resolve
- update `CHANGELOG.md` for notable behavior/ops/docs changes

## Audience Split Rule

- governance docs are canonical for policy
- human docs are procedures
- agent docs are overlays

## New Canonical Docs

Any new canonical doc must be registered in `docs/README.md`.
