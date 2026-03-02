mod web;

use std::{net::SocketAddr, sync::Arc, time::Duration};

use anyhow::{Context, Result};
use axum::{
    Json, Router,
    extract::State,
    http::{HeaderMap, StatusCode},
    response::{Html, IntoResponse},
    routing::{get, post},
};
use bominal_shared::{
    config::AppConfig,
    error::{ApiError, ApiErrorCode, ApiErrorStatus},
    http_client::build_http_client,
    queue::RuntimeQueueJob,
    supabase::{Jwks, SupabaseClaims, fetch_jwks, verify_supabase_jwt},
    telemetry::init_tracing,
};
use redis::{AsyncCommands, Client as RedisClient, RedisResult};
use serde::{Deserialize, Serialize};
use sqlx::{PgPool, postgres::PgPoolOptions};
use tokio::{net::TcpListener, signal, sync::RwLock};
use tower_http::{
    compression::CompressionLayer, cors::CorsLayer, services::ServeDir, trace::TraceLayer,
};
use tracing::{error, info, warn};
use uuid::Uuid;

pub(crate) struct AppState {
    config: AppConfig,
    db_pool: Option<PgPool>,
    redis_client: Option<RedisClient>,
    http_client: reqwest::Client,
    jwks_cache: RwLock<Option<JwksCacheEntry>>,
}

struct JwksCacheEntry {
    fetched_at: std::time::Instant,
    jwks: Jwks,
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

#[derive(Debug, Serialize)]
struct VerifySupabaseTokenResponse {
    valid: bool,
    claims: SupabaseClaims,
}

#[derive(Debug, Deserialize)]
struct EnqueueRuntimeJobRequest {
    job_id: Option<String>,
    user_id: String,
    kind: String,
    payload: serde_json::Value,
}

#[derive(Debug, Serialize)]
struct EnqueueRuntimeJobResponse {
    queued: bool,
    queue_key: String,
    job_id: String,
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

    Ok(AppState {
        config,
        db_pool,
        redis_client,
        http_client,
        jwks_cache: RwLock::new(None),
    })
}

pub(crate) fn build_router(state: Arc<AppState>) -> Router {
    let assets_dir =
        std::env::var("FRONTEND_ASSETS_DIR").unwrap_or_else(|_| "rust/frontend/dist".to_string());

    Router::new()
        .route("/", get(ssr_home))
        .route("/health/live", get(health_live))
        .route("/health/ready", get(health_ready))
        .route("/api/modules", get(list_modules))
        .route("/api/auth/supabase/verify", get(verify_supabase_token))
        .route("/api/auth/supabase/webhook", post(supabase_auth_webhook))
        .route("/api/runtime/queue/contract", get(runtime_queue_contract))
        .route("/api/runtime/queue/enqueue", post(enqueue_runtime_job))
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

async fn verify_supabase_token(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
) -> impl IntoResponse {
    let Some(token) = bearer_token(&headers) else {
        return ApiError::new(
            ApiErrorStatus::Unauthorized,
            ApiErrorCode::InvalidRequest,
            "missing bearer token",
            request_id_from_headers(&headers),
        )
        .into_response();
    };

    let jwks = match get_or_refresh_jwks(&state).await {
        Ok(value) => value,
        Err(err) => {
            error!(error = %err, "failed to load supabase jwks");
            return ApiError::new(
                ApiErrorStatus::ServiceUnavailable,
                ApiErrorCode::ServiceUnavailable,
                "jwks unavailable",
                request_id_from_headers(&headers),
            )
            .into_response();
        }
    };

    let claims = match verify_supabase_jwt(
        token,
        &jwks,
        &state.config.supabase.jwt_issuer,
        state.config.supabase.jwt_audience.as_deref(),
    ) {
        Ok(value) => value,
        Err(err) => {
            warn!(error = %err, "supabase token verification failed");
            return ApiError::new(
                ApiErrorStatus::Unauthorized,
                ApiErrorCode::Unauthorized,
                "invalid token",
                request_id_from_headers(&headers),
            )
            .into_response();
        }
    };

    (
        StatusCode::OK,
        Json(VerifySupabaseTokenResponse {
            valid: true,
            claims,
        }),
    )
        .into_response()
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
            return ApiError::new(
                ApiErrorStatus::Unauthorized,
                ApiErrorCode::Unauthorized,
                "invalid webhook secret",
                request_id_from_headers(&headers),
            )
            .into_response();
        }
    }

    if let Some(pool) = state.db_pool.as_ref()
        && let Err(err) = persist_auth_sync(pool, &payload).await
    {
        error!(error = %err, "failed to persist supabase auth sync payload");
        return ApiError::new(
            ApiErrorStatus::InternalServerError,
            ApiErrorCode::InternalError,
            "auth sync persistence failed",
            request_id_from_headers(&headers),
        )
        .into_response();
    }

    info!(
        event_type = %payload.event_type,
        user_id = payload.user_id.as_deref().unwrap_or("unknown"),
        has_email = payload.email.is_some(),
        "received supabase auth webhook"
    );

    (StatusCode::ACCEPTED, "ok").into_response()
}

async fn runtime_queue_contract(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    Json(QueueContractResponse {
        queue_key: state.config.redis.queue_key.clone(),
        dlq_key: state.config.redis.queue_dlq_key.clone(),
    })
}

async fn enqueue_runtime_job(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(payload): Json<EnqueueRuntimeJobRequest>,
) -> impl IntoResponse {
    let Some(redis_client) = state.redis_client.as_ref() else {
        return ApiError::new(
            ApiErrorStatus::ServiceUnavailable,
            ApiErrorCode::ServiceUnavailable,
            "redis unavailable",
            request_id_from_headers(&headers),
        )
        .into_response();
    };

    let job = RuntimeQueueJob {
        job_id: payload.job_id.unwrap_or_else(|| Uuid::new_v4().to_string()),
        user_id: payload.user_id,
        kind: payload.kind,
        payload: payload.payload,
        enqueued_at: chrono::Utc::now(),
    };

    let encoded = match serde_json::to_string(&job) {
        Ok(value) => value,
        Err(err) => {
            error!(error = %err, "failed to encode queue payload");
            return ApiError::new(
                ApiErrorStatus::BadRequest,
                ApiErrorCode::InvalidRequest,
                "invalid payload",
                request_id_from_headers(&headers),
            )
            .with_details(serde_json::json!({"stage": "encode"}))
            .into_response();
        }
    };

    let mut conn = match redis_client.get_multiplexed_async_connection().await {
        Ok(value) => value,
        Err(err) => {
            error!(error = %err, "failed to connect to redis");
            return ApiError::new(
                ApiErrorStatus::ServiceUnavailable,
                ApiErrorCode::ServiceUnavailable,
                "redis connection failed",
                request_id_from_headers(&headers),
            )
            .with_details(serde_json::json!({"stage": "connect"}))
            .into_response();
        }
    };

    let result: redis::RedisResult<usize> = conn
        .rpush(state.config.redis.queue_key.clone(), encoded)
        .await;

    if let Err(err) = result {
        error!(error = %err, "failed to enqueue queue payload");
        return ApiError::new(
            ApiErrorStatus::ServiceUnavailable,
            ApiErrorCode::ServiceUnavailable,
            "queue push failed",
            request_id_from_headers(&headers),
        )
        .with_details(serde_json::json!({"stage": "push"}))
        .into_response();
    }

    (
        StatusCode::ACCEPTED,
        Json(EnqueueRuntimeJobResponse {
            queued: true,
            queue_key: state.config.redis.queue_key.clone(),
            job_id: job.job_id,
        }),
    )
        .into_response()
}

async fn persist_auth_sync(pool: &PgPool, payload: &SupabaseAuthWebhook) -> Result<()> {
    let user_id = payload
        .user_id
        .as_deref()
        .context("supabase webhook payload missing user_id")?;

    sqlx::query(
        "insert into supabase_auth_user_sync (user_id, email, last_event_type, last_synced_at) values ($1, $2, $3, now()) on conflict (user_id) do update set email = excluded.email, last_event_type = excluded.last_event_type, last_synced_at = now()",
    )
    .bind(user_id)
    .bind(payload.email.as_deref())
    .bind(payload.event_type.as_str())
    .execute(pool)
    .await
    .context("failed to upsert supabase auth sync row")?;

    Ok(())
}

async fn get_or_refresh_jwks(state: &Arc<AppState>) -> Result<Jwks> {
    {
        let guard = state.jwks_cache.read().await;
        if let Some(entry) = guard.as_ref()
            && entry.fetched_at.elapsed()
                < Duration::from_secs(state.config.supabase.jwks_cache_seconds)
        {
            return Ok(entry.jwks.clone());
        }
    }

    let jwks = fetch_jwks(&state.http_client, &state.config.supabase.jwks_url).await?;

    {
        let mut guard = state.jwks_cache.write().await;
        *guard = Some(JwksCacheEntry {
            fetched_at: std::time::Instant::now(),
            jwks: jwks.clone(),
        });
    }

    Ok(jwks)
}

fn bearer_token(headers: &HeaderMap) -> Option<&str> {
    let auth_header = headers.get(axum::http::header::AUTHORIZATION)?;
    let raw = auth_header.to_str().ok()?;
    raw.strip_prefix("Bearer ")
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
