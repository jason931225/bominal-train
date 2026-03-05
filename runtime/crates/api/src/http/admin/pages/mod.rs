use std::sync::Arc;

use axum::Router;

use super::super::super::AppState;

pub(super) fn register(router: Router<Arc<AppState>>) -> Router<Arc<AppState>> {
    router
}
