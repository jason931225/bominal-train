mod guards;
mod handlers;
mod invites;
mod middleware;
mod service_identity;

use std::sync::Arc;

use axum::{
    Router,
    extract::{Request, State},
    middleware::Next,
    response::Response,
};

use super::super::AppState;

pub(super) fn compatibility_aliases_enabled(state: &AppState) -> bool {
    guards::compatibility_aliases_enabled(state)
}

pub(super) fn register_invites(
    router: Router<Arc<AppState>>,
    mount_aliases: bool,
) -> Router<Arc<AppState>> {
    invites::register_invites(router, mount_aliases)
}

pub(super) async fn require_service_jwt(
    state: State<Arc<AppState>>,
    request: Request,
    next: Next,
) -> Response {
    middleware::require_service_jwt(state, request, next).await
}
