use std::sync::Arc;

use axum::{
    extract::{Request, State},
    middleware::Next,
    response::Response,
};

use super::super::super::AppState;

pub(super) async fn require_service_jwt(
    State(state): State<Arc<AppState>>,
    request: Request,
    next: Next,
) -> Response {
    super::service_identity::require_service_jwt_impl(State(state), request, next).await
}
