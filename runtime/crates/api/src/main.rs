pub(crate) mod http;
pub(crate) mod services {
    pub(crate) mod auth_service;
    pub(crate) mod passkey_service;
    pub(crate) mod runtime_queue_service;
}
mod web;

use std::{net::SocketAddr, sync::Arc, time::Duration};

use anyhow::{Context, Result};
use axum::{
    Json, Router,
    extract::State,
    http::{HeaderMap, StatusCode},
    response::{Html, IntoResponse},
};
use bominal_shared::{
    config::{AppConfig, PasskeyProvider},
    http_client::build_http_client,
    telemetry::init_tracing,
};
use redis::{Client as RedisClient, RedisResult};
use serde::Serialize;
use sqlx::{PgPool, postgres::PgPoolOptions};
use tokio::{net::TcpListener, signal};
use tracing::{error, info};
use uuid::Uuid;
use webauthn_rs::prelude::{Webauthn, WebauthnBuilder};

pub(crate) struct AppState {
    config: AppConfig,
    db_pool: Option<PgPool>,
    redis_client: Option<RedisClient>,
    http_client: reqwest::Client,
    webauthn: Option<Webauthn>,
}

#[derive(Debug, Serialize)]
struct HealthResponse {
    ok: bool,
    service: &'static str,
    db: bool,
    redis: bool,
}

#[derive(Debug, Serialize)]
struct ModulesResponse {
    modules: Vec<ModuleCapability>,
}

#[derive(Debug, Serialize)]
struct ModuleCapability {
    name: &'static str,
    enabled: bool,
    auth_source: &'static str,
}

#[derive(Debug, Serialize)]
struct QueueContractResponse {
    queue_key: String,
    dlq_key: String,
}

fn request_id_from_headers(headers: &HeaderMap) -> String {
    headers
        .get("x-request-id")
        .and_then(|value| value.to_str().ok())
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToOwned::to_owned)
        .unwrap_or_else(|| Uuid::new_v4().to_string())
}

#[tokio::main]
async fn main() -> Result<()> {
    let config = AppConfig::from_env()?;
    let port = config.app_port;
    init_tracing("bominal-api", config.log_json)?;

    let state = Arc::new(build_state(config).await?);
    let router = build_router(state);

    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    let listener = TcpListener::bind(addr).await?;
    info!(listen = %addr, "bominal-api listening");

    axum::serve(listener, router)
        .with_graceful_shutdown(shutdown_signal())
        .await?;

    Ok(())
}

pub(crate) async fn build_state(config: AppConfig) -> Result<AppState> {
    let db_pool = if config.database_url.is_empty() {
        None
    } else {
        Some(
            PgPoolOptions::new()
                .max_connections(10)
                .connect_lazy(&config.database_url)
                .context("failed to create postgres pool")?,
        )
    };

    let redis_client = if config.redis.url.is_empty() {
        None
    } else {
        Some(RedisClient::open(config.redis.url.clone())?)
    };

    let http_client = build_http_client(Duration::from_secs(10))?;
    let webauthn = build_webauthn(&config)?;

    Ok(AppState {
        config,
        db_pool,
        redis_client,
        http_client,
        webauthn,
    })
}

fn build_webauthn(config: &AppConfig) -> Result<Option<Webauthn>> {
    if config.passkey.provider != PasskeyProvider::ServerWebauthn {
        return Ok(None);
    }

    let rp_origin = url::Url::parse(&config.passkey.webauthn_rp_origin)
        .context("WEBAUTHN_RP_ORIGIN must be a valid URL")?;
    let builder = WebauthnBuilder::new(&config.passkey.webauthn_rp_id, &rp_origin)
        .context("invalid WebAuthn relying party configuration")?
        .rp_name(&config.passkey.webauthn_rp_name);
    let webauthn = builder
        .build()
        .context("failed to construct webauthn server state")?;
    Ok(Some(webauthn))
}

pub(crate) fn build_router(state: Arc<AppState>) -> Router {
    http::build_router(state)
}

async fn ssr_home() -> impl IntoResponse {
    let body = web::render_home();
    Html(format!(
        "<!doctype html><html lang=\"en\"><head><meta charset=\"utf-8\" /><meta name=\"viewport\" content=\"width=device-width,initial-scale=1\" /><title>bominal | Rust SSR</title><link rel=\"stylesheet\" href=\"/assets/tailwind.css\" /></head><body class=\"min-h-screen bg-slate-50\">{body}</body></html>"
    ))
}

async fn ssr_auth(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let preflight = web::AuthPreflight {
        database_configured: !state.config.database_url.trim().is_empty(),
        redis_configured: !state.config.redis.url.trim().is_empty(),
        session_secret_configured: !state.config.session_secret.trim().is_empty(),
        invite_base_url_configured: !state.config.invite_base_url.trim().is_empty(),
        passkey_provider_server_only: state.config.passkey.provider
            == PasskeyProvider::ServerWebauthn,
        webauthn_rp_id_configured: !state.config.passkey.webauthn_rp_id.trim().is_empty(),
        webauthn_rp_origin_configured: !state.config.passkey.webauthn_rp_origin.trim().is_empty(),
    };
    let body = web::render_auth(&preflight);

    Html(format!(
        "<!doctype html><html lang=\"en\"><head><meta charset=\"utf-8\" /><meta name=\"viewport\" content=\"width=device-width,initial-scale=1\" /><title>bominal | Auth</title><link rel=\"stylesheet\" href=\"/assets/tailwind.css\" /></head><body class=\"min-h-screen bg-slate-50\">{body}</body></html>"
    ))
}

async fn health_live() -> impl IntoResponse {
    (
        StatusCode::OK,
        Json(HealthResponse {
            ok: true,
            service: "bominal-api",
            db: true,
            redis: true,
        }),
    )
}

async fn health_ready(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let db_ok = db_ready(state.db_pool.as_ref()).await;
    let redis_ok = redis_ready(state.redis_client.as_ref()).await;

    let status = if db_ok && redis_ok {
        StatusCode::OK
    } else {
        StatusCode::SERVICE_UNAVAILABLE
    };

    (
        status,
        Json(HealthResponse {
            ok: db_ok && redis_ok,
            service: "bominal-api",
            db: db_ok,
            redis: redis_ok,
        }),
    )
}

async fn list_modules() -> impl IntoResponse {
    Json(ModulesResponse {
        modules: vec![
            ModuleCapability {
                name: "train",
                enabled: true,
                auth_source: "local",
            },
            ModuleCapability {
                name: "auth",
                enabled: true,
                auth_source: "local",
            },
        ],
    })
}

async fn runtime_queue_contract(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    Json(QueueContractResponse {
        queue_key: state.config.redis.queue_key.clone(),
        dlq_key: state.config.redis.queue_dlq_key.clone(),
    })
}

async fn db_ready(pool: Option<&PgPool>) -> bool {
    let Some(pool) = pool else {
        return false;
    };

    match sqlx::query_scalar::<_, i32>("select 1")
        .fetch_one(pool)
        .await
    {
        Ok(value) => value == 1,
        Err(err) => {
            error!(error = %err, "database readiness probe failed");
            false
        }
    }
}

async fn redis_ready(redis_client: Option<&RedisClient>) -> bool {
    let Some(redis_client) = redis_client else {
        return false;
    };

    match redis_ping(redis_client).await {
        Ok(true) => true,
        Ok(false) => false,
        Err(err) => {
            error!(error = %err, "redis readiness probe failed");
            false
        }
    }
}

async fn redis_ping(redis_client: &RedisClient) -> RedisResult<bool> {
    let mut conn = redis_client.get_multiplexed_async_connection().await?;
    let response: String = redis::cmd("PING").query_async(&mut conn).await?;
    Ok(response.eq_ignore_ascii_case("PONG"))
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

    info!("shutdown signal received");
}
