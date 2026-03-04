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
use chrono::Utc;
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

async fn build_test_app_with_redis(redis_url: &str) -> Router {
    let mut config = test_config();
    config.redis.url = redis_url.to_string();
    let state = match api_main::build_state(config).await {
        Ok(state) => Arc::new(state),
        Err(err) => panic!("failed to build AppState with redis: {err}"),
    };

    api_main::build_router(state)
}

async fn write_session(redis_url: &str, session_id: &str, session: SessionFixture) {
    let client = match redis::Client::open(redis_url) {
        Ok(client) => client,
        Err(err) => panic!("failed to open redis client: {err}"),
    };
    let mut conn = match client.get_multiplexed_async_connection().await {
        Ok(conn) => conn,
        Err(err) => panic!("failed to connect to redis: {err}"),
    };
    let key = format!("{SESSION_KEY_PREFIX}{session_id}");
    let value = match serde_json::to_string(&session) {
        Ok(value) => value,
        Err(err) => panic!("failed to encode session fixture: {err}"),
    };
    if let Err(err) = conn.set_ex::<_, _, ()>(key, value, 3600).await {
        panic!("failed to write session fixture: {err}");
    }
}

fn user_session() -> SessionFixture {
    let now = Utc::now();
    SessionFixture {
        user_id: "4f0dfde0-97ed-4c2e-ab95-f87f5d4aa009".to_string(),
        email: "user@bominal.com".to_string(),
        role: AdminRole::User,
        issued_at: now,
        last_seen_at: now,
        step_up_verified_at: None,
    }
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
async fn train_api_station_regions_requires_authenticated_session() {
    let app = build_test_app().await;
    let request = match Request::builder()
        .uri("/api/train/stations/regions")
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
async fn dashboard_job_event_routes_reject_unknown_or_invalid_query_params() {
    let Some(redis_server) = RedisTestServer::start().await else {
        eprintln!("redis-server not available; skipping dashboard query validation test");
        return;
    };
    let app = build_test_app_with_redis(&redis_server.url).await;
    let session_id = "session-dashboard-query-validate";
    write_session(&redis_server.url, session_id, user_session()).await;

    let since_id_request = match Request::builder()
        .uri("/api/dashboard/jobs/job-1/events?since_id=10")
        .header(header::COOKIE, format!("bominal_session={session_id}"))
        .header(header::ACCEPT, "application/json")
        .body(axum::body::Body::empty())
    {
        Ok(request) => request,
        Err(err) => panic!("failed to build since_id request: {err}"),
    };
    let since_id_response = match app.clone().oneshot(since_id_request).await {
        Ok(response) => response,
        Err(err) => panic!("since_id request failed: {err}"),
    };
    assert_eq!(since_id_response.status(), StatusCode::BAD_REQUEST);
    let since_id_body = response_json(since_id_response).await;
    assert_eq!(since_id_body["code"], "invalid_request");
    assert!(since_id_body["request_id"].is_string());

    let limit_request = match Request::builder()
        .uri("/api/dashboard/jobs/job-1/events?limit=abc")
        .header(header::COOKIE, format!("bominal_session={session_id}"))
        .header(header::ACCEPT, "application/json")
        .body(axum::body::Body::empty())
    {
        Ok(request) => request,
        Err(err) => panic!("failed to build invalid limit request: {err}"),
    };
    let limit_response = match app.oneshot(limit_request).await {
        Ok(response) => response,
        Err(err) => panic!("invalid limit request failed: {err}"),
    };
    assert_eq!(limit_response.status(), StatusCode::BAD_REQUEST);
    let limit_body = response_json(limit_response).await;
    assert_eq!(limit_body["code"], "invalid_request");
    assert!(limit_body["request_id"].is_string());
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
