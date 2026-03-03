use std::{env, process, time::Duration as StdDuration};

use anyhow::Result;
use bominal_shared::repo::{
    MarkRuntimeJobV2TerminalParams, RuntimeJobV2Status, ScheduleRuntimeJobV2RetryParams,
    mark_runtime_job_v2_terminal_query, schedule_runtime_job_v2_retry_query,
};
use chrono::{DateTime, Duration, Utc};
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
    let execution = executor::ProviderExecutor
        .execute(&job, &config.payment_policy)
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
