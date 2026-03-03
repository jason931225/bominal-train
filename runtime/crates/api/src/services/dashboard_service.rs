use chrono::{DateTime, Utc};
use serde::Serialize;

use super::super::AppState;

#[derive(Debug, Clone, Serialize)]
pub(crate) struct DashboardSummary {
    pub(crate) total_jobs: i64,
    pub(crate) queued_jobs: i64,
    pub(crate) running_jobs: i64,
    pub(crate) failed_jobs: i64,
    pub(crate) support_request_id: String,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct RuntimeJobRecord {
    pub(crate) job_id: String,
    pub(crate) status: String,
    pub(crate) attempt_count: i32,
    pub(crate) next_run_at: Option<DateTime<Utc>>,
    pub(crate) updated_at: Option<DateTime<Utc>>,
    pub(crate) payload: serde_json::Value,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct RuntimeJobEventRecord {
    pub(crate) id: i64,
    pub(crate) event_type: String,
    pub(crate) event_payload: serde_json::Value,
    pub(crate) created_at: DateTime<Utc>,
}

#[derive(Debug)]
pub(crate) enum DashboardServiceError {
    InvalidRequest(&'static str),
    ServiceUnavailable(&'static str),
    NotFound(&'static str),
    Internal,
}

pub(crate) async fn load_dashboard_summary(
    state: &AppState,
    user_id: &str,
    request_id: String,
) -> Result<DashboardSummary, DashboardServiceError> {
    if user_id.trim().is_empty() {
        return Err(DashboardServiceError::InvalidRequest("user id is required"));
    }
    let pool = state
        .db_pool
        .as_ref()
        .ok_or(DashboardServiceError::ServiceUnavailable(
            "database unavailable",
        ))?;
    let row = sqlx::query_as::<_, (i64, i64, i64, i64)>(
        "select
            count(*)::bigint as total_jobs,
            count(*) filter (where status = 'queued')::bigint as queued_jobs,
            count(*) filter (where status = 'running')::bigint as running_jobs,
            count(*) filter (where status in ('failed', 'dead_lettered'))::bigint as failed_jobs
         from runtime_jobs
         where payload->>'user_id' = $1",
    )
    .bind(user_id)
    .fetch_one(pool)
    .await
    .map_err(|_| DashboardServiceError::Internal)?;
    Ok(DashboardSummary {
        total_jobs: row.0,
        queued_jobs: row.1,
        running_jobs: row.2,
        failed_jobs: row.3,
        support_request_id: request_id,
    })
}

pub(crate) async fn list_dashboard_jobs(
    state: &AppState,
    user_id: &str,
) -> Result<Vec<RuntimeJobRecord>, DashboardServiceError> {
    if user_id.trim().is_empty() {
        return Err(DashboardServiceError::InvalidRequest("user id is required"));
    }
    let pool = state
        .db_pool
        .as_ref()
        .ok_or(DashboardServiceError::ServiceUnavailable(
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
         where payload->>'user_id' = $1
         order by updated_at desc nulls last, created_at desc
         limit 200",
    )
    .bind(user_id)
    .fetch_all(pool)
    .await
    .map_err(|_| DashboardServiceError::Internal)?;
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

pub(crate) async fn get_dashboard_job(
    state: &AppState,
    user_id: &str,
    job_id: &str,
) -> Result<RuntimeJobRecord, DashboardServiceError> {
    if job_id.trim().is_empty() {
        return Err(DashboardServiceError::InvalidRequest("job id is required"));
    }
    let pool = state
        .db_pool
        .as_ref()
        .ok_or(DashboardServiceError::ServiceUnavailable(
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
           and payload->>'user_id' = $2
         limit 1",
    )
    .bind(job_id.trim())
    .bind(user_id)
    .fetch_optional(pool)
    .await
    .map_err(|_| DashboardServiceError::Internal)?;
    let Some(row) = row else {
        return Err(DashboardServiceError::NotFound("job not found"));
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

pub(crate) async fn list_dashboard_job_events(
    state: &AppState,
    user_id: &str,
    job_id: &str,
    since_id: Option<i64>,
) -> Result<Vec<RuntimeJobEventRecord>, DashboardServiceError> {
    let _ = get_dashboard_job(state, user_id, job_id).await?;
    let pool = state
        .db_pool
        .as_ref()
        .ok_or(DashboardServiceError::ServiceUnavailable(
            "database unavailable",
        ))?;
    let start_id = since_id.unwrap_or(0);
    let rows = sqlx::query_as::<_, (i64, String, serde_json::Value, DateTime<Utc>)>(
        "select id, event_type, event_payload, created_at
         from runtime_job_events
         where job_id = $1 and id > $2
         order by id asc",
    )
    .bind(job_id.trim())
    .bind(start_id)
    .fetch_all(pool)
    .await
    .map_err(|_| DashboardServiceError::Internal)?;
    Ok(rows
        .into_iter()
        .map(|row| RuntimeJobEventRecord {
            id: row.0,
            event_type: row.1,
            event_payload: row.2,
            created_at: row.3,
        })
        .collect())
}
