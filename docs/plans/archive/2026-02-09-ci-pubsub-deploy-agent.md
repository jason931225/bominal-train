# Pub/Sub CI Deploy Agent Implementation Plan (Archived)

> Archived on 2026-02-14.
> Implementation is complete; this plan is retained for historical traceability.

## Summary

The Pub/Sub deploy path replaced direct SSH deploy orchestration from GitHub Actions. CI now publishes deploy requests, and a VM-side systemd agent consumes requests and runs `infra/scripts/deploy.sh` locally.

## Implemented Artifacts

- `.github/workflows/deploy.yml`
- `infra/scripts/vm-deploy-agent-pubsub.sh`
- `infra/systemd/bominal-deploy-agent.service`
- `docs/DEPLOYMENT.md`

## Runtime Notes

- Current deploy mode is latest-only from CI-triggered publish flow.
- Deterministic rollback remains handled by `infra/scripts/deploy.sh` and deployment records in `/opt/bominal/deployments`.

## Archive Notes

- This plan is not an active execution plan.
- Follow active deployment policy and docs in `docs/DEPLOYMENT.md` and `docs/EXECUTION_PROTOCOL.md` for new deploy work.
