use std::{sync::Arc, time::Duration};

use axum::{
    Router,
    body::Body,
    http::{Request, StatusCode, header},
};
use bominal_shared::config::{
    AppConfig, EvervaultConfig, PasskeyConfig, PasskeyProvider, RedisConfig, RuntimeSchedule,
};
use tower::util::ServiceExt;
use uuid::Uuid;

#[allow(dead_code)]
#[path = "../src/main.rs"]
mod api_main;

fn test_config() -> AppConfig {
    AppConfig {
        app_env: "test".to_string(),
        app_host: "127.0.0.1".to_string(),
        app_port: 0,
        log_json: false,
        session_cookie_name: "bominal_session".to_string(),
        session_ttl_seconds: 3600,
        session_secret: "test-session-secret".to_string(),
        invite_base_url: "http://127.0.0.1:8000".to_string(),
        database_url: "".to_string(),
        redis: RedisConfig {
            url: "".to_string(),
            queue_key: "test:runtime:queue".to_string(),
            queue_dlq_key: "test:runtime:queue:dlq".to_string(),
            lease_prefix: "test:runtime:lease".to_string(),
            rate_limit_prefix: "test:runtime:rate".to_string(),
        },
        evervault: EvervaultConfig {
            relay_base_url: "https://relay.evervault.com".to_string(),
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
    }
}

async fn build_test_app() -> Router {
    let state = match api_main::build_state(test_config()).await {
        Ok(state) => Arc::new(state),
        Err(err) => panic!("failed to build AppState: {err}"),
    };

    api_main::build_router(state)
}

async fn request_status(app: Router, path: &str) -> StatusCode {
    let request = match Request::builder().uri(path).body(Body::empty()) {
        Ok(request) => request,
        Err(err) => panic!("failed to build request for {path}: {err}"),
    };

    match app.oneshot(request).await {
        Ok(response) => response.status(),
        Err(err) => panic!("request failed for {path}: {err}"),
    }
}

async fn request_response(app: Router, path: &str) -> axum::response::Response {
    let request = match Request::builder().uri(path).body(Body::empty()) {
        Ok(request) => request,
        Err(err) => panic!("failed to build request for {path}: {err}"),
    };

    match app.oneshot(request).await {
        Ok(response) => response,
        Err(err) => panic!("request failed for {path}: {err}"),
    }
}

#[tokio::test]
async fn exposes_health_and_ready_routes() {
    let app = build_test_app().await;

    let live_status = request_status(app.clone(), "/health").await;
    assert_eq!(live_status, StatusCode::OK);

    let ready_status = request_status(app, "/ready").await;
    assert_eq!(ready_status, StatusCode::SERVICE_UNAVAILABLE);
}

#[tokio::test]
async fn legacy_health_routes_are_removed() {
    let app = build_test_app().await;

    let old_live_status = request_status(app.clone(), "/health/live").await;
    assert_ne!(old_live_status, StatusCode::OK);
    assert_ne!(old_live_status, StatusCode::SERVICE_UNAVAILABLE);

    let old_ready_status = request_status(app, "/health/ready").await;
    assert_ne!(old_ready_status, StatusCode::OK);
    assert_ne!(old_ready_status, StatusCode::SERVICE_UNAVAILABLE);
}

#[tokio::test]
async fn health_response_propagates_request_id() {
    let app = build_test_app().await;
    let request = match Request::builder()
        .uri("/health")
        .header("x-request-id", "request-id-contract-test")
        .body(Body::empty())
    {
        Ok(request) => request,
        Err(err) => panic!("failed to build request: {err}"),
    };

    let response = match app.clone().oneshot(request).await {
        Ok(response) => response,
        Err(err) => panic!("request failed: {err}"),
    };
    let echoed = response
        .headers()
        .get("x-request-id")
        .and_then(|value| value.to_str().ok());

    assert_eq!(echoed, Some("request-id-contract-test"));

    let generated_response = request_response(app, "/health").await;
    let generated = generated_response
        .headers()
        .get(header::HeaderName::from_static("x-request-id"))
        .and_then(|value| value.to_str().ok());
    let generated = match generated {
        Some(value) => value,
        None => panic!("missing generated x-request-id header"),
    };

    assert!(Uuid::parse_str(generated).is_ok());
}

#[tokio::test]
async fn admin_maintenance_routes_require_admin_session() {
    let app = build_test_app().await;

    let dashboard_response = request_response(app.clone(), "/admin/maintenance").await;
    assert_eq!(dashboard_response.status(), StatusCode::UNAUTHORIZED);

    let metrics_response = request_response(app.clone(), "/admin/maintenance/metrics").await;
    assert_eq!(metrics_response.status(), StatusCode::UNAUTHORIZED);

    let public_metrics_status = request_status(app, "/metrics").await;
    assert_ne!(public_metrics_status, StatusCode::OK);
}
