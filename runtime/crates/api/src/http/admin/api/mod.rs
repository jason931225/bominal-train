use std::{convert::Infallible, sync::Arc, time::Duration};

use async_stream::stream;
use axum::{
    Json, Router,
    extract::{Path, Query, State},
    http::{HeaderMap, StatusCode, Uri},
    response::{
        IntoResponse,
        sse::{Event, KeepAlive, Sse},
    },
    routing::{get, patch, post, put},
};
use bominal_shared::error::{ApiError, ApiErrorCode, ApiErrorStatus};
use redis::AsyncCommands;

use super::super::super::{
    AppState, request_id_from_headers,
    services::{admin_service, auth_service, metrics_service},
};
use super::super::runtime_event_cursor;
use super::super::sse as canonical_sse;

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
struct UsersQuery {
    limit: Option<usize>,
    cursor: Option<String>,
    q: Option<String>,
    role: Option<String>,
    status: Option<String>,
    access: Option<String>,
}

#[derive(Debug, serde::Deserialize)]
struct SessionsQuery {
    limit: Option<usize>,
    cursor: Option<String>,
    q: Option<String>,
    role: Option<String>,
    revoked: Option<String>,
    step_up: Option<String>,
    user_id: Option<String>,
}

#[derive(Debug, serde::Deserialize)]
struct RuntimeJobsQuery {
    limit: Option<usize>,
    cursor: Option<String>,
    q: Option<String>,
    status: Option<String>,
    provider: Option<String>,
    operation: Option<String>,
}

#[derive(Debug, serde::Deserialize)]
struct ObservabilityEventsQuery {
    limit: Option<usize>,
    cursor: Option<String>,
    source: Option<String>,
    event_type: Option<String>,
    target_id: Option<String>,
    q: Option<String>,
}

#[derive(Debug, serde::Deserialize)]
struct TimeseriesQuery {
    window_minutes: Option<i64>,
}

#[derive(Debug, serde::Deserialize)]
struct ListIncidentsQuery {
    limit: Option<usize>,
    cursor: Option<String>,
    q: Option<String>,
    status: Option<String>,
    severity: Option<String>,
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
    items: Vec<super::super::super::services::dashboard_service::RuntimeJobEventRecord>,
    page: CursorPage,
}

#[derive(Debug, serde::Serialize)]
struct CursorPage {
    limit: usize,
    has_more: bool,
    next_cursor: Option<String>,
}

#[derive(Debug, serde::Serialize)]
struct CursorEnvelope<T, F> {
    items: Vec<T>,
    page: CursorPage,
    filters: F,
}

#[derive(Debug, serde::Serialize)]
struct UsersFilters {
    q: Option<String>,
    role: Option<String>,
    status: Option<String>,
    access: String,
}

#[derive(Debug, serde::Serialize)]
struct SessionsFilters {
    q: Option<String>,
    role: Option<String>,
    revoked: String,
    step_up: String,
    user_id: Option<String>,
}

#[derive(Debug, serde::Serialize)]
struct RuntimeJobsFilters {
    q: Option<String>,
    status: Option<String>,
    provider: Option<String>,
    operation: Option<String>,
}

#[derive(Debug, serde::Serialize)]
struct ObservabilityFilters {
    q: Option<String>,
    source: Option<String>,
    event_type: Option<String>,
    target_id: Option<String>,
}

#[derive(Debug, serde::Serialize)]
struct IncidentFilters {
    q: Option<String>,
    status: Option<String>,
    severity: Option<String>,
}

#[derive(Debug, Clone, serde::Serialize)]
struct RuntimeJobListItem {
    job_id: String,
    status: String,
    attempt_count: i32,
    next_run_at: Option<chrono::DateTime<chrono::Utc>>,
    updated_at: Option<chrono::DateTime<chrono::Utc>>,
    payload: serde_json::Value,
    provider: Option<String>,
    operation: Option<String>,
}

#[derive(Debug, Clone, serde::Serialize)]
struct ObservabilityEventListItem {
    occurred_at: chrono::DateTime<chrono::Utc>,
    source: String,
    event_type: String,
    target_id: Option<String>,
    detail: serde_json::Value,
    request_id: Option<String>,
}

const DEFAULT_CURSOR_LIMIT: usize = 25;
const MAX_CURSOR_LIMIT: usize = 100;

fn normalize_optional_query(raw: Option<&str>) -> Option<String> {
    raw.map(str::trim)
        .filter(|value| !value.is_empty())
        .map(|value| value.to_ascii_lowercase())
}

fn normalize_query_text(raw: Option<&str>) -> Option<String> {
    raw.map(str::trim)
        .filter(|value| !value.is_empty())
        .map(|value| value.to_string())
}

fn clamp_page_limit(limit: Option<usize>) -> usize {
    limit
        .unwrap_or(DEFAULT_CURSOR_LIMIT)
        .clamp(1, MAX_CURSOR_LIMIT)
}

fn encode_base64url(bytes: &[u8]) -> String {
    const ALPHABET: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-_";
    let mut out = String::with_capacity((bytes.len() * 4).div_ceil(3));
    let mut index = 0usize;
    while index + 3 <= bytes.len() {
        let chunk = ((bytes[index] as u32) << 16)
            | ((bytes[index + 1] as u32) << 8)
            | bytes[index + 2] as u32;
        out.push(ALPHABET[((chunk >> 18) & 0x3f) as usize] as char);
        out.push(ALPHABET[((chunk >> 12) & 0x3f) as usize] as char);
        out.push(ALPHABET[((chunk >> 6) & 0x3f) as usize] as char);
        out.push(ALPHABET[(chunk & 0x3f) as usize] as char);
        index += 3;
    }

    let remaining = bytes.len() - index;
    if remaining == 1 {
        let chunk = (bytes[index] as u32) << 16;
        out.push(ALPHABET[((chunk >> 18) & 0x3f) as usize] as char);
        out.push(ALPHABET[((chunk >> 12) & 0x3f) as usize] as char);
    } else if remaining == 2 {
        let chunk = ((bytes[index] as u32) << 16) | ((bytes[index + 1] as u32) << 8);
        out.push(ALPHABET[((chunk >> 18) & 0x3f) as usize] as char);
        out.push(ALPHABET[((chunk >> 12) & 0x3f) as usize] as char);
        out.push(ALPHABET[((chunk >> 6) & 0x3f) as usize] as char);
    }
    out
}

fn decode_base64url(input: &str) -> Result<Vec<u8>, admin_service::AdminServiceError> {
    let mut out = Vec::with_capacity((input.len() * 3) / 4 + 3);
    let mut buffer = 0u32;
    let mut bits = 0usize;

    for byte in input.bytes() {
        let sextet = match byte {
            b'A'..=b'Z' => (byte - b'A') as u32,
            b'a'..=b'z' => (byte - b'a' + 26) as u32,
            b'0'..=b'9' => (byte - b'0' + 52) as u32,
            b'-' => 62,
            b'_' => 63,
            _ => {
                return Err(admin_service::AdminServiceError::InvalidRequest(
                    "invalid cursor token",
                ));
            }
        };

        buffer = (buffer << 6) | sextet;
        bits += 6;

        if bits >= 8 {
            bits -= 8;
            out.push(((buffer >> bits) & 0xff) as u8);
            buffer &= (1u32 << bits) - 1;
        }
    }

    if bits > 0 && (buffer & ((1u32 << bits) - 1)) != 0 {
        return Err(admin_service::AdminServiceError::InvalidRequest(
            "invalid cursor token",
        ));
    }

    Ok(out)
}

fn decode_cursor_offset(raw: Option<&str>) -> Result<usize, admin_service::AdminServiceError> {
    let Some(raw) = raw.map(str::trim).filter(|value| !value.is_empty()) else {
        return Ok(0);
    };
    let bytes = decode_base64url(raw)?;
    let parsed = serde_json::from_slice::<serde_json::Value>(&bytes).map_err(|_| {
        admin_service::AdminServiceError::InvalidRequest("invalid cursor token payload")
    })?;
    let offset = parsed
        .get("offset")
        .and_then(|value| value.as_u64())
        .ok_or(admin_service::AdminServiceError::InvalidRequest(
            "invalid cursor token offset",
        ))?;
    usize::try_from(offset).map_err(|_| {
        admin_service::AdminServiceError::InvalidRequest("invalid cursor token offset")
    })
}

fn encode_cursor_offset(offset: usize) -> String {
    let payload = serde_json::json!({ "offset": offset });
    encode_base64url(payload.to_string().as_bytes())
}

fn paginate_items<T: Clone>(
    items: &[T],
    limit: usize,
    raw_cursor: Option<&str>,
) -> Result<(Vec<T>, CursorPage), admin_service::AdminServiceError> {
    let start = decode_cursor_offset(raw_cursor)?;
    if start > items.len() {
        return Err(admin_service::AdminServiceError::InvalidRequest(
            "cursor out of range",
        ));
    }
    let end = (start + limit).min(items.len());
    let page_items = items[start..end].to_vec();
    let has_more = end < items.len();
    let next_cursor = if has_more {
        Some(encode_cursor_offset(end))
    } else {
        None
    };

    Ok((
        page_items,
        CursorPage {
            limit,
            has_more,
            next_cursor,
        },
    ))
}

fn payload_text(payload: &serde_json::Value, key: &str) -> Option<String> {
    payload
        .get(key)
        .and_then(|value| value.as_str())
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToOwned::to_owned)
}

fn request_id_from_detail(detail: &serde_json::Value) -> Option<String> {
    detail
        .as_object()
        .and_then(|map| {
            ["request_id", "support_request_id", "trace_id"]
                .iter()
                .find_map(|key| map.get(*key).and_then(|value| value.as_str()))
        })
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToOwned::to_owned)
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
        .route("/api/admin/runtime/jobs/stream", get(stream_runtime_jobs))
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
            "/api/admin/observability/events/stream",
            get(stream_observability_events),
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

async fn list_users(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Query(query): Query<UsersQuery>,
) -> impl IntoResponse {
    if let Err(response) = require_admin_read(state.as_ref(), &headers).await {
        return response;
    }
    match admin_service::list_users(state.as_ref()).await {
        Ok(users) => {
            let limit = clamp_page_limit(query.limit);
            let q = normalize_query_text(query.q.as_deref());
            let role = normalize_optional_query(query.role.as_deref());
            let status = normalize_optional_query(query.status.as_deref());
            let access = normalize_optional_query(query.access.as_deref())
                .unwrap_or_else(|| "all".to_string());
            if role
                .as_deref()
                .is_some_and(|value| !matches!(value, "admin" | "operator" | "viewer" | "user"))
            {
                return map_admin_error(
                    admin_service::AdminServiceError::InvalidRequest("invalid role filter"),
                    &headers,
                )
                .into_response();
            }
            if !matches!(access.as_str(), "all" | "enabled" | "disabled") {
                return map_admin_error(
                    admin_service::AdminServiceError::InvalidRequest("invalid access filter"),
                    &headers,
                )
                .into_response();
            }

            let filtered = users
                .into_iter()
                .filter(|user| {
                    let matches_q = q.as_ref().is_none_or(|needle| {
                        let haystack =
                            format!("{} {}", user.email, user.user_id).to_ascii_lowercase();
                        haystack.contains(&needle.to_ascii_lowercase())
                    });
                    let matches_role = role
                        .as_ref()
                        .is_none_or(|value| user.role.eq_ignore_ascii_case(value));
                    let matches_status = status
                        .as_ref()
                        .is_none_or(|value| user.status.eq_ignore_ascii_case(value));
                    let matches_access = match access.as_str() {
                        "enabled" => user.access_enabled,
                        "disabled" => !user.access_enabled,
                        _ => true,
                    };
                    matches_q && matches_role && matches_status && matches_access
                })
                .collect::<Vec<_>>();

            match paginate_items(&filtered, limit, query.cursor.as_deref()) {
                Ok((items, page)) => (
                    StatusCode::OK,
                    Json(CursorEnvelope {
                        items,
                        page,
                        filters: UsersFilters {
                            q,
                            role,
                            status,
                            access,
                        },
                    }),
                )
                    .into_response(),
                Err(err) => map_admin_error(err, &headers).into_response(),
            }
        }
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
        admin_service::AppendAdminAuditInput {
            actor_user_id: Some(&session.user_id),
            actor_email: &session.email,
            action: "user_role_update",
            target_type: "user",
            target_id: &user_id,
            reason: &payload.reason,
            request_id: &request_id_from_headers(&headers),
            metadata: serde_json::json!({"role": payload.role}),
        },
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
        admin_service::AppendAdminAuditInput {
            actor_user_id: Some(&session.user_id),
            actor_email: &session.email,
            action: "user_access_update",
            target_type: "user",
            target_id: &user_id,
            reason: &payload.reason,
            request_id: &request_id_from_headers(&headers),
            metadata: serde_json::json!({"access_enabled": payload.access_enabled}),
        },
    )
    .await;
    StatusCode::NO_CONTENT.into_response()
}

async fn list_sessions(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Query(query): Query<SessionsQuery>,
) -> impl IntoResponse {
    if let Err(response) = require_admin_read(state.as_ref(), &headers).await {
        return response;
    }
    match admin_service::list_sessions(state.as_ref()).await {
        Ok(sessions) => {
            let limit = clamp_page_limit(query.limit);
            let q = normalize_query_text(query.q.as_deref());
            let role = normalize_optional_query(query.role.as_deref());
            let user_id = normalize_query_text(query.user_id.as_deref());
            let revoked = normalize_optional_query(query.revoked.as_deref())
                .unwrap_or_else(|| "all".to_string());
            let step_up = normalize_optional_query(query.step_up.as_deref())
                .unwrap_or_else(|| "all".to_string());

            if role
                .as_deref()
                .is_some_and(|value| !matches!(value, "admin" | "operator" | "viewer" | "user"))
            {
                return map_admin_error(
                    admin_service::AdminServiceError::InvalidRequest("invalid role filter"),
                    &headers,
                )
                .into_response();
            }
            if !matches!(revoked.as_str(), "all" | "active" | "revoked") {
                return map_admin_error(
                    admin_service::AdminServiceError::InvalidRequest("invalid revoked filter"),
                    &headers,
                )
                .into_response();
            }
            if !matches!(step_up.as_str(), "all" | "verified" | "missing") {
                return map_admin_error(
                    admin_service::AdminServiceError::InvalidRequest("invalid step_up filter"),
                    &headers,
                )
                .into_response();
            }

            let filtered = sessions
                .into_iter()
                .filter(|session| {
                    let matches_q = q.as_ref().is_none_or(|needle| {
                        let user_id_text = session.user_id.clone().unwrap_or_default();
                        let haystack = format!(
                            "{} {} {}",
                            session.email, session.session_id_hash, user_id_text
                        )
                        .to_ascii_lowercase();
                        haystack.contains(&needle.to_ascii_lowercase())
                    });
                    let matches_role = role
                        .as_ref()
                        .is_none_or(|value| session.role.eq_ignore_ascii_case(value));
                    let matches_user = user_id.as_ref().is_none_or(|value| {
                        session
                            .user_id
                            .as_ref()
                            .is_some_and(|candidate| candidate.eq_ignore_ascii_case(value))
                    });
                    let matches_revoked = match revoked.as_str() {
                        "active" => session.revoked_at.is_none(),
                        "revoked" => session.revoked_at.is_some(),
                        _ => true,
                    };
                    let matches_step_up = match step_up.as_str() {
                        "verified" => session.step_up_verified_at.is_some(),
                        "missing" => session.step_up_verified_at.is_none(),
                        _ => true,
                    };
                    matches_q && matches_role && matches_user && matches_revoked && matches_step_up
                })
                .collect::<Vec<_>>();

            match paginate_items(&filtered, limit, query.cursor.as_deref()) {
                Ok((items, page)) => (
                    StatusCode::OK,
                    Json(CursorEnvelope {
                        items,
                        page,
                        filters: SessionsFilters {
                            q,
                            role,
                            revoked,
                            step_up,
                            user_id,
                        },
                    }),
                )
                    .into_response(),
                Err(err) => map_admin_error(err, &headers).into_response(),
            }
        }
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
                admin_service::AppendAdminAuditInput {
                    actor_user_id: Some(&session.user_id),
                    actor_email: &session.email,
                    action: "sessions_revoke",
                    target_type: "user",
                    target_id: &user_id,
                    reason: &payload.reason,
                    request_id: &request_id_from_headers(&headers),
                    metadata: serde_json::json!({"revoked_rows": rows}),
                },
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
    Query(query): Query<RuntimeJobsQuery>,
) -> impl IntoResponse {
    if let Err(response) = require_admin_read(state.as_ref(), &headers).await {
        return response;
    }
    match admin_service::list_runtime_jobs(state.as_ref()).await {
        Ok(jobs) => {
            let limit = clamp_page_limit(query.limit);
            let q = normalize_query_text(query.q.as_deref());
            let status = normalize_optional_query(query.status.as_deref());
            let provider = normalize_optional_query(query.provider.as_deref());
            let operation = normalize_optional_query(query.operation.as_deref());

            let mapped = jobs
                .into_iter()
                .map(|job| {
                    let provider_value = payload_text(&job.payload, "provider")
                        .or_else(|| payload_text(&job.payload, "provider_name"));
                    let operation_value = payload_text(&job.payload, "operation")
                        .or_else(|| payload_text(&job.payload, "provider_operation"));
                    RuntimeJobListItem {
                        job_id: job.job_id,
                        status: job.status,
                        attempt_count: job.attempt_count,
                        next_run_at: job.next_run_at,
                        updated_at: job.updated_at,
                        payload: job.payload,
                        provider: provider_value,
                        operation: operation_value,
                    }
                })
                .collect::<Vec<_>>();

            let filtered = mapped
                .into_iter()
                .filter(|job| {
                    let matches_q = q.as_ref().is_none_or(|needle| {
                        let haystack = format!(
                            "{} {} {}",
                            job.job_id,
                            job.provider.as_deref().unwrap_or_default(),
                            job.operation.as_deref().unwrap_or_default()
                        )
                        .to_ascii_lowercase();
                        haystack.contains(&needle.to_ascii_lowercase())
                    });
                    let matches_status = status
                        .as_ref()
                        .is_none_or(|value| job.status.eq_ignore_ascii_case(value));
                    let matches_provider = provider.as_ref().is_none_or(|value| {
                        job.provider
                            .as_deref()
                            .is_some_and(|candidate| candidate.eq_ignore_ascii_case(value))
                    });
                    let matches_operation = operation.as_ref().is_none_or(|value| {
                        job.operation
                            .as_deref()
                            .is_some_and(|candidate| candidate.eq_ignore_ascii_case(value))
                    });
                    matches_q && matches_status && matches_provider && matches_operation
                })
                .collect::<Vec<_>>();

            match paginate_items(&filtered, limit, query.cursor.as_deref()) {
                Ok((items, page)) => (
                    StatusCode::OK,
                    Json(CursorEnvelope {
                        items,
                        page,
                        filters: RuntimeJobsFilters {
                            q,
                            status,
                            provider,
                            operation,
                        },
                    }),
                )
                    .into_response(),
                Err(err) => map_admin_error(err, &headers).into_response(),
            }
        }
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
    uri: Uri,
) -> impl IntoResponse {
    if let Err(response) = require_admin_read(state.as_ref(), &headers).await {
        return response;
    }
    let request = match runtime_event_cursor::parse_runtime_event_request(uri.query(), &job_id) {
        Ok(value) => value,
        Err(message) => {
            return map_admin_error(
                admin_service::AdminServiceError::InvalidRequest(message),
                &headers,
            )
            .into_response();
        }
    };
    if let Err(err) = admin_service::ensure_runtime_job_exists(state.as_ref(), &job_id).await {
        return map_admin_error(err, &headers).into_response();
    }
    match admin_service::list_runtime_job_events_page(
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
                Json(RuntimeEventsResponse {
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
        Err(err) => map_admin_error(err, &headers).into_response(),
    }
}

async fn stream_runtime_jobs(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
) -> impl IntoResponse {
    if let Err(response) = require_admin_read(state.as_ref(), &headers).await {
        return response;
    }
    let events = stream! {
        let mut last_fingerprint: Option<String> = None;
        let mut sent_sync = false;
        let sync_id = uuid::Uuid::new_v4().to_string();
        let mut seq: u64 = 0;
        loop {
            match admin_service::list_runtime_jobs(state.as_ref()).await {
                Ok(jobs) => {
                    let items = jobs
                        .into_iter()
                        .take(120)
                        .map(|job| RuntimeJobListItem {
                            provider: payload_text(&job.payload, "provider")
                                .or_else(|| payload_text(&job.payload, "provider_name")),
                            operation: payload_text(&job.payload, "operation")
                                .or_else(|| payload_text(&job.payload, "provider_operation")),
                            job_id: job.job_id,
                            status: job.status,
                            attempt_count: job.attempt_count,
                            next_run_at: job.next_run_at,
                            updated_at: job.updated_at,
                            payload: job.payload,
                        })
                        .collect::<Vec<_>>();
                    let fingerprint = items
                        .iter()
                        .map(|job| {
                            format!(
                                "{}|{}|{}|{}",
                                job.job_id,
                                job.status,
                                job.attempt_count,
                                job.updated_at
                                    .map(|value| value.timestamp_millis().to_string())
                                    .unwrap_or_default()
                            )
                        })
                        .collect::<Vec<_>>()
                        .join(";");
                    if !sent_sync {
                        sent_sync = true;
                        last_fingerprint = Some(fingerprint);
                        let payload = serde_json::json!({ "items": items });
                        yield Ok::<Event, Infallible>(
                            canonical_sse::sync_event(
                                "admin.runtime_jobs",
                                "runtime_jobs",
                                "all",
                                &sync_id,
                                seq,
                                "runtime_jobs.v1",
                                payload,
                            ),
                        );
                    } else if last_fingerprint.as_ref() != Some(&fingerprint) {
                        seq = seq.saturating_add(1);
                        last_fingerprint = Some(fingerprint);
                        yield Ok::<Event, Infallible>(
                            canonical_sse::delta_event(
                                "admin.runtime_jobs",
                                "runtime_jobs",
                                "all",
                                &sync_id,
                                seq,
                                "runtime_jobs.v1",
                                vec![canonical_sse::op_upsert(
                                    "/items",
                                    serde_json::json!(items),
                                )],
                            ),
                        );
                    }
                }
                Err(_) => {
                    yield Ok::<Event, Infallible>(canonical_sse::error_event(
                        "runtime jobs stream temporarily unavailable",
                    ));
                }
            }
            tokio::time::sleep(Duration::from_secs(3)).await;
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

async fn stream_runtime_job_events(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Path(job_id): Path<String>,
    uri: Uri,
) -> impl IntoResponse {
    if let Err(response) = require_admin_read(state.as_ref(), &headers).await {
        return response;
    }
    let request = match runtime_event_cursor::parse_runtime_event_request(uri.query(), &job_id) {
        Ok(value) => value,
        Err(message) => {
            return map_admin_error(
                admin_service::AdminServiceError::InvalidRequest(message),
                &headers,
            )
            .into_response();
        }
    };
    if let Err(err) = admin_service::ensure_runtime_job_exists(state.as_ref(), &job_id).await {
        return map_admin_error(err, &headers).into_response();
    }
    let mut after_id = request.after_id;
    let limit = request.limit;
    let events = stream! {
        let sync_id = uuid::Uuid::new_v4().to_string();
        let mut seq: u64 = 0;
        yield Ok::<Event, Infallible>(
            canonical_sse::sync_event(
                "admin.runtime_job_events",
                "runtime_job_events",
                job_id.as_str(),
                &sync_id,
                seq,
                "runtime_job_events.v1",
                serde_json::json!({
                    "job_id": job_id,
                    "events": [],
                }),
            ),
        );
        loop {
            match admin_service::list_runtime_job_events_page(state.as_ref(), &job_id, after_id, limit).await {
                Ok(page) => {
                    for event in page.items {
                        after_id = event.id;
                        seq = seq.saturating_add(1);
                        let payload = serde_json::json!({
                            "id": event.id,
                            "event_type": event.event_type,
                            "event_payload": event.event_payload,
                            "created_at": event.created_at,
                        });
                        yield Ok::<Event, Infallible>(
                            canonical_sse::delta_event(
                                "admin.runtime_job_events",
                                "runtime_job_events",
                                job_id.as_str(),
                                &sync_id,
                                seq,
                                "runtime_job_events.v1",
                                vec![canonical_sse::op_append("/events", payload)],
                            ),
                        );
                    }
                    if page.has_more {
                        continue;
                    }
                }
                Err(_) => {
                    yield Ok::<Event, Infallible>(canonical_sse::error_event(
                        "stream temporarily unavailable",
                    ));
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
        admin_service::AppendAdminAuditInput {
            actor_user_id: Some(&session.user_id),
            actor_email: &session.email,
            action: "runtime_retry",
            target_type: "runtime_job",
            target_id: &job_id,
            reason: &payload.reason,
            request_id: &request_id_from_headers(&headers),
            metadata: serde_json::json!({}),
        },
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
        admin_service::AppendAdminAuditInput {
            actor_user_id: Some(&session.user_id),
            actor_email: &session.email,
            action: "runtime_requeue",
            target_type: "runtime_job",
            target_id: &job_id,
            reason: &payload.reason,
            request_id: &request_id_from_headers(&headers),
            metadata: serde_json::json!({}),
        },
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
        admin_service::AppendAdminAuditInput {
            actor_user_id: Some(&session.user_id),
            actor_email: &session.email,
            action: "runtime_cancel",
            target_type: "runtime_job",
            target_id: &job_id,
            reason: &payload.reason,
            request_id: &request_id_from_headers(&headers),
            metadata: serde_json::json!({}),
        },
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
        admin_service::AppendAdminAuditInput {
            actor_user_id: Some(&session.user_id),
            actor_email: &session.email,
            action: "kill_switch_update",
            target_type: "kill_switch",
            target_id: &flag,
            reason: &payload.reason,
            request_id: &request_id_from_headers(&headers),
            metadata: serde_json::json!({ "enabled": payload.enabled }),
        },
    )
    .await;
    StatusCode::NO_CONTENT.into_response()
}

async fn list_observability_events(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Query(query): Query<ObservabilityEventsQuery>,
) -> impl IntoResponse {
    if let Err(response) = require_admin_read(state.as_ref(), &headers).await {
        return response;
    }
    match admin_service::list_observability_events(state.as_ref()).await {
        Ok(events) => {
            let limit = clamp_page_limit(query.limit);
            let q = normalize_query_text(query.q.as_deref());
            let source = normalize_optional_query(query.source.as_deref());
            let event_type = normalize_optional_query(query.event_type.as_deref());
            let target_id = normalize_query_text(query.target_id.as_deref());

            let mapped = events
                .into_iter()
                .map(|event| ObservabilityEventListItem {
                    request_id: request_id_from_detail(&event.detail),
                    occurred_at: event.occurred_at,
                    source: event.source,
                    event_type: event.event_type,
                    target_id: event.target_id,
                    detail: event.detail,
                })
                .collect::<Vec<_>>();

            let filtered = mapped
                .into_iter()
                .filter(|event| {
                    let matches_q = q.as_ref().is_none_or(|needle| {
                        let haystack = format!(
                            "{} {} {} {}",
                            event.source,
                            event.event_type,
                            event.target_id.as_deref().unwrap_or_default(),
                            event.detail
                        )
                        .to_ascii_lowercase();
                        haystack.contains(&needle.to_ascii_lowercase())
                    });
                    let matches_source = source
                        .as_ref()
                        .is_none_or(|value| event.source.eq_ignore_ascii_case(value));
                    let matches_type = event_type
                        .as_ref()
                        .is_none_or(|value| event.event_type.eq_ignore_ascii_case(value));
                    let matches_target = target_id.as_ref().is_none_or(|value| {
                        event
                            .target_id
                            .as_deref()
                            .is_some_and(|candidate| candidate.eq_ignore_ascii_case(value))
                    });
                    matches_q && matches_source && matches_type && matches_target
                })
                .collect::<Vec<_>>();

            match paginate_items(&filtered, limit, query.cursor.as_deref()) {
                Ok((items, page)) => (
                    StatusCode::OK,
                    Json(CursorEnvelope {
                        items,
                        page,
                        filters: ObservabilityFilters {
                            q,
                            source,
                            event_type,
                            target_id,
                        },
                    }),
                )
                    .into_response(),
                Err(err) => map_admin_error(err, &headers).into_response(),
            }
        }
        Err(err) => map_admin_error(err, &headers).into_response(),
    }
}

async fn stream_observability_events(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
) -> impl IntoResponse {
    if let Err(response) = require_admin_read(state.as_ref(), &headers).await {
        return response;
    }
    let events = stream! {
        let mut last_fingerprint: Option<String> = None;
        let mut sent_sync = false;
        let sync_id = uuid::Uuid::new_v4().to_string();
        let mut seq: u64 = 0;
        loop {
            match admin_service::list_observability_events(state.as_ref()).await {
                Ok(events) => {
                    let items = events
                        .into_iter()
                        .take(120)
                        .map(|event| ObservabilityEventListItem {
                            request_id: request_id_from_detail(&event.detail),
                            occurred_at: event.occurred_at,
                            source: event.source,
                            event_type: event.event_type,
                            target_id: event.target_id,
                            detail: event.detail,
                        })
                        .collect::<Vec<_>>();
                    let fingerprint = items
                        .iter()
                        .map(|event| {
                            format!(
                                "{}|{}|{}|{}",
                                event.occurred_at.timestamp_millis(),
                                event.source,
                                event.event_type,
                                event.target_id.as_deref().unwrap_or_default()
                            )
                        })
                        .collect::<Vec<_>>()
                        .join(";");
                    if !sent_sync {
                        sent_sync = true;
                        last_fingerprint = Some(fingerprint);
                        let next_cursor = (!items.is_empty()).then(|| encode_cursor_offset(items.len()));
                        let payload = serde_json::json!({
                            "items": items,
                            "cursor": next_cursor,
                        });
                        yield Ok::<Event, Infallible>(
                            canonical_sse::sync_event(
                                "admin.observability_events",
                                "observability_events",
                                "all",
                                &sync_id,
                                seq,
                                "observability_events.v1",
                                payload,
                            ),
                        );
                    } else if last_fingerprint.as_ref() != Some(&fingerprint) {
                        seq = seq.saturating_add(1);
                        last_fingerprint = Some(fingerprint);
                        yield Ok::<Event, Infallible>(
                            canonical_sse::delta_event(
                                "admin.observability_events",
                                "observability_events",
                                "all",
                                &sync_id,
                                seq,
                                "observability_events.v1",
                                vec![canonical_sse::op_upsert(
                                    "/items",
                                    serde_json::json!(items),
                                )],
                            ),
                        );
                    }
                }
                Err(_) => {
                    yield Ok::<Event, Infallible>(canonical_sse::error_event(
                        "observability stream temporarily unavailable",
                    ));
                }
            }
            tokio::time::sleep(Duration::from_secs(3)).await;
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
        Some(500),
    )
    .await
    {
        Ok(incidents) => {
            let limit = clamp_page_limit(query.limit);
            let q = normalize_query_text(query.q.as_deref());
            let status = normalize_optional_query(query.status.as_deref());
            let severity = normalize_optional_query(query.severity.as_deref());

            let filtered = incidents
                .into_iter()
                .filter(|incident| {
                    let matches_q = q.as_ref().is_none_or(|needle| {
                        let haystack = format!(
                            "{} {}",
                            incident.title,
                            incident.summary.as_deref().unwrap_or_default()
                        )
                        .to_ascii_lowercase();
                        haystack.contains(&needle.to_ascii_lowercase())
                    });
                    let matches_status = status
                        .as_ref()
                        .is_none_or(|value| incident.status.eq_ignore_ascii_case(value));
                    let matches_severity = severity
                        .as_ref()
                        .is_none_or(|value| incident.severity.eq_ignore_ascii_case(value));
                    matches_q && matches_status && matches_severity
                })
                .collect::<Vec<_>>();

            match paginate_items(&filtered, limit, query.cursor.as_deref()) {
                Ok((items, page)) => (
                    StatusCode::OK,
                    Json(CursorEnvelope {
                        items,
                        page,
                        filters: IncidentFilters {
                            q,
                            status,
                            severity,
                        },
                    }),
                )
                    .into_response(),
                Err(err) => map_admin_error(err, &headers).into_response(),
            }
        }
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
        admin_service::AppendAdminAuditInput {
            actor_user_id: Some(&session.user_id),
            actor_email: &session.email,
            action: "incident_create",
            target_type: "incident",
            target_id: &incident.id,
            reason,
            request_id: &request_id_from_headers(&headers),
            metadata: serde_json::json!({"severity": incident.severity, "status": incident.status}),
        },
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
        admin_service::AppendAdminAuditInput {
            actor_user_id: Some(&session.user_id),
            actor_email: &session.email,
            action: "incident_status_update",
            target_type: "incident",
            target_id: &incident_id,
            reason: &payload.reason,
            request_id: &request_id_from_headers(&headers),
            metadata: serde_json::json!({"status": updated.status}),
        },
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
