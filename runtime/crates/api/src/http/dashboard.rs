use std::{convert::Infallible, sync::Arc, time::Duration};

use async_stream::stream;
use axum::{
    Json, Router,
    extract::{Path, Query, State},
    http::{HeaderMap, StatusCode},
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

#[derive(Debug, serde::Deserialize)]
struct JobEventsQuery {
    since_id: Option<i64>,
}

#[derive(Debug, serde::Serialize)]
struct DashboardEventsResponse {
    events: Vec<dashboard_service::RuntimeJobEventRecord>,
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
    Query(query): Query<JobEventsQuery>,
) -> impl IntoResponse {
    let session = match auth_service::require_session_state(state.as_ref(), &headers).await {
        Ok(session) => session,
        Err(err) => return map_auth_error(err, &headers).into_response(),
    };
    match dashboard_service::list_dashboard_job_events(
        state.as_ref(),
        &session.user_id,
        &job_id,
        query.since_id,
    )
    .await
    {
        Ok(events) => (StatusCode::OK, Json(DashboardEventsResponse { events })).into_response(),
        Err(err) => map_dashboard_error(err, &headers).into_response(),
    }
}

async fn stream_dashboard_job_events(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Path(job_id): Path<String>,
    Query(query): Query<JobEventsQuery>,
) -> impl IntoResponse {
    let session = match auth_service::require_session_state(state.as_ref(), &headers).await {
        Ok(session) => session,
        Err(err) => return map_auth_error(err, &headers).into_response(),
    };
    let session_user_id = session.user_id;
    let start_since = query.since_id.unwrap_or(0);
    let event_stream = stream! {
        let mut since_id = start_since;
        loop {
            match dashboard_service::list_dashboard_job_events(
                state.as_ref(),
                &session_user_id,
                &job_id,
                Some(since_id),
            )
            .await
            {
                Ok(events) => {
                    for event in events {
                        since_id = event.id;
                        let payload = serde_json::json!({
                            "id": event.id,
                            "event_type": event.event_type,
                            "event_payload": event.event_payload,
                            "created_at": event.created_at,
                        });
                        yield Ok::<Event, Infallible>(
                            Event::default()
                                .id(event.id.to_string())
                                .event("job_event")
                                .data(payload.to_string()),
                        );
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
