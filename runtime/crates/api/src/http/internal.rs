use std::sync::Arc;

use axum::{Router, routing::get};

use super::super::AppState;

pub(super) fn register(router: Router<Arc<AppState>>) -> Router<Arc<AppState>> {
    router
        .route("/", get(super::super::ssr_home))
        .route("/auth", get(super::super::ssr_auth))
        .route(
            "/admin/maintenance",
            get(super::super::ssr_admin_maintenance),
        )
        .route(
            "/admin/maintenance/metrics",
            get(super::super::admin_maintenance_metrics),
        )
        .route("/health", get(super::super::health_live))
        .route("/ready", get(super::super::health_ready))
}
