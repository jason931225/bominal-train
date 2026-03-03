use bominal_shared::{
    crypto::{RedactionMode, redact_json},
    repo::{InsertRuntimeJobV2Params, insert_runtime_job_v2_query},
};
use chrono::{DateTime, Utc};
use sqlx::PgPool;
use tracing::error;
use uuid::Uuid;

use super::super::AppState;

#[derive(Debug, serde::Deserialize)]
pub(crate) struct CreateProviderJobRequest {
    pub(crate) provider: String,
    pub(crate) operation: String,
    pub(crate) idempotency_key: Option<String>,
    pub(crate) payload: serde_json::Value,
}

#[derive(Debug, serde::Serialize)]
pub(crate) struct CreateProviderJobResult {
    pub(crate) accepted: bool,
    pub(crate) job_id: String,
    pub(crate) status: String,
}

#[derive(Debug, serde::Serialize)]
pub(crate) struct ProviderJobResult {
    pub(crate) job_id: String,
    pub(crate) provider: String,
    pub(crate) operation: String,
    pub(crate) status: String,
}

#[derive(Debug, serde::Serialize)]
pub(crate) struct ProviderJobEvent {
    pub(crate) sequence: i64,
    pub(crate) event_type: String,
    pub(crate) occurred_at: chrono::DateTime<Utc>,
    pub(crate) detail: serde_json::Value,
}

#[derive(Debug)]
pub(crate) enum ProviderJobsError {
    ValidationFailed,
    PersistenceUnavailable,
    DuplicateConflict,
    NotFound,
    PersistenceFailure,
}

pub(crate) async fn create_provider_job(
    state: &AppState,
    payload: CreateProviderJobRequest,
) -> Result<CreateProviderJobResult, ProviderJobsError> {
    validate_create_job_request(&payload)?;

    let Some(pool) = state.db_pool.as_ref() else {
        return Err(ProviderJobsError::PersistenceUnavailable);
    };

    let provider = payload.provider.trim().to_string();
    let operation = payload.operation.trim().to_string();
    let idempotency_key = payload
        .idempotency_key
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToOwned::to_owned);
    let job_id = idempotency_key
        .clone()
        .unwrap_or_else(|| Uuid::new_v4().to_string());
    let idempotency_scope = idempotency_key
        .as_ref()
        .map(|_| format!("provider:{}:{}", provider, operation));
    let now = Utc::now();
    let runtime_payload = serde_json::json!({
        "provider": provider,
        "operation": operation,
        "payload": payload.payload
    });

    let params = InsertRuntimeJobV2Params {
        job_id: job_id.as_str(),
        payload: &runtime_payload,
        next_run_at: Some(now),
        idempotency_scope: idempotency_scope.as_deref(),
        idempotency_key: idempotency_key.as_deref(),
        max_attempts: 5,
        created_at: now,
    };

    let inserted = match insert_runtime_job_v2_query(&params).execute(pool).await {
        Ok(_) => true,
        Err(sqlx::Error::Database(database_error)) if database_error.is_unique_violation() => false,
        Err(err) => {
            error!(error = %err, "failed to insert runtime v2 provider job");
            return Err(ProviderJobsError::PersistenceFailure);
        }
    };

    let status = if inserted {
        let queued_event_detail = redact_json(
            &serde_json::json!({
                "provider": provider,
                "operation": operation,
                "state": "queued"
            }),
            RedactionMode::Mask,
        );

        if let Err(err) =
            insert_provider_job_event(pool, job_id.as_str(), "queued", &queued_event_detail, now)
                .await
        {
            error!(error = ?err, "failed to persist provider job queued event");
            return Err(ProviderJobsError::PersistenceFailure);
        }

        "queued".to_string()
    } else {
        load_runtime_job_status(pool, job_id.as_str()).await?
    };

    Ok(CreateProviderJobResult {
        accepted: true,
        job_id,
        status,
    })
}

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

pub(crate) async fn list_provider_job_events(
    state: &AppState,
    job_id: &str,
) -> Result<Vec<ProviderJobEvent>, ProviderJobsError> {
    validate_job_id(job_id)?;

    let Some(pool) = state.db_pool.as_ref() else {
        return Err(ProviderJobsError::PersistenceUnavailable);
    };

    let rows = sqlx::query_as::<_, (i64, String, DateTime<Utc>, String)>(
        "select id, event_type, created_at, event_payload::text from runtime_job_events where job_id = $1 order by created_at asc, id asc",
    )
    .bind(job_id)
    .fetch_all(pool)
    .await
    .map_err(|err| {
        error!(error = %err, "failed to load runtime job events");
        ProviderJobsError::PersistenceFailure
    })?;

    if rows.is_empty() && !runtime_job_exists(pool, job_id).await? {
        return Err(ProviderJobsError::NotFound);
    }

    let mut events = Vec::with_capacity(rows.len());
    for row in rows {
        let detail_raw = serde_json::from_str::<serde_json::Value>(&row.3).map_err(|err| {
            error!(error = %err, "failed to decode runtime job event payload");
            ProviderJobsError::PersistenceFailure
        })?;
        let detail = redact_json(&detail_raw, RedactionMode::Mask);

        events.push(ProviderJobEvent {
            sequence: row.0,
            event_type: row.1,
            occurred_at: row.2,
            detail,
        });
    }

    Ok(events)
}

fn validate_create_job_request(
    payload: &CreateProviderJobRequest,
) -> Result<(), ProviderJobsError> {
    if payload.provider.trim().is_empty() || payload.operation.trim().is_empty() {
        return Err(ProviderJobsError::ValidationFailed);
    }

    let _ = &payload.payload;

    Ok(())
}

fn validate_job_id(job_id: &str) -> Result<(), ProviderJobsError> {
    if job_id.trim().is_empty() {
        return Err(ProviderJobsError::ValidationFailed);
    }

    Ok(())
}

async fn insert_provider_job_event(
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

async fn load_runtime_job_status(pool: &PgPool, job_id: &str) -> Result<String, ProviderJobsError> {
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

async fn runtime_job_exists(pool: &PgPool, job_id: &str) -> Result<bool, ProviderJobsError> {
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
