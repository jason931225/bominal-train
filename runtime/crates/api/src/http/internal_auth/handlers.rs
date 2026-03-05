use std::sync::Arc;

use axum::{
    Json,
    extract::State,
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
};
use bominal_shared::error::{ApiError, ApiErrorCode, ApiErrorStatus};

use super::super::super::{AppState, request_id_from_headers, services::auth_service};

pub(super) async fn create_invite(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(payload): Json<auth_service::CreateInviteRequest>,
) -> impl IntoResponse {
    match auth_service::create_invite(state.as_ref(), payload).await {
        Ok(response) => (StatusCode::CREATED, Json(response)).into_response(),
        Err(auth_service::AuthServiceError::InvalidRequest(message))
        | Err(auth_service::AuthServiceError::Conflict(message)) => ApiError::new(
            ApiErrorStatus::BadRequest,
            ApiErrorCode::InvalidRequest,
            message,
            request_id_from_headers(&headers),
        )
        .into_response(),
        Err(auth_service::AuthServiceError::Unauthorized(message))
        | Err(auth_service::AuthServiceError::NotFound(message)) => ApiError::new(
            ApiErrorStatus::Unauthorized,
            ApiErrorCode::Unauthorized,
            message,
            request_id_from_headers(&headers),
        )
        .into_response(),
        Err(auth_service::AuthServiceError::ServiceUnavailable(message)) => ApiError::new(
            ApiErrorStatus::ServiceUnavailable,
            ApiErrorCode::ServiceUnavailable,
            message,
            request_id_from_headers(&headers),
        )
        .into_response(),
        Err(auth_service::AuthServiceError::Internal) => ApiError::new(
            ApiErrorStatus::InternalServerError,
            ApiErrorCode::InternalError,
            "failed to create invite",
            request_id_from_headers(&headers),
        )
        .into_response(),
    }
}
