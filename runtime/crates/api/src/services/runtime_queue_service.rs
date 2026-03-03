use bominal_shared::{
    queue::RuntimeQueueJob,
    repo::{RepoError, insert_runtime_job},
};
use redis::AsyncCommands;
use tracing::{error, info};
use uuid::Uuid;

use super::super::AppState;

#[derive(Debug, serde::Deserialize)]
pub(crate) struct EnqueueRuntimeJobRequest {
    pub(crate) job_id: Option<String>,
    pub(crate) user_id: String,
    pub(crate) kind: String,
    pub(crate) payload: serde_json::Value,
}

#[derive(Debug)]
pub(crate) struct EnqueueRuntimeJobResult {
    pub(crate) queue_key: String,
    pub(crate) job_id: String,
}

#[derive(Debug)]
pub(crate) enum EnqueueRuntimeJobError {
    ValidationFailed,
    EncodeFailed,
    DuplicateJobConflict,
    PersistenceUnavailable,
    RedisUnavailable,
    RedisConnectionFailed,
    QueuePushFailed,
    PersistenceFailure,
}

enum PersistRuntimeJobOutcome {
    Inserted,
    DuplicateIdempotent,
}

pub(crate) async fn enqueue_runtime_job(
    state: &AppState,
    payload: EnqueueRuntimeJobRequest,
) -> Result<EnqueueRuntimeJobResult, EnqueueRuntimeJobError> {
    validate_enqueue_request(&payload)?;

    let job = RuntimeQueueJob {
        job_id: payload.job_id.unwrap_or_else(|| Uuid::new_v4().to_string()),
        user_id: payload.user_id,
        kind: payload.kind,
        payload: payload.payload,
        enqueued_at: chrono::Utc::now(),
    };

    let persist_outcome = persist_runtime_job(state, &job).await?;
    if matches!(persist_outcome, PersistRuntimeJobOutcome::Inserted) {
        if let Err(err) = push_runtime_queue_job(state, &job).await {
            compensate_persisted_runtime_job_on_push_failure(state, &job.job_id).await;
            return Err(err);
        }
    }

    Ok(EnqueueRuntimeJobResult {
        queue_key: state.config.redis.queue_key.clone(),
        job_id: job.job_id,
    })
}

fn validate_enqueue_request(
    payload: &EnqueueRuntimeJobRequest,
) -> Result<(), EnqueueRuntimeJobError> {
    if payload.user_id.trim().is_empty() || payload.kind.trim().is_empty() {
        return Err(EnqueueRuntimeJobError::ValidationFailed);
    }

    Ok(())
}

async fn persist_runtime_job(
    state: &AppState,
    job: &RuntimeQueueJob,
) -> Result<PersistRuntimeJobOutcome, EnqueueRuntimeJobError> {
    let Some(pool) = state.db_pool.as_ref() else {
        error!("runtime job persistence requires configured database pool");
        return Err(EnqueueRuntimeJobError::PersistenceUnavailable);
    };

    let persisted_payload = serde_json::to_value(job).map_err(|err| {
        error!(error = %err, "failed to encode runtime job persistence payload");
        EnqueueRuntimeJobError::EncodeFailed
    })?;

    if let Err(err) =
        insert_runtime_job(pool, &job.job_id, &persisted_payload, job.enqueued_at).await
    {
        match err {
            RepoError::JobAlreadyExists { .. } => {
                return resolve_duplicate_runtime_job(pool, job).await;
            }
            _ => {
                error!(error = %err, "failed to persist runtime job row");
                return Err(EnqueueRuntimeJobError::PersistenceFailure);
            }
        }
    }

    Ok(PersistRuntimeJobOutcome::Inserted)
}

async fn resolve_duplicate_runtime_job(
    pool: &sqlx::PgPool,
    requested_job: &RuntimeQueueJob,
) -> Result<PersistRuntimeJobOutcome, EnqueueRuntimeJobError> {
    let existing_payload =
        sqlx::query_scalar::<_, String>("select payload::text from runtime_jobs where job_id = $1")
            .bind(&requested_job.job_id)
            .fetch_optional(pool)
            .await
            .map_err(|err| {
                error!(
                    error = %err,
                    job_id = %requested_job.job_id,
                    "failed to load existing runtime job row after duplicate insert"
                );
                EnqueueRuntimeJobError::PersistenceFailure
            })?;

    let Some(existing_payload) = existing_payload else {
        error!(
            job_id = %requested_job.job_id,
            "duplicate insert reported but runtime job row is missing"
        );
        return Err(EnqueueRuntimeJobError::PersistenceFailure);
    };

    let existing_job: RuntimeQueueJob = serde_json::from_str(&existing_payload).map_err(|err| {
        error!(
            error = %err,
            job_id = %requested_job.job_id,
            "failed to decode existing runtime job payload"
        );
        EnqueueRuntimeJobError::PersistenceFailure
    })?;

    if existing_job.user_id == requested_job.user_id
        && existing_job.kind == requested_job.kind
        && existing_job.payload == requested_job.payload
    {
        info!(
            job_id = %requested_job.job_id,
            "runtime job already persisted with identical payload; returning idempotent enqueue response"
        );
        return Ok(PersistRuntimeJobOutcome::DuplicateIdempotent);
    }

    info!(
        job_id = %requested_job.job_id,
        "runtime job duplicate detected with mismatched canonical request fields"
    );
    Err(EnqueueRuntimeJobError::DuplicateJobConflict)
}

async fn push_runtime_queue_job(
    state: &AppState,
    job: &RuntimeQueueJob,
) -> Result<(), EnqueueRuntimeJobError> {
    let Some(redis_client) = state.redis_client.as_ref() else {
        return Err(EnqueueRuntimeJobError::RedisUnavailable);
    };

    let encoded = serde_json::to_string(job).map_err(|err| {
        error!(error = %err, "failed to encode runtime queue payload");
        EnqueueRuntimeJobError::EncodeFailed
    })?;

    let mut conn = redis_client
        .get_multiplexed_async_connection()
        .await
        .map_err(|err| {
            error!(error = %err, "failed to connect to redis");
            EnqueueRuntimeJobError::RedisConnectionFailed
        })?;

    let result: redis::RedisResult<usize> = conn
        .rpush(state.config.redis.queue_key.clone(), encoded)
        .await;

    if let Err(err) = result {
        error!(error = %err, "failed to enqueue queue payload");
        return Err(EnqueueRuntimeJobError::QueuePushFailed);
    }

    Ok(())
}

async fn compensate_persisted_runtime_job_on_push_failure(state: &AppState, job_id: &str) {
    let Some(pool) = state.db_pool.as_ref() else {
        return;
    };

    if let Err(err) = sqlx::query("delete from runtime_jobs where job_id = $1")
        .bind(job_id)
        .execute(pool)
        .await
    {
        error!(
            error = %err,
            job_id = %job_id,
            "failed to compensate persisted runtime job row after queue push failure"
        );
    }
}
