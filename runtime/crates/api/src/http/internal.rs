use std::sync::Arc;

use axum::{Router, routing::get};

use super::super::AppState;

pub(super) fn register(router: Router<Arc<AppState>>) -> Router<Arc<AppState>> {
    router
        .route("/favicon.ico", get(super::super::favicon_placeholder))
        .route("/", get(super::super::ssr_auth_landing))
        .route("/auth", get(super::super::ssr_auth_alias))
        .route("/dev/ui", get(super::super::ssr_dev_ui))
        .route("/dashboard", get(super::super::ssr_dashboard))
        .route("/dashboard/train", get(super::super::ssr_dashboard_train))
        .route("/dashboard/jobs", get(super::super::ssr_dashboard_jobs))
        .route(
            "/dashboard/jobs/{job_id}",
            get(super::super::ssr_dashboard_job_detail),
        )
        .route(
            "/dashboard/settings",
            get(super::super::ssr_dashboard_settings),
        )
        .route(
            "/dashboard/settings/providers",
            get(super::super::ssr_dashboard_settings_providers),
        )
        .route(
            "/dashboard/payment",
            get(super::super::ssr_dashboard_payment),
        )
        .route(
            "/admin/maintenance",
            get(super::super::ssr_admin_maintenance),
        )
        .route("/admin/users", get(super::super::ssr_admin_users))
        .route("/admin/runtime", get(super::super::ssr_admin_runtime))
        .route(
            "/admin/observability",
            get(super::super::ssr_admin_observability),
        )
        .route("/admin/security", get(super::super::ssr_admin_security))
        .route("/admin/config", get(super::super::ssr_admin_config))
        .route("/admin/audit", get(super::super::ssr_admin_audit))
        .route(
            "/admin/maintenance/metrics",
            get(super::super::admin_maintenance_metrics),
        )
        .route("/health", get(super::super::health_live))
        .route("/ready", get(super::super::health_ready))
}
