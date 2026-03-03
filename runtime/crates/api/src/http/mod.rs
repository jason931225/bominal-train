use std::{
    sync::Arc,
    time::{Duration, Instant},
};

use axum::{
    Router,
    extract::{MatchedPath, Request, State},
    http::{HeaderName, StatusCode},
    middleware::{self, Next},
    response::{IntoResponse, Redirect, Response},
};
use bominal_shared::error::{ApiError, ApiErrorCode, ApiErrorStatus};
use metrics::{counter, histogram};
use tower::{ServiceBuilder, limit::ConcurrencyLimitLayer};
use tower_http::{
    classify::ServerErrorsFailureClass,
    compression::CompressionLayer,
    cors::CorsLayer,
    limit::RequestBodyLimitLayer,
    request_id::{MakeRequestUuid, PropagateRequestIdLayer, SetRequestIdLayer},
    services::ServeDir,
    timeout::TimeoutLayer,
    trace::TraceLayer,
};
use tracing::{Span, info, info_span};

use super::AppState;

mod admin;
mod auth;
mod dashboard;
mod internal;
mod internal_auth;
mod internal_auth_invites;
mod internal_provider_jobs;
mod internal_providers_srt;
mod modules;
mod runtime_queue;

const DEFAULT_HTTP_REQUEST_TIMEOUT_SECONDS: u64 = 30;
const DEFAULT_HTTP_REQUEST_BODY_LIMIT_BYTES: usize = 2 * 1024 * 1024;
const DEFAULT_HTTP_CONCURRENCY_LIMIT: usize = 32;

pub(crate) fn build_router(state: Arc<AppState>) -> Router {
    let assets_dir = std::env::var("FRONTEND_ASSETS_DIR")
        .unwrap_or_else(|_| "runtime/frontend/dist".to_string());
    let request_timeout = Duration::from_secs(parse_u64_env(
        "HTTP_REQUEST_TIMEOUT_SECONDS",
        DEFAULT_HTTP_REQUEST_TIMEOUT_SECONDS,
    ));
    let request_body_limit_bytes = parse_usize_env(
        "HTTP_REQUEST_BODY_LIMIT_BYTES",
        DEFAULT_HTTP_REQUEST_BODY_LIMIT_BYTES,
    );
    let concurrency_limit =
        parse_usize_env("HTTP_CONCURRENCY_LIMIT", DEFAULT_HTTP_CONCURRENCY_LIMIT);

    info!(
        timeout_seconds = request_timeout.as_secs(),
        request_body_limit_bytes, concurrency_limit, "http guardrails configured",
    );

    let router = Router::<Arc<AppState>>::new();
    let router = internal::register(router);
    let router = register_internal_api(router, state.clone());
    let router = modules::register(router);
    let router = auth::register(router);
    let router = dashboard::register(router);
    let router = admin::register(router);
    let router = runtime_queue::register(router);
    let request_id_header = HeaderName::from_static("x-request-id");
    let trace_layer = TraceLayer::new_for_http()
        .make_span_with(|request: &axum::http::Request<axum::body::Body>| {
            let request_id = super::request_id_from_headers(request.headers());
            let route = matched_route(request);
            info_span!(
                "http.request",
                request_id = %request_id,
                method = %request.method(),
                route = %route,
                status = tracing::field::Empty,
                latency_ms = tracing::field::Empty,
            )
        })
        .on_response(
            |response: &axum::http::Response<_>, latency: Duration, span: &Span| {
                let status = response.status().as_u16();
                let latency_ms = latency.as_millis() as u64;
                span.record("status", status);
                span.record("latency_ms", latency_ms);
                info!(
                    parent: span,
                    status,
                    latency_ms,
                    "http request completed",
                );
            },
        )
        .on_failure(
            |failure: ServerErrorsFailureClass, latency: Duration, span: &Span| {
                info!(
                    parent: span,
                    failure = ?failure,
                    latency_ms = latency.as_millis() as u64,
                    "http request failed",
                );
            },
        );

    router
        .nest_service("/assets", ServeDir::new(assets_dir))
        .layer(middleware::from_fn(record_http_metrics))
        .layer(middleware::from_fn_with_state(
            state.clone(),
            enforce_canonical_hosts,
        ))
        .layer(
            ServiceBuilder::new()
                .layer(SetRequestIdLayer::new(
                    request_id_header.clone(),
                    MakeRequestUuid,
                ))
                .layer(PropagateRequestIdLayer::new(request_id_header))
                .layer(trace_layer)
                .layer(RequestBodyLimitLayer::new(request_body_limit_bytes))
                .layer(ConcurrencyLimitLayer::new(concurrency_limit))
                .layer(CompressionLayer::new())
                .layer(CorsLayer::permissive())
                .layer(TimeoutLayer::with_status_code(
                    StatusCode::REQUEST_TIMEOUT,
                    request_timeout,
                )),
        )
        .with_state(state)
}

async fn enforce_canonical_hosts(
    State(state): State<Arc<AppState>>,
    request: Request,
    next: Next,
) -> Response {
    let host = request
        .headers()
        .get("x-forwarded-host")
        .and_then(|value| value.to_str().ok())
        .or_else(|| {
            request
                .headers()
                .get("host")
                .and_then(|value| value.to_str().ok())
        })
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(|value| {
            value
                .split(':')
                .next()
                .unwrap_or(value)
                .to_ascii_lowercase()
        });

    let Some(host) = host else {
        return next.run(request).await;
    };
    if is_local_host(&host) {
        return next.run(request).await;
    }

    let path = request.uri().path();
    let path_and_query = request
        .uri()
        .path_and_query()
        .map(|value| value.as_str())
        .unwrap_or(path);
    let admin_host = state.config.admin_app_host.to_ascii_lowercase();
    let user_host = state.config.user_app_host.to_ascii_lowercase();

    if path.starts_with("/api/admin") && host != admin_host {
        return ApiError::new(
            ApiErrorStatus::Unauthorized,
            ApiErrorCode::Unauthorized,
            "admin api must be requested via admin host",
            super::request_id_from_headers(request.headers()),
        )
        .into_response();
    }
    if path.starts_with("/admin") && host != admin_host {
        return Redirect::permanent(&format!("https://{}{}", admin_host, path_and_query))
            .into_response();
    }
    if matches!(path, "/" | "/auth") || path.starts_with("/dashboard") {
        if host == admin_host {
            return Redirect::permanent(&format!("https://{}{}", user_host, path_and_query))
                .into_response();
        }
    }

    next.run(request).await
}

fn is_local_host(value: &str) -> bool {
    value == "localhost" || value == "127.0.0.1" || value.ends_with(".local") || value == "0.0.0.0"
}

fn register_internal_api(
    router: Router<Arc<AppState>>,
    state: Arc<AppState>,
) -> Router<Arc<AppState>> {
    let mount_aliases = internal_auth::compatibility_aliases_enabled(state.as_ref());

    let internal_router = Router::<Arc<AppState>>::new();
    let internal_router = internal_providers_srt::register(internal_router, mount_aliases);
    let internal_router = internal_provider_jobs::register(internal_router, mount_aliases);
    let internal_router = internal_auth_invites::register(internal_router, mount_aliases);
    let internal_router = internal_router.layer(middleware::from_fn_with_state(
        state,
        internal_auth::require_service_jwt,
    ));

    router.merge(internal_router)
}

async fn record_http_metrics(request: Request, next: Next) -> Response {
    let method = request.method().to_string();
    let route = matched_route(&request);
    let start = Instant::now();
    let response = next.run(request).await;
    let status = response.status();

    counter!(
        "http_requests_total",
        "method" => method.clone(),
        "route" => route.clone(),
        "status_class" => status_class(status).to_string(),
    )
    .increment(1);
    histogram!(
        "http_request_duration_seconds",
        "method" => method.clone(),
        "route" => route.clone(),
    )
    .record(start.elapsed().as_secs_f64());

    if status.is_server_error() {
        counter!(
            "http_errors_total",
            "method" => method,
            "route" => route,
            "error_code" => status.as_u16().to_string(),
        )
        .increment(1);
    }

    response
}

fn matched_route<B>(request: &axum::http::Request<B>) -> String {
    request
        .extensions()
        .get::<MatchedPath>()
        .map(MatchedPath::as_str)
        .unwrap_or_else(|| request.uri().path())
        .to_owned()
}

fn status_class(status: StatusCode) -> &'static str {
    match status.as_u16() {
        100..=199 => "1xx",
        200..=299 => "2xx",
        300..=399 => "3xx",
        400..=499 => "4xx",
        _ => "5xx",
    }
}

fn parse_u64_env(key: &str, default: u64) -> u64 {
    std::env::var(key)
        .ok()
        .and_then(|raw| raw.parse::<u64>().ok())
        .filter(|value| *value > 0)
        .unwrap_or(default)
}

fn parse_usize_env(key: &str, default: usize) -> usize {
    std::env::var(key)
        .ok()
        .and_then(|raw| raw.parse::<usize>().ok())
        .filter(|value| *value > 0)
        .unwrap_or(default)
}
