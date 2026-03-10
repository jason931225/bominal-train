use std::{sync::Arc, time::Duration};

use anyhow::Result;
use bominal_shared::{config::AppConfig, telemetry::init_tracing};
use chrono::Utc;
use sqlx::{PgPool, postgres::PgPoolOptions};
use tokio::{
    signal,
    sync::watch,
    task::JoinHandle,
    time::{MissedTickBehavior, interval},
};
use tracing::{error, info, warn};

mod runtime;
mod train_tasks;

#[derive(Clone)]
struct WorkerState {
    config: Arc<AppConfig>,
    runtime_v2: runtime::RuntimeExecutionConfig,
    db_pool: Option<PgPool>,
}

#[tokio::main]
async fn main() -> Result<()> {
    let config = Arc::new(AppConfig::from_env()?);
    init_tracing("bominal-worker", config.log_json)?;

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
        spawn_loop(
            "train_tasks",
            state.clone(),
            shutdown_rx.clone(),
            train_tasks_loop,
        ),
        spawn_loop(
            "scheduler",
            state.clone(),
            shutdown_rx.clone(),
            scheduler_loop,
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

    Ok(WorkerState {
        runtime_v2: runtime::RuntimeExecutionConfig::from_env(config.app_env.as_str()),
        config,
        db_pool,
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
                if let Some(pool) = state.db_pool.as_ref()
                    && let Err(err) = runtime::process_next_job(pool, &state.runtime_v2).await
                {
                    warn!(error = %err, "runtime v2 poll tick failed");
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
                    match runtime::recover_expired_jobs(pool, Utc::now()).await {
                        Ok(recovered) if recovered > 0 => {
                            info!(recovered, "reconcile loop re-queued expired runtime v2 jobs");
                        }
                        Ok(_) => {}
                        Err(err) => {
                            warn!(error = %err, "runtime v2 reconcile tick failed");
                        }
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

async fn train_tasks_loop(state: Arc<WorkerState>, mut shutdown_rx: watch::Receiver<bool>) {
    let mut ticker = interval(Duration::from_millis(1250));
    ticker.set_missed_tick_behavior(MissedTickBehavior::Skip);

    loop {
        tokio::select! {
            _ = ticker.tick() => {
                if let Some(pool) = state.db_pool.as_ref()
                    && let Err(err) = train_tasks::process_due_train_task(pool, &state.runtime_v2).await
                {
                    warn!(error = %err, "train task tick failed");
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

async fn scheduler_loop(state: Arc<WorkerState>, mut shutdown_rx: watch::Receiver<bool>) {
    let mut ticker = interval(Duration::from_secs(30));
    ticker.set_missed_tick_behavior(MissedTickBehavior::Skip);

    loop {
        tokio::select! {
            _ = ticker.tick() => {
                if let Some(pool) = state.db_pool.as_ref()
                    && let Err(err) = train_tasks::process_scheduled_tasks(pool, &state.runtime_v2).await
                {
                    warn!(error = %err, "worker scheduler tick failed");
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
