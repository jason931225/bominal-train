use std::sync::Arc;

use axum::{
    Json, Router,
    extract::{Path, Query, State},
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
    routing::{delete, get, post, put},
};
use bominal_shared::error::{ApiError, ApiErrorCode, ApiErrorStatus};
use tracing::{error, warn};

use super::super::{
    AppState, request_id_from_headers,
    services::{auth_service, payment_method_service, train_service},
};

pub(super) fn register(router: Router<Arc<AppState>>) -> Router<Arc<AppState>> {
    router
        .route("/api/train/preflight", get(get_preflight))
        .route("/api/train/stations/suggest", get(get_station_suggestions))
        .route(
            "/api/train/search",
            post(create_search).get(list_search_history),
        )
        .route("/api/train/search/{search_id}", get(get_search))
        .route(
            "/api/train/providers/{provider}/credentials",
            put(put_provider_credentials).delete(delete_provider_credentials),
        )
        .route(
            "/api/train/providers/{provider}/payment-method",
            put(put_provider_payment_method),
        )
        .route(
            "/api/train/payment-methods",
            get(list_payment_methods).put(put_universal_payment_method),
        )
        .route(
            "/api/train/payment-methods/{payment_method_ref}",
            delete(delete_payment_method),
        )
}

async fn get_preflight(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
) -> impl IntoResponse {
    let session = match auth_service::require_session_state(state.as_ref(), &headers).await {
        Ok(session) => session,
        Err(err) => return map_auth_error(err, &headers).into_response(),
    };

    match train_service::load_preflight(state.as_ref(), &session.user_id).await {
        Ok(result) => (StatusCode::OK, Json(result)).into_response(),
        Err(err) => map_train_error(err, &headers).into_response(),
    }
}

async fn get_station_suggestions(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Query(query): Query<train_service::StationSuggestQuery>,
) -> impl IntoResponse {
    if let Err(err) = auth_service::require_session_state(state.as_ref(), &headers).await {
        return map_auth_error(err, &headers).into_response();
    }

    match train_service::suggest_stations(state.as_ref(), query).await {
        Ok(result) => (StatusCode::OK, Json(result)).into_response(),
        Err(err) => map_train_error(err, &headers).into_response(),
    }
}

async fn create_search(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(payload): Json<train_service::CreateTrainSearchRequest>,
) -> impl IntoResponse {
    let session = match auth_service::require_session_state(state.as_ref(), &headers).await {
        Ok(session) => session,
        Err(err) => return map_auth_error(err, &headers).into_response(),
    };

    match train_service::create_search(state.as_ref(), &session.user_id, payload).await {
        Ok(result) => (StatusCode::ACCEPTED, Json(result)).into_response(),
        Err(err) => map_train_error(err, &headers).into_response(),
    }
}

async fn get_search(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Path(search_id): Path<String>,
) -> impl IntoResponse {
    let session = match auth_service::require_session_state(state.as_ref(), &headers).await {
        Ok(session) => session,
        Err(err) => return map_auth_error(err, &headers).into_response(),
    };

    match train_service::get_search(state.as_ref(), &session.user_id, &search_id).await {
        Ok(result) => (StatusCode::OK, Json(result)).into_response(),
        Err(err) => map_train_error(err, &headers).into_response(),
    }
}

async fn list_search_history(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Query(query): Query<train_service::SearchHistoryQuery>,
) -> impl IntoResponse {
    let session = match auth_service::require_session_state(state.as_ref(), &headers).await {
        Ok(session) => session,
        Err(err) => return map_auth_error(err, &headers).into_response(),
    };

    let limit = query.limit.unwrap_or(20);
    match train_service::list_search_history(state.as_ref(), &session.user_id, limit).await {
        Ok(result) => (StatusCode::OK, Json(result)).into_response(),
        Err(err) => map_train_error(err, &headers).into_response(),
    }
}

async fn put_provider_credentials(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Path(provider): Path<String>,
    Json(payload): Json<train_service::PutTrainProviderCredentialsRequest>,
) -> impl IntoResponse {
    let session = match auth_service::require_session_state(state.as_ref(), &headers).await {
        Ok(session) => session,
        Err(err) => return map_auth_error(err, &headers).into_response(),
    };

    match train_service::put_provider_credentials_for_user(
        state.as_ref(),
        &session.user_id,
        &provider,
        payload,
    )
    .await
    {
        Ok(result) => (StatusCode::ACCEPTED, Json(result)).into_response(),
        Err(err) => map_train_error(err, &headers).into_response(),
    }
}

async fn delete_provider_credentials(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Path(provider): Path<String>,
) -> impl IntoResponse {
    let session = match auth_service::require_session_state(state.as_ref(), &headers).await {
        Ok(session) => session,
        Err(err) => return map_auth_error(err, &headers).into_response(),
    };

    match train_service::delete_provider_credentials_for_user(
        state.as_ref(),
        &session.user_id,
        &provider,
    )
    .await
    {
        Ok(result) => (StatusCode::OK, Json(result)).into_response(),
        Err(err) => map_train_error(err, &headers).into_response(),
    }
}

async fn put_provider_payment_method(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Path(provider): Path<String>,
    Json(payload): Json<train_service::PutTrainPaymentMethodRequest>,
) -> impl IntoResponse {
    let session = match auth_service::require_session_state(state.as_ref(), &headers).await {
        Ok(session) => session,
        Err(err) => return map_auth_error(err, &headers).into_response(),
    };

    match train_service::put_payment_method_for_user(
        state.as_ref(),
        &session.user_id,
        &provider,
        payload,
    )
    .await
    {
        Ok(result) => (StatusCode::ACCEPTED, Json(result)).into_response(),
        Err(err) => map_train_error(err, &headers).into_response(),
    }
}

async fn put_universal_payment_method(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(payload): Json<train_service::PutTrainPaymentMethodRequest>,
) -> impl IntoResponse {
    let session = match auth_service::require_session_state(state.as_ref(), &headers).await {
        Ok(session) => session,
        Err(err) => return map_auth_error(err, &headers).into_response(),
    };

    match train_service::put_payment_method_for_user(
        state.as_ref(),
        &session.user_id,
        payment_method_service::UNIVERSAL_PAYMENT_PROVIDER,
        payload,
    )
    .await
    {
        Ok(result) => (StatusCode::ACCEPTED, Json(result)).into_response(),
        Err(err) => map_train_error(err, &headers).into_response(),
    }
}

async fn list_payment_methods(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
) -> impl IntoResponse {
    let session = match auth_service::require_session_state(state.as_ref(), &headers).await {
        Ok(session) => session,
        Err(err) => return map_auth_error(err, &headers).into_response(),
    };

    match train_service::list_payment_methods_for_user(state.as_ref(), &session.user_id).await {
        Ok(result) => (StatusCode::OK, Json(result)).into_response(),
        Err(err) => map_train_error(err, &headers).into_response(),
    }
}

async fn delete_payment_method(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Path(payment_method_ref): Path<String>,
) -> impl IntoResponse {
    let session = match auth_service::require_session_state(state.as_ref(), &headers).await {
        Ok(session) => session,
        Err(err) => return map_auth_error(err, &headers).into_response(),
    };

    match train_service::delete_payment_method_for_user(
        state.as_ref(),
        &session.user_id,
        &payment_method_ref,
    )
    .await
    {
        Ok(result) => (StatusCode::OK, Json(result)).into_response(),
        Err(err) => map_train_error(err, &headers).into_response(),
    }
}

fn map_auth_error(error: auth_service::AuthServiceError, headers: &HeaderMap) -> ApiError {
    match error {
        auth_service::AuthServiceError::InvalidRequest(message)
        | auth_service::AuthServiceError::Conflict(message) => ApiError::new(
            ApiErrorStatus::BadRequest,
            ApiErrorCode::InvalidRequest,
            message,
            request_id_from_headers(headers),
        ),
        auth_service::AuthServiceError::Unauthorized(message)
        | auth_service::AuthServiceError::NotFound(message) => ApiError::new(
            ApiErrorStatus::Unauthorized,
            ApiErrorCode::Unauthorized,
            message,
            request_id_from_headers(headers),
        ),
        auth_service::AuthServiceError::ServiceUnavailable(message) => ApiError::new(
            ApiErrorStatus::ServiceUnavailable,
            ApiErrorCode::ServiceUnavailable,
            message,
            request_id_from_headers(headers),
        ),
        auth_service::AuthServiceError::Internal => ApiError::new(
            ApiErrorStatus::InternalServerError,
            ApiErrorCode::InternalError,
            "authentication service failure",
            request_id_from_headers(headers),
        ),
    }
}

fn map_train_error(error: train_service::TrainServiceError, headers: &HeaderMap) -> ApiError {
    let request_id = request_id_from_headers(headers);
    match &error {
        train_service::TrainServiceError::ServiceUnavailable(message) => {
            warn!(message = %message, "train service unavailable");
        }
        train_service::TrainServiceError::Internal => {
            error!("train service internal failure");
        }
        _ => {}
    }

    match error {
        train_service::TrainServiceError::InvalidRequest(message) => ApiError::new(
            ApiErrorStatus::BadRequest,
            ApiErrorCode::InvalidRequest,
            message,
            request_id.clone(),
        ),
        train_service::TrainServiceError::Unauthorized(message) => ApiError::new(
            ApiErrorStatus::Unauthorized,
            ApiErrorCode::Unauthorized,
            message,
            request_id.clone(),
        ),
        train_service::TrainServiceError::NotFound(message) => ApiError::new(
            ApiErrorStatus::Unauthorized,
            ApiErrorCode::Unauthorized,
            message,
            request_id.clone(),
        ),
        train_service::TrainServiceError::ServiceUnavailable(message) => ApiError::new(
            ApiErrorStatus::ServiceUnavailable,
            ApiErrorCode::ServiceUnavailable,
            message,
            request_id.clone(),
        ),
        train_service::TrainServiceError::Internal => ApiError::new(
            ApiErrorStatus::InternalServerError,
            ApiErrorCode::InternalError,
            "train service failure",
            request_id,
        ),
    }
}
