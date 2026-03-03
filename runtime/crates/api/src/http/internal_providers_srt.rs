use std::sync::Arc;

use axum::{
    Json, Router,
    extract::State,
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
    routing::put,
};
use bominal_shared::error::{ApiError, ApiErrorCode, ApiErrorStatus};

use super::super::{
    AppState, request_id_from_headers,
    services::{payment_method_service, provider_credentials_service},
};

pub(super) fn register(
    router: Router<Arc<AppState>>,
    mount_aliases: bool,
) -> Router<Arc<AppState>> {
    let router = router
        .route(
            "/internal/v1/providers/srt/credentials",
            put(put_srt_credentials),
        )
        .route(
            "/internal/v1/providers/srt/payment-method",
            put(put_srt_payment_method),
        );

    if mount_aliases {
        return router
            .route(
                "/api/internal/providers/srt/credentials",
                put(put_srt_credentials),
            )
            .route(
                "/api/internal/providers/srt/payment-method",
                put(put_srt_payment_method),
            );
    }

    router
}

async fn put_srt_credentials(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(payload): Json<provider_credentials_service::PutSrtCredentialsRequest>,
) -> impl IntoResponse {
    match provider_credentials_service::put_srt_credentials(state.as_ref(), payload).await {
        Ok(result) => (StatusCode::ACCEPTED, Json(result)).into_response(),
        Err(provider_credentials_service::PutSrtCredentialsError::ValidationFailed) => {
            ApiError::new(
                ApiErrorStatus::BadRequest,
                ApiErrorCode::InvalidRequest,
                "invalid provider credentials payload",
                request_id_from_headers(&headers),
            )
            .into_response()
        }
        Err(provider_credentials_service::PutSrtCredentialsError::PersistenceUnavailable)
        | Err(provider_credentials_service::PutSrtCredentialsError::CryptoUnavailable) => {
            ApiError::new(
                ApiErrorStatus::ServiceUnavailable,
                ApiErrorCode::ServiceUnavailable,
                "provider credentials persistence unavailable",
                request_id_from_headers(&headers),
            )
            .into_response()
        }
        Err(provider_credentials_service::PutSrtCredentialsError::PersistenceFailure) => {
            ApiError::new(
                ApiErrorStatus::InternalServerError,
                ApiErrorCode::InternalError,
                "provider credentials persistence failed",
                request_id_from_headers(&headers),
            )
            .into_response()
        }
    }
}

async fn put_srt_payment_method(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(payload): Json<payment_method_service::PutSrtPaymentMethodRequest>,
) -> impl IntoResponse {
    match payment_method_service::put_srt_payment_method(state.as_ref(), payload).await {
        Ok(result) => (StatusCode::ACCEPTED, Json(result)).into_response(),
        Err(payment_method_service::PutSrtPaymentMethodError::ValidationFailed) => ApiError::new(
            ApiErrorStatus::BadRequest,
            ApiErrorCode::InvalidRequest,
            "invalid payment payload",
            request_id_from_headers(&headers),
        )
        .into_response(),
        Err(payment_method_service::PutSrtPaymentMethodError::PersistenceUnavailable)
        | Err(payment_method_service::PutSrtPaymentMethodError::CryptoUnavailable) => {
            ApiError::new(
                ApiErrorStatus::ServiceUnavailable,
                ApiErrorCode::ServiceUnavailable,
                "payment method persistence unavailable",
                request_id_from_headers(&headers),
            )
            .into_response()
        }
        Err(payment_method_service::PutSrtPaymentMethodError::PersistenceFailure) => ApiError::new(
            ApiErrorStatus::InternalServerError,
            ApiErrorCode::InternalError,
            "payment method persistence failed",
            request_id_from_headers(&headers),
        )
        .into_response(),
    }
}
