# Deprecation Inventory (2026-02-14)

This inventory tracks runtime artifacts under deprecation governance.

## DEP-2026-02-14-001

- path: `infra/docker-compose.deploy.yml.deprecated`
- owner: Infra / Deployment
- reason deprecated: superseded by canonical production topology in `infra/docker-compose.prod.yml` and script-driven deploy flow in `infra/scripts/deploy-zero-downtime.sh`
- active replacement:
  - `infra/docker-compose.prod.yml`
  - `infra/scripts/deploy-zero-downtime.sh`
- known callers:
  - no active runtime/script callers in repository automation
  - historical/manual operator usage only
- removal gate criteria:
  - compatibility notices published in operator docs
  - no active runtime references in `infra/scripts/**` or compose files
  - deprecation guard test passes
- compatibility window opened: `2026-02-14`
- removed at: `2026-02-14`
- status: `removed`
- removal evidence:
  - artifact deleted: `infra/docker-compose.deploy.yml.deprecated`
  - guard test: `infra/tests/test_deprecation_references.sh`
