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
    routing::{get, patch, post, put},
};
use bominal_shared::error::{ApiError, ApiErrorCode, ApiErrorStatus};
use redis::AsyncCommands;

use super::super::{
    AppState, request_id_from_headers,
    services::{admin_service, auth_service, metrics_service},
};

#[derive(Debug, serde::Deserialize)]
struct SensitiveMutationRequest {
    reason: String,
    confirm_target: String,
}

#[derive(Debug, serde::Deserialize)]
struct UpdateRoleRequest {
    role: String,
    reason: String,
    confirm_target: String,
}

#[derive(Debug, serde::Deserialize)]
struct UpdateAccessRequest {
    access_enabled: bool,
    reason: String,
    confirm_target: String,
}

#[derive(Debug, serde::Deserialize)]
struct UpdateKillSwitchRequest {
    enabled: bool,
    reason: String,
    confirm_target: String,
}

#[derive(Debug, serde::Deserialize)]
struct EventsQuery {
    since_id: Option<i64>,
}

#[derive(Debug, serde::Deserialize)]
struct TimeseriesQuery {
    window_minutes: Option<i64>,
}

#[derive(Debug, serde::Deserialize)]
struct ListIncidentsQuery {
    status: Option<String>,
    severity: Option<String>,
    limit: Option<usize>,
}

#[derive(Debug, serde::Deserialize)]
struct CreateIncidentRequest {
    title: String,
    severity: String,
    summary: Option<String>,
    context: Option<serde_json::Value>,
    reason: String,
}

#[derive(Debug, serde::Deserialize)]
struct UpdateIncidentStatusRequest {
    status: String,
    summary: Option<String>,
    reason: String,
    confirm_target: String,
}

#[derive(Debug, serde::Serialize)]
struct RuntimeEventsResponse {
    events: Vec<super::super::services::dashboard_service::RuntimeJobEventRecord>,
}

pub(super) fn register(router: Router<Arc<AppState>>) -> Router<Arc<AppState>> {
    router
        .route("/api/admin/maintenance/summary", get(maintenance_summary))
        .route("/api/admin/system/status", get(system_status))
        .route(
            "/api/admin/maintenance/metrics/summary",
            get(metrics_summary),
        )
        .route("/api/admin/capabilities", get(admin_capabilities))
        .route("/api/admin/users", get(list_users))
        .route("/api/admin/users/{user_id}/role", patch(update_user_role))
        .route(
            "/api/admin/users/{user_id}/access",
            patch(update_user_access),
        )
        .route("/api/admin/sessions", get(list_sessions))
        .route(
            "/api/admin/users/{user_id}/sessions/revoke",
            post(revoke_user_sessions),
        )
        .route("/api/admin/runtime/jobs", get(list_runtime_jobs))
        .route("/api/admin/runtime/jobs/{job_id}", get(get_runtime_job))
        .route(
            "/api/admin/runtime/jobs/{job_id}/events",
            get(get_runtime_job_events),
        )
        .route(
            "/api/admin/runtime/jobs/{job_id}/events/stream",
            get(stream_runtime_job_events),
        )
        .route(
            "/api/admin/runtime/jobs/{job_id}/retry",
            post(retry_runtime_job),
        )
        .route(
            "/api/admin/runtime/jobs/{job_id}/requeue",
            post(requeue_runtime_job),
        )
        .route(
            "/api/admin/runtime/jobs/{job_id}/cancel",
            post(cancel_runtime_job),
        )
        .route("/api/admin/runtime/kill-switches", get(list_kill_switches))
        .route(
            "/api/admin/runtime/kill-switches/{flag}",
            put(update_kill_switch),
        )
        .route(
            "/api/admin/observability/events",
            get(list_observability_events),
        )
        .route(
            "/api/admin/observability/timeseries",
            get(observability_timeseries),
        )
        .route(
            "/api/admin/incidents",
            get(list_incidents).post(create_incident),
        )
        .route(
            "/api/admin/incidents/{incident_id}/status",
            patch(update_incident_status),
        )
        .route("/api/admin/config/redacted", get(get_redacted_config))
        .route("/api/admin/audit", get(list_admin_audit))
}

async fn maintenance_summary(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
) -> impl IntoResponse {
    let session = match require_admin_read(state.as_ref(), &headers).await {
        Ok(value) => value,
        Err(response) => return response,
    };
    let db_ok = db_ready(state.as_ref()).await;
    let redis_ok = redis_ready(state.as_ref()).await;
    let ready_ok = db_ok && redis_ok;
    (
        StatusCode::OK,
        Json(serde_json::json!({
            "admin_email": session.email,
            "role": auth_service::admin_role_as_str(&session.role),
            "health": { "ok": true, "path": "/health" },
            "readiness": { "ok": ready_ok, "path": "/ready", "db": db_ok, "redis": redis_ok },
            "metrics_path": "/admin/maintenance/metrics",
        })),
    )
        .into_response()
}

async fn system_status(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
) -> impl IntoResponse {
    if let Err(response) = require_admin_read(state.as_ref(), &headers).await {
        return response;
    }
    let db_ok = db_ready(state.as_ref()).await;
    let redis_ok = redis_ready(state.as_ref()).await;
    let status = if db_ok && redis_ok {
        "ready"
    } else {
        "degraded"
    };
    (
        StatusCode::OK,
        Json(serde_json::json!({
            "status": status,
            "liveness": { "ok": true, "path": "/health" },
            "readiness": { "ok": db_ok && redis_ok, "path": "/ready", "db": db_ok, "redis": redis_ok }
        })),
    )
        .into_response()
}

async fn metrics_summary(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
) -> impl IntoResponse {
    if let Err(response) = require_admin_read(state.as_ref(), &headers).await {
        return response;
    }
    let db_ok = db_ready(state.as_ref()).await;
    let redis_ok = redis_ready(state.as_ref()).await;
    let metrics =
        metrics_service::summarize_metrics(&state.metrics_handle.render(), true, db_ok && redis_ok);
    (StatusCode::OK, Json(metrics)).into_response()
}

async fn admin_capabilities(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
) -> impl IntoResponse {
    let session = match require_admin_read(state.as_ref(), &headers).await {
        Ok(value) => value,
        Err(response) => return response,
    };
    (
        StatusCode::OK,
        Json(serde_json::json!({
            "role": auth_service::admin_role_as_str(&session.role),
            "can_read": admin_service::role_allows_admin_read(&session.role),
            "can_mutate": admin_service::role_allows_admin_mutation(&session.role),
            "step_up_required_for_mutation": true,
            "hosts": {
                "admin_host": state.config.admin_app_host,
                "user_host": state.config.user_app_host
            },
            "available_modules": [
                "maintenance",
                "users",
                "runtime",
                "observability",
                "security",
                "config",
                "audit"
            ],
        })),
    )
        .into_response()
}

async fn list_users(State(state): State<Arc<AppState>>, headers: HeaderMap) -> impl IntoResponse {
    if let Err(response) = require_admin_read(state.as_ref(), &headers).await {
        return response;
    }
    match admin_service::list_users(state.as_ref()).await {
        Ok(users) => (StatusCode::OK, Json(serde_json::json!({ "users": users }))).into_response(),
        Err(err) => map_admin_error(err, &headers).into_response(),
    }
}

async fn update_user_role(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Path(user_id): Path<String>,
    Json(payload): Json<UpdateRoleRequest>,
) -> impl IntoResponse {
    let session = match require_admin_mutation(state.as_ref(), &headers).await {
        Ok(value) => value,
        Err(response) => return response,
    };
    if let Err(err) = admin_service::validate_sensitive_confirmation(
        &payload.reason,
        &payload.confirm_target,
        &user_id,
    ) {
        return map_admin_error(err, &headers).into_response();
    }
    if let Err(err) = admin_service::set_user_role(
        state.as_ref(),
        &user_id,
        &payload.role,
        Some(&session.user_id),
    )
    .await
    {
        return map_admin_error(err, &headers).into_response();
    }
    let _ = admin_service::append_admin_audit(
        state.as_ref(),
        Some(&session.user_id),
        &session.email,
        "user_role_update",
        "user",
        &user_id,
        &payload.reason,
        &request_id_from_headers(&headers),
        serde_json::json!({"role": payload.role}),
    )
    .await;
    StatusCode::NO_CONTENT.into_response()
}

async fn update_user_access(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Path(user_id): Path<String>,
    Json(payload): Json<UpdateAccessRequest>,
) -> impl IntoResponse {
    let session = match require_admin_mutation(state.as_ref(), &headers).await {
        Ok(value) => value,
        Err(response) => return response,
    };
    if let Err(err) = admin_service::validate_sensitive_confirmation(
        &payload.reason,
        &payload.confirm_target,
        &user_id,
    ) {
        return map_admin_error(err, &headers).into_response();
    }
    if let Err(err) = admin_service::set_user_access(
        state.as_ref(),
        &user_id,
        payload.access_enabled,
        Some(&session.user_id),
    )
    .await
    {
        return map_admin_error(err, &headers).into_response();
    }
    let _ = admin_service::append_admin_audit(
        state.as_ref(),
        Some(&session.user_id),
        &session.email,
        "user_access_update",
        "user",
        &user_id,
        &payload.reason,
        &request_id_from_headers(&headers),
        serde_json::json!({"access_enabled": payload.access_enabled}),
    )
    .await;
    StatusCode::NO_CONTENT.into_response()
}

async fn list_sessions(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
) -> impl IntoResponse {
    if let Err(response) = require_admin_read(state.as_ref(), &headers).await {
        return response;
    }
    match admin_service::list_sessions(state.as_ref()).await {
        Ok(sessions) => (
            StatusCode::OK,
            Json(serde_json::json!({ "sessions": sessions })),
        )
            .into_response(),
        Err(err) => map_admin_error(err, &headers).into_response(),
    }
}

async fn revoke_user_sessions(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Path(user_id): Path<String>,
    Json(payload): Json<SensitiveMutationRequest>,
) -> impl IntoResponse {
    let session = match require_admin_mutation(state.as_ref(), &headers).await {
        Ok(value) => value,
        Err(response) => return response,
    };
    if let Err(err) = admin_service::validate_sensitive_confirmation(
        &payload.reason,
        &payload.confirm_target,
        &user_id,
    ) {
        return map_admin_error(err, &headers).into_response();
    }
    match auth_service::revoke_user_sessions(state.as_ref(), &user_id, &payload.reason).await {
        Ok(rows) => {
            let _ = admin_service::append_admin_audit(
                state.as_ref(),
                Some(&session.user_id),
                &session.email,
                "sessions_revoke",
                "user",
                &user_id,
                &payload.reason,
                &request_id_from_headers(&headers),
                serde_json::json!({"revoked_rows": rows}),
            )
            .await;
            (StatusCode::OK, Json(serde_json::json!({ "revoked": rows }))).into_response()
        }
        Err(err) => map_auth_error(err, &headers).into_response(),
    }
}

async fn list_runtime_jobs(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
) -> impl IntoResponse {
    if let Err(response) = require_admin_read(state.as_ref(), &headers).await {
        return response;
    }
    match admin_service::list_runtime_jobs(state.as_ref()).await {
        Ok(jobs) => (StatusCode::OK, Json(serde_json::json!({ "jobs": jobs }))).into_response(),
        Err(err) => map_admin_error(err, &headers).into_response(),
    }
}

async fn get_runtime_job(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Path(job_id): Path<String>,
) -> impl IntoResponse {
    if let Err(response) = require_admin_read(state.as_ref(), &headers).await {
        return response;
    }
    match admin_service::get_runtime_job(state.as_ref(), &job_id).await {
        Ok(job) => (StatusCode::OK, Json(job)).into_response(),
        Err(err) => map_admin_error(err, &headers).into_response(),
    }
}

async fn get_runtime_job_events(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Path(job_id): Path<String>,
    Query(query): Query<EventsQuery>,
) -> impl IntoResponse {
    if let Err(response) = require_admin_read(state.as_ref(), &headers).await {
        return response;
    }
    match admin_service::list_runtime_job_events(state.as_ref(), &job_id, query.since_id).await {
        Ok(events) => (StatusCode::OK, Json(RuntimeEventsResponse { events })).into_response(),
        Err(err) => map_admin_error(err, &headers).into_response(),
    }
}

async fn stream_runtime_job_events(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Path(job_id): Path<String>,
    Query(query): Query<EventsQuery>,
) -> impl IntoResponse {
    if let Err(response) = require_admin_read(state.as_ref(), &headers).await {
        return response;
    }
    let mut since_id = query.since_id.unwrap_or(0);
    let events = stream! {
        loop {
            match admin_service::list_runtime_job_events(state.as_ref(), &job_id, Some(since_id)).await {
                Ok(batch) => {
                    for event in batch {
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
    Sse::new(events)
        .keep_alive(
            KeepAlive::new()
                .interval(Duration::from_secs(15))
                .text("heartbeat"),
        )
        .into_response()
}

async fn retry_runtime_job(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Path(job_id): Path<String>,
    Json(payload): Json<SensitiveMutationRequest>,
) -> impl IntoResponse {
    let session = match require_admin_mutation(state.as_ref(), &headers).await {
        Ok(value) => value,
        Err(response) => return response,
    };
    if let Err(err) = admin_service::validate_sensitive_confirmation(
        &payload.reason,
        &payload.confirm_target,
        &job_id,
    ) {
        return map_admin_error(err, &headers).into_response();
    }
    if let Err(err) = admin_service::retry_runtime_job(state.as_ref(), &job_id).await {
        return map_admin_error(err, &headers).into_response();
    }
    let _ = admin_service::append_admin_audit(
        state.as_ref(),
        Some(&session.user_id),
        &session.email,
        "runtime_retry",
        "runtime_job",
        &job_id,
        &payload.reason,
        &request_id_from_headers(&headers),
        serde_json::json!({}),
    )
    .await;
    StatusCode::NO_CONTENT.into_response()
}

async fn requeue_runtime_job(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Path(job_id): Path<String>,
    Json(payload): Json<SensitiveMutationRequest>,
) -> impl IntoResponse {
    let session = match require_admin_mutation(state.as_ref(), &headers).await {
        Ok(value) => value,
        Err(response) => return response,
    };
    if let Err(err) = admin_service::validate_sensitive_confirmation(
        &payload.reason,
        &payload.confirm_target,
        &job_id,
    ) {
        return map_admin_error(err, &headers).into_response();
    }
    if let Err(err) = admin_service::requeue_runtime_job(state.as_ref(), &job_id).await {
        return map_admin_error(err, &headers).into_response();
    }
    let _ = admin_service::append_admin_audit(
        state.as_ref(),
        Some(&session.user_id),
        &session.email,
        "runtime_requeue",
        "runtime_job",
        &job_id,
        &payload.reason,
        &request_id_from_headers(&headers),
        serde_json::json!({}),
    )
    .await;
    StatusCode::NO_CONTENT.into_response()
}

async fn cancel_runtime_job(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Path(job_id): Path<String>,
    Json(payload): Json<SensitiveMutationRequest>,
) -> impl IntoResponse {
    let session = match require_admin_mutation(state.as_ref(), &headers).await {
        Ok(value) => value,
        Err(response) => return response,
    };
    if let Err(err) = admin_service::validate_sensitive_confirmation(
        &payload.reason,
        &payload.confirm_target,
        &job_id,
    ) {
        return map_admin_error(err, &headers).into_response();
    }
    if let Err(err) = admin_service::cancel_runtime_job(state.as_ref(), &job_id).await {
        return map_admin_error(err, &headers).into_response();
    }
    let _ = admin_service::append_admin_audit(
        state.as_ref(),
        Some(&session.user_id),
        &session.email,
        "runtime_cancel",
        "runtime_job",
        &job_id,
        &payload.reason,
        &request_id_from_headers(&headers),
        serde_json::json!({}),
    )
    .await;
    StatusCode::NO_CONTENT.into_response()
}

async fn list_kill_switches(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
) -> impl IntoResponse {
    if let Err(response) = require_admin_read(state.as_ref(), &headers).await {
        return response;
    }
    match admin_service::list_kill_switches(state.as_ref()).await {
        Ok(flags) => (StatusCode::OK, Json(serde_json::json!({ "flags": flags }))).into_response(),
        Err(err) => map_admin_error(err, &headers).into_response(),
    }
}

async fn update_kill_switch(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Path(flag): Path<String>,
    Json(payload): Json<UpdateKillSwitchRequest>,
) -> impl IntoResponse {
    let session = match require_admin_mutation(state.as_ref(), &headers).await {
        Ok(value) => value,
        Err(response) => return response,
    };
    if let Err(err) = admin_service::validate_sensitive_confirmation(
        &payload.reason,
        &payload.confirm_target,
        &flag,
    ) {
        return map_admin_error(err, &headers).into_response();
    }
    if let Err(err) = admin_service::upsert_kill_switch(
        state.as_ref(),
        &flag,
        payload.enabled,
        &payload.reason,
        Some(&session.user_id),
    )
    .await
    {
        return map_admin_error(err, &headers).into_response();
    }
    let _ = admin_service::append_admin_audit(
        state.as_ref(),
        Some(&session.user_id),
        &session.email,
        "kill_switch_update",
        "kill_switch",
        &flag,
        &payload.reason,
        &request_id_from_headers(&headers),
        serde_json::json!({ "enabled": payload.enabled }),
    )
    .await;
    StatusCode::NO_CONTENT.into_response()
}

async fn list_observability_events(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
) -> impl IntoResponse {
    if let Err(response) = require_admin_read(state.as_ref(), &headers).await {
        return response;
    }
    match admin_service::list_observability_events(state.as_ref()).await {
        Ok(events) => (
            StatusCode::OK,
            Json(serde_json::json!({ "events": events })),
        )
            .into_response(),
        Err(err) => map_admin_error(err, &headers).into_response(),
    }
}

async fn observability_timeseries(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Query(query): Query<TimeseriesQuery>,
) -> impl IntoResponse {
    if let Err(response) = require_admin_read(state.as_ref(), &headers).await {
        return response;
    }
    let requested_window = query.window_minutes.unwrap_or(240).clamp(15, 1440);
    match admin_service::list_observability_timeseries(state.as_ref(), Some(requested_window)).await
    {
        Ok(points) => (
            StatusCode::OK,
            Json(serde_json::json!({
                "window_minutes": requested_window,
                "points": points
            })),
        )
            .into_response(),
        Err(err) => map_admin_error(err, &headers).into_response(),
    }
}

async fn list_incidents(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Query(query): Query<ListIncidentsQuery>,
) -> impl IntoResponse {
    if let Err(response) = require_admin_read(state.as_ref(), &headers).await {
        return response;
    }
    match admin_service::list_incidents(
        state.as_ref(),
        query.status.as_deref(),
        query.severity.as_deref(),
        query.limit,
    )
    .await
    {
        Ok(incidents) => (
            StatusCode::OK,
            Json(serde_json::json!({ "incidents": incidents })),
        )
            .into_response(),
        Err(err) => map_admin_error(err, &headers).into_response(),
    }
}

async fn create_incident(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(payload): Json<CreateIncidentRequest>,
) -> impl IntoResponse {
    let session = match require_admin_mutation(state.as_ref(), &headers).await {
        Ok(value) => value,
        Err(response) => return response,
    };
    let reason = payload.reason.trim();
    if reason.len() < 8 {
        return map_admin_error(
            admin_service::AdminServiceError::InvalidRequest(
                "reason must be at least 8 characters",
            ),
            &headers,
        )
        .into_response();
    }
    let incident = match admin_service::create_incident(
        state.as_ref(),
        &payload.title,
        &payload.severity,
        payload.summary.as_deref(),
        payload.context.unwrap_or_else(|| serde_json::json!({})),
        Some(&session.user_id),
    )
    .await
    {
        Ok(value) => value,
        Err(err) => return map_admin_error(err, &headers).into_response(),
    };
    let _ = admin_service::append_admin_audit(
        state.as_ref(),
        Some(&session.user_id),
        &session.email,
        "incident_create",
        "incident",
        &incident.id,
        reason,
        &request_id_from_headers(&headers),
        serde_json::json!({"severity": incident.severity, "status": incident.status}),
    )
    .await;
    (StatusCode::CREATED, Json(incident)).into_response()
}

async fn update_incident_status(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Path(incident_id): Path<String>,
    Json(payload): Json<UpdateIncidentStatusRequest>,
) -> impl IntoResponse {
    let session = match require_admin_mutation(state.as_ref(), &headers).await {
        Ok(value) => value,
        Err(response) => return response,
    };
    if let Err(err) = admin_service::validate_sensitive_confirmation(
        &payload.reason,
        &payload.confirm_target,
        &incident_id,
    ) {
        return map_admin_error(err, &headers).into_response();
    }
    let updated = match admin_service::update_incident_status(
        state.as_ref(),
        &incident_id,
        &payload.status,
        payload.summary.as_deref(),
    )
    .await
    {
        Ok(value) => value,
        Err(err) => return map_admin_error(err, &headers).into_response(),
    };
    let _ = admin_service::append_admin_audit(
        state.as_ref(),
        Some(&session.user_id),
        &session.email,
        "incident_status_update",
        "incident",
        &incident_id,
        &payload.reason,
        &request_id_from_headers(&headers),
        serde_json::json!({"status": updated.status}),
    )
    .await;
    (StatusCode::OK, Json(updated)).into_response()
}

async fn get_redacted_config(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
) -> impl IntoResponse {
    if let Err(response) = require_admin_read(state.as_ref(), &headers).await {
        return response;
    }
    let config = admin_service::redacted_config_snapshot(state.as_ref());
    (
        StatusCode::OK,
        Json(serde_json::json!({ "config": config })),
    )
        .into_response()
}

async fn list_admin_audit(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
) -> impl IntoResponse {
    if let Err(response) = require_admin_read(state.as_ref(), &headers).await {
        return response;
    }
    match admin_service::list_admin_audit(state.as_ref()).await {
        Ok(entries) => (
            StatusCode::OK,
            Json(serde_json::json!({ "entries": entries })),
        )
            .into_response(),
        Err(err) => map_admin_error(err, &headers).into_response(),
    }
}

async fn require_admin_read(
    state: &AppState,
    headers: &HeaderMap,
) -> Result<auth_service::SessionState, axum::response::Response> {
    let session = match auth_service::require_admin_session_state(state, headers).await {
        Ok(value) => value,
        Err(err) => return Err(map_auth_error(err, headers).into_response()),
    };
    if !admin_service::role_allows_admin_read(&session.role) {
        return Err(ApiError::new(
            ApiErrorStatus::Unauthorized,
            ApiErrorCode::Unauthorized,
            "admin access required",
            request_id_from_headers(headers),
        )
        .into_response());
    }
    Ok(session)
}

async fn require_admin_mutation(
    state: &AppState,
    headers: &HeaderMap,
) -> Result<auth_service::SessionState, axum::response::Response> {
    let session = require_admin_read(state, headers).await?;
    if !admin_service::role_allows_admin_mutation(&session.role) {
        return Err(ApiError::new(
            ApiErrorStatus::Unauthorized,
            ApiErrorCode::Unauthorized,
            "admin mutation role required",
            request_id_from_headers(headers),
        )
        .into_response());
    }
    if !auth_service::ensure_recent_step_up(&session, state.config.step_up_ttl_seconds) {
        return Err(ApiError::new(
            ApiErrorStatus::Unauthorized,
            ApiErrorCode::Unauthorized,
            "recent passkey step-up required",
            request_id_from_headers(headers),
        )
        .into_response());
    }
    Ok(session)
}

async fn db_ready(state: &AppState) -> bool {
    let Some(pool) = state.db_pool.as_ref() else {
        return false;
    };
    sqlx::query_scalar::<_, i32>("select 1")
        .fetch_one(pool)
        .await
        .map(|value| value == 1)
        .unwrap_or(false)
}

async fn redis_ready(state: &AppState) -> bool {
    let Some(redis_client) = state.redis_client.as_ref() else {
        return false;
    };
    let mut conn = match redis_client.get_multiplexed_async_connection().await {
        Ok(conn) => conn,
        Err(_) => return false,
    };
    conn.ping::<String>().await.is_ok()
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

fn map_admin_error(error: admin_service::AdminServiceError, headers: &HeaderMap) -> ApiError {
    match error {
        admin_service::AdminServiceError::InvalidRequest(message) => ApiError::new(
            ApiErrorStatus::BadRequest,
            ApiErrorCode::InvalidRequest,
            message,
            request_id_from_headers(headers),
        ),
        admin_service::AdminServiceError::NotFound(message) => ApiError::new(
            ApiErrorStatus::Unauthorized,
            ApiErrorCode::Unauthorized,
            message,
            request_id_from_headers(headers),
        ),
        admin_service::AdminServiceError::ServiceUnavailable(message) => ApiError::new(
            ApiErrorStatus::ServiceUnavailable,
            ApiErrorCode::ServiceUnavailable,
            message,
            request_id_from_headers(headers),
        ),
        admin_service::AdminServiceError::Internal => ApiError::new(
            ApiErrorStatus::InternalServerError,
            ApiErrorCode::InternalError,
            "admin service failure",
            request_id_from_headers(headers),
        ),
    }
}
