use std::collections::BTreeMap;

use bominal_shared::config::AdminRole;
use chrono::{DateTime, Utc};
use serde::Serialize;
use uuid::Uuid;

use super::super::AppState;
use super::auth_service;
use super::dashboard_service::{RuntimeJobEventRecord, RuntimeJobRecord};

#[derive(Debug)]
pub(crate) enum AdminServiceError {
    InvalidRequest(&'static str),
    NotFound(&'static str),
    ServiceUnavailable(&'static str),
    Internal,
}

#[derive(Debug, Clone)]
pub(crate) struct RuntimeJobEventsPage {
    pub(crate) items: Vec<RuntimeJobEventRecord>,
    pub(crate) has_more: bool,
    pub(crate) next_after_id: Option<i64>,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct AdminUserRecord {
    pub(crate) user_id: String,
    pub(crate) email: String,
    pub(crate) status: String,
    pub(crate) role: String,
    pub(crate) access_enabled: bool,
    pub(crate) updated_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct SessionRecord {
    pub(crate) session_id_hash: String,
    pub(crate) user_id: Option<String>,
    pub(crate) email: String,
    pub(crate) role: String,
    pub(crate) issued_at: DateTime<Utc>,
    pub(crate) last_seen_at: DateTime<Utc>,
    pub(crate) step_up_verified_at: Option<DateTime<Utc>>,
    pub(crate) revoked_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct KillSwitchRecord {
    pub(crate) flag: String,
    pub(crate) enabled: bool,
    pub(crate) reason: String,
    pub(crate) updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct ObservabilityEventRecord {
    pub(crate) occurred_at: DateTime<Utc>,
    pub(crate) source: String,
    pub(crate) event_type: String,
    pub(crate) target_id: Option<String>,
    pub(crate) detail: serde_json::Value,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct AuditRecord {
    pub(crate) id: String,
    pub(crate) actor_user_id: Option<String>,
    pub(crate) actor_email: String,
    pub(crate) action: String,
    pub(crate) target_type: String,
    pub(crate) target_id: String,
    pub(crate) reason: String,
    pub(crate) request_id: String,
    pub(crate) metadata: serde_json::Value,
    pub(crate) created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct IncidentRecord {
    pub(crate) id: String,
    pub(crate) title: String,
    pub(crate) severity: String,
    pub(crate) status: String,
    pub(crate) summary: Option<String>,
    pub(crate) context: serde_json::Value,
    pub(crate) opened_at: DateTime<Utc>,
    pub(crate) resolved_at: Option<DateTime<Utc>>,
    pub(crate) created_by: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct ObservabilityTimeseriesPoint {
    pub(crate) bucket: DateTime<Utc>,
    pub(crate) total_events: i64,
    pub(crate) error_events: i64,
    pub(crate) runtime_events: i64,
    pub(crate) provider_events: i64,
    pub(crate) auth_events: i64,
    pub(crate) admin_events: i64,
}

pub(crate) fn role_allows_admin_read(role: &AdminRole) -> bool {
    matches!(
        role,
        AdminRole::Admin | AdminRole::Operator | AdminRole::Viewer
    )
}

pub(crate) fn role_allows_admin_mutation(role: &AdminRole) -> bool {
    auth_service::admin_role_can_mutate(role)
}

pub(crate) fn validate_sensitive_confirmation(
    reason: &str,
    confirm_target: &str,
    expected_target: &str,
) -> Result<(), AdminServiceError> {
    let reason = reason.trim();
    let confirm_target = confirm_target.trim();
    if reason.len() < 8 {
        return Err(AdminServiceError::InvalidRequest(
            "reason must be at least 8 characters",
        ));
    }
    if reason.len() > 500 {
        return Err(AdminServiceError::InvalidRequest("reason too long"));
    }
    if confirm_target != expected_target {
        return Err(AdminServiceError::InvalidRequest(
            "typed confirmation target mismatch",
        ));
    }
    Ok(())
}

fn clamp_limit(limit: Option<usize>, default_value: usize, max_value: usize) -> i64 {
    let value = limit.unwrap_or(default_value);
    value.clamp(1, max_value) as i64
}

fn normalize_incident_severity(value: &str) -> Option<&'static str> {
    match value.trim().to_ascii_lowercase().as_str() {
        "sev1" => Some("sev1"),
        "sev2" => Some("sev2"),
        "sev3" => Some("sev3"),
        "sev4" => Some("sev4"),
        _ => None,
    }
}

fn normalize_incident_status(value: &str) -> Option<&'static str> {
    match value.trim().to_ascii_lowercase().as_str() {
        "open" => Some("open"),
        "monitoring" => Some("monitoring"),
        "resolved" => Some("resolved"),
        _ => None,
    }
}

pub(crate) async fn list_users(
    state: &AppState,
) -> Result<Vec<AdminUserRecord>, AdminServiceError> {
    let pool = state
        .db_pool
        .as_ref()
        .ok_or(AdminServiceError::ServiceUnavailable(
            "database unavailable",
        ))?;
    let rows = sqlx::query_as::<
        _,
        (
            Uuid,
            String,
            String,
            Option<String>,
            Option<bool>,
            Option<DateTime<Utc>>,
        ),
    >(
        "select u.id, u.email::text, u.status,
                r.role, r.access_enabled, r.updated_at
         from auth_users u
         left join auth_user_role_bindings r on r.user_id = u.id
         order by u.created_at desc",
    )
    .fetch_all(pool)
    .await
    .map_err(|_| AdminServiceError::Internal)?;
    Ok(rows
        .into_iter()
        .map(|row| AdminUserRecord {
            user_id: row.0.to_string(),
            email: row.1,
            status: row.2,
            role: row.3.unwrap_or_else(|| "user".to_string()),
            access_enabled: row.4.unwrap_or(true),
            updated_at: row.5,
        })
        .collect())
}

pub(crate) async fn set_user_role(
    state: &AppState,
    user_id: &str,
    role: &str,
    actor_user_id: Option<&str>,
) -> Result<(), AdminServiceError> {
    let role = role.trim().to_ascii_lowercase();
    if !matches!(role.as_str(), "admin" | "operator" | "viewer" | "user") {
        return Err(AdminServiceError::InvalidRequest("invalid role"));
    }
    let user_uuid = Uuid::parse_str(user_id)
        .map_err(|_| AdminServiceError::InvalidRequest("invalid user id"))?;
    let pool = state
        .db_pool
        .as_ref()
        .ok_or(AdminServiceError::ServiceUnavailable(
            "database unavailable",
        ))?;
    let actor_uuid = actor_user_id.and_then(|value| Uuid::parse_str(value).ok());
    sqlx::query(
        "insert into auth_user_role_bindings (user_id, role, access_enabled, updated_at, updated_by)
         values ($1, $2, true, now(), $3)
         on conflict (user_id)
         do update set role = excluded.role, updated_at = now(), updated_by = excluded.updated_by",
    )
    .bind(user_uuid)
    .bind(role)
    .bind(actor_uuid)
    .execute(pool)
    .await
    .map_err(|_| AdminServiceError::Internal)?;
    Ok(())
}

pub(crate) async fn set_user_access(
    state: &AppState,
    user_id: &str,
    access_enabled: bool,
    actor_user_id: Option<&str>,
) -> Result<(), AdminServiceError> {
    let user_uuid = Uuid::parse_str(user_id)
        .map_err(|_| AdminServiceError::InvalidRequest("invalid user id"))?;
    let pool = state
        .db_pool
        .as_ref()
        .ok_or(AdminServiceError::ServiceUnavailable(
            "database unavailable",
        ))?;
    let actor_uuid = actor_user_id.and_then(|value| Uuid::parse_str(value).ok());
    sqlx::query(
        "insert into auth_user_role_bindings (user_id, role, access_enabled, updated_at, updated_by)
         values ($1, 'user', $2, now(), $3)
         on conflict (user_id)
         do update set access_enabled = excluded.access_enabled, updated_at = now(), updated_by = excluded.updated_by",
    )
    .bind(user_uuid)
    .bind(access_enabled)
    .bind(actor_uuid)
    .execute(pool)
    .await
    .map_err(|_| AdminServiceError::Internal)?;
    Ok(())
}

pub(crate) async fn list_sessions(
    state: &AppState,
) -> Result<Vec<SessionRecord>, AdminServiceError> {
    let pool = state
        .db_pool
        .as_ref()
        .ok_or(AdminServiceError::ServiceUnavailable(
            "database unavailable",
        ))?;
    let rows = sqlx::query_as::<_, (String, Option<Uuid>, String, String, DateTime<Utc>, DateTime<Utc>, Option<DateTime<Utc>>, Option<DateTime<Utc>>)>(
        "select session_id_hash, user_id, email, role, issued_at, last_seen_at, step_up_verified_at, revoked_at
         from auth_sessions
         order by last_seen_at desc
         limit 500",
    )
    .fetch_all(pool)
    .await
    .map_err(|_| AdminServiceError::Internal)?;
    Ok(rows
        .into_iter()
        .map(|row| SessionRecord {
            session_id_hash: row.0,
            user_id: row.1.map(|value| value.to_string()),
            email: row.2,
            role: row.3,
            issued_at: row.4,
            last_seen_at: row.5,
            step_up_verified_at: row.6,
            revoked_at: row.7,
        })
        .collect())
}

pub(crate) async fn list_runtime_jobs(
    state: &AppState,
) -> Result<Vec<RuntimeJobRecord>, AdminServiceError> {
    let pool = state
        .db_pool
        .as_ref()
        .ok_or(AdminServiceError::ServiceUnavailable(
            "database unavailable",
        ))?;
    let rows = sqlx::query_as::<
        _,
        (
            String,
            String,
            i32,
            Option<DateTime<Utc>>,
            Option<DateTime<Utc>>,
            serde_json::Value,
        ),
    >(
        "select job_id, status, attempt_count, next_run_at, updated_at, payload
         from runtime_jobs
         order by updated_at desc nulls last, created_at desc
         limit 500",
    )
    .fetch_all(pool)
    .await
    .map_err(|_| AdminServiceError::Internal)?;
    Ok(rows
        .into_iter()
        .map(|row| RuntimeJobRecord {
            job_id: row.0,
            status: row.1,
            attempt_count: row.2,
            next_run_at: row.3,
            updated_at: row.4,
            payload: row.5,
        })
        .collect())
}

pub(crate) async fn get_runtime_job(
    state: &AppState,
    job_id: &str,
) -> Result<RuntimeJobRecord, AdminServiceError> {
    let pool = state
        .db_pool
        .as_ref()
        .ok_or(AdminServiceError::ServiceUnavailable(
            "database unavailable",
        ))?;
    let row = sqlx::query_as::<
        _,
        (
            String,
            String,
            i32,
            Option<DateTime<Utc>>,
            Option<DateTime<Utc>>,
            serde_json::Value,
        ),
    >(
        "select job_id, status, attempt_count, next_run_at, updated_at, payload
         from runtime_jobs
         where job_id = $1
         limit 1",
    )
    .bind(job_id.trim())
    .fetch_optional(pool)
    .await
    .map_err(|_| AdminServiceError::Internal)?;
    let Some(row) = row else {
        return Err(AdminServiceError::NotFound("runtime job not found"));
    };
    Ok(RuntimeJobRecord {
        job_id: row.0,
        status: row.1,
        attempt_count: row.2,
        next_run_at: row.3,
        updated_at: row.4,
        payload: row.5,
    })
}

pub(crate) async fn ensure_runtime_job_exists(
    state: &AppState,
    job_id: &str,
) -> Result<(), AdminServiceError> {
    let _ = get_runtime_job(state, job_id).await?;
    Ok(())
}

pub(crate) async fn list_runtime_job_events_page(
    state: &AppState,
    job_id: &str,
    after_id: i64,
    limit: usize,
) -> Result<RuntimeJobEventsPage, AdminServiceError> {
    if after_id < 0 {
        return Err(AdminServiceError::InvalidRequest(
            "invalid cursor token payload",
        ));
    }
    if limit == 0 {
        return Err(AdminServiceError::InvalidRequest(
            "invalid limit query parameter",
        ));
    }
    let pool = state
        .db_pool
        .as_ref()
        .ok_or(AdminServiceError::ServiceUnavailable(
            "database unavailable",
        ))?;
    let limit_plus_one = i64::try_from(limit.saturating_add(1))
        .map_err(|_| AdminServiceError::InvalidRequest("invalid limit query parameter"))?;
    let rows = sqlx::query_as::<_, (i64, String, serde_json::Value, DateTime<Utc>)>(
        "select id, event_type, event_payload, created_at
         from runtime_job_events
         where job_id = $1 and id > $2
         order by id asc
         limit $3",
    )
    .bind(job_id.trim())
    .bind(after_id)
    .bind(limit_plus_one)
    .fetch_all(pool)
    .await
    .map_err(|_| AdminServiceError::Internal)?;
    let has_more = rows.len() > limit;
    let items = rows
        .into_iter()
        .take(limit)
        .map(|row| RuntimeJobEventRecord {
            id: row.0,
            event_type: row.1,
            event_payload: row.2,
            created_at: row.3,
        })
        .collect::<Vec<_>>();
    let next_after_id = items.last().map(|item| item.id);

    Ok(RuntimeJobEventsPage {
        items,
        has_more,
        next_after_id,
    })
}

pub(crate) async fn retry_runtime_job(
    state: &AppState,
    job_id: &str,
) -> Result<(), AdminServiceError> {
    update_runtime_job_to_queued(state, job_id, "admin_retry").await
}

pub(crate) async fn requeue_runtime_job(
    state: &AppState,
    job_id: &str,
) -> Result<(), AdminServiceError> {
    update_runtime_job_to_queued(state, job_id, "admin_requeue").await
}

pub(crate) async fn cancel_runtime_job(
    state: &AppState,
    job_id: &str,
) -> Result<(), AdminServiceError> {
    let pool = state
        .db_pool
        .as_ref()
        .ok_or(AdminServiceError::ServiceUnavailable(
            "database unavailable",
        ))?;
    let result = sqlx::query(
        "update runtime_jobs
         set status = 'failed', last_error = 'cancelled_by_admin', updated_at = now()
         where job_id = $1 and status in ('queued', 'running')",
    )
    .bind(job_id.trim())
    .execute(pool)
    .await
    .map_err(|_| AdminServiceError::Internal)?;
    if result.rows_affected() == 0 {
        return Err(AdminServiceError::NotFound("runtime job not found"));
    }
    insert_runtime_job_event(
        state,
        job_id,
        "cancelled",
        serde_json::json!({"source": "admin"}),
    )
    .await?;
    Ok(())
}

async fn update_runtime_job_to_queued(
    state: &AppState,
    job_id: &str,
    event_type: &str,
) -> Result<(), AdminServiceError> {
    let pool = state
        .db_pool
        .as_ref()
        .ok_or(AdminServiceError::ServiceUnavailable(
            "database unavailable",
        ))?;
    let result = sqlx::query(
        "update runtime_jobs
         set status = 'queued', next_run_at = now(), updated_at = now()
         where job_id = $1",
    )
    .bind(job_id.trim())
    .execute(pool)
    .await
    .map_err(|_| AdminServiceError::Internal)?;
    if result.rows_affected() == 0 {
        return Err(AdminServiceError::NotFound("runtime job not found"));
    }
    insert_runtime_job_event(
        state,
        job_id,
        event_type,
        serde_json::json!({"source": "admin"}),
    )
    .await?;
    Ok(())
}

async fn insert_runtime_job_event(
    state: &AppState,
    job_id: &str,
    event_type: &str,
    payload: serde_json::Value,
) -> Result<(), AdminServiceError> {
    let pool = state
        .db_pool
        .as_ref()
        .ok_or(AdminServiceError::ServiceUnavailable(
            "database unavailable",
        ))?;
    sqlx::query(
        "insert into runtime_job_events (job_id, event_type, event_payload, created_at)
         values ($1, $2, cast($3 as jsonb), now())",
    )
    .bind(job_id.trim())
    .bind(event_type)
    .bind(payload)
    .execute(pool)
    .await
    .map_err(|_| AdminServiceError::Internal)?;
    Ok(())
}

pub(crate) async fn list_kill_switches(
    state: &AppState,
) -> Result<Vec<KillSwitchRecord>, AdminServiceError> {
    let pool = state
        .db_pool
        .as_ref()
        .ok_or(AdminServiceError::ServiceUnavailable(
            "database unavailable",
        ))?;
    let rows = sqlx::query_as::<_, (String, bool, String, DateTime<Utc>)>(
        "select flag, enabled, reason, updated_at
         from admin_runtime_flags
         order by flag asc",
    )
    .fetch_all(pool)
    .await
    .map_err(|_| AdminServiceError::Internal)?;
    Ok(rows
        .into_iter()
        .map(|row| KillSwitchRecord {
            flag: row.0,
            enabled: row.1,
            reason: row.2,
            updated_at: row.3,
        })
        .collect())
}

pub(crate) async fn upsert_kill_switch(
    state: &AppState,
    flag: &str,
    enabled: bool,
    reason: &str,
    actor_user_id: Option<&str>,
) -> Result<(), AdminServiceError> {
    let flag = flag.trim().to_ascii_lowercase();
    if !matches!(
        flag.as_str(),
        "runtime_ingest" | "runtime_dispatch" | "provider_calls"
    ) {
        return Err(AdminServiceError::InvalidRequest(
            "unknown kill switch flag",
        ));
    }
    let pool = state
        .db_pool
        .as_ref()
        .ok_or(AdminServiceError::ServiceUnavailable(
            "database unavailable",
        ))?;
    let actor_uuid = actor_user_id.and_then(|value| Uuid::parse_str(value).ok());
    sqlx::query(
        "insert into admin_runtime_flags (flag, enabled, reason, updated_at, updated_by)
         values ($1, $2, $3, now(), $4)
         on conflict (flag)
         do update set enabled = excluded.enabled, reason = excluded.reason, updated_at = now(), updated_by = excluded.updated_by",
    )
    .bind(flag)
    .bind(enabled)
    .bind(reason.trim())
    .bind(actor_uuid)
    .execute(pool)
    .await
    .map_err(|_| AdminServiceError::Internal)?;
    Ok(())
}

pub(crate) async fn list_observability_events(
    state: &AppState,
) -> Result<Vec<ObservabilityEventRecord>, AdminServiceError> {
    let pool = state
        .db_pool
        .as_ref()
        .ok_or(AdminServiceError::ServiceUnavailable(
            "database unavailable",
        ))?;
    let rows = sqlx::query_as::<_, (DateTime<Utc>, String, String, Option<String>, serde_json::Value)>(
        "select occurred_at, source, event_type, target_id, detail
         from (
           select e.created_at as occurred_at, 'runtime_job_events'::text as source, e.event_type, e.job_id as target_id, e.event_payload as detail
           from runtime_job_events e
           union all
           select l.created_at as occurred_at, 'provider_contract_ledger'::text as source, l.operation as event_type, l.job_id as target_id,
                  jsonb_build_object('provider', l.provider, 'outcome', l.outcome, 'status', l.response_status_code) as detail
           from provider_contract_ledger l
           union all
           select a.created_at as occurred_at, 'auth_audit_log'::text as source, a.event_type, coalesce(a.user_id::text, 'unknown') as target_id,
                  coalesce(a.metadata, '{}'::jsonb) as detail
           from auth_audit_log a
           union all
           select aa.created_at as occurred_at, 'admin_audit_log'::text as source, aa.action as event_type, aa.target_id,
                  coalesce(aa.metadata, '{}'::jsonb) as detail
           from admin_audit_log aa
         ) unified
         order by occurred_at desc
         limit 400",
    )
    .fetch_all(pool)
    .await
    .map_err(|_| AdminServiceError::Internal)?;
    Ok(rows
        .into_iter()
        .map(|row| ObservabilityEventRecord {
            occurred_at: row.0,
            source: row.1,
            event_type: row.2,
            target_id: row.3,
            detail: row.4,
        })
        .collect())
}

pub(crate) async fn list_observability_timeseries(
    state: &AppState,
    window_minutes: Option<i64>,
) -> Result<Vec<ObservabilityTimeseriesPoint>, AdminServiceError> {
    let pool = state
        .db_pool
        .as_ref()
        .ok_or(AdminServiceError::ServiceUnavailable(
            "database unavailable",
        ))?;
    let minutes = window_minutes.unwrap_or(240).clamp(15, 1440);
    let rows = sqlx::query_as::<_, (DateTime<Utc>, i64, i64, i64, i64, i64, i64)>(
        "with bounds as (
           select now() - ($1::int * interval '1 minute') as since_at
         ),
         events as (
           select date_trunc('minute', e.created_at) as bucket,
                  1::bigint as total_events,
                  case
                    when lower(e.event_type) like '%fail%' then 1::bigint
                    when lower(e.event_type) like '%error%' then 1::bigint
                    when lower(e.event_type) in ('cancelled', 'dead_lettered') then 1::bigint
                    else 0::bigint
                  end as error_events,
                  1::bigint as runtime_events,
                  0::bigint as provider_events,
                  0::bigint as auth_events,
                  0::bigint as admin_events
             from runtime_job_events e, bounds b
            where e.created_at >= b.since_at
           union all
           select date_trunc('minute', l.created_at) as bucket,
                  1::bigint as total_events,
                  case
                    when lower(coalesce(l.outcome, '')) in ('failed', 'error', 'timeout', 'rejected') then 1::bigint
                    else 0::bigint
                  end as error_events,
                  0::bigint as runtime_events,
                  1::bigint as provider_events,
                  0::bigint as auth_events,
                  0::bigint as admin_events
             from provider_contract_ledger l, bounds b
            where l.created_at >= b.since_at
           union all
           select date_trunc('minute', a.created_at) as bucket,
                  1::bigint as total_events,
                  0::bigint as error_events,
                  0::bigint as runtime_events,
                  0::bigint as provider_events,
                  1::bigint as auth_events,
                  0::bigint as admin_events
             from auth_audit_log a, bounds b
            where a.created_at >= b.since_at
           union all
           select date_trunc('minute', aa.created_at) as bucket,
                  1::bigint as total_events,
                  0::bigint as error_events,
                  0::bigint as runtime_events,
                  0::bigint as provider_events,
                  0::bigint as auth_events,
                  1::bigint as admin_events
             from admin_audit_log aa, bounds b
            where aa.created_at >= b.since_at
         )
         select bucket,
                sum(total_events) as total_events,
                sum(error_events) as error_events,
                sum(runtime_events) as runtime_events,
                sum(provider_events) as provider_events,
                sum(auth_events) as auth_events,
                sum(admin_events) as admin_events
           from events
       group by bucket
       order by bucket asc",
    )
    .bind(minutes as i32)
    .fetch_all(pool)
    .await
    .map_err(|_| AdminServiceError::Internal)?;
    Ok(rows
        .into_iter()
        .map(|row| ObservabilityTimeseriesPoint {
            bucket: row.0,
            total_events: row.1,
            error_events: row.2,
            runtime_events: row.3,
            provider_events: row.4,
            auth_events: row.5,
            admin_events: row.6,
        })
        .collect())
}

pub(crate) async fn list_incidents(
    state: &AppState,
    status: Option<&str>,
    severity: Option<&str>,
    limit: Option<usize>,
) -> Result<Vec<IncidentRecord>, AdminServiceError> {
    let pool = state
        .db_pool
        .as_ref()
        .ok_or(AdminServiceError::ServiceUnavailable(
            "database unavailable",
        ))?;
    let status_filter = match status {
        Some(value) if !value.trim().is_empty() => Some(
            normalize_incident_status(value)
                .ok_or(AdminServiceError::InvalidRequest("invalid incident status"))?
                .to_string(),
        ),
        _ => None,
    };
    let severity_filter = match severity {
        Some(value) if !value.trim().is_empty() => Some(
            normalize_incident_severity(value)
                .ok_or(AdminServiceError::InvalidRequest(
                    "invalid incident severity",
                ))?
                .to_string(),
        ),
        _ => None,
    };
    let rows = sqlx::query_as::<
        _,
        (
            Uuid,
            String,
            String,
            String,
            Option<String>,
            serde_json::Value,
            DateTime<Utc>,
            Option<DateTime<Utc>>,
            Option<Uuid>,
        ),
    >(
        "select id, title, severity, status, summary, context_json, opened_at, resolved_at, created_by
           from admin_incidents
          where ($1::text is null or status = $1::text)
            and ($2::text is null or severity = $2::text)
       order by opened_at desc
          limit $3",
    )
    .bind(status_filter)
    .bind(severity_filter)
    .bind(clamp_limit(limit, 120, 500))
    .fetch_all(pool)
    .await
    .map_err(|_| AdminServiceError::Internal)?;
    Ok(rows
        .into_iter()
        .map(|row| IncidentRecord {
            id: row.0.to_string(),
            title: row.1,
            severity: row.2,
            status: row.3,
            summary: row.4,
            context: row.5,
            opened_at: row.6,
            resolved_at: row.7,
            created_by: row.8.map(|value| value.to_string()),
        })
        .collect())
}

pub(crate) async fn create_incident(
    state: &AppState,
    title: &str,
    severity: &str,
    summary: Option<&str>,
    context: serde_json::Value,
    created_by: Option<&str>,
) -> Result<IncidentRecord, AdminServiceError> {
    let title = title.trim();
    if title.len() < 3 {
        return Err(AdminServiceError::InvalidRequest(
            "incident title must be at least 3 characters",
        ));
    }
    if title.len() > 140 {
        return Err(AdminServiceError::InvalidRequest("incident title too long"));
    }
    let severity = normalize_incident_severity(severity).ok_or(
        AdminServiceError::InvalidRequest("invalid incident severity"),
    )?;
    let summary = summary
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(|value| value.to_string());
    if let Some(value) = summary.as_ref() {
        if value.len() > 600 {
            return Err(AdminServiceError::InvalidRequest(
                "incident summary too long",
            ));
        }
    }
    let actor_uuid = match created_by {
        Some(value) => Some(
            Uuid::parse_str(value)
                .map_err(|_| AdminServiceError::InvalidRequest("invalid actor user id"))?,
        ),
        None => None,
    };

    let pool = state
        .db_pool
        .as_ref()
        .ok_or(AdminServiceError::ServiceUnavailable(
            "database unavailable",
        ))?;
    let row = sqlx::query_as::<
        _,
        (
            Uuid,
            String,
            String,
            String,
            Option<String>,
            serde_json::Value,
            DateTime<Utc>,
            Option<DateTime<Utc>>,
            Option<Uuid>,
        ),
    >(
        "insert into admin_incidents
         (id, title, severity, status, summary, context_json, opened_at, created_by)
         values ($1, $2, $3, 'open', $4, cast($5 as jsonb), now(), $6)
         returning id, title, severity, status, summary, context_json, opened_at, resolved_at, created_by",
    )
    .bind(Uuid::new_v4())
    .bind(title)
    .bind(severity)
    .bind(summary)
    .bind(context)
    .bind(actor_uuid)
    .fetch_one(pool)
    .await
    .map_err(|_| AdminServiceError::Internal)?;

    Ok(IncidentRecord {
        id: row.0.to_string(),
        title: row.1,
        severity: row.2,
        status: row.3,
        summary: row.4,
        context: row.5,
        opened_at: row.6,
        resolved_at: row.7,
        created_by: row.8.map(|value| value.to_string()),
    })
}

pub(crate) async fn update_incident_status(
    state: &AppState,
    incident_id: &str,
    status: &str,
    summary: Option<&str>,
) -> Result<IncidentRecord, AdminServiceError> {
    let incident_uuid = Uuid::parse_str(incident_id.trim())
        .map_err(|_| AdminServiceError::InvalidRequest("invalid incident id"))?;
    let status = normalize_incident_status(status)
        .ok_or(AdminServiceError::InvalidRequest("invalid incident status"))?;
    let summary = summary
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(|value| value.to_string());
    if let Some(value) = summary.as_ref() {
        if value.len() > 600 {
            return Err(AdminServiceError::InvalidRequest(
                "incident summary too long",
            ));
        }
    }
    let pool = state
        .db_pool
        .as_ref()
        .ok_or(AdminServiceError::ServiceUnavailable(
            "database unavailable",
        ))?;
    let row = sqlx::query_as::<
        _,
        (
            Uuid,
            String,
            String,
            String,
            Option<String>,
            serde_json::Value,
            DateTime<Utc>,
            Option<DateTime<Utc>>,
            Option<Uuid>,
        ),
    >(
        "update admin_incidents
            set status = $2,
                summary = coalesce($3, summary),
                resolved_at = case
                  when $2 = 'resolved' then coalesce(resolved_at, now())
                  else null
                end
          where id = $1
      returning id, title, severity, status, summary, context_json, opened_at, resolved_at, created_by",
    )
    .bind(incident_uuid)
    .bind(status)
    .bind(summary)
    .fetch_optional(pool)
    .await
    .map_err(|_| AdminServiceError::Internal)?;
    let Some(row) = row else {
        return Err(AdminServiceError::NotFound("incident not found"));
    };
    Ok(IncidentRecord {
        id: row.0.to_string(),
        title: row.1,
        severity: row.2,
        status: row.3,
        summary: row.4,
        context: row.5,
        opened_at: row.6,
        resolved_at: row.7,
        created_by: row.8.map(|value| value.to_string()),
    })
}

pub(crate) fn redacted_config_snapshot(state: &AppState) -> BTreeMap<String, serde_json::Value> {
    let mut out = BTreeMap::new();
    out.insert(
        "app_env".to_string(),
        serde_json::json!(state.config.app_env),
    );
    out.insert(
        "app_host".to_string(),
        serde_json::json!(state.config.app_host),
    );
    out.insert(
        "app_port".to_string(),
        serde_json::json!(state.config.app_port),
    );
    out.insert(
        "session_cookie_name".to_string(),
        serde_json::json!(state.config.session_cookie_name),
    );
    out.insert(
        "session_cookie_domain".to_string(),
        serde_json::json!(state.config.session_cookie_domain),
    );
    out.insert(
        "session_ttl_seconds".to_string(),
        serde_json::json!(state.config.session_ttl_seconds),
    );
    out.insert(
        "step_up_ttl_seconds".to_string(),
        serde_json::json!(state.config.step_up_ttl_seconds),
    );
    out.insert(
        "user_app_host".to_string(),
        serde_json::json!(state.config.user_app_host),
    );
    out.insert(
        "admin_app_host".to_string(),
        serde_json::json!(state.config.admin_app_host),
    );
    out.insert(
        "passkey_provider".to_string(),
        serde_json::json!(state.config.passkey.provider),
    );
    out.insert(
        "webauthn_rp_id".to_string(),
        serde_json::json!(state.config.passkey.webauthn_rp_id),
    );
    out.insert(
        "webauthn_rp_origin".to_string(),
        serde_json::json!(state.config.passkey.webauthn_rp_origin),
    );
    out.insert(
        "database_url_present".to_string(),
        serde_json::json!(!state.config.database_url.trim().is_empty()),
    );
    out.insert(
        "redis_url_present".to_string(),
        serde_json::json!(!state.config.redis.url.trim().is_empty()),
    );
    out.insert(
        "session_secret_present".to_string(),
        serde_json::json!(!state.config.session_secret.trim().is_empty()),
    );
    out.insert(
        "invite_base_url".to_string(),
        serde_json::json!(state.config.invite_base_url),
    );
    out
}

pub(crate) async fn append_admin_audit(
    state: &AppState,
    actor_user_id: Option<&str>,
    actor_email: &str,
    action: &str,
    target_type: &str,
    target_id: &str,
    reason: &str,
    request_id: &str,
    metadata: serde_json::Value,
) -> Result<(), AdminServiceError> {
    let pool = state
        .db_pool
        .as_ref()
        .ok_or(AdminServiceError::ServiceUnavailable(
            "database unavailable",
        ))?;
    let actor_uuid = actor_user_id.and_then(|value| Uuid::parse_str(value).ok());
    sqlx::query(
        "insert into admin_audit_log
         (id, actor_user_id, actor_email, action, target_type, target_id, reason, request_id, metadata, created_at)
         values ($1, $2, $3, $4, $5, $6, $7, $8, cast($9 as jsonb), now())",
    )
    .bind(Uuid::new_v4())
    .bind(actor_uuid)
    .bind(actor_email)
    .bind(action)
    .bind(target_type)
    .bind(target_id)
    .bind(reason.trim())
    .bind(request_id)
    .bind(metadata)
    .execute(pool)
    .await
    .map_err(|_| AdminServiceError::Internal)?;
    Ok(())
}

pub(crate) async fn list_admin_audit(
    state: &AppState,
) -> Result<Vec<AuditRecord>, AdminServiceError> {
    let pool = state
        .db_pool
        .as_ref()
        .ok_or(AdminServiceError::ServiceUnavailable(
            "database unavailable",
        ))?;
    let rows = sqlx::query_as::<_, (Uuid, Option<Uuid>, String, String, String, String, String, String, serde_json::Value, DateTime<Utc>)>(
        "select id, actor_user_id, actor_email, action, target_type, target_id, reason, request_id, metadata, created_at
         from admin_audit_log
         order by created_at desc
         limit 400",
    )
    .fetch_all(pool)
    .await
    .map_err(|_| AdminServiceError::Internal)?;
    Ok(rows
        .into_iter()
        .map(|row| AuditRecord {
            id: row.0.to_string(),
            actor_user_id: row.1.map(|value| value.to_string()),
            actor_email: row.2,
            action: row.3,
            target_type: row.4,
            target_id: row.5,
            reason: row.6,
            request_id: row.7,
            metadata: row.8,
            created_at: row.9,
        })
        .collect())
}
