use std::sync::Arc;

use axum::{Router, routing::get};

use super::super::AppState;

pub(super) fn register(router: Router<Arc<AppState>>) -> Router<Arc<AppState>> {
    router
        .route("/", get(super::super::ssr_home))
        .route("/health/live", get(super::super::health_live))
        .route("/health/ready", get(super::super::health_ready))
}
