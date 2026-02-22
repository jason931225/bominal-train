# Release Versioning

Human-readable versioning in bominal is enforced as SemVer-to-commit parity.

## Source Of Truth

- Canonical mapping file: `docs/releases/version-map.json`
- Each release entry MUST include:
  - semantic version (`X.Y.Z`)
  - exact git commit hash
  - declared bump type (`major`, `minor`, `patch`)
- A version MUST map to one commit, and a release commit MUST map to one version.

## Current Policy (Pre-1.0)

- Project versioning currently remains in `v0.0.N`.
- `N` is computed from commit history and enforced by `infra/scripts/version_guard.py`.
- The active baseline anchor is repository root:
  - `v0.0.1` -> `d2061306546cbb981fa14f75ca07fc9de9a7e2fd`
- Current and new commits continue resolving in the `0.0.#` line until an explicit `1.0.0` promotion is approved.

## Bump Rules

- While pre-1.0 mode is active, commits resolve as `0.0.#`.
- When explicit release milestones are added to `version-map.json`, bump validation applies:
  - `major`: `X+1.0.0`
  - `minor`: `X.Y+1.0`
  - `patch`: `X.Y.Z+1`

Invalid bump transitions are rejected by CI.

## CI Enforcement

- Validation script: `infra/scripts/version_guard.py`
- CI test: `infra/tests/test_versioning.sh`
- Infra workflow gate: `.github/workflows/infra-tests.yml`

## Usage

Validate registry:

```bash
python3 infra/scripts/version_guard.py validate
```

Resolve a commit to a human-readable version:

```bash
python3 infra/scripts/version_guard.py resolve --commit HEAD
```

Show baseline mapping:

```bash
python3 infra/scripts/version_guard.py baseline-version
python3 infra/scripts/version_guard.py baseline-commit
```
