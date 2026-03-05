use chrono::{DateTime, Utc};

use crate::AppState;

use super::super::dashboard_service::{RuntimeJobEventRecord, RuntimeJobRecord};
use super::AdminServiceError;

#[derive(Debug, Clone)]
pub(crate) struct RuntimeJobEventsPage {
    pub(crate) items: Vec<RuntimeJobEventRecord>,
    pub(crate) has_more: bool,
    pub(crate) next_after_id: Option<i64>,
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
