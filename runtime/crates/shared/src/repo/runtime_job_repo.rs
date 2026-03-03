use chrono::{DateTime, Utc};
use serde_json::Value;
use sqlx::PgPool;
use std::fmt;
use thiserror::Error;

pub const REPO_RUNTIME_JOB_INSERT_SQL: &str = "insert into runtime_jobs (job_id, status, payload, attempt_count, created_at, updated_at) values ($1, 'queued', cast($2 as jsonb), 0, $3, $3) returning job_id, status, attempt_count, payload::text, last_error, processed_at, created_at, updated_at";
pub const REPO_RUNTIME_JOB_TRANSITION_SQL: &str = "update runtime_jobs set status = $2, attempt_count = case when $2 = 'running' then attempt_count + 1 else attempt_count end, last_error = case when $2 = 'failed' then $3 else null end, processed_at = case when $2 in ('completed', 'failed') then $4 else null end, updated_at = $5 where job_id = $1 and status = $6 returning job_id, status, attempt_count, payload::text, last_error, processed_at, created_at, updated_at";
const REPO_RUNTIME_JOB_STATUS_SQL: &str = "select status from runtime_jobs where job_id = $1";

type RuntimeJobRow = (
    String,
    String,
    i32,
    String,
    Option<String>,
    Option<DateTime<Utc>>,
    DateTime<Utc>,
    DateTime<Utc>,
);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RuntimeJobStatus {
    Queued,
    Running,
    Completed,
    Failed,
}

impl RuntimeJobStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Queued => "queued",
            Self::Running => "running",
            Self::Completed => "completed",
            Self::Failed => "failed",
        }
    }

    fn from_db(value: &str) -> Result<Self, RepoError> {
        match value {
            "queued" => Ok(Self::Queued),
            "running" => Ok(Self::Running),
            "completed" => Ok(Self::Completed),
            "failed" => Ok(Self::Failed),
            other => Err(RepoError::UnknownRuntimeJobStatus {
                status: other.to_owned(),
            }),
        }
    }
}

impl fmt::Display for RuntimeJobStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str((*self).as_str())
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RuntimeJobRecord {
    pub job_id: String,
    pub status: RuntimeJobStatus,
    pub attempt_count: i32,
    pub payload: Value,
    pub last_error: Option<String>,
    pub processed_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Error)]
pub enum RepoError {
    #[error("repository query failed: {0}")]
    Sqlx(#[from] sqlx::Error),
    #[error("repository payload decode failed: {0}")]
    PayloadDecode(#[from] serde_json::Error),
    #[error("runtime job `{job_id}` already exists")]
    JobAlreadyExists { job_id: String },
    #[error("runtime job `{job_id}` not found")]
    JobNotFound { job_id: String },
    #[error("invalid runtime job status transition from `{from}` to `{to}`")]
    InvalidStatusTransition {
        from: RuntimeJobStatus,
        to: RuntimeJobStatus,
    },
    #[error("unknown runtime job status `{status}`")]
    UnknownRuntimeJobStatus { status: String },
}

pub async fn insert_runtime_job(
    pool: &PgPool,
    job_id: &str,
    payload: &Value,
    queued_at: DateTime<Utc>,
) -> Result<RuntimeJobRecord, RepoError> {
    let payload_json = serde_json::to_string(payload)?;

    let row = match sqlx::query_as::<_, RuntimeJobRow>(REPO_RUNTIME_JOB_INSERT_SQL)
        .bind(job_id)
        .bind(payload_json)
        .bind(queued_at)
        .fetch_one(pool)
        .await
    {
        Ok(value) => value,
        Err(sqlx::Error::Database(database_error)) if database_error.is_unique_violation() => {
            return Err(RepoError::JobAlreadyExists {
                job_id: job_id.to_owned(),
            });
        }
        Err(err) => return Err(RepoError::Sqlx(err)),
    };

    runtime_job_record_from_row(row)
}

pub async fn transition_runtime_job_status(
    pool: &PgPool,
    job_id: &str,
    next_status: RuntimeJobStatus,
    transitioned_at: DateTime<Utc>,
    last_error: Option<&str>,
) -> Result<RuntimeJobRecord, RepoError> {
    if next_status == RuntimeJobStatus::Queued {
        return transition_error_from_current_state(pool, job_id, next_status).await;
    }

    let required_from = match next_status {
        RuntimeJobStatus::Running => RuntimeJobStatus::Queued,
        RuntimeJobStatus::Completed | RuntimeJobStatus::Failed => RuntimeJobStatus::Running,
        RuntimeJobStatus::Queued => RuntimeJobStatus::Queued,
    };

    let row = sqlx::query_as::<_, RuntimeJobRow>(REPO_RUNTIME_JOB_TRANSITION_SQL)
        .bind(job_id)
        .bind(next_status.as_str())
        .bind(last_error)
        .bind(transitioned_at)
        .bind(transitioned_at)
        .bind(required_from.as_str())
        .fetch_optional(pool)
        .await?;

    if let Some(value) = row {
        return runtime_job_record_from_row(value);
    }

    transition_error_from_current_state(pool, job_id, next_status).await
}

fn runtime_job_record_from_row(row: RuntimeJobRow) -> Result<RuntimeJobRecord, RepoError> {
    Ok(RuntimeJobRecord {
        job_id: row.0,
        status: RuntimeJobStatus::from_db(&row.1)?,
        attempt_count: row.2,
        payload: serde_json::from_str(&row.3)?,
        last_error: row.4,
        processed_at: row.5,
        created_at: row.6,
        updated_at: row.7,
    })
}

async fn transition_error_from_current_state(
    pool: &PgPool,
    job_id: &str,
    next_status: RuntimeJobStatus,
) -> Result<RuntimeJobRecord, RepoError> {
    let current_status = sqlx::query_scalar::<_, String>(REPO_RUNTIME_JOB_STATUS_SQL)
        .bind(job_id)
        .fetch_optional(pool)
        .await?;

    match current_status {
        Some(value) => Err(RepoError::InvalidStatusTransition {
            from: RuntimeJobStatus::from_db(&value)?,
            to: next_status,
        }),
        None => Err(RepoError::JobNotFound {
            job_id: job_id.to_owned(),
        }),
    }
}
