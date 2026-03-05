---
applyTo: "runtime/frontend/**"
---

This directory builds runtime-served assets, not a separate SPA.

Review for:

- generated-only `dist/**` discipline
- CSS/asset budget creep
- deterministic build output in CI and Docker
- Tailwind, Lightning CSS, or lightweight-charts changes that increase shipped bytes or break served asset names
- coordination with Rust SSR/runtime code when assets, paths, or DOM expectations change

Suggest build and budget verification for touched frontend changes.
