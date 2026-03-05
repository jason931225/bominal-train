use std::sync::Arc;

use axum::Router;

use super::super::AppState;

mod api;
mod pages;

pub(super) fn register(router: Router<Arc<AppState>>) -> Router<Arc<AppState>> {
    let router = pages::register(router);
    api::register(router)
}
