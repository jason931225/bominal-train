use bominal_shared::crypto::{RedactionMode, redact_json};
use chrono::{DateTime, Utc};
use serde_json::Value;
use sqlx::PgPool;
use tracing::error;

use crate::AppState;

use super::{
    ProviderJobEvent, ProviderJobEventsPage, ProviderJobResult, ProviderJobsError,
    mapping::validate_job_id, state::runtime_job_exists,
};

pub(crate) async fn get_provider_job(
    state: &AppState,
    job_id: &str,
) -> Result<ProviderJobResult, ProviderJobsError> {
    validate_job_id(job_id)?;

    let Some(pool) = state.db_pool.as_ref() else {
        return Err(ProviderJobsError::PersistenceUnavailable);
    };

    let row = sqlx::query_as::<_, (String, String)>(
        "select status, payload::text from runtime_jobs where job_id = $1",
    )
    .bind(job_id)
    .fetch_optional(pool)
    .await
    .map_err(|err| {
        error!(error = %err, "failed to load runtime job row");
        ProviderJobsError::PersistenceFailure
    })?
    .ok_or(ProviderJobsError::NotFound)?;

    let payload = serde_json::from_str::<serde_json::Value>(&row.1).map_err(|err| {
        error!(error = %err, "failed to decode runtime job payload json");
        ProviderJobsError::PersistenceFailure
    })?;
    let provider = payload
        .get("provider")
        .and_then(serde_json::Value::as_str)
        .unwrap_or("unknown")
        .to_string();
    let operation = payload
        .get("operation")
        .and_then(serde_json::Value::as_str)
        .unwrap_or("unknown")
        .to_string();

    Ok(ProviderJobResult {
        job_id: job_id.to_string(),
        provider,
        operation,
        status: row.0,
    })
}

pub(crate) async fn list_provider_job_events_page(
    state: &AppState,
    job_id: &str,
    after_id: i64,
    limit: usize,
) -> Result<ProviderJobEventsPage, ProviderJobsError> {
    validate_job_id(job_id)?;
    if after_id < 0 || limit == 0 {
        return Err(ProviderJobsError::ValidationFailed);
    }

    let Some(pool) = state.db_pool.as_ref() else {
        return Err(ProviderJobsError::PersistenceUnavailable);
    };

    let limit_plus_one =
        i64::try_from(limit.saturating_add(1)).map_err(|_| ProviderJobsError::ValidationFailed)?;
    let rows = sqlx::query_as::<_, (i64, String, DateTime<Utc>, String)>(
        "select id, event_type, created_at, event_payload::text
         from runtime_job_events
         where job_id = $1 and id > $2
         order by id asc
         limit $3",
    )
    .bind(job_id)
    .bind(after_id)
    .bind(limit_plus_one)
    .fetch_all(pool)
    .await
    .map_err(|err| {
        error!(error = %err, "failed to load runtime job events");
        ProviderJobsError::PersistenceFailure
    })?;

    if rows.is_empty() && !runtime_job_exists(pool, job_id).await? {
        return Err(ProviderJobsError::NotFound);
    }

    let has_more = rows.len() > limit;
    let mut items = Vec::with_capacity(rows.len().min(limit));
    for row in rows.into_iter().take(limit) {
        let detail_raw = serde_json::from_str::<serde_json::Value>(&row.3).map_err(|err| {
            error!(error = %err, "failed to decode runtime job event payload");
            ProviderJobsError::PersistenceFailure
        })?;
        let detail = redact_json(&detail_raw, RedactionMode::Mask);

        items.push(ProviderJobEvent {
            sequence: row.0,
            event_type: row.1,
            occurred_at: row.2,
            detail,
        });
    }
    let next_after_id = items.last().map(|item| item.sequence);

    Ok(ProviderJobEventsPage {
        items,
        has_more,
        next_after_id,
    })
}
