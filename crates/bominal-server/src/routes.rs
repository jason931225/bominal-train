//! Router setup with all routes and middleware layers.

use std::sync::Arc;
use std::time::Instant;

use axum::Router;
use axum::http::{HeaderValue, Method};
use axum::routing::{delete, get, patch, post};
use tower_http::compression::CompressionLayer;
use tower_http::cors::CorsLayer;
use tower_http::limit::RequestBodyLimitLayer;
use tower_http::request_id::{PropagateRequestIdLayer, SetRequestIdLayer};
use tower_http::services::{ServeDir, ServeFile};
use tower_http::set_header::SetResponseHeaderLayer;
use tower_http::trace::TraceLayer;
use webauthn_rs::WebauthnBuilder;

use crate::auth;
use crate::cards;
use crate::config::AppConfig;
use crate::middleware;
use crate::passkey;
use crate::providers;
use crate::rate_limit::RateLimiter;
use crate::reservations;
use crate::runner;
use crate::search;
use crate::sse;
use crate::state::SharedState;
use crate::tasks;

/// Build the API route tree. Separated from `create_router` so integration
/// tests can mount it directly without the background runner.
pub fn api_routes() -> Router<SharedState> {
    // Rate-limited auth routes (20 req/min per IP)
    let auth_limiter = RateLimiter::new(20);
    auth_limiter.clone().spawn_cleanup();

    let rate_limited_auth = Router::new()
        .route("/auth/register", post(auth::register))
        .route("/auth/login", post(auth::login))
        .route("/auth/forgot-password", post(auth::forgot_password))
        .route("/auth/reset-password", post(auth::reset_password))
        .route("/auth/passkey/login/start", post(passkey::login_start))
        .route("/auth/passkey/login/finish", post(passkey::login_finish))
        .layer(axum::middleware::from_fn(move |req, next| {
            let limiter = auth_limiter.clone();
            crate::rate_limit::rate_limit_middleware(limiter, req, next)
        }));

    // Auth routes that don't need rate limiting (already require session)
    let auth_no_limit = Router::new()
        .route("/auth/logout", post(auth::logout))
        .route("/auth/me", get(auth::me))
        .route("/auth/verify-email", post(auth::verify_email))
        .route("/auth/resend-verification", post(auth::resend_verification))
        .route(
            "/auth/passkey/register/start",
            post(passkey::register_start),
        )
        .route(
            "/auth/passkey/register/finish",
            post(passkey::register_finish),
        );

    // Search endpoints (30 req/min per IP — triggers external provider calls)
    let search_limiter = RateLimiter::new(30);
    search_limiter.clone().spawn_cleanup();

    let rate_limited_search = Router::new()
        .route("/search", post(search::search_trains))
        .layer(axum::middleware::from_fn(move |req, next| {
            let limiter = search_limiter.clone();
            crate::rate_limit::rate_limit_middleware(limiter, req, next)
        }));

    // Payment endpoints (10 req/min per IP)
    let payment_limiter = RateLimiter::new(10);
    payment_limiter.clone().spawn_cleanup();

    let rate_limited_payments = Router::new()
        .route(
            "/reservations/{pnr}/pay",
            post(reservations::pay_reservation),
        )
        .route(
            "/reservations/{pnr}/refund",
            post(reservations::refund_reservation),
        )
        .layer(axum::middleware::from_fn(move |req, next| {
            let limiter = payment_limiter.clone();
            crate::rate_limit::rate_limit_middleware(limiter, req, next)
        }));

    Router::new()
        .merge(rate_limited_auth)
        .merge(auth_no_limit)
        .merge(rate_limited_search)
        .merge(rate_limited_payments)
        // Provider credentials
        .route(
            "/providers",
            get(providers::list_providers).post(providers::add_provider),
        )
        .route("/providers/{provider}", delete(providers::delete_provider))
        // Station suggest
        .route("/stations/{provider}", get(search::list_stations))
        .route(
            "/stations/{provider}/suggest",
            get(search::suggest_stations),
        )
        // Tasks
        .route("/tasks", get(tasks::list_tasks).post(tasks::create_task))
        .route(
            "/tasks/{id}",
            get(tasks::get_task)
                .patch(tasks::update_task)
                .delete(tasks::delete_task),
        )
        // SSE — real-time task updates
        .route("/tasks/events", get(sse::task_events))
        // Reservations (read-only + cancel)
        .route("/reservations", get(reservations::list_reservations))
        .route(
            "/reservations/{pnr}/tickets",
            get(reservations::ticket_detail),
        )
        .route(
            "/reservations/{pnr}/cancel",
            post(reservations::cancel_reservation),
        )
        // Payment cards
        .route("/cards", get(cards::list_cards).post(cards::add_card))
        .route(
            "/cards/{id}",
            patch(cards::update_card).delete(cards::delete_card),
        )
}

/// Build the Axum router with all middleware layers.
pub async fn create_router(
    config: &AppConfig,
    prometheus_handle: metrics_exporter_prometheus::PrometheusHandle,
) -> anyhow::Result<Router> {
    let db = bominal_db::create_pool(&config.database_url).await?;
    let start_time = Instant::now();

    let event_bus = sse::EventBus::new();
    let email = bominal_email::EmailClient::new(&config.resend_api_key, &config.email_from);
    let encryption_key =
        bominal_domain::crypto::encryption::EncryptionKey::from_hex(&config.encryption_key)?;
    let evervault = bominal_domain::evervault::EvervaultConfig::new(
        &config.ev_team_id,
        &config.ev_app_id,
        &config.ev_api_key,
        &config.ev_srt_domain,
        &config.ev_ktx_domain,
    );

    // Build the WebAuthn instance from the app's origin
    let rp_origin = url::Url::parse(&config.app_base_url)
        .map_err(|e| anyhow::anyhow!("Invalid app_base_url: {e}"))?;
    let rp_id = rp_origin
        .host_str()
        .ok_or_else(|| anyhow::anyhow!("app_base_url has no host"))?;
    let mut wa_builder = WebauthnBuilder::new(rp_id, &rp_origin)
        .map_err(|e| anyhow::anyhow!("WebAuthn config error: {e}"))?
        .rp_name("Bominal");
    // Only skip port checks for non-HTTPS (local dev) origins
    if !config.app_base_url.starts_with("https://") {
        wa_builder = wa_builder.allow_any_port(true);
    }
    let webauthn = wa_builder
        .build()
        .map_err(|e| anyhow::anyhow!("WebAuthn build error: {e}"))?;

    let state = SharedState {
        db,
        start_time,
        event_bus,
        email,
        encryption_key,
        evervault,
        app_base_url: config.app_base_url.clone(),
        prometheus_handle,
        webauthn: Arc::new(webauthn),
    };

    // Spawn session cleanup background job
    crate::session_cleanup::spawn_cleanup(state.db.clone());

    // Start the background reservation task runner
    runner::spawn_runner(
        state.db.clone(),
        state.event_bus.clone(),
        state.email.clone(),
        state.encryption_key.clone(),
        state.evervault.clone(),
        config.app_base_url.clone(),
    );

    let api = api_routes();

    // SPA static file serving: SvelteKit build output from `frontend/build/`
    // Falls back to index.html for client-side routing.
    let spa_dir = "frontend/build";
    let spa_fallback = ServeFile::new(format!("{spa_dir}/index.html"));

    let app = Router::new()
        .nest("/api", api)
        .route("/health", get(health_check))
        .route("/metrics", get(metrics_endpoint))
        .fallback_service(
            ServeDir::new(spa_dir)
                .precompressed_br()
                .precompressed_gzip()
                .fallback(spa_fallback),
        )
        .layer(CompressionLayer::new().gzip(true).br(true))
        .layer(RequestBodyLimitLayer::new(1_048_576)) // 1 MB
        .layer(SetResponseHeaderLayer::if_not_present(
            axum::http::header::X_CONTENT_TYPE_OPTIONS,
            HeaderValue::from_static("nosniff"),
        ))
        .layer(SetResponseHeaderLayer::if_not_present(
            axum::http::header::X_FRAME_OPTIONS,
            HeaderValue::from_static("DENY"),
        ))
        .layer(SetResponseHeaderLayer::if_not_present(
            axum::http::header::REFERRER_POLICY,
            HeaderValue::from_static("strict-origin-when-cross-origin"),
        ))
        .layer(SetResponseHeaderLayer::if_not_present(
            axum::http::header::CONTENT_SECURITY_POLICY,
            HeaderValue::from_static(
                "default-src 'self'; script-src 'self' 'unsafe-inline' https://js.evervault.com; style-src 'self' 'unsafe-inline'; img-src 'self' data:; connect-src 'self' https://*.evervault.com",
            ),
        ))
        .layer(
            CorsLayer::new()
                .allow_origin(
                    config
                        .app_base_url
                        .parse::<HeaderValue>()
                        .unwrap_or_else(|_| HeaderValue::from_static("http://localhost:3000")),
                )
                .allow_methods([Method::GET, Method::POST, Method::PATCH, Method::DELETE])
                .allow_credentials(true)
                .allow_headers([axum::http::header::CONTENT_TYPE, axum::http::header::COOKIE]),
        )
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(|request: &axum::http::Request<_>| {
                    let request_id = request
                        .headers()
                        .get("x-request-id")
                        .and_then(|v| v.to_str().ok())
                        .unwrap_or("unknown");

                    tracing::info_span!(
                        "http_request",
                        method = %request.method(),
                        uri = %request.uri(),
                        request_id = %request_id,
                    )
                })
                .on_response(
                    |response: &axum::http::Response<_>,
                     latency: std::time::Duration,
                     _span: &tracing::Span| {
                        tracing::info!(
                            status = %response.status().as_u16(),
                            latency_ms = %latency.as_millis(),
                            "response"
                        );
                    },
                ),
        )
        .layer(PropagateRequestIdLayer::new(middleware::request_id_header()))
        .layer(SetRequestIdLayer::new(
            middleware::request_id_header(),
            middleware::RequestIdGenerator,
        ))
        .with_state(state);

    // Add HSTS header only when running over HTTPS (production)
    let app = if config.app_base_url.starts_with("https://") {
        app.layer(SetResponseHeaderLayer::if_not_present(
            axum::http::header::STRICT_TRANSPORT_SECURITY,
            HeaderValue::from_static("max-age=63072000; includeSubDomains; preload"),
        ))
    } else {
        app
    };

    Ok(app)
}

async fn health_check(
    axum::extract::State(state): axum::extract::State<SharedState>,
) -> axum::Json<serde_json::Value> {
    let uptime = state.start_time.elapsed().as_secs();

    axum::Json(serde_json::json!({
        "status": "ok",
        "version": env!("CARGO_PKG_VERSION"),
        "uptime_seconds": uptime,
    }))
}

async fn metrics_endpoint(
    axum::extract::State(state): axum::extract::State<SharedState>,
) -> String {
    state.prometheus_handle.render()
}
