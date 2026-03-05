# Performance Baseline

This directory holds repeatable baseline probes for runtime/internal APIs.

Current baseline script:

- `perf/k6/internal-api-baseline.js`

It covers:

- internal provider job create/get/events path (`/internal/v1/provider-jobs*`)
- internal provider credential write path (`/internal/v1/providers/{provider}/credentials`)
- internal provider payment write path (`/internal/v1/providers/{provider}/payment-method`)
- admin observability timeseries (`/api/admin/observability/timeseries`) when `K6_ADMIN_COOKIE` is provided
- admin runtime jobs SSE endpoint reachability (`/api/admin/runtime/jobs/stream`) when `K6_ADMIN_COOKIE` is provided

Run manually with GitHub Actions:

- workflow: `.github/workflows/perf.yml`
- required inputs: `base_url`, `service_token`
- optional input: `admin_cookie`

The workflow uploads `perf-baseline-k6-summary` as the baseline artifact.
