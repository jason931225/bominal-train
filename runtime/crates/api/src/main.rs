pub(crate) mod http;
mod i18n;
pub(crate) mod services;
mod web;

use std::{
    net::SocketAddr,
    sync::{Arc, Mutex, OnceLock},
    time::{Duration, Instant},
};

use anyhow::{Context, Result};
use axum::{
    Json, Router,
    extract::{Path, State},
    http::{
        HeaderMap, StatusCode,
        header::{self, HeaderValue},
    },
    response::{Html, IntoResponse, Redirect},
};
use bominal_shared::{
    config::{AppConfig, PasskeyProvider},
    error::{ApiError, ApiErrorCode, ApiErrorStatus},
    http_client::build_http_client,
    telemetry::init_tracing,
};
use metrics_exporter_prometheus::{PrometheusBuilder, PrometheusHandle};
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
    metrics_handle: PrometheusHandle,
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

fn init_metrics_recorder() -> Result<PrometheusHandle> {
    static PROMETHEUS_HANDLE: OnceLock<Mutex<Option<PrometheusHandle>>> = OnceLock::new();
    let slot = PROMETHEUS_HANDLE.get_or_init(|| Mutex::new(None));
    let mut guard = slot
        .lock()
        .map_err(|_| anyhow::anyhow!("metrics handle lock poisoned"))?;
    if let Some(handle) = guard.as_ref() {
        return Ok(handle.clone());
    }

    let handle = PrometheusBuilder::new()
        .install_recorder()
        .context("failed to install prometheus metrics recorder")?;
    *guard = Some(handle.clone());
    Ok(handle)
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
    let metrics_handle = init_metrics_recorder()?;

    Ok(AppState {
        config,
        db_pool,
        redis_client,
        metrics_handle,
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

async fn favicon_placeholder() -> impl IntoResponse {
    Redirect::permanent("/assets/icons/brand/favicon.svg")
}

async fn ssr_auth_landing(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
) -> impl IntoResponse {
    let request_id = request_id_from_headers(&headers);
    let render_started_at = Instant::now();
    let theme_mode = theme_mode_from_headers(&state.config, &headers);
    let locale = i18n::locale_from_headers(&headers);
    let body = web::render_auth_landing();
    info!(
        request_id = %request_id,
        route = "/",
        render_ms = render_started_at.elapsed().as_millis(),
        "ssr render complete",
    );
    Html(render_document(
        "bominal | Authenticate",
        &body,
        &theme_mode,
        locale,
    ))
}

async fn ssr_auth_alias() -> impl IntoResponse {
    Redirect::permanent("/")
}

async fn ssr_dev_ui(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
) -> axum::response::Response {
    if state.config.app_env.eq_ignore_ascii_case("production") {
        return StatusCode::NOT_FOUND.into_response();
    }

    let request_id = request_id_from_headers(&headers);
    let render_started_at = Instant::now();
    let theme_mode = theme_mode_from_headers(&state.config, &headers);
    let locale = i18n::locale_from_headers(&headers);
    let body = web::render_dev_ui();
    info!(
        request_id = %request_id,
        route = "/dev/ui",
        render_ms = render_started_at.elapsed().as_millis(),
        "ssr render complete",
    );
    Html(render_document("bominal | UI", &body, &theme_mode, locale)).into_response()
}

async fn ssr_dashboard(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
) -> axum::response::Response {
    let request_id = request_id_from_headers(&headers);
    let session =
        match services::auth_service::require_session_state(state.as_ref(), &headers).await {
            Ok(value) => value,
            Err(err) => return map_auth_service_error(err, &request_id).into_response(),
        };
    let body = web::render_dashboard_overview(&session.email);
    let theme_mode = theme_mode_from_headers(&state.config, &headers);
    let locale = i18n::locale_from_headers(&headers);
    Html(render_document(
        "bominal | Dashboard",
        &body,
        &theme_mode,
        locale,
    ))
    .into_response()
}

async fn ssr_dashboard_jobs(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
) -> axum::response::Response {
    let request_id = request_id_from_headers(&headers);
    let session =
        match services::auth_service::require_session_state(state.as_ref(), &headers).await {
            Ok(value) => value,
            Err(err) => return map_auth_service_error(err, &request_id).into_response(),
        };
    let body = web::render_dashboard_jobs(&session.email);
    let theme_mode = theme_mode_from_headers(&state.config, &headers);
    let locale = i18n::locale_from_headers(&headers);
    Html(render_document(
        "bominal | Jobs",
        &body,
        &theme_mode,
        locale,
    ))
    .into_response()
}

async fn ssr_dashboard_job_detail(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Path(job_id): Path<String>,
) -> axum::response::Response {
    let request_id = request_id_from_headers(&headers);
    let session =
        match services::auth_service::require_session_state(state.as_ref(), &headers).await {
            Ok(value) => value,
            Err(err) => return map_auth_service_error(err, &request_id).into_response(),
        };
    let body = web::render_dashboard_job_detail(&session.email, &job_id);
    let theme_mode = theme_mode_from_headers(&state.config, &headers);
    let locale = i18n::locale_from_headers(&headers);
    Html(render_document(
        "bominal | Job Detail",
        &body,
        &theme_mode,
        locale,
    ))
    .into_response()
}

async fn ssr_dashboard_settings(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
) -> axum::response::Response {
    let request_id = request_id_from_headers(&headers);
    let session =
        match services::auth_service::require_session_state(state.as_ref(), &headers).await {
            Ok(value) => value,
            Err(err) => return map_auth_service_error(err, &request_id).into_response(),
        };
    let body = web::render_dashboard_settings(&session.email);
    let theme_mode = theme_mode_from_headers(&state.config, &headers);
    let locale = i18n::locale_from_headers(&headers);
    Html(render_document(
        "bominal | Settings",
        &body,
        &theme_mode,
        locale,
    ))
    .into_response()
}

async fn ssr_dashboard_train(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
) -> axum::response::Response {
    let request_id = request_id_from_headers(&headers);
    let session =
        match services::auth_service::require_session_state(state.as_ref(), &headers).await {
            Ok(value) => value,
            Err(err) => return map_auth_service_error(err, &request_id).into_response(),
        };
    let body = web::render_dashboard_train(&session.email);
    let theme_mode = theme_mode_from_headers(&state.config, &headers);
    let locale = i18n::locale_from_headers(&headers);
    Html(render_document(
        "bominal | Train",
        &body,
        &theme_mode,
        locale,
    ))
    .into_response()
}

async fn ssr_dashboard_settings_providers(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
) -> axum::response::Response {
    let request_id = request_id_from_headers(&headers);
    let session =
        match services::auth_service::require_session_state(state.as_ref(), &headers).await {
            Ok(value) => value,
            Err(err) => return map_auth_service_error(err, &request_id).into_response(),
        };
    let body = web::render_dashboard_settings_providers(&session.email);
    let theme_mode = theme_mode_from_headers(&state.config, &headers);
    let locale = i18n::locale_from_headers(&headers);
    Html(render_document(
        "bominal | Settings · Provider",
        &body,
        &theme_mode,
        locale,
    ))
    .into_response()
}

async fn ssr_dashboard_payment(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
) -> axum::response::Response {
    let request_id = request_id_from_headers(&headers);
    let session =
        match services::auth_service::require_session_state(state.as_ref(), &headers).await {
            Ok(value) => value,
            Err(err) => return map_auth_service_error(err, &request_id).into_response(),
        };
    let body = web::render_dashboard_payment(&session.email);
    let theme_mode = theme_mode_from_headers(&state.config, &headers);
    let locale = i18n::locale_from_headers(&headers);
    Html(render_document(
        "bominal | Settings · Payment",
        &body,
        &theme_mode,
        locale,
    ))
    .into_response()
}

async fn ssr_admin_maintenance(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
) -> axum::response::Response {
    let request_id = request_id_from_headers(&headers);
    let admin_user =
        match services::auth_service::require_admin_session_user(state.as_ref(), &headers).await {
            Ok(user) => user,
            Err(err) => return map_auth_service_error(err, &request_id).into_response(),
        };

    let render_started_at = Instant::now();
    let db_ok = db_ready(state.db_pool.as_ref()).await;
    let redis_ok = redis_ready(state.redis_client.as_ref()).await;
    let ready_ok = db_ok && redis_ok;
    let body = web::render_admin_maintenance(&web::AdminMaintenanceView {
        admin_email: admin_user.email,
        db_ok,
        redis_ok,
        ready_ok,
        health_path: "/health",
        ready_path: "/ready",
        metrics_path: "/admin/maintenance/metrics",
        metrics_snapshot: state.metrics_handle.render(),
    });
    info!(
        request_id = %request_id,
        route = "/admin/maintenance",
        render_ms = render_started_at.elapsed().as_millis(),
        "ssr render complete",
    );

    let theme_mode = theme_mode_from_headers(&state.config, &headers);
    let locale = i18n::locale_from_headers(&headers);
    Html(render_document(
        "bominal | Admin Maintenance",
        &body,
        &theme_mode,
        locale,
    ))
    .into_response()
}

async fn ssr_admin_users(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
) -> axum::response::Response {
    ssr_admin_shell_page(state, headers, "users", "bominal | Admin Users").await
}

async fn ssr_admin_runtime(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
) -> axum::response::Response {
    ssr_admin_shell_page(state, headers, "runtime", "bominal | Admin Runtime").await
}

async fn ssr_admin_observability(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
) -> axum::response::Response {
    ssr_admin_shell_page(
        state,
        headers,
        "observability",
        "bominal | Admin Observability",
    )
    .await
}

async fn ssr_admin_security(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
) -> axum::response::Response {
    ssr_admin_shell_page(state, headers, "security", "bominal | Admin Security").await
}

async fn ssr_admin_config(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
) -> axum::response::Response {
    ssr_admin_shell_page(state, headers, "config", "bominal | Admin Config").await
}

async fn ssr_admin_audit(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
) -> axum::response::Response {
    ssr_admin_shell_page(state, headers, "audit", "bominal | Admin Audit").await
}

async fn ssr_admin_shell_page(
    state: Arc<AppState>,
    headers: HeaderMap,
    section: &'static str,
    title: &'static str,
) -> axum::response::Response {
    let request_id = request_id_from_headers(&headers);
    let admin_user =
        match services::auth_service::require_admin_session_user(state.as_ref(), &headers).await {
            Ok(user) => user,
            Err(err) => return map_auth_service_error(err, &request_id).into_response(),
        };
    let theme_mode = theme_mode_from_headers(&state.config, &headers);
    let body = web::render_admin_section(&admin_user.email, section);
    let locale = i18n::locale_from_headers(&headers);
    Html(render_document(title, &body, &theme_mode, locale)).into_response()
}

async fn admin_maintenance_metrics(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
) -> axum::response::Response {
    let request_id = request_id_from_headers(&headers);
    if let Err(err) =
        services::auth_service::require_admin_session_user(state.as_ref(), &headers).await
    {
        return map_auth_service_error(err, &request_id).into_response();
    }

    let mut response = state.metrics_handle.render().into_response();
    response.headers_mut().insert(
        header::CONTENT_TYPE,
        HeaderValue::from_static("text/plain; version=0.0.4; charset=utf-8"),
    );
    response
}

fn map_auth_service_error(
    error: services::auth_service::AuthServiceError,
    request_id: &str,
) -> ApiError {
    match error {
        services::auth_service::AuthServiceError::InvalidRequest(message)
        | services::auth_service::AuthServiceError::Conflict(message) => ApiError::new(
            ApiErrorStatus::BadRequest,
            ApiErrorCode::InvalidRequest,
            message,
            request_id.to_string(),
        ),
        services::auth_service::AuthServiceError::Unauthorized(message)
        | services::auth_service::AuthServiceError::NotFound(message) => ApiError::new(
            ApiErrorStatus::Unauthorized,
            ApiErrorCode::Unauthorized,
            message,
            request_id.to_string(),
        ),
        services::auth_service::AuthServiceError::ServiceUnavailable(message) => ApiError::new(
            ApiErrorStatus::ServiceUnavailable,
            ApiErrorCode::ServiceUnavailable,
            message,
            request_id.to_string(),
        ),
        services::auth_service::AuthServiceError::Internal => ApiError::new(
            ApiErrorStatus::InternalServerError,
            ApiErrorCode::InternalError,
            "internal error",
            request_id.to_string(),
        ),
    }
}

fn render_document(title: &str, body: &str, theme_mode: &str, locale: i18n::UiLocale) -> String {
    const THEME_TOGGLE_SCRIPT: &str = r#"
<script>
(() => {
  const toggles = Array.from(document.querySelectorAll('[data-theme-toggle]'));
  if (!toggles.length || !document.body) return;

  const getStoredMode = () => document.body.dataset.themeMode === 'dark' ? 'dark' : 'light';
  const setStoredMode = (mode) => {
    const normalized = mode === 'dark' ? 'dark' : 'light';
    document.documentElement.dataset.themeMode = normalized;
    document.body.dataset.themeMode = normalized;
  };

  const setLabels = () => {
    const stored = getStoredMode();
    const label = stored === 'dark' ? 'Theme: Dark' : 'Theme: Light';
    const hint = 'Click to toggle between dark and light.';
    toggles.forEach((button) => {
      const compact = button.hasAttribute('data-theme-toggle-compact');
      if (compact) {
        button.dataset.themeEffective = stored;
      } else {
        button.textContent = label;
      }
      button.title = hint;
      button.setAttribute('aria-label', hint);
    });
  };

  const persistMode = async (mode) => {
    const response = await fetch('/api/ui/theme', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json', 'Accept': 'application/json' },
      body: JSON.stringify({ mode }),
    });
    if (!response.ok) {
      throw new Error('failed_to_persist_theme');
    }
  };

  let persistInFlight = false;
  toggles.forEach((button) => {
    button.addEventListener('click', async (event) => {
      event.preventDefault();
      if (persistInFlight) return;
      const previousMode = getStoredMode();
      const nextMode = previousMode === 'dark' ? 'light' : 'dark';

      setStoredMode(nextMode);
      setLabels();
      persistInFlight = true;
      try {
        await persistMode(nextMode);
        setLabels();
      } catch (_err) {
        setStoredMode(previousMode);
        setLabels();
        if (!button.hasAttribute('data-theme-toggle-compact')) {
          button.textContent = 'Theme unavailable';
        }
      } finally {
        persistInFlight = false;
      }
    });
  });

  setLabels();
})();
</script>
"#;

    let html_lang = locale.as_html_lang();
    let locale_code = locale.as_cookie_value();
    format!(
        "<!doctype html><html lang=\"{html_lang}\" data-theme-mode=\"{theme_mode}\" data-locale=\"{locale_code}\"><head><meta charset=\"utf-8\" /><meta name=\"viewport\" content=\"width=device-width,initial-scale=1\" /><title>{title}</title><link rel=\"preconnect\" href=\"https://fonts.googleapis.com\" /><link rel=\"preconnect\" href=\"https://fonts.gstatic.com\" crossorigin /><link rel=\"stylesheet\" href=\"https://fonts.googleapis.com/css2?family=Noto+Sans+JP:wght@400;500;600;700&family=Noto+Sans+KR:wght@400;500;600;700&display=swap\" /><link rel=\"stylesheet\" href=\"/assets/tailwind.css\" /></head><body class=\"stripe theme\" data-theme-mode=\"{theme_mode}\" data-locale=\"{locale_code}\">{body}{THEME_TOGGLE_SCRIPT}</body></html>"
    )
}

fn theme_mode_from_headers(config: &AppConfig, headers: &HeaderMap) -> String {
    let Some(raw_cookie) = headers
        .get(header::COOKIE)
        .and_then(|value| value.to_str().ok())
    else {
        return "light".to_string();
    };
    let mut selected = "light".to_string();
    for pair in raw_cookie.split(';') {
        let mut parts = pair.trim().splitn(2, '=');
        let Some(key) = parts.next() else {
            continue;
        };
        let Some(value) = parts.next() else {
            continue;
        };
        if key == config.ui_theme_cookie_name {
            let value = value.trim().to_ascii_lowercase();
            if matches!(value.as_str(), "light" | "dark") {
                selected = value;
            }
        }
    }
    selected
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
