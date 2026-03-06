use std::time::Duration as StdDuration;

use anyhow::Result;
use bominal_shared::repo::{
    ClaimRuntimeJobV2LeaseParams, HeartbeatRuntimeJobV2LeaseParams,
    claim_runtime_job_v2_lease_query, heartbeat_runtime_job_v2_lease_query,
    release_runtime_job_v2_lease_query,
};
use chrono::{DateTime, Duration, Utc};
use sqlx::{PgPool, Row};
use tokio::{
    sync::watch,
    task::JoinHandle,
    time::{MissedTickBehavior, interval},
};
use tracing::{debug, warn};

#[derive(Debug, Clone)]
pub struct ClaimedLease {
    pub job_id: String,
    pub lease_owner: String,
    pub lease_token: String,
    pub acquired_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct QueueSnapshot {
    pub queued_jobs: i64,
    pub ready_jobs: i64,
    pub oldest_ready_age_seconds: Option<f64>,
}

pub async fn claim_next_job(
    pool: &PgPool,
    lease_owner: &str,
    now: DateTime<Utc>,
    lease_ttl: Duration,
) -> Result<Option<ClaimedLease>> {
    let lease_token = build_lease_token(lease_owner, now);
    let params = ClaimRuntimeJobV2LeaseParams {
        lease_owner,
        now,
        lease_token: lease_token.as_str(),
        lease_expires_at: now + lease_ttl,
    };
    let row = claim_runtime_job_v2_lease_query(&params)
        .fetch_optional(pool)
        .await?;

    let Some(row) = row else {
        return Ok(None);
    };

    Ok(Some(ClaimedLease {
        job_id: row.try_get("job_id")?,
        lease_owner: row.try_get("lease_owner")?,
        lease_token: row.try_get("lease_token")?,
        acquired_at: row.try_get("acquired_at")?,
        expires_at: row.try_get("expires_at")?,
    }))
}

pub async fn heartbeat_lease(
    pool: &PgPool,
    job_id: &str,
    lease_token: &str,
    now: DateTime<Utc>,
    lease_ttl: Duration,
) -> Result<bool> {
    let params = HeartbeatRuntimeJobV2LeaseParams {
        job_id,
        lease_token,
        lease_expires_at: now + lease_ttl,
        now,
    };
    let row = heartbeat_runtime_job_v2_lease_query(&params)
        .fetch_optional(pool)
        .await?;
    Ok(row.is_some())
}

pub async fn release_lease(pool: &PgPool, job_id: &str, lease_token: &str) -> Result<bool> {
    let released = release_runtime_job_v2_lease_query(job_id, lease_token)
        .execute(pool)
        .await?
        .rows_affected();
    Ok(released > 0)
}

const RUNTIME_JOB_QUEUE_SNAPSHOT_SQL: &str = "with ready as ( select coalesce(next_run_at, created_at) as ready_at from runtime_jobs where status = 'queued' and (next_run_at is null or next_run_at <= $1) ) select (select count(*) from runtime_jobs where status = 'queued') as queued_jobs, (select count(*) from ready) as ready_jobs, (select extract(epoch from ($1 - min(ready_at))) from ready) as oldest_ready_age_seconds";

pub async fn queue_snapshot(pool: &PgPool, now: DateTime<Utc>) -> Result<QueueSnapshot> {
    let row = sqlx::query(RUNTIME_JOB_QUEUE_SNAPSHOT_SQL)
        .bind(now)
        .fetch_one(pool)
        .await?;

    Ok(QueueSnapshot {
        queued_jobs: row.try_get("queued_jobs")?,
        ready_jobs: row.try_get("ready_jobs")?,
        oldest_ready_age_seconds: row.try_get("oldest_ready_age_seconds")?,
    })
}

pub struct LeaseHeartbeat {
    stop_tx: watch::Sender<bool>,
    task: JoinHandle<()>,
}

impl LeaseHeartbeat {
    pub fn spawn(
        pool: PgPool,
        job_id: String,
        lease_token: String,
        heartbeat_interval: StdDuration,
        lease_ttl: Duration,
    ) -> Self {
        let (stop_tx, mut stop_rx) = watch::channel(false);
        let task = tokio::spawn(async move {
            let mut ticker = interval(heartbeat_interval);
            ticker.set_missed_tick_behavior(MissedTickBehavior::Skip);

            loop {
                tokio::select! {
                    _ = ticker.tick() => {
                        let now = Utc::now();
                        match heartbeat_lease(&pool, &job_id, &lease_token, now, lease_ttl).await {
                            Ok(true) => {
                                debug!(job_id = %job_id, "runtime job lease heartbeat extended");
                            }
                            Ok(false) => {
                                warn!(job_id = %job_id, "runtime job lease heartbeat lost ownership");
                                return;
                            }
                            Err(err) => {
                                warn!(job_id = %job_id, error = %err, "runtime job lease heartbeat failed");
                                return;
                            }
                        }
                    }
                    changed = stop_rx.changed() => {
                        if changed.is_err() || *stop_rx.borrow() {
                            return;
                        }
                    }
                }
            }
        });

        Self { stop_tx, task }
    }

    pub async fn stop(self) {
        let _ = self.stop_tx.send(true);
        let _ = self.task.await;
    }
}

const RECOVER_EXPIRED_RUNNING_JOBS_SQL: &str = "with expired as ( delete from runtime_job_leases where expires_at <= $1 returning job_id ) update runtime_jobs j set status = 'queued', next_run_at = coalesce(j.next_run_at, $1), updated_at = $1, last_error = case when j.last_error is null then 'lease_expired_requeued' else j.last_error end from expired e where j.job_id = e.job_id and j.status = 'running'";

pub async fn recover_expired_running_jobs(pool: &PgPool, now: DateTime<Utc>) -> Result<u64> {
    let updated = sqlx::query(RECOVER_EXPIRED_RUNNING_JOBS_SQL)
        .bind(now)
        .execute(pool)
        .await?
        .rows_affected();
    Ok(updated)
}

fn build_lease_token(lease_owner: &str, now: DateTime<Utc>) -> String {
    format!(
        "{lease_owner}:{}:{}",
        now.timestamp(),
        now.timestamp_subsec_nanos()
    )
}
