mod web;

use std::{net::SocketAddr, sync::Arc, time::Duration};

use anyhow::Result;
use axum::{
    Json, Router,
    extract::State,
    http::{HeaderMap, StatusCode},
    response::{Html, IntoResponse},
    routing::{get, post},
};
use bominal_shared::{config::AppConfig, http_client::build_http_client, telemetry::init_tracing};
use redis::{Client as RedisClient, RedisResult};
use serde::{Deserialize, Serialize};
use sqlx::{PgPool, postgres::PgPoolOptions};
use tokio::{net::TcpListener, signal};
use tower_http::{
    compression::CompressionLayer, cors::CorsLayer, services::ServeDir, trace::TraceLayer,
};
use tracing::{error, info};

#[derive(Clone)]
struct AppState {
    config: AppConfig,
    db_pool: Option<PgPool>,
    redis_client: Option<RedisClient>,
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

#[derive(Debug, Deserialize)]
struct SupabaseAuthWebhook {
    #[serde(rename = "type")]
    event_type: String,
    user_id: Option<String>,
    email: Option<String>,
}

#[tokio::main]
async fn main() -> Result<()> {
    let config = AppConfig::from_env()?;
    let port = config.app_port;
    init_tracing("bominal-rust-api", config.log_json)?;

    let state = Arc::new(build_state(config).await?);
    let router = build_router(state);

    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    let listener = TcpListener::bind(addr).await?;
    info!(listen = %addr, "bominal-rust-api listening");

    axum::serve(listener, router)
        .with_graceful_shutdown(shutdown_signal())
        .await?;

    Ok(())
}

async fn build_state(config: AppConfig) -> Result<AppState> {
    let db_pool = if config.database_url.is_empty() {
        None
    } else {
        Some(
            PgPoolOptions::new()
                .max_connections(10)
                .connect_lazy(&config.database_url)?,
        )
    };

    let redis_client = if config.redis.url.is_empty() {
        None
    } else {
        Some(RedisClient::open(config.redis.url.clone())?)
    };

    let _http = build_http_client(Duration::from_secs(10))?;

    Ok(AppState {
        config,
        db_pool,
        redis_client,
    })
}

fn build_router(state: Arc<AppState>) -> Router {
    let assets_dir =
        std::env::var("FRONTEND_ASSETS_DIR").unwrap_or_else(|_| "rust/frontend/dist".to_string());

    Router::new()
        .route("/", get(ssr_home))
        .route("/health/live", get(health_live))
        .route("/health/ready", get(health_ready))
        .route("/api/modules", get(list_modules))
        .route("/api/auth/supabase/webhook", post(supabase_auth_webhook))
        .nest_service("/assets", ServeDir::new(assets_dir))
        .layer(TraceLayer::new_for_http())
        .layer(CompressionLayer::new())
        .layer(CorsLayer::permissive())
        .with_state(state)
}

async fn ssr_home() -> impl IntoResponse {
    let body = web::render_home();
    Html(format!(
        "<!doctype html><html lang=\"en\"><head><meta charset=\"utf-8\" /><meta name=\"viewport\" content=\"width=device-width,initial-scale=1\" /><title>bominal | Rust SSR</title><link rel=\"stylesheet\" href=\"/assets/tailwind.css\" /></head><body class=\"min-h-screen bg-slate-50\">{body}</body></html>"
    ))
}

async fn health_live() -> impl IntoResponse {
    (
        StatusCode::OK,
        Json(HealthResponse {
            ok: true,
            service: "bominal-rust-api",
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
            service: "bominal-rust-api",
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
                auth_source: "supabase",
            },
            ModuleCapability {
                name: "auth",
                enabled: true,
                auth_source: "supabase",
            },
        ],
    })
}

async fn supabase_auth_webhook(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(payload): Json<SupabaseAuthWebhook>,
) -> impl IntoResponse {
    if let Some(expected_secret) = state.config.supabase.auth_webhook_secret.as_deref() {
        let provided = headers
            .get("x-bominal-supabase-webhook-secret")
            .and_then(|value| value.to_str().ok())
            .unwrap_or("");

        if provided != expected_secret {
            return (StatusCode::UNAUTHORIZED, "invalid webhook secret").into_response();
        }
    }

    info!(
        event_type = %payload.event_type,
        user_id = payload.user_id.as_deref().unwrap_or("unknown"),
        has_email = payload.email.is_some(),
        "received supabase auth webhook"
    );

    (StatusCode::ACCEPTED, "ok").into_response()
}

async fn db_ready(pool: Option<&PgPool>) -> bool {
    let Some(pool) = pool else {
        return false;
    };

    match sqlx::query_scalar::<_, i64>("select 1")
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
