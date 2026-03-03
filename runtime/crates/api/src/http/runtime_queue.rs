use std::sync::Arc;

use axum::{
    Json, Router,
    extract::State,
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
    routing::{get, post},
};
use bominal_shared::error::{ApiError, ApiErrorCode, ApiErrorStatus};

use super::super::{AppState, request_id_from_headers, services::runtime_queue_service};

#[derive(Debug, serde::Serialize)]
struct EnqueueRuntimeJobResponse {
    queued: bool,
    queue_key: String,
    job_id: String,
}

pub(super) fn register(router: Router<Arc<AppState>>) -> Router<Arc<AppState>> {
    router
        .route(
            "/api/runtime/queue/contract",
            get(super::super::runtime_queue_contract),
        )
        .route("/api/runtime/queue/enqueue", post(enqueue_runtime_job))
}

async fn enqueue_runtime_job(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(payload): Json<runtime_queue_service::EnqueueRuntimeJobRequest>,
) -> impl IntoResponse {
    match runtime_queue_service::enqueue_runtime_job(state.as_ref(), payload).await {
        Ok(result) => (
            StatusCode::ACCEPTED,
            Json(EnqueueRuntimeJobResponse {
                queued: true,
                queue_key: result.queue_key,
                job_id: result.job_id,
            }),
        )
            .into_response(),
        Err(runtime_queue_service::EnqueueRuntimeJobError::ValidationFailed) => ApiError::new(
            ApiErrorStatus::BadRequest,
            ApiErrorCode::InvalidRequest,
            "invalid payload",
            request_id_from_headers(&headers),
        )
        .with_details(serde_json::json!({"stage": "validate"}))
        .into_response(),
        Err(runtime_queue_service::EnqueueRuntimeJobError::EncodeFailed) => ApiError::new(
            ApiErrorStatus::BadRequest,
            ApiErrorCode::InvalidRequest,
            "invalid payload",
            request_id_from_headers(&headers),
        )
        .with_details(serde_json::json!({"stage": "encode"}))
        .into_response(),
        Err(runtime_queue_service::EnqueueRuntimeJobError::DuplicateJobConflict) => ApiError::new(
            ApiErrorStatus::BadRequest,
            ApiErrorCode::InvalidRequest,
            "job_id conflicts with existing runtime job payload",
            request_id_from_headers(&headers),
        )
        .with_details(serde_json::json!({"stage": "persist"}))
        .into_response(),
        Err(runtime_queue_service::EnqueueRuntimeJobError::PersistenceUnavailable) => {
            ApiError::new(
                ApiErrorStatus::ServiceUnavailable,
                ApiErrorCode::ServiceUnavailable,
                "runtime persistence unavailable",
                request_id_from_headers(&headers),
            )
            .with_details(serde_json::json!({"stage": "persist"}))
            .into_response()
        }
        Err(runtime_queue_service::EnqueueRuntimeJobError::RedisUnavailable) => ApiError::new(
            ApiErrorStatus::ServiceUnavailable,
            ApiErrorCode::ServiceUnavailable,
            "redis unavailable",
            request_id_from_headers(&headers),
        )
        .into_response(),
        Err(runtime_queue_service::EnqueueRuntimeJobError::RedisConnectionFailed) => ApiError::new(
            ApiErrorStatus::ServiceUnavailable,
            ApiErrorCode::ServiceUnavailable,
            "redis connection failed",
            request_id_from_headers(&headers),
        )
        .with_details(serde_json::json!({"stage": "connect"}))
        .into_response(),
        Err(runtime_queue_service::EnqueueRuntimeJobError::QueuePushFailed) => ApiError::new(
            ApiErrorStatus::ServiceUnavailable,
            ApiErrorCode::ServiceUnavailable,
            "queue push failed",
            request_id_from_headers(&headers),
        )
        .with_details(serde_json::json!({"stage": "push"}))
        .into_response(),
        Err(runtime_queue_service::EnqueueRuntimeJobError::PersistenceFailure) => ApiError::new(
            ApiErrorStatus::InternalServerError,
            ApiErrorCode::InternalError,
            "runtime job persistence failed",
            request_id_from_headers(&headers),
        )
        .into_response(),
    }
}
