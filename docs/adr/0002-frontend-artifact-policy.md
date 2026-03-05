# ADR 0002: Frontend Artifact Ownership Policy

- Status: Accepted
- Date: 2026-03-05

## Context

Tracked generated frontend outputs create drift and review noise, and they can diverge from the release image build path.

## Decision

- Treat `runtime/frontend/dist/**` as generated-only output, not a tracked source artifact.
- Build release frontend assets from source during Docker image builds.
- Keep CI checks that fail on tracked generated dist artifacts.
- Keep local developer workflows (`npm ci`, CSS build scripts, bootstrap/dev scripts) for fast iteration without changing source-of-truth rules.

## Consequences

- Release artifacts are reproducible from committed sources.
- CI and CD no longer depend on repository-committed generated CSS.
- Pull requests stay focused on source changes, not generated file churn.
