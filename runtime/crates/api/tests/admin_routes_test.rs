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

fn admin_session(role: AdminRole, stepped_up: bool) -> SessionFixture {
    let now = Utc::now();
    SessionFixture {
        user_id: "4f0dfde0-97ed-4c2e-ab95-f87f5d4aa001".to_string(),
        email: "ops@bominal.com".to_string(),
        role,
        issued_at: now,
        last_seen_at: now,
        step_up_verified_at: stepped_up.then_some(now - ChronoDuration::seconds(5)),
    }
}

#[tokio::test]
async fn admin_pages_require_authenticated_admin_session() {
    let app = build_test_app("").await;
    let request = match Request::builder()
        .uri("/admin/maintenance")
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
}

#[tokio::test]
async fn admin_api_rejects_non_admin_host() {
    let app = build_test_app("").await;
    let request = match Request::builder()
        .uri("/api/admin/users")
        .header(header::HOST, "www.bominal.com")
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
    assert_eq!(
        body["message"],
        "admin api must be requested via admin host"
    );
}

#[tokio::test]
async fn admin_mutations_require_recent_step_up() {
    let Some(redis_server) = RedisTestServer::start().await else {
        eprintln!("redis-server not available; skipping admin step-up test");
        return;
    };

    let app = build_test_app(&redis_server.url).await;
    let session_id = "session-step-up-required";
    write_session(
        &redis_server.url,
        session_id,
        admin_session(AdminRole::Admin, false),
    )
    .await;

    let request = match Request::builder()
        .method("POST")
        .uri("/api/admin/runtime/jobs/job-123/retry")
        .header(header::HOST, "ops.bominal.com")
        .header(header::COOKIE, format!("bominal_session={session_id}"))
        .header(header::CONTENT_TYPE, "application/json")
        .body(axum::body::Body::from(
            serde_json::json!({
                "reason": "manual retry",
                "confirm_target": "job-123"
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

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    let body = response_json(response).await;
    assert_eq!(body["code"], "unauthorized");
    assert_eq!(body["message"], "recent passkey step-up required");
}

#[tokio::test]
async fn admin_mutations_require_reason_and_typed_confirmation() {
    let Some(redis_server) = RedisTestServer::start().await else {
        eprintln!("redis-server not available; skipping admin confirmation test");
        return;
    };

    let app = build_test_app(&redis_server.url).await;
    let session_id = "session-confirm-required";
    write_session(
        &redis_server.url,
        session_id,
        admin_session(AdminRole::Admin, true),
    )
    .await;

    let request = match Request::builder()
        .method("POST")
        .uri("/api/admin/runtime/jobs/job-456/requeue")
        .header(header::HOST, "ops.bominal.com")
        .header(header::COOKIE, format!("bominal_session={session_id}"))
        .header(header::CONTENT_TYPE, "application/json")
        .body(axum::body::Body::from(
            serde_json::json!({
                "reason": "",
                "confirm_target": "wrong-target"
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
    assert_eq!(body["code"], "invalid_request");
}

#[tokio::test]
async fn viewer_role_can_read_but_cannot_mutate_admin_routes() {
    let Some(redis_server) = RedisTestServer::start().await else {
        eprintln!("redis-server not available; skipping admin role matrix test");
        return;
    };

    let app = build_test_app(&redis_server.url).await;
    let session_id = "session-viewer-role";
    write_session(
        &redis_server.url,
        session_id,
        admin_session(AdminRole::Viewer, true),
    )
    .await;

    let read_request = match Request::builder()
        .uri("/api/admin/users")
        .header(header::HOST, "ops.bominal.com")
        .header(header::COOKIE, format!("bominal_session={session_id}"))
        .body(axum::body::Body::empty())
    {
        Ok(request) => request,
        Err(err) => panic!("failed to build read request: {err}"),
    };

    let read_response = match app.clone().oneshot(read_request).await {
        Ok(response) => response,
        Err(err) => panic!("read request failed: {err}"),
    };
    assert_eq!(read_response.status(), StatusCode::SERVICE_UNAVAILABLE);

    let mutate_request = match Request::builder()
        .method("POST")
        .uri("/api/admin/runtime/jobs/job-789/cancel")
        .header(header::HOST, "ops.bominal.com")
        .header(header::COOKIE, format!("bominal_session={session_id}"))
        .header(header::CONTENT_TYPE, "application/json")
        .body(axum::body::Body::from(
            serde_json::json!({
                "reason": "cancel test",
                "confirm_target": "job-789"
            })
            .to_string(),
        )) {
        Ok(request) => request,
        Err(err) => panic!("failed to build mutation request: {err}"),
    };

    let mutate_response = match app.oneshot(mutate_request).await {
        Ok(response) => response,
        Err(err) => panic!("mutation request failed: {err}"),
    };

    assert_eq!(mutate_response.status(), StatusCode::UNAUTHORIZED);
    let body = response_json(mutate_response).await;
    assert_eq!(body["message"], "admin mutation role required");
}
