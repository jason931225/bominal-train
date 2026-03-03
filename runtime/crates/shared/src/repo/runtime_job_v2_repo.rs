use chrono::{DateTime, Utc};
use serde_json::Value;
use sqlx::{Postgres, postgres::PgArguments, query::Query};

pub const RUNTIME_JOB_V2_INSERT_SQL: &str = "insert into runtime_jobs (job_id, status, payload, attempt_count, next_run_at, idempotency_scope, idempotency_key, max_attempts, created_at, updated_at) values ($1, 'queued', cast($2 as jsonb), 0, $3, $4, $5, $6, $7, $7) returning job_id, status, attempt_count, next_run_at, last_error, processed_at, idempotency_scope, idempotency_key, max_attempts, created_at, updated_at";
pub const RUNTIME_JOB_V2_CLAIM_NEXT_LEASE_SQL: &str = "with candidate as ( select j.job_id from runtime_jobs j left join runtime_job_leases l on l.job_id = j.job_id and l.expires_at > $2 where j.status = 'queued' and (j.next_run_at is null or j.next_run_at <= $2) and l.job_id is null order by j.next_run_at nulls first, j.created_at for update of j skip locked limit 1 ), claimed as ( update runtime_jobs j set status = 'running', attempt_count = j.attempt_count + 1, updated_at = $2 from candidate c where j.job_id = c.job_id returning j.job_id ) insert into runtime_job_leases (job_id, lease_owner, lease_token, acquired_at, expires_at) select c.job_id, $1, $3, $2, $4 from claimed c on conflict (job_id) do update set lease_owner = excluded.lease_owner, lease_token = excluded.lease_token, acquired_at = excluded.acquired_at, expires_at = excluded.expires_at where runtime_job_leases.expires_at <= $2 returning job_id, lease_owner, lease_token, acquired_at, expires_at";
pub const RUNTIME_JOB_V2_HEARTBEAT_LEASE_SQL: &str = "update runtime_job_leases set expires_at = $3 where job_id = $1 and lease_token = $2 and expires_at > $4 returning job_id, lease_owner, lease_token, acquired_at, expires_at";
pub const RUNTIME_JOB_V2_RELEASE_LEASE_SQL: &str =
    "delete from runtime_job_leases where job_id = $1 and lease_token = $2";
pub const RUNTIME_JOB_V2_SCHEDULE_RETRY_SQL: &str = "update runtime_jobs set status = 'queued', next_run_at = $2, last_error = $3, updated_at = $4 where job_id = $1 and status = 'running' returning job_id, status, attempt_count, next_run_at, last_error, processed_at, idempotency_scope, idempotency_key, max_attempts, created_at, updated_at";
pub const RUNTIME_JOB_V2_MARK_TERMINAL_SQL: &str = "update runtime_jobs set status = $2, last_error = $3, processed_at = $4, updated_at = $4 where job_id = $1 and status = 'running' returning job_id, status, attempt_count, next_run_at, last_error, processed_at, idempotency_scope, idempotency_key, max_attempts, created_at, updated_at";
pub const RUNTIME_JOB_V2_INSERT_DEAD_LETTER_SQL: &str = "insert into runtime_job_dead_letters (job_id, dead_letter_key, failure_kind, error_message_redacted, error_context_redacted, payload_redacted, attempt_count, created_at) values ($1, $2, $3, $4, cast($5 as jsonb), cast($6 as jsonb), $7, $8) on conflict (job_id) do update set dead_letter_key = excluded.dead_letter_key, failure_kind = excluded.failure_kind, error_message_redacted = excluded.error_message_redacted, error_context_redacted = excluded.error_context_redacted, payload_redacted = excluded.payload_redacted, attempt_count = excluded.attempt_count, created_at = excluded.created_at returning id, job_id, dead_letter_key, failure_kind, error_message_redacted, error_context_redacted::text, payload_redacted::text, attempt_count, created_at";
pub const RUNTIME_JOB_V2_MARK_DEAD_LETTERED_SQL: &str = "update runtime_jobs set status = 'dead_lettered', last_error = $2, processed_at = $3, updated_at = $3 where job_id = $1 and status in ('running', 'failed') returning job_id, status, attempt_count, next_run_at, last_error, processed_at, idempotency_scope, idempotency_key, max_attempts, created_at, updated_at";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RuntimeJobV2Status {
    Queued,
    Running,
    Completed,
    Failed,
    DeadLettered,
}

impl RuntimeJobV2Status {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Queued => "queued",
            Self::Running => "running",
            Self::Completed => "completed",
            Self::Failed => "failed",
            Self::DeadLettered => "dead_lettered",
        }
    }
}

#[derive(Debug, Clone)]
pub struct InsertRuntimeJobV2Params<'a> {
    pub job_id: &'a str,
    pub payload: &'a Value,
    pub next_run_at: Option<DateTime<Utc>>,
    pub idempotency_scope: Option<&'a str>,
    pub idempotency_key: Option<&'a str>,
    pub max_attempts: i32,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone)]
pub struct ClaimRuntimeJobV2LeaseParams<'a> {
    pub lease_owner: &'a str,
    pub now: DateTime<Utc>,
    pub lease_token: &'a str,
    pub lease_expires_at: DateTime<Utc>,
}

#[derive(Debug, Clone)]
pub struct HeartbeatRuntimeJobV2LeaseParams<'a> {
    pub job_id: &'a str,
    pub lease_token: &'a str,
    pub lease_expires_at: DateTime<Utc>,
    pub now: DateTime<Utc>,
}

#[derive(Debug, Clone)]
pub struct ScheduleRuntimeJobV2RetryParams<'a> {
    pub job_id: &'a str,
    pub next_run_at: DateTime<Utc>,
    pub last_error: &'a str,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone)]
pub struct MarkRuntimeJobV2TerminalParams<'a> {
    pub job_id: &'a str,
    pub status: RuntimeJobV2Status,
    pub last_error: Option<&'a str>,
    pub processed_at: DateTime<Utc>,
}

#[derive(Debug, Clone)]
pub struct InsertRuntimeJobDeadLetterParams<'a> {
    pub job_id: &'a str,
    pub dead_letter_key: Option<&'a str>,
    pub failure_kind: &'a str,
    pub error_message_redacted: Option<&'a str>,
    pub error_context_redacted: &'a Value,
    pub payload_redacted: &'a Value,
    pub attempt_count: i32,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone)]
pub struct MarkRuntimeJobV2DeadLetteredParams<'a> {
    pub job_id: &'a str,
    pub last_error: Option<&'a str>,
    pub processed_at: DateTime<Utc>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct RuntimeJobV2Record {
    pub job_id: String,
    pub status: RuntimeJobV2Status,
    pub attempt_count: i32,
    pub next_run_at: Option<DateTime<Utc>>,
    pub last_error: Option<String>,
    pub processed_at: Option<DateTime<Utc>>,
    pub idempotency_scope: Option<String>,
    pub idempotency_key: Option<String>,
    pub max_attempts: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct RuntimeJobV2LeaseRecord {
    pub job_id: String,
    pub lease_owner: String,
    pub lease_token: String,
    pub acquired_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct RuntimeJobDeadLetterRecord {
    pub id: i64,
    pub job_id: String,
    pub dead_letter_key: Option<String>,
    pub failure_kind: String,
    pub error_message_redacted: Option<String>,
    pub error_context_redacted: Value,
    pub payload_redacted: Value,
    pub attempt_count: i32,
    pub created_at: DateTime<Utc>,
}

pub trait RuntimeJobV2RepoContract {
    fn insert_runtime_job_v2_query<'q>(
        params: &'q InsertRuntimeJobV2Params<'q>,
    ) -> Query<'q, Postgres, PgArguments>;

    fn claim_runtime_job_v2_lease_query<'q>(
        params: &'q ClaimRuntimeJobV2LeaseParams<'q>,
    ) -> Query<'q, Postgres, PgArguments>;

    fn heartbeat_runtime_job_v2_lease_query<'q>(
        params: &'q HeartbeatRuntimeJobV2LeaseParams<'q>,
    ) -> Query<'q, Postgres, PgArguments>;

    fn release_runtime_job_v2_lease_query<'q>(
        job_id: &'q str,
        lease_token: &'q str,
    ) -> Query<'q, Postgres, PgArguments>;

    fn schedule_runtime_job_v2_retry_query<'q>(
        params: &'q ScheduleRuntimeJobV2RetryParams<'q>,
    ) -> Query<'q, Postgres, PgArguments>;

    fn mark_runtime_job_v2_terminal_query<'q>(
        params: &'q MarkRuntimeJobV2TerminalParams<'q>,
    ) -> Query<'q, Postgres, PgArguments>;

    fn insert_runtime_job_dead_letter_query<'q>(
        params: &'q InsertRuntimeJobDeadLetterParams<'q>,
    ) -> Query<'q, Postgres, PgArguments>;

    fn mark_runtime_job_v2_dead_lettered_query<'q>(
        params: &'q MarkRuntimeJobV2DeadLetteredParams<'q>,
    ) -> Query<'q, Postgres, PgArguments>;
}

#[derive(Debug, Clone, Copy, Default)]
pub struct SqlRuntimeJobV2RepoContract;

impl RuntimeJobV2RepoContract for SqlRuntimeJobV2RepoContract {
    fn insert_runtime_job_v2_query<'q>(
        params: &'q InsertRuntimeJobV2Params<'q>,
    ) -> Query<'q, Postgres, PgArguments> {
        sqlx::query(RUNTIME_JOB_V2_INSERT_SQL)
            .bind(params.job_id)
            .bind(params.payload)
            .bind(params.next_run_at)
            .bind(params.idempotency_scope)
            .bind(params.idempotency_key)
            .bind(params.max_attempts)
            .bind(params.created_at)
    }

    fn claim_runtime_job_v2_lease_query<'q>(
        params: &'q ClaimRuntimeJobV2LeaseParams<'q>,
    ) -> Query<'q, Postgres, PgArguments> {
        sqlx::query(RUNTIME_JOB_V2_CLAIM_NEXT_LEASE_SQL)
            .bind(params.lease_owner)
            .bind(params.now)
            .bind(params.lease_token)
            .bind(params.lease_expires_at)
    }

    fn heartbeat_runtime_job_v2_lease_query<'q>(
        params: &'q HeartbeatRuntimeJobV2LeaseParams<'q>,
    ) -> Query<'q, Postgres, PgArguments> {
        sqlx::query(RUNTIME_JOB_V2_HEARTBEAT_LEASE_SQL)
            .bind(params.job_id)
            .bind(params.lease_token)
            .bind(params.lease_expires_at)
            .bind(params.now)
    }

    fn release_runtime_job_v2_lease_query<'q>(
        job_id: &'q str,
        lease_token: &'q str,
    ) -> Query<'q, Postgres, PgArguments> {
        sqlx::query(RUNTIME_JOB_V2_RELEASE_LEASE_SQL)
            .bind(job_id)
            .bind(lease_token)
    }

    fn schedule_runtime_job_v2_retry_query<'q>(
        params: &'q ScheduleRuntimeJobV2RetryParams<'q>,
    ) -> Query<'q, Postgres, PgArguments> {
        sqlx::query(RUNTIME_JOB_V2_SCHEDULE_RETRY_SQL)
            .bind(params.job_id)
            .bind(params.next_run_at)
            .bind(params.last_error)
            .bind(params.updated_at)
    }

    fn mark_runtime_job_v2_terminal_query<'q>(
        params: &'q MarkRuntimeJobV2TerminalParams<'q>,
    ) -> Query<'q, Postgres, PgArguments> {
        sqlx::query(RUNTIME_JOB_V2_MARK_TERMINAL_SQL)
            .bind(params.job_id)
            .bind(params.status.as_str())
            .bind(params.last_error)
            .bind(params.processed_at)
    }

    fn insert_runtime_job_dead_letter_query<'q>(
        params: &'q InsertRuntimeJobDeadLetterParams<'q>,
    ) -> Query<'q, Postgres, PgArguments> {
        sqlx::query(RUNTIME_JOB_V2_INSERT_DEAD_LETTER_SQL)
            .bind(params.job_id)
            .bind(params.dead_letter_key)
            .bind(params.failure_kind)
            .bind(params.error_message_redacted)
            .bind(params.error_context_redacted)
            .bind(params.payload_redacted)
            .bind(params.attempt_count)
            .bind(params.created_at)
    }

    fn mark_runtime_job_v2_dead_lettered_query<'q>(
        params: &'q MarkRuntimeJobV2DeadLetteredParams<'q>,
    ) -> Query<'q, Postgres, PgArguments> {
        sqlx::query(RUNTIME_JOB_V2_MARK_DEAD_LETTERED_SQL)
            .bind(params.job_id)
            .bind(params.last_error)
            .bind(params.processed_at)
    }
}

pub fn insert_runtime_job_v2_query<'q>(
    params: &'q InsertRuntimeJobV2Params<'q>,
) -> Query<'q, Postgres, PgArguments> {
    <SqlRuntimeJobV2RepoContract as RuntimeJobV2RepoContract>::insert_runtime_job_v2_query(params)
}

pub fn claim_runtime_job_v2_lease_query<'q>(
    params: &'q ClaimRuntimeJobV2LeaseParams<'q>,
) -> Query<'q, Postgres, PgArguments> {
    <SqlRuntimeJobV2RepoContract as RuntimeJobV2RepoContract>::claim_runtime_job_v2_lease_query(
        params,
    )
}

pub fn heartbeat_runtime_job_v2_lease_query<'q>(
    params: &'q HeartbeatRuntimeJobV2LeaseParams<'q>,
) -> Query<'q, Postgres, PgArguments> {
    <SqlRuntimeJobV2RepoContract as RuntimeJobV2RepoContract>::heartbeat_runtime_job_v2_lease_query(
        params,
    )
}

pub fn release_runtime_job_v2_lease_query<'q>(
    job_id: &'q str,
    lease_token: &'q str,
) -> Query<'q, Postgres, PgArguments> {
    <SqlRuntimeJobV2RepoContract as RuntimeJobV2RepoContract>::release_runtime_job_v2_lease_query(
        job_id,
        lease_token,
    )
}

pub fn schedule_runtime_job_v2_retry_query<'q>(
    params: &'q ScheduleRuntimeJobV2RetryParams<'q>,
) -> Query<'q, Postgres, PgArguments> {
    <SqlRuntimeJobV2RepoContract as RuntimeJobV2RepoContract>::schedule_runtime_job_v2_retry_query(
        params,
    )
}

pub fn mark_runtime_job_v2_terminal_query<'q>(
    params: &'q MarkRuntimeJobV2TerminalParams<'q>,
) -> Query<'q, Postgres, PgArguments> {
    <SqlRuntimeJobV2RepoContract as RuntimeJobV2RepoContract>::mark_runtime_job_v2_terminal_query(
        params,
    )
}

pub fn insert_runtime_job_dead_letter_query<'q>(
    params: &'q InsertRuntimeJobDeadLetterParams<'q>,
) -> Query<'q, Postgres, PgArguments> {
    <SqlRuntimeJobV2RepoContract as RuntimeJobV2RepoContract>::insert_runtime_job_dead_letter_query(
        params,
    )
}

pub fn mark_runtime_job_v2_dead_lettered_query<'q>(
    params: &'q MarkRuntimeJobV2DeadLetteredParams<'q>,
) -> Query<'q, Postgres, PgArguments> {
    <SqlRuntimeJobV2RepoContract as RuntimeJobV2RepoContract>::mark_runtime_job_v2_dead_lettered_query(params)
}
