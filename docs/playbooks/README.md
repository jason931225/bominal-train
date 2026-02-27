# Playbooks

Reusable, high-signal procedures for complex and expensive workflows.

## When to add a playbook

Add or update a playbook when a task required:
- Non-obvious sequencing.
- Network/payload reverse engineering.
- Repeated retries to get deterministic behavior.
- Special safety checks before destructive/expensive actions.

## Required format

Use `docs/playbooks/TEMPLATE.md` and keep all sections complete.

## Current playbooks

- `docs/playbooks/daily-operations-chores.md` - routine daily workflow for validation, hygiene, and coordination.
- `docs/playbooks/new-module-feature-workflow.md` - standardized end-to-end workflow for new module/feature delivery.
- `docs/playbooks/resy-widget-form-data-capture.md` - Resy widget/form-data capture and replay protocol.
- `docs/playbooks/restaurant-provider-adapter-workflow.md` - canonical implementation workflow for restaurant provider adapters.
- `docs/playbooks/provider-adapter-contract-template.md` - reusable contract template for onboarding new providers.
- `docs/playbooks/release-1.0.0-deploy-readiness.md` - release readiness and production deployment checklist for `1.0.0` promotion.
