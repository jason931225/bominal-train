use std::{convert::Infallible, sync::Arc, time::Duration};

use async_stream::stream;
use axum::{
    Json, Router,
    extract::{Path, State},
    http::{HeaderMap, StatusCode, Uri},
    response::{
        IntoResponse,
        sse::{Event, KeepAlive, Sse},
    },
    routing::get,
};
use bominal_shared::error::{ApiError, ApiErrorCode, ApiErrorStatus};

use super::super::{
    AppState, request_id_from_headers,
    services::{auth_service, dashboard_service},
};
use super::runtime_event_cursor;

#[derive(Debug, serde::Serialize)]
struct DashboardEventsResponse {
    items: Vec<dashboard_service::RuntimeJobEventRecord>,
    page: CursorPage,
}

#[derive(Debug, serde::Serialize)]
struct CursorPage {
    limit: usize,
    has_more: bool,
    next_cursor: Option<String>,
}

pub(super) fn register(router: Router<Arc<AppState>>) -> Router<Arc<AppState>> {
    router
        .route("/api/dashboard/summary", get(get_dashboard_summary))
        .route("/api/dashboard/jobs", get(list_dashboard_jobs))
        .route("/api/dashboard/jobs/{job_id}", get(get_dashboard_job))
        .route(
            "/api/dashboard/jobs/{job_id}/events",
            get(get_dashboard_job_events),
        )
        .route(
            "/api/dashboard/jobs/{job_id}/events/stream",
            get(stream_dashboard_job_events),
        )
}

async fn get_dashboard_summary(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
) -> impl IntoResponse {
    let session = match auth_service::require_session_state(state.as_ref(), &headers).await {
        Ok(session) => session,
        Err(err) => return map_auth_error(err, &headers).into_response(),
    };
    let request_id = request_id_from_headers(&headers);
    match dashboard_service::load_dashboard_summary(state.as_ref(), &session.user_id, request_id)
        .await
    {
        Ok(summary) => (StatusCode::OK, Json(summary)).into_response(),
        Err(err) => map_dashboard_error(err, &headers).into_response(),
    }
}

async fn list_dashboard_jobs(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
) -> impl IntoResponse {
    let session = match auth_service::require_session_state(state.as_ref(), &headers).await {
        Ok(session) => session,
        Err(err) => return map_auth_error(err, &headers).into_response(),
    };
    match dashboard_service::list_dashboard_jobs(state.as_ref(), &session.user_id).await {
        Ok(jobs) => (StatusCode::OK, Json(serde_json::json!({ "jobs": jobs }))).into_response(),
        Err(err) => map_dashboard_error(err, &headers).into_response(),
    }
}

async fn get_dashboard_job(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Path(job_id): Path<String>,
) -> impl IntoResponse {
    let session = match auth_service::require_session_state(state.as_ref(), &headers).await {
        Ok(session) => session,
        Err(err) => return map_auth_error(err, &headers).into_response(),
    };
    match dashboard_service::get_dashboard_job(state.as_ref(), &session.user_id, &job_id).await {
        Ok(job) => (StatusCode::OK, Json(job)).into_response(),
        Err(err) => map_dashboard_error(err, &headers).into_response(),
    }
}

async fn get_dashboard_job_events(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Path(job_id): Path<String>,
    uri: Uri,
) -> impl IntoResponse {
    let session = match auth_service::require_session_state(state.as_ref(), &headers).await {
        Ok(session) => session,
        Err(err) => return map_auth_error(err, &headers).into_response(),
    };
    let request = match runtime_event_cursor::parse_runtime_event_request(uri.query(), &job_id) {
        Ok(value) => value,
        Err(message) => {
            return map_dashboard_error(
                dashboard_service::DashboardServiceError::InvalidRequest(message),
                &headers,
            )
            .into_response();
        }
    };
    if let Err(err) =
        dashboard_service::ensure_dashboard_job_access(state.as_ref(), &session.user_id, &job_id)
            .await
    {
        return map_dashboard_error(err, &headers).into_response();
    }
    match dashboard_service::list_dashboard_job_events_page(
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
                Json(DashboardEventsResponse {
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
        Err(err) => map_dashboard_error(err, &headers).into_response(),
    }
}

async fn stream_dashboard_job_events(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Path(job_id): Path<String>,
    uri: Uri,
) -> impl IntoResponse {
    let session = match auth_service::require_session_state(state.as_ref(), &headers).await {
        Ok(session) => session,
        Err(err) => return map_auth_error(err, &headers).into_response(),
    };
    let request = match runtime_event_cursor::parse_runtime_event_request(uri.query(), &job_id) {
        Ok(value) => value,
        Err(message) => {
            return map_dashboard_error(
                dashboard_service::DashboardServiceError::InvalidRequest(message),
                &headers,
            )
            .into_response();
        }
    };
    let session_user_id = session.user_id;
    if let Err(err) =
        dashboard_service::ensure_dashboard_job_access(state.as_ref(), &session_user_id, &job_id)
            .await
    {
        return map_dashboard_error(err, &headers).into_response();
    }
    let start_after_id = request.after_id;
    let limit = request.limit;
    let event_stream = stream! {
        let mut after_id = start_after_id;
        loop {
            match dashboard_service::list_dashboard_job_events_page(
                state.as_ref(),
                &job_id,
                after_id,
                limit,
            )
            .await
            {
                Ok(page) => {
                    for event in page.items {
                        after_id = event.id;
                        let payload = serde_json::json!({
                            "id": event.id,
                            "event_type": event.event_type,
                            "event_payload": event.event_payload,
                            "created_at": event.created_at,
                        });
                        yield Ok::<Event, Infallible>(
                            Event::default()
                                .event("job_event")
                                .data(payload.to_string()),
                        );
                    }
                    if page.has_more {
                        continue;
                    }
                }
                Err(_) => {
                    yield Ok::<Event, Infallible>(
                        Event::default()
                            .event("error")
                            .data("{\"message\":\"stream temporarily unavailable\"}"),
                    );
                }
            }
            tokio::time::sleep(Duration::from_secs(2)).await;
        }
    };

    Sse::new(event_stream)
        .keep_alive(
            KeepAlive::new()
                .interval(Duration::from_secs(15))
                .text("heartbeat"),
        )
        .into_response()
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

fn map_dashboard_error(
    error: dashboard_service::DashboardServiceError,
    headers: &HeaderMap,
) -> ApiError {
    match error {
        dashboard_service::DashboardServiceError::InvalidRequest(message) => ApiError::new(
            ApiErrorStatus::BadRequest,
            ApiErrorCode::InvalidRequest,
            message,
            request_id_from_headers(headers),
        ),
        dashboard_service::DashboardServiceError::NotFound(message) => ApiError::new(
            ApiErrorStatus::Unauthorized,
            ApiErrorCode::Unauthorized,
            message,
            request_id_from_headers(headers),
        ),
        dashboard_service::DashboardServiceError::ServiceUnavailable(message) => ApiError::new(
            ApiErrorStatus::ServiceUnavailable,
            ApiErrorCode::ServiceUnavailable,
            message,
            request_id_from_headers(headers),
        ),
        dashboard_service::DashboardServiceError::Internal => ApiError::new(
            ApiErrorStatus::InternalServerError,
            ApiErrorCode::InternalError,
            "dashboard service failure",
            request_id_from_headers(headers),
        ),
    }
}
