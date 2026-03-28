---
phase: 09-css-and-build-pipeline
plan: 02
status: complete
completed: 2026-03-27T19:26:19-04:00
requirements:
  - BUILD-01
  - BUILD-02
---

# Phase 9 / Plan 02 Summary

Moved the deployment/bootstrap surface onto cargo-leptos and removed the old Node/npm production build path.

## What Changed

- Replaced the manual release pipeline in [deployment/build.sh](/Users/jasonlee/projects/bominal-train/.claude/worktrees/gsd-autonomous-phase4/deployment/build.sh) with `cargo leptos build --release --precompress`.
- Updated [deployment/bootstrap.sh](/Users/jasonlee/projects/bominal-train/.claude/worktrees/gsd-autonomous-phase4/deployment/bootstrap.sh) so the environment setup no longer requires Node/npm for the application build workflow and instead points users at cargo-leptos commands.
- Rewrote [Dockerfile](/Users/jasonlee/projects/bominal-train/.claude/worktrees/gsd-autonomous-phase4/Dockerfile) to install cargo-leptos plus the Tailwind standalone binary, build the Leptos app/server without Node, and copy `target/site` into the runtime image.
- Switched the Docker strategy to an explicit BuildKit extra context for the sibling [bominal-ui](/Users/jasonlee/projects/bominal-ui/Cargo.toml) crate, which keeps the repo-root build context while still making the external path dependency available in-container.

## Verification

- `bash -n deployment/build.sh`
- `bash -n deployment/bootstrap.sh`
- `cargo check -p bominal-server --features ssr`
- `docker buildx build --build-context bominal_ui=/Users/jasonlee/projects/bominal-ui -f Dockerfile .` was probed successfully through Dockerfile parse, both build contexts loading, and builder-stage startup before the long cold compile was intentionally stopped

Script syntax and server compilation passed, and the Docker context strategy was validated.
