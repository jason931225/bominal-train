use std::sync::Arc;

use axum::{Router, routing::get};

use super::super::AppState;

pub(super) fn register(router: Router<Arc<AppState>>) -> Router<Arc<AppState>> {
    router.route("/api/modules", get(super::super::list_modules))
}
