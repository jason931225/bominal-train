use std::sync::Arc;

use axum::{
    Json, Router,
    extract::{Path, State},
    http::{HeaderMap, StatusCode, Uri},
    response::IntoResponse,
    routing::{get, post},
};
use bominal_shared::error::{ApiError, ApiErrorCode, ApiErrorStatus};

use super::{
    super::{AppState, request_id_from_headers, services::provider_jobs_service},
    runtime_event_cursor,
};

#[derive(Debug, serde::Serialize)]
struct ProviderJobEventsResponse {
    items: Vec<provider_jobs_service::ProviderJobEvent>,
    page: CursorPage,
}

#[derive(Debug, serde::Serialize)]
struct CursorPage {
    limit: usize,
    has_more: bool,
    next_cursor: Option<String>,
}

pub(super) fn register(
    router: Router<Arc<AppState>>,
    mount_aliases: bool,
) -> Router<Arc<AppState>> {
    let router = router
        .route("/internal/v1/provider-jobs", post(create_provider_job))
        .route("/internal/v1/provider-jobs/{job_id}", get(get_provider_job))
        .route(
            "/internal/v1/provider-jobs/{job_id}/events",
            get(get_provider_job_events),
        );

    if mount_aliases {
        return router
            .route("/api/internal/provider-jobs", post(create_provider_job))
            .route(
                "/api/internal/provider-jobs/{job_id}",
                get(get_provider_job),
            )
            .route(
                "/api/internal/provider-jobs/{job_id}/events",
                get(get_provider_job_events),
            );
    }

    router
}

async fn create_provider_job(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(payload): Json<provider_jobs_service::CreateProviderJobRequest>,
) -> impl IntoResponse {
    match provider_jobs_service::create_provider_job(state.as_ref(), payload).await {
        Ok(result) => (StatusCode::ACCEPTED, Json(result)).into_response(),
        Err(provider_jobs_service::ProviderJobsError::ValidationFailed) => ApiError::new(
            ApiErrorStatus::BadRequest,
            ApiErrorCode::InvalidRequest,
            "invalid provider job payload",
            request_id_from_headers(&headers),
        )
        .into_response(),
        Err(provider_jobs_service::ProviderJobsError::PersistenceUnavailable) => ApiError::new(
            ApiErrorStatus::ServiceUnavailable,
            ApiErrorCode::ServiceUnavailable,
            "provider job persistence unavailable",
            request_id_from_headers(&headers),
        )
        .into_response(),
        Err(provider_jobs_service::ProviderJobsError::DuplicateConflict) => ApiError::new(
            ApiErrorStatus::BadRequest,
            ApiErrorCode::InvalidRequest,
            "provider job id conflicts with existing job",
            request_id_from_headers(&headers),
        )
        .into_response(),
        Err(provider_jobs_service::ProviderJobsError::NotFound) => ApiError::new(
            ApiErrorStatus::BadRequest,
            ApiErrorCode::InvalidRequest,
            "provider job not found",
            request_id_from_headers(&headers),
        )
        .into_response(),
        Err(provider_jobs_service::ProviderJobsError::PersistenceFailure) => ApiError::new(
            ApiErrorStatus::InternalServerError,
            ApiErrorCode::InternalError,
            "provider job persistence failed",
            request_id_from_headers(&headers),
        )
        .into_response(),
    }
}

async fn get_provider_job(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Path(job_id): Path<String>,
) -> impl IntoResponse {
    match provider_jobs_service::get_provider_job(state.as_ref(), &job_id).await {
        Ok(result) => (StatusCode::OK, Json(result)).into_response(),
        Err(provider_jobs_service::ProviderJobsError::ValidationFailed) => ApiError::new(
            ApiErrorStatus::BadRequest,
            ApiErrorCode::InvalidRequest,
            "invalid provider job id",
            request_id_from_headers(&headers),
        )
        .into_response(),
        Err(provider_jobs_service::ProviderJobsError::PersistenceUnavailable) => ApiError::new(
            ApiErrorStatus::ServiceUnavailable,
            ApiErrorCode::ServiceUnavailable,
            "provider job persistence unavailable",
            request_id_from_headers(&headers),
        )
        .into_response(),
        Err(provider_jobs_service::ProviderJobsError::DuplicateConflict) => ApiError::new(
            ApiErrorStatus::BadRequest,
            ApiErrorCode::InvalidRequest,
            "provider job id conflicts with existing job",
            request_id_from_headers(&headers),
        )
        .into_response(),
        Err(provider_jobs_service::ProviderJobsError::NotFound) => ApiError::new(
            ApiErrorStatus::BadRequest,
            ApiErrorCode::InvalidRequest,
            "provider job not found",
            request_id_from_headers(&headers),
        )
        .into_response(),
        Err(provider_jobs_service::ProviderJobsError::PersistenceFailure) => ApiError::new(
            ApiErrorStatus::InternalServerError,
            ApiErrorCode::InternalError,
            "provider job load failed",
            request_id_from_headers(&headers),
        )
        .into_response(),
    }
}

async fn get_provider_job_events(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Path(job_id): Path<String>,
    uri: Uri,
) -> impl IntoResponse {
    let request = match runtime_event_cursor::parse_runtime_event_request(uri.query(), &job_id) {
        Ok(value) => value,
        Err(message) => {
            return ApiError::new(
                ApiErrorStatus::BadRequest,
                ApiErrorCode::InvalidRequest,
                message,
                request_id_from_headers(&headers),
            )
            .into_response();
        }
    };

    match provider_jobs_service::list_provider_job_events_page(
        state.as_ref(),
        &job_id,
        request.after_id,
        request.limit,
    )
    .await
    {
        Ok(events_page) => {
            let next_cursor = events_page.next_after_id.and_then(|after_id| {
                runtime_event_cursor::encode_runtime_event_cursor(&job_id, after_id).ok()
            });
            (
                StatusCode::OK,
                Json(ProviderJobEventsResponse {
                    items: events_page.items,
                    page: CursorPage {
                        limit: request.limit,
                        has_more: events_page.has_more,
                        next_cursor,
                    },
                }),
            )
                .into_response()
        }
        Err(provider_jobs_service::ProviderJobsError::ValidationFailed) => ApiError::new(
            ApiErrorStatus::BadRequest,
            ApiErrorCode::InvalidRequest,
            "invalid provider job id",
            request_id_from_headers(&headers),
        )
        .into_response(),
        Err(provider_jobs_service::ProviderJobsError::PersistenceUnavailable) => ApiError::new(
            ApiErrorStatus::ServiceUnavailable,
            ApiErrorCode::ServiceUnavailable,
            "provider job persistence unavailable",
            request_id_from_headers(&headers),
        )
        .into_response(),
        Err(provider_jobs_service::ProviderJobsError::DuplicateConflict) => ApiError::new(
            ApiErrorStatus::BadRequest,
            ApiErrorCode::InvalidRequest,
            "provider job id conflicts with existing job",
            request_id_from_headers(&headers),
        )
        .into_response(),
        Err(provider_jobs_service::ProviderJobsError::NotFound) => ApiError::new(
            ApiErrorStatus::BadRequest,
            ApiErrorCode::InvalidRequest,
            "provider job not found",
            request_id_from_headers(&headers),
        )
        .into_response(),
        Err(provider_jobs_service::ProviderJobsError::PersistenceFailure) => ApiError::new(
            ApiErrorStatus::InternalServerError,
            ApiErrorCode::InternalError,
            "provider job events load failed",
            request_id_from_headers(&headers),
        )
        .into_response(),
    }
}
