# Deploy Runbook

## Purpose

Execute production deploys for `bominal` using the protected GitHub CD workflow path.

## Preconditions

- Deploy source is protected `main`; the current production CD workflow does not deploy from tag refs.
- GitHub `production` environment is configured with required variables.
- VM deploy scripts and runtime env contracts are present and current.

## Procedure

1. Confirm scope and linked issue/PR metadata are complete.
2. Confirm release artifact identity (image digest/version metadata).
3. Trigger `.github/workflows/cd.yml` from the `main` ref through the approved path.
   For `workflow_dispatch`, set the required `deploy=true` input or the workflow will only publish images without executing the production deploy job.
4. Verify deploy sequence:
   - remote deploy script execution
   - migration application
   - service restart
   - health checks (`/health`, `/ready`)
5. Capture deployment decision log and post-deploy verification summary.

## Success Criteria

- API and worker are healthy.
- No active rollback trigger criteria are met.
- Deployment evidence is recorded in workflow logs and release notes.

## References

- `docs/MANUAL.md#deployment-and-rollback-standard`
- `docs/playbooks/RUST_PRODUCTION_CUTOVER.md`
