use std::{env, process, time::Duration as StdDuration};

use anyhow::Result;
use bominal_shared::repo::{
    MarkRuntimeJobV2TerminalParams, RuntimeJobV2Status, ScheduleRuntimeJobV2RetryParams,
    mark_runtime_job_v2_terminal_query, schedule_runtime_job_v2_retry_query,
};
use chrono::{DateTime, Duration, Utc};
use serde_json::json;
use sqlx::PgPool;
use tracing::{info, warn};

pub mod dlq;
pub mod executor;
pub mod lease;
pub mod retry;

#[derive(Debug, Clone)]
pub struct RuntimeExecutionConfig {
    pub lease_owner: String,
    pub lease_ttl: Duration,
    pub heartbeat_interval: StdDuration,
    pub retry_policy: retry::RetryPolicy,
    pub payment_policy: executor::PaymentExecutionPolicy,
}

impl RuntimeExecutionConfig {
    pub fn from_env(app_env: &str) -> Self {
        let lease_seconds = env_i64("WORKER_RUNTIME_V2_LEASE_SECONDS", 30).max(5);
        let heartbeat_seconds = env_u64(
            "WORKER_RUNTIME_V2_HEARTBEAT_SECONDS",
            (lease_seconds / 3).max(1) as u64,
        );
        let base_retry_seconds = env_i64("WORKER_RUNTIME_V2_RETRY_BASE_SECONDS", 5).max(1);
        let max_retry_seconds =
            env_i64("WORKER_RUNTIME_V2_RETRY_MAX_SECONDS", 300).max(base_retry_seconds);
        let lease_owner =
            env::var("WORKER_RUNTIME_V2_LEASE_OWNER").unwrap_or_else(|_| default_lease_owner());

        Self {
            lease_owner,
            lease_ttl: Duration::seconds(lease_seconds),
            heartbeat_interval: StdDuration::from_secs(heartbeat_seconds),
            retry_policy: retry::RetryPolicy {
                base_delay: Duration::seconds(base_retry_seconds),
                max_delay: Duration::seconds(max_retry_seconds),
            },
            payment_policy: executor::PaymentExecutionPolicy::from_env(app_env),
        }
    }
}

pub async fn process_next_job(
    pool: &PgPool,
    config: &RuntimeExecutionConfig,
) -> Result<Option<String>> {
    let now = Utc::now();
    let Some(lease) =
        lease::claim_next_job(pool, &config.lease_owner, now, config.lease_ttl).await?
    else {
        return Ok(None);
    };
    let claimed_job_id = lease.job_id.clone();
    info!(
        job_id = %lease.job_id,
        lease_owner = %lease.lease_owner,
        acquired_at = %lease.acquired_at,
        expires_at = %lease.expires_at,
        "claimed runtime v2 job lease"
    );

    let Some(job) = executor::load_claimed_job(pool, lease.job_id.as_str()).await? else {
        warn!(
            job_id = %lease.job_id,
            "claimed runtime job lease but runtime_jobs row is missing"
        );
        if let Err(err) =
            lease::release_lease(pool, lease.job_id.as_str(), lease.lease_token.as_str()).await
        {
            warn!(job_id = %lease.job_id, error = %err, "failed to release missing-job lease");
        }
        return Ok(Some(claimed_job_id));
    };

    let heartbeat = lease::LeaseHeartbeat::spawn(
        pool.clone(),
        lease.job_id.clone(),
        lease.lease_token.clone(),
        config.heartbeat_interval,
        config.lease_ttl,
    );
    let inferred_provider = job.inferred_provider();
    let operation_name = job
        .payload
        .get("operation")
        .and_then(serde_json::Value::as_str)
        .or_else(|| {
            job.persisted_payload
                .get("operation")
                .and_then(serde_json::Value::as_str)
        })
        .unwrap_or(job.kind.as_str())
        .to_string();
    insert_runtime_job_event_best_effort(
        pool,
        job.job_id.as_str(),
        "running",
        json!({
            "provider": inferred_provider,
            "operation": operation_name,
            "attempt_count": job.attempt_count,
            "state": "running",
        }),
    )
    .await;

    let execution = executor::ProviderExecutor
        .execute(pool, &job, &config.payment_policy)
        .await;
    heartbeat.stop().await;

    let transition_result: Result<()> = match execution {
        Ok(success) => {
            let processed_at = Utc::now();
            let params = MarkRuntimeJobV2TerminalParams {
                job_id: job.job_id.as_str(),
                status: RuntimeJobV2Status::Completed,
                last_error: None,
                processed_at,
            };
            let rows = mark_runtime_job_v2_terminal_query(&params)
                .execute(pool)
                .await?
                .rows_affected();
            if rows == 0 {
                info!(
                    job_id = %job.job_id,
                    "runtime job completed transition skipped due idempotent state"
                );
            } else {
                info!(
                    job_id = %job.job_id,
                    provider = %success.provider,
                    operation = %success.operation,
                    result = %success.result_redacted,
                    "runtime job completed"
                );
                insert_runtime_job_event_best_effort(
                    pool,
                    job.job_id.as_str(),
                    "completed",
                    json!({
                        "provider": success.provider,
                        "operation": success.operation,
                        "state": "completed",
                        "result": success.result_redacted,
                    }),
                )
                .await;
            }
            Ok(())
        }
        Err(error) => {
            let now = Utc::now();
            let failure = retry::plan_failure(
                now,
                job.attempt_count,
                job.max_attempts,
                &error.kind,
                &config.retry_policy,
            );

            match failure.action {
                retry::FailureAction::ScheduleRetry { next_run_at } => {
                    let last_error = dlq::redacted_message(&error);
                    let params = ScheduleRuntimeJobV2RetryParams {
                        job_id: job.job_id.as_str(),
                        next_run_at,
                        last_error: last_error.as_str(),
                        updated_at: now,
                    };
                    let rows = schedule_runtime_job_v2_retry_query(&params)
                        .execute(pool)
                        .await?
                        .rows_affected();
                    if rows == 0 {
                        info!(
                            job_id = %job.job_id,
                            "runtime job retry transition skipped due idempotent state"
                        );
                    } else {
                        info!(
                            job_id = %job.job_id,
                            next_run_at = %next_run_at,
                            "runtime job scheduled for retry"
                        );
                        insert_runtime_job_event_best_effort(
                            pool,
                            job.job_id.as_str(),
                            "retry_scheduled",
                            json!({
                                "provider": job.inferred_provider(),
                                "operation": operation_name_for_event(&job),
                                "state": "retry_scheduled",
                                "next_run_at": next_run_at,
                                "error_class": error.kind.as_str(),
                                "message": error.safe_message(),
                            }),
                        )
                        .await;
                    }
                }
                retry::FailureAction::DeadLetter { failure_kind } => {
                    let processed_at = now;
                    let last_error = dlq::redacted_message(&error);
                    let failed_params = MarkRuntimeJobV2TerminalParams {
                        job_id: job.job_id.as_str(),
                        status: RuntimeJobV2Status::Failed,
                        last_error: Some(last_error.as_str()),
                        processed_at,
                    };
                    let _ = mark_runtime_job_v2_terminal_query(&failed_params)
                        .execute(pool)
                        .await?;

                    let dead_lettered =
                        dlq::write_dead_letter(pool, &job, &error, failure_kind, processed_at)
                            .await?;
                    if dead_lettered {
                        info!(
                            job_id = %job.job_id,
                            failure_kind,
                            "runtime job transitioned to dead-letter"
                        );
                        insert_runtime_job_event_best_effort(
                            pool,
                            job.job_id.as_str(),
                            "dead_lettered",
                            json!({
                                "provider": job.inferred_provider(),
                                "operation": operation_name_for_event(&job),
                                "state": "dead_lettered",
                                "failure_kind": failure_kind,
                                "error_class": error.kind.as_str(),
                                "error_reason": error_reason_for_event(&error),
                                "message": error.safe_message(),
                            }),
                        )
                        .await;
                    } else {
                        info!(
                            job_id = %job.job_id,
                            failure_kind,
                            "runtime job dead-letter transition skipped due idempotent state"
                        );
                    }
                }
            }

            Ok(())
        }
    };

    if let Err(err) =
        lease::release_lease(pool, lease.job_id.as_str(), lease.lease_token.as_str()).await
    {
        warn!(
            job_id = %lease.job_id,
            error = %err,
            "failed to release runtime job lease after execution"
        );
    }

    transition_result?;
    Ok(Some(claimed_job_id))
}

pub async fn recover_expired_jobs(pool: &PgPool, now: DateTime<Utc>) -> Result<u64> {
    lease::recover_expired_running_jobs(pool, now).await
}

fn env_i64(key: &str, default: i64) -> i64 {
    env::var(key)
        .ok()
        .and_then(|value| value.parse::<i64>().ok())
        .unwrap_or(default)
}

fn env_u64(key: &str, default: u64) -> u64 {
    env::var(key)
        .ok()
        .and_then(|value| value.parse::<u64>().ok())
        .unwrap_or(default)
}

fn default_lease_owner() -> String {
    let host = env::var("HOSTNAME")
        .ok()
        .filter(|value| !value.trim().is_empty())
        .unwrap_or_else(|| "worker".to_string());
    format!("{}:{}", host, process::id())
}

fn operation_name_for_event(job: &executor::ClaimedRuntimeJob) -> String {
    job.payload
        .get("operation")
        .and_then(serde_json::Value::as_str)
        .or_else(|| {
            job.persisted_payload
                .get("operation")
                .and_then(serde_json::Value::as_str)
        })
        .unwrap_or(job.kind.as_str())
        .to_string()
}

fn error_reason_for_event(error: &executor::ExecutionError) -> String {
    error
        .context
        .get("class")
        .and_then(serde_json::Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .unwrap_or(error.kind.as_str())
        .to_string()
}

async fn insert_runtime_job_event_best_effort(
    pool: &PgPool,
    job_id: &str,
    event_type: &str,
    event_payload: serde_json::Value,
) {
    if let Err(err) = sqlx::query(
        "insert into runtime_job_events (job_id, event_type, event_payload, created_at) values ($1, $2, cast($3 as jsonb), now())",
    )
    .bind(job_id)
    .bind(event_type)
    .bind(event_payload)
    .execute(pool)
    .await
    {
        warn!(
            job_id = %job_id,
            event_type = %event_type,
            error = %err,
            "failed to persist runtime job event",
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn error_reason_for_event_prefers_context_class_over_kind() {
        let error = executor::ExecutionError::new(
            executor::ExecutionErrorKind::Fatal,
            "non-retryable execution failure",
            json!({ "class": "missing_session" }),
        );

        assert_eq!(error_reason_for_event(&error), "missing_session");
    }

    #[test]
    fn error_reason_for_event_falls_back_to_error_kind() {
        let error = executor::ExecutionError::new(
            executor::ExecutionErrorKind::RateLimited,
            "provider rate limited",
            json!({}),
        );

        assert_eq!(error_reason_for_event(&error), "rate_limited");
    }
}
