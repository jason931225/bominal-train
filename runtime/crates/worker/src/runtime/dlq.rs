use anyhow::Result;
use bominal_shared::{
    crypto::{RedactionMode, redact_json},
    repo::{
        InsertRuntimeJobDeadLetterParams, MarkRuntimeJobV2DeadLetteredParams,
        insert_runtime_job_dead_letter_query, mark_runtime_job_v2_dead_lettered_query,
    },
};
use chrono::{DateTime, Utc};
use serde_json::json;
use sqlx::PgPool;

use super::executor::{ClaimedRuntimeJob, ExecutionError};

pub async fn write_dead_letter(
    pool: &PgPool,
    job: &ClaimedRuntimeJob,
    error: &ExecutionError,
    failure_kind: &str,
    now: DateTime<Utc>,
) -> Result<bool> {
    let dead_letter_key = build_dead_letter_key(&job.job_id, now);
    let error_context = redact_json(
        &json!({
            "error_kind": error.kind.as_str(),
            "message": error.safe_message(),
            "message_detail": error.message,
            "context": error.context,
            "job": {
                "kind": job.kind,
                "provider": job.inferred_provider(),
                "user_id": job.user_id,
                "attempt_count": job.attempt_count,
                "max_attempts": job.max_attempts,
                "idempotency_scope": job.idempotency_scope,
                "idempotency_key": job.idempotency_key,
            }
        }),
        RedactionMode::Mask,
    );
    let payload_redacted = redact_json(&job.persisted_payload, RedactionMode::Mask);
    let error_message = error.safe_message();

    let insert_params = InsertRuntimeJobDeadLetterParams {
        job_id: job.job_id.as_str(),
        dead_letter_key: Some(dead_letter_key.as_str()),
        failure_kind,
        error_message_redacted: Some(error_message),
        error_context_redacted: &error_context,
        payload_redacted: &payload_redacted,
        attempt_count: job.attempt_count,
        created_at: now,
    };
    let inserted = insert_runtime_job_dead_letter_query(&insert_params)
        .execute(pool)
        .await?
        .rows_affected()
        > 0;

    let mark_params = MarkRuntimeJobV2DeadLetteredParams {
        job_id: job.job_id.as_str(),
        last_error: Some(error_message),
        processed_at: now,
    };
    let marked = mark_runtime_job_v2_dead_lettered_query(&mark_params)
        .execute(pool)
        .await?
        .rows_affected()
        > 0;

    Ok(inserted || marked)
}

pub fn redacted_message(error: &ExecutionError) -> String {
    error.safe_message().to_string()
}

fn build_dead_letter_key(job_id: &str, now: DateTime<Utc>) -> String {
    format!("runtime-job-v2:{job_id}:{}", now.timestamp_millis())
}
