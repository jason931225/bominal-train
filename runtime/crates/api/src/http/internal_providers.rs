use std::sync::Arc;

use axum::{
    Json, Router,
    extract::{Path, State},
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
            "/internal/v1/providers/{provider}/credentials",
            put(put_provider_credentials),
        )
        .route(
            "/internal/v1/providers/{provider}/payment-method",
            put(put_provider_payment_method),
        );

    if mount_aliases {
        return router
            .route(
                "/api/internal/providers/{provider}/credentials",
                put(put_provider_credentials),
            )
            .route(
                "/api/internal/providers/{provider}/payment-method",
                put(put_provider_payment_method),
            );
    }

    router
}

async fn put_provider_credentials(
    State(state): State<Arc<AppState>>,
    Path(provider): Path<String>,
    headers: HeaderMap,
    Json(payload): Json<provider_credentials_service::PutProviderCredentialsRequest>,
) -> impl IntoResponse {
    match provider_credentials_service::put_provider_credentials(
        state.as_ref(),
        provider.as_str(),
        payload,
    )
    .await
    {
        Ok(result) => (StatusCode::ACCEPTED, Json(result)).into_response(),
        Err(provider_credentials_service::PutProviderCredentialsError::ValidationFailed) => {
            ApiError::new(
                ApiErrorStatus::BadRequest,
                ApiErrorCode::InvalidRequest,
                "invalid provider credentials payload",
                request_id_from_headers(&headers),
            )
            .into_response()
        }
        Err(provider_credentials_service::PutProviderCredentialsError::PersistenceUnavailable)
        | Err(provider_credentials_service::PutProviderCredentialsError::CryptoUnavailable) => {
            ApiError::new(
                ApiErrorStatus::ServiceUnavailable,
                ApiErrorCode::ServiceUnavailable,
                "provider credentials persistence unavailable",
                request_id_from_headers(&headers),
            )
            .into_response()
        }
        Err(provider_credentials_service::PutProviderCredentialsError::PersistenceFailure) => {
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

async fn put_provider_payment_method(
    State(state): State<Arc<AppState>>,
    Path(provider): Path<String>,
    headers: HeaderMap,
    Json(payload): Json<payment_method_service::PutProviderPaymentMethodRequest>,
) -> impl IntoResponse {
    match payment_method_service::put_provider_payment_method(
        state.as_ref(),
        provider.as_str(),
        payload,
    )
    .await
    {
        Ok(result) => (StatusCode::ACCEPTED, Json(result)).into_response(),
        Err(payment_method_service::PutProviderPaymentMethodError::ValidationFailed) => {
            ApiError::new(
                ApiErrorStatus::BadRequest,
                ApiErrorCode::InvalidRequest,
                "invalid payment payload",
                request_id_from_headers(&headers),
            )
            .into_response()
        }
        Err(payment_method_service::PutProviderPaymentMethodError::PersistenceUnavailable)
        | Err(payment_method_service::PutProviderPaymentMethodError::CryptoUnavailable) => {
            ApiError::new(
                ApiErrorStatus::ServiceUnavailable,
                ApiErrorCode::ServiceUnavailable,
                "payment method persistence unavailable",
                request_id_from_headers(&headers),
            )
            .into_response()
        }
        Err(payment_method_service::PutProviderPaymentMethodError::PersistenceFailure) => {
            ApiError::new(
                ApiErrorStatus::InternalServerError,
                ApiErrorCode::InternalError,
                "payment method persistence failed",
                request_id_from_headers(&headers),
            )
            .into_response()
        }
    }
}
