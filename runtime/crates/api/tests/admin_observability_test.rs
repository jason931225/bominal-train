use std::{
    process::{Child, Command, Stdio},
    sync::Arc,
    time::Duration,
};

use axum::{
    Router,
    body::to_bytes,
    http::{Request, StatusCode, header},
};
use bominal_shared::config::{
    AdminRole, AppConfig, EvervaultConfig, PasskeyConfig, PasskeyProvider, RedisConfig,
    RuntimeSchedule,
};
use chrono::{Duration as ChronoDuration, Utc};
use redis::AsyncCommands;
use tower::util::ServiceExt;

#[allow(dead_code)]
#[path = "../src/main.rs"]
mod api_main;

const SESSION_KEY_PREFIX: &str = "auth:session:";

#[derive(Clone, serde::Serialize)]
struct SessionFixture {
    user_id: String,
    email: String,
    role: AdminRole,
    issued_at: chrono::DateTime<Utc>,
    last_seen_at: chrono::DateTime<Utc>,
    step_up_verified_at: Option<chrono::DateTime<Utc>>,
}

struct RedisTestServer {
    child: Child,
    url: String,
}

impl RedisTestServer {
    async fn start() -> Option<Self> {
        if !Command::new("redis-server")
            .arg("--version")
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .ok()?
            .success()
        {
            return None;
        }

        let port = {
            let listener = std::net::TcpListener::bind("127.0.0.1:0").ok()?;
            listener.local_addr().ok()?.port()
        };

        let child = Command::new("redis-server")
            .arg("--save")
            .arg("")
            .arg("--appendonly")
            .arg("no")
            .arg("--bind")
            .arg("127.0.0.1")
            .arg("--port")
            .arg(port.to_string())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .ok()?;

        let url = format!("redis://127.0.0.1:{port}/0");
        for _ in 0..40 {
            if let Ok(client) = redis::Client::open(url.as_str()) {
                if let Ok(mut conn) = client.get_multiplexed_async_connection().await {
                    let pong: redis::RedisResult<String> =
                        redis::cmd("PING").query_async(&mut conn).await;
                    if matches!(pong.as_deref(), Ok("PONG")) {
                        return Some(Self { child, url });
                    }
                }
            }
            tokio::time::sleep(Duration::from_millis(100)).await;
        }

        None
    }
}

impl Drop for RedisTestServer {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

fn test_config(redis_url: &str) -> AppConfig {
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
        database_url: String::new(),
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

async fn build_test_app(redis_url: &str) -> Router {
    let state = match api_main::build_state(test_config(redis_url)).await {
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

async fn write_admin_session(redis_url: &str, session_id: &str) {
    let client = match redis::Client::open(redis_url) {
        Ok(client) => client,
        Err(err) => panic!("failed to open redis client: {err}"),
    };
    let mut conn = match client.get_multiplexed_async_connection().await {
        Ok(conn) => conn,
        Err(err) => panic!("failed to connect to redis: {err}"),
    };

    let now = Utc::now();
    let session = SessionFixture {
        user_id: "164f207e-fcc7-4ca4-8ec9-0f73f7dc5f8f".to_string(),
        email: "ops@bominal.com".to_string(),
        role: AdminRole::Admin,
        issued_at: now,
        last_seen_at: now,
        step_up_verified_at: Some(now - ChronoDuration::seconds(5)),
    };
    let value = match serde_json::to_string(&session) {
        Ok(value) => value,
        Err(err) => panic!("failed to encode session fixture: {err}"),
    };
    let key = format!("{SESSION_KEY_PREFIX}{session_id}");
    if let Err(err) = conn.set_ex::<_, _, ()>(key, value, 3600).await {
        panic!("failed to write session fixture: {err}");
    }
}

#[tokio::test]
async fn health_and_ready_are_explicitly_distinct() {
    let app = build_test_app("").await;

    let health_request = match Request::builder()
        .uri("/health")
        .body(axum::body::Body::empty())
    {
        Ok(request) => request,
        Err(err) => panic!("failed to build health request: {err}"),
    };
    let health_response = match app.clone().oneshot(health_request).await {
        Ok(response) => response,
        Err(err) => panic!("health request failed: {err}"),
    };
    assert_eq!(health_response.status(), StatusCode::OK);
    let health_json = response_json(health_response).await;
    assert_eq!(health_json["ok"], true);
    assert_eq!(health_json["service"], "bominal-api");

    let ready_request = match Request::builder()
        .uri("/ready")
        .body(axum::body::Body::empty())
    {
        Ok(request) => request,
        Err(err) => panic!("failed to build ready request: {err}"),
    };
    let ready_response = match app.oneshot(ready_request).await {
        Ok(response) => response,
        Err(err) => panic!("ready request failed: {err}"),
    };
    assert_eq!(ready_response.status(), StatusCode::SERVICE_UNAVAILABLE);
    let ready_json = response_json(ready_response).await;
    assert_eq!(ready_json["ok"], false);
    assert_eq!(ready_json["db"], false);
    assert_eq!(ready_json["redis"], false);
}

#[tokio::test]
async fn admin_metrics_summary_route_requires_admin_session() {
    let app = build_test_app("").await;
    let request = match Request::builder()
        .uri("/api/admin/maintenance/metrics/summary")
        .header(header::HOST, "ops.bominal.com")
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
async fn admin_metrics_endpoints_work_with_admin_session() {
    let Some(redis_server) = RedisTestServer::start().await else {
        eprintln!("redis-server not available; skipping admin metrics test");
        return;
    };

    let app = build_test_app(&redis_server.url).await;
    let session_id = "metrics-admin-session";
    write_admin_session(&redis_server.url, session_id).await;

    let text_request = match Request::builder()
        .uri("/admin/maintenance/metrics")
        .header(header::HOST, "ops.bominal.com")
        .header(header::COOKIE, format!("bominal_session={session_id}"))
        .body(axum::body::Body::empty())
    {
        Ok(request) => request,
        Err(err) => panic!("failed to build metrics text request: {err}"),
    };
    let text_response = match app.clone().oneshot(text_request).await {
        Ok(response) => response,
        Err(err) => panic!("metrics text request failed: {err}"),
    };

    assert_eq!(text_response.status(), StatusCode::OK);
    let content_type = text_response
        .headers()
        .get(header::CONTENT_TYPE)
        .and_then(|value| value.to_str().ok())
        .unwrap_or_default();
    assert!(content_type.starts_with("text/plain"));

    let text_body = match to_bytes(text_response.into_body(), usize::MAX).await {
        Ok(bytes) => bytes,
        Err(err) => panic!("failed to read metrics text body: {err}"),
    };
    let text = match String::from_utf8(text_body.to_vec()) {
        Ok(text) => text,
        Err(err) => panic!("metrics body not valid utf-8: {err}"),
    };
    assert!(text.contains("http_requests_total") || !text.trim().is_empty());

    let summary_request = match Request::builder()
        .uri("/api/admin/maintenance/metrics/summary")
        .header(header::HOST, "ops.bominal.com")
        .header(header::COOKIE, format!("bominal_session={session_id}"))
        .body(axum::body::Body::empty())
    {
        Ok(request) => request,
        Err(err) => panic!("failed to build metrics summary request: {err}"),
    };
    let summary_response = match app.oneshot(summary_request).await {
        Ok(response) => response,
        Err(err) => panic!("metrics summary request failed: {err}"),
    };

    assert_eq!(summary_response.status(), StatusCode::OK);
    let summary = response_json(summary_response).await;
    assert!(summary["liveness_ok"].is_boolean());
    assert!(summary["readiness_ok"].is_boolean());
    assert!(summary["error_rate"].is_number());
    assert!(summary["raw_excerpt"].is_array());
}
