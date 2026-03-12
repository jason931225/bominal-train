//! Router setup with all routes and middleware layers.

use std::time::Instant;

use axum::body::Body;
use axum::http::Request;
use axum::response::IntoResponse;
use axum::routing::{delete, get, patch, post};
use axum::Router;
use leptos::prelude::provide_context;
use tower_http::request_id::{PropagateRequestIdLayer, SetRequestIdLayer};
use tower_http::services::ServeFile;
use tower_http::trace::TraceLayer;

use crate::auth;
use crate::cards;
use crate::config::AppConfig;
use crate::middleware;
use crate::providers;
use crate::reservations;
use crate::runner;
use crate::search;
use crate::sse;
use crate::state::SharedState;
use crate::tasks;

/// Build the API route tree. Separated from `create_router` so integration
/// tests can mount it directly without Leptos SSR or the background runner.
pub fn api_routes() -> Router<SharedState> {
    Router::new()
        // Auth
        .route("/auth/register", post(auth::register))
        .route("/auth/login", post(auth::login))
        .route("/auth/logout", post(auth::logout))
        .route("/auth/me", get(auth::me))
        .route("/auth/verify-email", post(auth::verify_email))
        .route("/auth/resend-verification", post(auth::resend_verification))
        .route("/auth/forgot-password", post(auth::forgot_password))
        .route("/auth/reset-password", post(auth::reset_password))
        // Provider credentials
        .route(
            "/providers",
            get(providers::list_providers).post(providers::add_provider),
        )
        .route("/providers/{provider}", delete(providers::delete_provider))
        // Search & station suggest
        .route("/search", post(search::search_trains))
        .route("/stations/{provider}", get(search::list_stations))
        .route(
            "/stations/{provider}/suggest",
            get(search::suggest_stations),
        )
        // Tasks
        .route(
            "/tasks",
            get(tasks::list_tasks).post(tasks::create_task),
        )
        .route(
            "/tasks/{id}",
            get(tasks::get_task)
                .patch(tasks::update_task)
                .delete(tasks::delete_task),
        )
        // SSE — real-time task updates
        .route("/tasks/events", get(sse::task_events))
        // Reservations
        .route("/reservations", get(reservations::list_reservations))
        .route(
            "/reservations/{pnr}/tickets",
            get(reservations::ticket_detail),
        )
        .route(
            "/reservations/{pnr}/cancel",
            post(reservations::cancel_reservation),
        )
        .route(
            "/reservations/{pnr}/pay",
            post(reservations::pay_reservation),
        )
        // Payment cards
        .route(
            "/cards",
            get(cards::list_cards).post(cards::add_card),
        )
        .route(
            "/cards/{id}",
            patch(cards::update_card).delete(cards::delete_card),
        )
}

/// Build the Axum router with all middleware layers.
pub async fn create_router(config: &AppConfig) -> anyhow::Result<Router> {
    let db = bominal_db::create_pool(&config.database_url).await?;
    let start_time = Instant::now();

    let event_bus = sse::EventBus::new();
    let email = bominal_email::EmailClient::new(&config.resend_api_key, &config.email_from);
    let encryption_key = bominal_domain::crypto::encryption::EncryptionKey::from_hex(
        &config.encryption_key,
    )?;

    // Start the background reservation task runner
    runner::spawn_runner(
        db.clone(),
        event_bus.clone(),
        email.clone(),
        encryption_key.clone(),
        config.app_base_url.clone(),
    );

    let state = SharedState {
        db,
        start_time,
        event_bus,
        email,
        encryption_key,
        app_base_url: config.app_base_url.clone(),
    };

    let api = api_routes();

    // Context provider for Leptos server functions and SSR rendering
    let sfn_db = state.db.clone();
    let sfn_key = state.encryption_key.clone();
    let sfn_email = state.email.clone();
    let sfn_base_url = config.app_base_url.clone();
    let context_fn = move || {
        provide_context(sfn_db.clone());
        provide_context(sfn_key.clone());
        provide_context(sfn_email.clone());
        provide_context(bominal_frontend::api::auth::AppBaseUrl(
            sfn_base_url.clone(),
        ));
    };

    // Server function handler (POST /sfn/*)
    let sfn_context = context_fn.clone();
    let server_fn_handler = move |req: Request<Body>| {
        let ctx = sfn_context.clone();
        async move {
            leptos_axum::handle_server_fns_with_context(ctx, req)
                .await
                .into_response()
        }
    };

    // Leptos SSR page renderer (fallback for all non-API routes)
    let page_renderer = leptos_axum::render_app_to_stream_with_context(
        context_fn,
        bominal_frontend::app::shell,
    );

    let app = Router::new()
        .nest("/api", api)
        .route("/sfn/{*fn_name}", post(server_fn_handler))
        .route_service(
            "/style.css",
            ServeFile::new("crates/bominal-frontend/style/main.css"),
        )
        .route("/health", get(health_check))
        .route("/metrics", get(metrics_endpoint))
        .fallback(page_renderer)
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
        .layer(PropagateRequestIdLayer::new(
            middleware::request_id_header(),
        ))
        .layer(SetRequestIdLayer::new(
            middleware::request_id_header(),
            middleware::RequestIdGenerator,
        ))
        .with_state(state);

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

async fn metrics_endpoint() -> String {
    // Prometheus metrics will be collected here
    String::new()
}
