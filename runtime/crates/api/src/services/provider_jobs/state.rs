use chrono::{DateTime, Utc};
use sqlx::PgPool;
use tracing::error;

use super::ProviderJobsError;

pub(super) async fn insert_provider_job_event(
    pool: &PgPool,
    job_id: &str,
    event_type: &str,
    event_payload: &serde_json::Value,
    occurred_at: DateTime<Utc>,
) -> Result<(), ProviderJobsError> {
    sqlx::query(
        "insert into runtime_job_events (job_id, event_type, event_payload, created_at) values ($1, $2, cast($3 as jsonb), $4)",
    )
    .bind(job_id)
    .bind(event_type)
    .bind(event_payload)
    .bind(occurred_at)
    .execute(pool)
    .await
    .map_err(|err| {
        error!(error = %err, "failed to insert runtime job event row");
        ProviderJobsError::PersistenceFailure
    })?;

    Ok(())
}

pub(super) async fn load_runtime_job_status(
    pool: &PgPool,
    job_id: &str,
) -> Result<String, ProviderJobsError> {
    let status =
        sqlx::query_scalar::<_, String>("select status from runtime_jobs where job_id = $1")
            .bind(job_id)
            .fetch_optional(pool)
            .await
            .map_err(|err| {
                error!(error = %err, "failed to load runtime job status");
                ProviderJobsError::PersistenceFailure
            })?
            .ok_or(ProviderJobsError::DuplicateConflict)?;

    Ok(status)
}

pub(super) async fn runtime_job_exists(
    pool: &PgPool,
    job_id: &str,
) -> Result<bool, ProviderJobsError> {
    let exists = sqlx::query_scalar::<_, i64>("select 1 from runtime_jobs where job_id = $1")
        .bind(job_id)
        .fetch_optional(pool)
        .await
        .map_err(|err| {
            error!(error = %err, "failed to check runtime job existence");
            ProviderJobsError::PersistenceFailure
        })?
        .is_some();

    Ok(exists)
}
