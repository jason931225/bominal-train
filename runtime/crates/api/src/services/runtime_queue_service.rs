use std::{future::Future, pin::Pin};

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

type PersistRuntimeJobFuture<'a> = Pin<
    Box<dyn Future<Output = Result<PersistRuntimeJobOutcome, EnqueueRuntimeJobError>> + Send + 'a>,
>;
type PersistRuntimeJobHook =
    for<'a> fn(&'a AppState, &'a RuntimeQueueJob) -> PersistRuntimeJobFuture<'a>;

type PushRuntimeQueueJobFuture<'a> =
    Pin<Box<dyn Future<Output = Result<(), EnqueueRuntimeJobError>> + Send + 'a>>;
type PushRuntimeQueueJobHook =
    for<'a> fn(&'a AppState, &'a RuntimeQueueJob) -> PushRuntimeQueueJobFuture<'a>;

type CompensateRuntimeJobFuture<'a> = Pin<Box<dyn Future<Output = ()> + Send + 'a>>;
type CompensateRuntimeJobHook = for<'a> fn(&'a AppState, &'a str) -> CompensateRuntimeJobFuture<'a>;

#[derive(Clone, Copy)]
struct RuntimeQueueHooks {
    persist_runtime_job: PersistRuntimeJobHook,
    push_runtime_queue_job: PushRuntimeQueueJobHook,
    compensate_persisted_runtime_job: CompensateRuntimeJobHook,
}

impl RuntimeQueueHooks {
    fn live() -> Self {
        Self {
            persist_runtime_job: persist_runtime_job_hook,
            push_runtime_queue_job: push_runtime_queue_job_hook,
            compensate_persisted_runtime_job: compensate_persisted_runtime_job_on_push_failure_hook,
        }
    }
}

fn persist_runtime_job_hook<'a>(
    state: &'a AppState,
    job: &'a RuntimeQueueJob,
) -> PersistRuntimeJobFuture<'a> {
    Box::pin(persist_runtime_job(state, job))
}

fn push_runtime_queue_job_hook<'a>(
    state: &'a AppState,
    job: &'a RuntimeQueueJob,
) -> PushRuntimeQueueJobFuture<'a> {
    Box::pin(push_runtime_queue_job(state, job))
}

fn compensate_persisted_runtime_job_on_push_failure_hook<'a>(
    state: &'a AppState,
    job_id: &'a str,
) -> CompensateRuntimeJobFuture<'a> {
    Box::pin(compensate_persisted_runtime_job_on_push_failure(
        state, job_id,
    ))
}

type RedisConnectAndPushFuture<'a> =
    Pin<Box<dyn Future<Output = Result<(), RedisPushTransportError>> + Send + 'a>>;
type RedisConnectAndPushHook = for<'a> fn(&'a AppState, String) -> RedisConnectAndPushFuture<'a>;

#[derive(Debug)]
enum RedisPushTransportError {
    Connection(redis::RedisError),
    Push(redis::RedisError),
}

#[derive(Clone, Copy)]
struct RuntimeQueueRedisPushHooks {
    connect_and_push: RedisConnectAndPushHook,
}

impl RuntimeQueueRedisPushHooks {
    fn live() -> Self {
        Self {
            connect_and_push: connect_and_push_runtime_queue_live,
        }
    }
}

fn connect_and_push_runtime_queue_live<'a>(
    state: &'a AppState,
    encoded: String,
) -> RedisConnectAndPushFuture<'a> {
    Box::pin(async move {
        let Some(redis_client) = state.redis_client.as_ref() else {
            return Err(RedisPushTransportError::Connection(
                redis::RedisError::from((
                    redis::ErrorKind::InvalidClientConfig,
                    "redis client unavailable",
                )),
            ));
        };

        let mut conn = redis_client
            .get_multiplexed_async_connection()
            .await
            .map_err(RedisPushTransportError::Connection)?;

        conn.rpush(state.config.redis.queue_key.clone(), encoded)
            .await
            .map(|_: usize| ())
            .map_err(RedisPushTransportError::Push)
    })
}

pub(crate) async fn enqueue_runtime_job(
    state: &AppState,
    payload: EnqueueRuntimeJobRequest,
) -> Result<EnqueueRuntimeJobResult, EnqueueRuntimeJobError> {
    enqueue_runtime_job_with_hooks(state, payload, RuntimeQueueHooks::live()).await
}

async fn enqueue_runtime_job_with_hooks(
    state: &AppState,
    payload: EnqueueRuntimeJobRequest,
    hooks: RuntimeQueueHooks,
) -> Result<EnqueueRuntimeJobResult, EnqueueRuntimeJobError> {
    validate_enqueue_request(&payload)?;

    let job = RuntimeQueueJob {
        job_id: payload.job_id.unwrap_or_else(|| Uuid::new_v4().to_string()),
        user_id: payload.user_id,
        kind: payload.kind,
        payload: payload.payload,
        enqueued_at: chrono::Utc::now(),
    };

    let persist_outcome = (hooks.persist_runtime_job)(state, &job).await?;
    if matches!(persist_outcome, PersistRuntimeJobOutcome::Inserted)
        && let Err(err) = (hooks.push_runtime_queue_job)(state, &job).await
    {
        (hooks.compensate_persisted_runtime_job)(state, &job.job_id).await;
        return Err(err);
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
    push_runtime_queue_job_with_hooks(state, job, RuntimeQueueRedisPushHooks::live()).await
}

async fn push_runtime_queue_job_with_hooks(
    state: &AppState,
    job: &RuntimeQueueJob,
    hooks: RuntimeQueueRedisPushHooks,
) -> Result<(), EnqueueRuntimeJobError> {
    if state.redis_client.is_none() {
        return Err(EnqueueRuntimeJobError::RedisUnavailable);
    }

    let encoded = serde_json::to_string(job).map_err(|err| {
        error!(error = %err, "failed to encode runtime queue payload");
        EnqueueRuntimeJobError::EncodeFailed
    })?;

    match (hooks.connect_and_push)(state, encoded).await {
        Ok(()) => Ok(()),
        Err(RedisPushTransportError::Connection(err)) => {
            error!(error = %err, "failed to connect to redis");
            Err(EnqueueRuntimeJobError::RedisConnectionFailed)
        }
        Err(RedisPushTransportError::Push(err)) => {
            error!(error = %err, "failed to enqueue queue payload");
            Err(EnqueueRuntimeJobError::QueuePushFailed)
        }
    }
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

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use bominal_shared::config::{
        AppConfig, EvervaultConfig, PasskeyConfig, PasskeyProvider, RedisConfig, RuntimeSchedule,
    };

    use super::*;

    fn test_state(redis_client: Option<redis::Client>) -> AppState {
        AppState {
            config: AppConfig {
                app_env: "test".to_string(),
                app_host: "127.0.0.1".to_string(),
                app_port: 8080,
                log_json: false,
                session_cookie_name: "bominal_session".to_string(),
                session_ttl_seconds: 3600,
                session_secret: "test-session-secret".to_string(),
                invite_base_url: "http://127.0.0.1:8000".to_string(),
                database_url: String::new(),
                redis: RedisConfig {
                    url: "redis://127.0.0.1:6379".to_string(),
                    queue_key: "runtime:test:queue".to_string(),
                    queue_dlq_key: "runtime:test:queue:dlq".to_string(),
                    lease_prefix: "runtime:test:lease".to_string(),
                    rate_limit_prefix: "runtime:test:rate".to_string(),
                },
                evervault: EvervaultConfig {
                    relay_base_url: "https://relay.example.test".to_string(),
                    app_id: None,
                },
                resend: None,
                passkey: PasskeyConfig {
                    provider: PasskeyProvider::ServerWebauthn,
                    webauthn_rp_id: "localhost".to_string(),
                    webauthn_rp_origin: "http://localhost:8000".to_string(),
                    webauthn_rp_name: "bominal".to_string(),
                    webauthn_challenge_ttl_seconds: 300,
                },
                runtime: RuntimeSchedule {
                    poll_interval: Duration::from_secs(1),
                    reconcile_interval: Duration::from_secs(1),
                    watch_interval: Duration::from_secs(1),
                    key_rotation_interval: Duration::from_secs(1),
                },
            },
            db_pool: None,
            redis_client,
            metrics_handle: super::super::super::init_metrics_recorder()
                .expect("metrics recorder should initialize for tests"),
            http_client: reqwest::Client::new(),
            webauthn: None,
        }
    }

    fn runtime_job_fixture() -> RuntimeQueueJob {
        RuntimeQueueJob {
            job_id: "job-1".to_string(),
            user_id: "user-1".to_string(),
            kind: "train.refresh".to_string(),
            payload: serde_json::json!({"provider": "srt"}),
            enqueued_at: chrono::Utc::now(),
        }
    }

    fn redis_client_fixture() -> redis::Client {
        match redis::Client::open("redis://127.0.0.1:6379") {
            Ok(client) => client,
            Err(err) => panic!("test redis url must parse: {err}"),
        }
    }

    fn enqueue_request_fixture() -> EnqueueRuntimeJobRequest {
        EnqueueRuntimeJobRequest {
            job_id: Some("job-1".to_string()),
            user_id: "user-1".to_string(),
            kind: "train.refresh".to_string(),
            payload: serde_json::json!({"provider": "srt"}),
        }
    }

    fn should_not_persist<'a>(
        _: &'a AppState,
        _: &'a RuntimeQueueJob,
    ) -> PersistRuntimeJobFuture<'a> {
        panic!("persist hook must not be called");
    }

    fn persist_duplicate_conflict<'a>(
        _: &'a AppState,
        _: &'a RuntimeQueueJob,
    ) -> PersistRuntimeJobFuture<'a> {
        Box::pin(async { Err(EnqueueRuntimeJobError::DuplicateJobConflict) })
    }

    fn should_not_push<'a>(
        _: &'a AppState,
        _: &'a RuntimeQueueJob,
    ) -> PushRuntimeQueueJobFuture<'a> {
        panic!("push hook must not be called");
    }

    fn noop_compensate<'a>(_: &'a AppState, _: &'a str) -> CompensateRuntimeJobFuture<'a> {
        Box::pin(async {})
    }

    fn should_not_connect_and_push<'a>(
        _: &'a AppState,
        _: String,
    ) -> RedisConnectAndPushFuture<'a> {
        panic!("connect_and_push hook must not be called");
    }

    fn connection_failure<'a>(_: &'a AppState, _: String) -> RedisConnectAndPushFuture<'a> {
        Box::pin(async {
            Err(RedisPushTransportError::Connection(
                redis::RedisError::from((redis::ErrorKind::Io, "deterministic connection failure")),
            ))
        })
    }

    fn push_failure<'a>(_: &'a AppState, _: String) -> RedisConnectAndPushFuture<'a> {
        Box::pin(async {
            Err(RedisPushTransportError::Push(redis::RedisError::from((
                redis::ErrorKind::Io,
                "deterministic push failure",
            ))))
        })
    }

    #[tokio::test]
    async fn enqueue_runtime_job_rejects_blank_user_id() {
        let state = test_state(None);
        let payload = EnqueueRuntimeJobRequest {
            user_id: "   ".to_string(),
            ..enqueue_request_fixture()
        };

        let result = enqueue_runtime_job_with_hooks(
            &state,
            payload,
            RuntimeQueueHooks {
                persist_runtime_job: should_not_persist,
                push_runtime_queue_job: should_not_push,
                compensate_persisted_runtime_job: noop_compensate,
            },
        )
        .await;

        assert!(matches!(
            result,
            Err(EnqueueRuntimeJobError::ValidationFailed)
        ));
    }

    #[tokio::test]
    async fn enqueue_runtime_job_rejects_blank_kind() {
        let state = test_state(None);
        let payload = EnqueueRuntimeJobRequest {
            kind: "\n\t".to_string(),
            ..enqueue_request_fixture()
        };

        let result = enqueue_runtime_job_with_hooks(
            &state,
            payload,
            RuntimeQueueHooks {
                persist_runtime_job: should_not_persist,
                push_runtime_queue_job: should_not_push,
                compensate_persisted_runtime_job: noop_compensate,
            },
        )
        .await;

        assert!(matches!(
            result,
            Err(EnqueueRuntimeJobError::ValidationFailed)
        ));
    }

    #[tokio::test]
    async fn push_runtime_queue_job_returns_redis_unavailable_without_client() {
        let state = test_state(None);
        let result = push_runtime_queue_job_with_hooks(
            &state,
            &runtime_job_fixture(),
            RuntimeQueueRedisPushHooks {
                connect_and_push: should_not_connect_and_push,
            },
        )
        .await;

        assert!(matches!(
            result,
            Err(EnqueueRuntimeJobError::RedisUnavailable)
        ));
    }

    #[tokio::test]
    async fn push_runtime_queue_job_maps_connection_failure_deterministically() {
        let state = test_state(Some(redis_client_fixture()));

        let result = push_runtime_queue_job_with_hooks(
            &state,
            &runtime_job_fixture(),
            RuntimeQueueRedisPushHooks {
                connect_and_push: connection_failure,
            },
        )
        .await;

        assert!(matches!(
            result,
            Err(EnqueueRuntimeJobError::RedisConnectionFailed)
        ));
    }

    #[tokio::test]
    async fn push_runtime_queue_job_maps_push_failure_deterministically() {
        let state = test_state(Some(redis_client_fixture()));

        let result = push_runtime_queue_job_with_hooks(
            &state,
            &runtime_job_fixture(),
            RuntimeQueueRedisPushHooks {
                connect_and_push: push_failure,
            },
        )
        .await;

        assert!(matches!(
            result,
            Err(EnqueueRuntimeJobError::QueuePushFailed)
        ));
    }

    #[tokio::test]
    async fn enqueue_runtime_job_propagates_duplicate_job_conflict() {
        let state = test_state(None);
        let result = enqueue_runtime_job_with_hooks(
            &state,
            enqueue_request_fixture(),
            RuntimeQueueHooks {
                persist_runtime_job: persist_duplicate_conflict,
                push_runtime_queue_job: should_not_push,
                compensate_persisted_runtime_job: noop_compensate,
            },
        )
        .await;

        assert!(matches!(
            result,
            Err(EnqueueRuntimeJobError::DuplicateJobConflict)
        ));
    }
}
