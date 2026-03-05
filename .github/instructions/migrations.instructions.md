---
applyTo: "runtime/migrations/**"
---

Treat every migration as deploy-critical.

Require:

- forward-safe and operationally reversible changes
- compatibility with old and new code during rolling or partial deploys
- expand/contract instead of destructive rename/drop where possible
- careful defaults, backfills, and index creation to avoid long locks or large table rewrites
- explicit handling for nullability, uniqueness, data backfill, and rollback
- matching query/model/test updates when schema contracts change

Flag missing evidence for migration safety or rollout compatibility.
