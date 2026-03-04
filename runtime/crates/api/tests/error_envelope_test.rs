use std::{sync::Arc, time::Duration};

use axum::{
    Router,
    body::to_bytes,
    http::{StatusCode, header, request::Builder as RequestBuilder},
};
use bominal_shared::config::{
    AppConfig, EvervaultConfig, PasskeyConfig, PasskeyProvider, RedisConfig, RuntimeSchedule,
};
use tower::util::ServiceExt;

#[allow(dead_code)]
#[path = "../src/main.rs"]
mod api_main;

fn test_config(redis_url: &str, database_url: &str) -> AppConfig {
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

async fn build_test_app(redis_url: &str, database_url: &str) -> Router {
    let state = match api_main::build_state(test_config(redis_url, database_url)).await {
        Ok(state) => Arc::new(state),
        Err(err) => panic!("failed to construct test AppState: {err}"),
    };

    api_main::build_router(state)
}

fn request_builder(method: &str, path: &str) -> RequestBuilder {
    axum::http::Request::builder()
        .method(method)
        .uri(path)
        .header(header::CONTENT_TYPE, "application/json")
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

fn assert_envelope_fields(body: &serde_json::Value, expected_code: &str) {
    assert_eq!(body["code"], expected_code);
    assert!(body["message"].is_string());
    assert!(body["request_id"].is_string());
}

#[tokio::test]
async fn error_envelope_password_signin_missing_fields() {
    let app = build_test_app("redis://127.0.0.1:6379", "").await;

    let request = match request_builder("POST", "/api/auth/password/signin")
        .header("x-request-id", "req-from-header")
        .body(axum::body::Body::from(
            serde_json::json!({
                "email": "",
                "password": ""
            })
            .to_string(),
        )) {
        Ok(request) => request,
        Err(err) => panic!("failed to build request: {err}"),
    };

    let response = match app.oneshot(request).await {
        Ok(response) => response,
        Err(err) => panic!("request failed: {err}"),
    };

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    let body = response_json(response).await;
    assert_envelope_fields(&body, "invalid_request");
    assert_eq!(body["request_id"], "req-from-header");
}

#[tokio::test]
async fn error_envelope_invite_accept_missing_fields() {
    let app = build_test_app("redis://127.0.0.1:6379", "").await;

    let request =
        match request_builder("POST", "/api/auth/invite/accept").body(axum::body::Body::from(
            serde_json::json!({
                "invite_token": "",
                "email": "",
                "password": ""
            })
            .to_string(),
        )) {
            Ok(request) => request,
            Err(err) => panic!("failed to build request: {err}"),
        };

    let response = match app.oneshot(request).await {
        Ok(response) => response,
        Err(err) => panic!("request failed: {err}"),
    };

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    let body = response_json(response).await;
    assert_envelope_fields(&body, "invalid_request");
}

#[tokio::test]
async fn error_envelope_queue_connect_failure() {
    let app = build_test_app("redis://127.0.0.1:1", "").await;

    let request =
        match request_builder("POST", "/api/runtime/queue/enqueue").body(axum::body::Body::from(
            serde_json::json!({
                "user_id": "user-123",
                "kind": "train.search",
                "payload": {"origin": "SEO", "destination": "BUS"}
            })
            .to_string(),
        )) {
            Ok(request) => request,
            Err(err) => panic!("failed to build request: {err}"),
        };

    let response = match app.oneshot(request).await {
        Ok(response) => response,
        Err(err) => panic!("request failed: {err}"),
    };

    assert_eq!(response.status(), StatusCode::SERVICE_UNAVAILABLE);
    let body = response_json(response).await;
    assert_envelope_fields(&body, "service_unavailable");
    assert_eq!(body["message"], "runtime persistence unavailable");
    assert_eq!(body["details"]["stage"], "persist");
}

#[tokio::test]
async fn error_envelope_queue_redis_unavailable_branch() {
    let app = build_test_app("", "").await;

    let request = match request_builder("POST", "/api/runtime/queue/enqueue")
        .header("x-request-id", "redis-unavailable-request-id")
        .body(axum::body::Body::from(
            serde_json::json!({
                "user_id": "user-123",
                "kind": "train.search",
                "payload": {"origin": "SEO", "destination": "BUS"}
            })
            .to_string(),
        )) {
        Ok(request) => request,
        Err(err) => panic!("failed to build request: {err}"),
    };

    let response = match app.oneshot(request).await {
        Ok(response) => response,
        Err(err) => panic!("request failed: {err}"),
    };

    assert_eq!(response.status(), StatusCode::SERVICE_UNAVAILABLE);
    let body = response_json(response).await;
    assert_envelope_fields(&body, "service_unavailable");
    assert_eq!(body["message"], "runtime persistence unavailable");
    assert_eq!(body["request_id"], "redis-unavailable-request-id");
    assert_eq!(body["details"]["stage"], "persist");
}
