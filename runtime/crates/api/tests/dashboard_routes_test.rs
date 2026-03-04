use std::{sync::Arc, time::Duration};

use axum::{
    Router,
    body::to_bytes,
    http::{Request, StatusCode, header},
};
use bominal_shared::config::{
    AppConfig, EvervaultConfig, PasskeyConfig, PasskeyProvider, RedisConfig, RuntimeSchedule,
};
use tower::util::ServiceExt;

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
        session_cookie_domain: None,
        session_ttl_seconds: 3600,
        step_up_ttl_seconds: 600,
        session_secret: "test-session-secret".to_string(),
        invite_base_url: "http://127.0.0.1:8000".to_string(),
        user_app_host: "www.bominal.com".to_string(),
        admin_app_host: "ops.bominal.com".to_string(),
        ui_theme_cookie_name: "bominal_theme".to_string(),
        station_catalog_json_path: "data/train/station_catalog.v1.json".to_string(),
        station_catalog_source_mode: bominal_shared::config::StationCatalogSourceMode::RepoOnly,
        database_url: String::new(),
        redis: RedisConfig {
            url: String::new(),
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

async fn response_json(response: axum::response::Response) -> serde_json::Value {
    let body = match to_bytes(response.into_body(), usize::MAX).await {
        Ok(bytes) => bytes,
        Err(err) => panic!("failed to read response body: {err}"),
    };

    match serde_json::from_slice::<serde_json::Value>(&body) {
        Ok(value) => value,
        Err(err) => panic!("response body is not valid JSON: {err}"),
    }
}

#[tokio::test]
async fn dashboard_page_requires_authenticated_session() {
    let app = build_test_app().await;
    let request = match Request::builder()
        .uri("/dashboard")
        .body(axum::body::Body::empty())
    {
        Ok(request) => request,
        Err(err) => panic!("failed to build request: {err}"),
    };

    let response = match app.oneshot(request).await {
        Ok(response) => response,
        Err(err) => panic!("request failed: {err}"),
    };

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    let body = response_json(response).await;
    assert_eq!(body["code"], "unauthorized");
    assert!(body["request_id"].is_string());
}

#[tokio::test]
async fn dashboard_train_page_requires_authenticated_session() {
    let app = build_test_app().await;
    let request = match Request::builder()
        .uri("/dashboard/train")
        .body(axum::body::Body::empty())
    {
        Ok(request) => request,
        Err(err) => panic!("failed to build request: {err}"),
    };

    let response = match app.oneshot(request).await {
        Ok(response) => response,
        Err(err) => panic!("request failed: {err}"),
    };

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    let body = response_json(response).await;
    assert_eq!(body["code"], "unauthorized");
}

#[tokio::test]
async fn dashboard_provider_security_page_requires_authenticated_session() {
    let app = build_test_app().await;
    let request = match Request::builder()
        .uri("/dashboard/settings/providers")
        .body(axum::body::Body::empty())
    {
        Ok(request) => request,
        Err(err) => panic!("failed to build request: {err}"),
    };

    let response = match app.oneshot(request).await {
        Ok(response) => response,
        Err(err) => panic!("request failed: {err}"),
    };

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    let body = response_json(response).await;
    assert_eq!(body["code"], "unauthorized");
}

#[tokio::test]
async fn dashboard_payment_page_requires_authenticated_session() {
    let app = build_test_app().await;
    let request = match Request::builder()
        .uri("/dashboard/payment")
        .body(axum::body::Body::empty())
    {
        Ok(request) => request,
        Err(err) => panic!("failed to build request: {err}"),
    };

    let response = match app.oneshot(request).await {
        Ok(response) => response,
        Err(err) => panic!("request failed: {err}"),
    };

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    let body = response_json(response).await;
    assert_eq!(body["code"], "unauthorized");
}

#[tokio::test]
async fn dashboard_api_requires_authenticated_session() {
    let app = build_test_app().await;
    let request = match Request::builder()
        .uri("/api/dashboard/summary")
        .header(header::ACCEPT, "application/json")
        .body(axum::body::Body::empty())
    {
        Ok(request) => request,
        Err(err) => panic!("failed to build request: {err}"),
    };

    let response = match app.oneshot(request).await {
        Ok(response) => response,
        Err(err) => panic!("request failed: {err}"),
    };

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    let body = response_json(response).await;
    assert_eq!(body["code"], "unauthorized");
    assert!(body["request_id"].is_string());
}

#[tokio::test]
async fn train_api_preflight_requires_authenticated_session() {
    let app = build_test_app().await;
    let request = match Request::builder()
        .uri("/api/train/preflight")
        .header(header::ACCEPT, "application/json")
        .body(axum::body::Body::empty())
    {
        Ok(request) => request,
        Err(err) => panic!("failed to build request: {err}"),
    };

    let response = match app.oneshot(request).await {
        Ok(response) => response,
        Err(err) => panic!("request failed: {err}"),
    };

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    let body = response_json(response).await;
    assert_eq!(body["code"], "unauthorized");
}

#[tokio::test]
async fn password_change_api_requires_authenticated_session() {
    let app = build_test_app().await;
    let request = match Request::builder()
        .method("POST")
        .uri("/api/auth/password/change")
        .header(header::CONTENT_TYPE, "application/json")
        .header(header::ACCEPT, "application/json")
        .body(axum::body::Body::from(
            r#"{"current_password":"old","new_password":"StrongPass#2026"}"#,
        )) {
        Ok(request) => request,
        Err(err) => panic!("failed to build request: {err}"),
    };

    let response = match app.oneshot(request).await {
        Ok(response) => response,
        Err(err) => panic!("request failed: {err}"),
    };

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    let body = response_json(response).await;
    assert_eq!(body["code"], "unauthorized");
    assert!(body["request_id"].is_string());
}

#[tokio::test]
async fn dashboard_sse_route_requires_authenticated_session() {
    let app = build_test_app().await;
    let request = match Request::builder()
        .uri("/api/dashboard/jobs/job-1/events/stream")
        .header(header::ACCEPT, "text/event-stream")
        .body(axum::body::Body::empty())
    {
        Ok(request) => request,
        Err(err) => panic!("failed to build request: {err}"),
    };

    let response = match app.oneshot(request).await {
        Ok(response) => response,
        Err(err) => panic!("request failed: {err}"),
    };

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    let body = response_json(response).await;
    assert_eq!(body["code"], "unauthorized");
}

#[tokio::test]
async fn root_landing_has_passkey_first_primary_cta() {
    let app = build_test_app().await;
    let request = match Request::builder().uri("/").body(axum::body::Body::empty()) {
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

    assert!(html.contains("Authenticate with passkey"));
    assert!(html.contains("Sign in with email"));
}
