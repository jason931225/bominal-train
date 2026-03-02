use std::sync::Arc;

use anyhow::Result;
use bominal_shared::{config::AppConfig, queue::RuntimeQueueJob, telemetry::init_tracing};
use redis::{AsyncCommands, Client as RedisClient};
use sqlx::{PgPool, postgres::PgPoolOptions};
use tokio::{
    signal,
    sync::watch,
    task::JoinHandle,
    time::{MissedTickBehavior, interval},
};
use tracing::{error, info, warn};

#[derive(Clone)]
struct WorkerState {
    config: Arc<AppConfig>,
    db_pool: Option<PgPool>,
    redis_client: Option<RedisClient>,
}

#[tokio::main]
async fn main() -> Result<()> {
    let config = Arc::new(AppConfig::from_env()?);
    init_tracing("bominal-rust-worker", config.log_json)?;

    let state = Arc::new(build_state(config.clone()).await?);
    let (shutdown_tx, shutdown_rx) = watch::channel(false);

    let mut tasks = vec![
        spawn_loop("poll", state.clone(), shutdown_rx.clone(), poll_loop),
        spawn_loop(
            "reconcile",
            state.clone(),
            shutdown_rx.clone(),
            reconcile_loop,
        ),
        spawn_loop("watch", state.clone(), shutdown_rx.clone(), watch_loop),
        spawn_loop(
            "rotation",
            state.clone(),
            shutdown_rx.clone(),
            rotation_loop,
        ),
    ];

    shutdown_signal().await;
    info!("worker shutdown signal received");
    let _ = shutdown_tx.send(true);

    for task in tasks.drain(..) {
        if let Err(err) = task.await {
            error!(error = %err, "worker task join error");
        }
    }

    info!("worker shutdown complete");
    Ok(())
}

async fn build_state(config: Arc<AppConfig>) -> Result<WorkerState> {
    let db_pool = if config.database_url.is_empty() {
        None
    } else {
        Some(
            PgPoolOptions::new()
                .max_connections(5)
                .connect_lazy(&config.database_url)?,
        )
    };

    let redis_client = if config.redis.url.is_empty() {
        None
    } else {
        Some(RedisClient::open(config.redis.url.clone())?)
    };

    Ok(WorkerState {
        config,
        db_pool,
        redis_client,
    })
}

fn spawn_loop<F, Fut>(
    name: &'static str,
    state: Arc<WorkerState>,
    shutdown_rx: watch::Receiver<bool>,
    run: F,
) -> JoinHandle<()>
where
    F: Fn(Arc<WorkerState>, watch::Receiver<bool>) -> Fut + Send + Sync + 'static,
    Fut: std::future::Future<Output = ()> + Send + 'static,
{
    tokio::spawn(async move {
        info!(loop_name = name, "worker loop started");
        run(state, shutdown_rx).await;
        info!(loop_name = name, "worker loop stopped");
    })
}

async fn poll_loop(state: Arc<WorkerState>, mut shutdown_rx: watch::Receiver<bool>) {
    let mut ticker = interval(state.config.runtime.poll_interval);
    ticker.set_missed_tick_behavior(MissedTickBehavior::Skip);

    loop {
        tokio::select! {
            _ = ticker.tick() => {
                if let Err(err) = poll_queue_once(&state).await {
                    warn!(error = %err, "queue poll failed");
                }
            }
            changed = shutdown_rx.changed() => {
                if changed.is_ok() && *shutdown_rx.borrow() {
                    return;
                }
            }
        }
    }
}

async fn reconcile_loop(state: Arc<WorkerState>, mut shutdown_rx: watch::Receiver<bool>) {
    let mut ticker = interval(state.config.runtime.reconcile_interval);
    ticker.set_missed_tick_behavior(MissedTickBehavior::Skip);

    loop {
        tokio::select! {
            _ = ticker.tick() => {
                if let Some(pool) = state.db_pool.as_ref() {
                    if let Err(err) = sqlx::query_scalar::<_, i64>("select 1").fetch_one(pool).await {
                        warn!(error = %err, "reconcile probe failed");
                    }
                }
            }
            changed = shutdown_rx.changed() => {
                if changed.is_ok() && *shutdown_rx.borrow() {
                    return;
                }
            }
        }
    }
}

async fn watch_loop(state: Arc<WorkerState>, mut shutdown_rx: watch::Receiver<bool>) {
    let mut ticker = interval(state.config.runtime.watch_interval);
    ticker.set_missed_tick_behavior(MissedTickBehavior::Skip);

    loop {
        tokio::select! {
            _ = ticker.tick() => {
                let queue_key = state.config.redis.queue_key.as_str();
                info!(queue_key, "watch loop heartbeat");
            }
            changed = shutdown_rx.changed() => {
                if changed.is_ok() && *shutdown_rx.borrow() {
                    return;
                }
            }
        }
    }
}

async fn rotation_loop(state: Arc<WorkerState>, mut shutdown_rx: watch::Receiver<bool>) {
    let mut ticker = interval(state.config.runtime.key_rotation_interval);
    ticker.set_missed_tick_behavior(MissedTickBehavior::Skip);

    loop {
        tokio::select! {
            _ = ticker.tick() => {
                info!("rotation schedule tick");
            }
            changed = shutdown_rx.changed() => {
                if changed.is_ok() && *shutdown_rx.borrow() {
                    return;
                }
            }
        }
    }
}

async fn poll_queue_once(state: &WorkerState) -> Result<()> {
    let Some(client) = state.redis_client.as_ref() else {
        return Ok(());
    };

    let mut conn = client.get_multiplexed_async_connection().await?;
    let payload: Option<String> = conn.lpop(&state.config.redis.queue_key, None).await?;

    let Some(raw_payload) = payload else {
        return Ok(());
    };

    match serde_json::from_str::<RuntimeQueueJob>(&raw_payload) {
        Ok(job) => {
            info!(
                queue = %state.config.redis.queue_key,
                job_id = %job.job_id,
                user_id = %job.user_id,
                kind = %job.kind,
                "processed runtime queue job"
            );
        }
        Err(err) => {
            warn!(
                error = %err,
                queue = %state.config.redis.queue_key,
                dlq = %state.config.redis.queue_dlq_key,
                "failed to decode runtime queue payload; moving to dlq"
            );
            let _: usize = conn
                .rpush(&state.config.redis.queue_dlq_key, raw_payload)
                .await?;
        }
    }

    Ok(())
}

async fn shutdown_signal() {
    let ctrl_c = async {
        if let Err(err) = signal::ctrl_c().await {
            error!(error = %err, "failed to listen for Ctrl+C");
        }
    };

    #[cfg(unix)]
    let terminate = async {
        use tokio::signal::unix::{SignalKind, signal};

        match signal(SignalKind::terminate()) {
            Ok(mut stream) => {
                let _ = stream.recv().await;
            }
            Err(err) => error!(error = %err, "failed to install SIGTERM handler"),
        }
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {}
        _ = terminate => {}
    }
}
