use std::{sync::Arc, time::Duration};

use axum::{Router, body::to_bytes, http::StatusCode};
use bominal_shared::config::{
    AppConfig, EvervaultConfig, PasskeyConfig, PasskeyProvider, RedisConfig, RuntimeSchedule,
};
use tower::util::ServiceExt;

#[allow(dead_code)]
#[path = "../src/main.rs"]
mod api_main;

fn test_config(
    database_url: &str,
    redis_url: &str,
    session_secret: &str,
    invite_base_url: &str,
    passkey_provider: PasskeyProvider,
    rp_id: &str,
    rp_origin: &str,
) -> AppConfig {
    AppConfig {
        app_env: "test".to_string(),
        app_host: "127.0.0.1".to_string(),
        app_port: 0,
        log_json: false,
        session_cookie_name: "bominal_session".to_string(),
        session_ttl_seconds: 3600,
        session_secret: session_secret.to_string(),
        invite_base_url: invite_base_url.to_string(),
        database_url: database_url.to_string(),
        redis: RedisConfig {
            url: redis_url.to_string(),
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
            provider: passkey_provider,
            webauthn_rp_id: rp_id.to_string(),
            webauthn_rp_origin: rp_origin.to_string(),
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

async fn build_test_app(config: AppConfig) -> Router {
    let state = match api_main::build_state(config).await {
        Ok(state) => Arc::new(state),
        Err(err) => panic!("failed to construct test AppState: {err}"),
    };

    api_main::build_router(state)
}

#[tokio::test]
async fn auth_page_shows_missing_preflight_when_local_auth_not_configured() {
    let app = build_test_app(test_config(
        "",
        "",
        "",
        "",
        PasskeyProvider::ServerWebauthn,
        "localhost",
        "http://localhost:8000",
    ))
    .await;
    let request = match axum::http::Request::builder()
        .method("GET")
        .uri("/auth")
        .body(axum::body::Body::empty())
    {
        Ok(request) => request,
        Err(err) => panic!("failed to build request: {err}"),
    };

    let response = match app.oneshot(request).await {
        Ok(response) => response,
        Err(err) => panic!("request failed: {err}"),
    };

    assert_eq!(response.status(), StatusCode::OK);
    let body = match to_bytes(response.into_body(), usize::MAX).await {
        Ok(body) => body,
        Err(err) => panic!("failed to read response body: {err}"),
    };
    let html = match String::from_utf8(body.to_vec()) {
        Ok(html) => html,
        Err(err) => panic!("response body is not valid utf-8: {err}"),
    };

    assert!(html.contains("Auth Workspace"));
    assert!(html.contains("DATABASE_URL"));
    assert!(html.contains("REDIS_URL"));
    assert!(html.contains("SESSION_SECRET"));
    assert!(html.contains("PASSKEY_PROVIDER=server_webauthn"));
    assert!(html.contains("Missing"));
    assert!(html.contains("Set missing keys in <code>env/local/runtime.env</code>"));
}

#[tokio::test]
async fn auth_page_shows_ready_preflight_when_local_auth_is_configured() {
    let app = build_test_app(test_config(
        "postgresql://localhost:5432/bominal",
        "redis://127.0.0.1:6379",
        "test-session-secret",
        "http://127.0.0.1:8000",
        PasskeyProvider::ServerWebauthn,
        "localhost",
        "http://localhost:8000",
    ))
    .await;
    let request = match axum::http::Request::builder()
        .method("GET")
        .uri("/auth")
        .body(axum::body::Body::empty())
    {
        Ok(request) => request,
        Err(err) => panic!("failed to build request: {err}"),
    };

    let response = match app.oneshot(request).await {
        Ok(response) => response,
        Err(err) => panic!("request failed: {err}"),
    };

    assert_eq!(response.status(), StatusCode::OK);
    let body = match to_bytes(response.into_body(), usize::MAX).await {
        Ok(body) => body,
        Err(err) => panic!("failed to read response body: {err}"),
    };
    let html = match String::from_utf8(body.to_vec()) {
        Ok(html) => html,
        Err(err) => panic!("response body is not valid utf-8: {err}"),
    };

    assert!(html.contains("Auth Workspace"));
    assert!(html.contains("Invite Accept"));
    assert!(html.contains("Password Sign In"));
    assert!(html.contains("Passkey Sign In"));
    assert!(html.contains("Passkey Register"));
    assert!(html.contains("Local auth runtime contract is ready."));
}
