use std::{sync::Arc, time::Duration};

use axum::{
    Json, Router,
    body::to_bytes,
    http::{StatusCode, header, request::Builder as RequestBuilder},
    routing::get,
};
use bominal_shared::config::{
    AppConfig, EvervaultConfig, RedisConfig, RuntimeSchedule, SupabaseConfig,
};
use tower::util::ServiceExt;

#[allow(dead_code)]
#[path = "../src/main.rs"]
mod api_main;

fn test_config(
    redis_url: &str,
    database_url: &str,
    jwks_url: String,
    webhook_secret: Option<&str>,
) -> AppConfig {
    AppConfig {
        app_env: "test".to_string(),
        app_host: "127.0.0.1".to_string(),
        app_port: 0,
        log_json: false,
        database_url: database_url.to_string(),
        supabase: SupabaseConfig {
            url: "https://example.supabase.co".to_string(),
            jwt_issuer: "https://example.supabase.co/auth/v1".to_string(),
            jwt_audience: Some("authenticated".to_string()),
            jwks_url,
            jwks_cache_seconds: 300,
            auth_webhook_secret: webhook_secret.map(str::to_string),
        },
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
        runtime: RuntimeSchedule {
            poll_interval: Duration::from_secs(1),
            reconcile_interval: Duration::from_secs(1),
            watch_interval: Duration::from_secs(1),
            key_rotation_interval: Duration::from_secs(1),
        },
    }
}

async fn build_test_app(
    redis_url: &str,
    database_url: &str,
    jwks_url: String,
    webhook_secret: Option<&str>,
) -> Router {
    let state = match api_main::build_state(test_config(
        redis_url,
        database_url,
        jwks_url,
        webhook_secret,
    ))
    .await
    {
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

async fn spawn_jwks_server() -> String {
    let router = Router::new().route(
        "/.well-known/jwks.json",
        get(|| async { Json(serde_json::json!({ "keys": [] })) }),
    );

    let listener = match tokio::net::TcpListener::bind("127.0.0.1:0").await {
        Ok(listener) => listener,
        Err(err) => panic!("failed to bind jwks test server: {err}"),
    };
    let addr = match listener.local_addr() {
        Ok(addr) => addr,
        Err(err) => panic!("failed to read jwks test server addr: {err}"),
    };

    tokio::spawn(async move {
        if let Err(err) = axum::serve(listener, router).await {
            panic!("jwks test server failed: {err}");
        }
    });

    format!("http://{addr}/.well-known/jwks.json")
}

#[tokio::test]
async fn error_envelope_missing_bearer_token() {
    let jwks_url = spawn_jwks_server().await;
    let app = build_test_app("redis://127.0.0.1:6379", "", jwks_url, None).await;

    let request = match request_builder("GET", "/api/auth/supabase/verify")
        .header("x-request-id", "req-from-header")
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
    assert_envelope_fields(&body, "invalid_request");
    assert_eq!(body["request_id"], "req-from-header");
}

#[tokio::test]
async fn error_envelope_invalid_token() {
    let jwks_url = spawn_jwks_server().await;
    let app = build_test_app("redis://127.0.0.1:6379", "", jwks_url, None).await;

    let request = match request_builder("GET", "/api/auth/supabase/verify")
        .header(header::AUTHORIZATION, "Bearer not-a-jwt")
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
    assert_envelope_fields(&body, "unauthorized");
}

#[tokio::test]
async fn error_envelope_webhook_invalid_secret() {
    let jwks_url = spawn_jwks_server().await;
    let app = build_test_app(
        "redis://127.0.0.1:6379",
        "",
        jwks_url,
        Some("expected-secret"),
    )
    .await;

    let request = match request_builder("POST", "/api/auth/supabase/webhook")
        .header("x-bominal-supabase-webhook-secret", "wrong-secret")
        .body(axum::body::Body::from(
            serde_json::json!({
                "type": "user.updated",
                "user_id": "user-123",
                "email": "user@example.com"
            })
            .to_string(),
        ))
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
    assert_envelope_fields(&body, "unauthorized");
}

#[tokio::test]
async fn error_envelope_queue_connect_failure() {
    let jwks_url = spawn_jwks_server().await;
    let app = build_test_app("redis://127.0.0.1:1", "", jwks_url, None).await;

    let request = match request_builder("POST", "/api/runtime/queue/enqueue").body(
        axum::body::Body::from(
            serde_json::json!({
                "user_id": "user-123",
                "kind": "train.search",
                "payload": {"origin": "SEO", "destination": "BUS"}
            })
            .to_string(),
        ),
    ) {
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
    assert_eq!(body["details"]["stage"], "connect");
}

#[tokio::test]
async fn error_envelope_queue_redis_unavailable_branch() {
    let jwks_url = spawn_jwks_server().await;
    let app = build_test_app("", "", jwks_url, None).await;

    let request = match request_builder("POST", "/api/runtime/queue/enqueue")
        .header("x-request-id", "redis-unavailable-request-id")
        .body(axum::body::Body::from(
            serde_json::json!({
                "user_id": "user-123",
                "kind": "train.search",
                "payload": {"origin": "SEO", "destination": "BUS"}
            })
            .to_string(),
        ))
    {
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
    assert_eq!(body["message"], "redis unavailable");
    assert_eq!(body["request_id"], "redis-unavailable-request-id");
    assert!(body.get("details").is_none());
}

#[tokio::test]
async fn error_envelope_jwks_unavailable_branch() {
    let app = build_test_app(
        "redis://127.0.0.1:6379",
        "",
        "http://127.0.0.1:1/.well-known/jwks.json".to_string(),
        None,
    )
    .await;

    let request = match request_builder("GET", "/api/auth/supabase/verify")
        .header(header::AUTHORIZATION, "Bearer not-a-jwt")
        .header("x-request-id", "jwks-request-id")
        .body(axum::body::Body::empty())
    {
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
    assert_eq!(body["message"], "jwks unavailable");
    assert_eq!(body["request_id"], "jwks-request-id");
}

#[tokio::test]
async fn error_envelope_auth_sync_persistence_failure_branch() {
    let jwks_url = spawn_jwks_server().await;
    let app = build_test_app(
        "redis://127.0.0.1:6379",
        "postgres://postgres:postgres@127.0.0.1:1/bominal_test",
        jwks_url,
        None,
    )
    .await;

    let request = match request_builder("POST", "/api/auth/supabase/webhook")
        .header("x-request-id", "auth-sync-request-id")
        .body(axum::body::Body::from(
            serde_json::json!({
                "type": "user.updated",
                "user_id": "user-123",
                "email": "user@example.com"
            })
            .to_string(),
        ))
    {
        Ok(request) => request,
        Err(err) => panic!("failed to build request: {err}"),
    };

    let response = match app.oneshot(request).await {
        Ok(response) => response,
        Err(err) => panic!("request failed: {err}"),
    };

    assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
    let body = response_json(response).await;
    assert_envelope_fields(&body, "internal_error");
    assert_eq!(body["message"], "auth sync persistence failed");
    assert_eq!(body["request_id"], "auth-sync-request-id");
}
