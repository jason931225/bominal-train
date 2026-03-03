use std::sync::Arc;

use axum::{
    Json, Router,
    extract::State,
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
    routing::{get, post},
};
use bominal_shared::{
    error::{ApiError, ApiErrorCode, ApiErrorStatus},
    supabase::SupabaseClaims,
};

use super::super::{AppState, request_id_from_headers, services::auth_service};

#[derive(Debug, serde::Serialize)]
struct VerifySupabaseTokenResponse {
    valid: bool,
    claims: SupabaseClaims,
}

pub(super) fn register(router: Router<Arc<AppState>>) -> Router<Arc<AppState>> {
    router
        .route("/api/auth/supabase/verify", get(verify_supabase_token))
        .route("/api/auth/supabase/webhook", post(supabase_auth_webhook))
}

async fn verify_supabase_token(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
) -> impl IntoResponse {
    match auth_service::verify_supabase_token(state.as_ref(), &headers).await {
        Ok(claims) => (
            StatusCode::OK,
            Json(VerifySupabaseTokenResponse {
                valid: true,
                claims,
            }),
        )
            .into_response(),
        Err(auth_service::VerifySupabaseTokenError::MissingBearerToken) => ApiError::new(
            ApiErrorStatus::Unauthorized,
            ApiErrorCode::InvalidRequest,
            "missing bearer token",
            request_id_from_headers(&headers),
        )
        .into_response(),
        Err(auth_service::VerifySupabaseTokenError::JwksUnavailable) => ApiError::new(
            ApiErrorStatus::ServiceUnavailable,
            ApiErrorCode::ServiceUnavailable,
            "jwks unavailable",
            request_id_from_headers(&headers),
        )
        .into_response(),
        Err(auth_service::VerifySupabaseTokenError::Unauthorized) => ApiError::new(
            ApiErrorStatus::Unauthorized,
            ApiErrorCode::Unauthorized,
            "invalid token",
            request_id_from_headers(&headers),
        )
        .into_response(),
    }
}

async fn supabase_auth_webhook(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(payload): Json<auth_service::SupabaseAuthWebhook>,
) -> impl IntoResponse {
    match auth_service::process_supabase_auth_webhook(state.as_ref(), &headers, &payload).await {
        Ok(()) => (StatusCode::ACCEPTED, "ok").into_response(),
        Err(auth_service::SupabaseAuthWebhookError::SecretMismatch) => ApiError::new(
            ApiErrorStatus::Unauthorized,
            ApiErrorCode::Unauthorized,
            "invalid webhook secret",
            request_id_from_headers(&headers),
        )
        .into_response(),
        Err(auth_service::SupabaseAuthWebhookError::PersistenceFailure) => ApiError::new(
            ApiErrorStatus::InternalServerError,
            ApiErrorCode::InternalError,
            "auth sync persistence failed",
            request_id_from_headers(&headers),
        )
        .into_response(),
    }
}
