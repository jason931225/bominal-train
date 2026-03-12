//! Shared test infrastructure for bominal-server integration tests.
//!
//! Each test gets its own Postgres schema for full isolation and
//! parallel-safe execution.

use std::time::Instant;

use axum::body::Body;
use axum::http::{Method, Request, StatusCode};
use axum::Router;
use http_body_util::BodyExt;
use sqlx::PgPool;
use tower::ServiceExt;
use uuid::Uuid;

use bominal_domain::crypto::encryption::EncryptionKey;
use bominal_server::evervault::EvervaultConfig;
use bominal_server::routes::api_routes;
use bominal_server::sse::EventBus;
use bominal_server::state::SharedState;

/// Fixed 64-hex-char key used across all tests.
const TEST_ENCRYPTION_KEY: &str =
    "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef";

pub struct TestApp {
    pub router: Router,
    pub pool: PgPool,
    pub encryption_key: EncryptionKey,
    schema_name: String,
    admin_pool: PgPool,
}

impl TestApp {
    /// Spin up an isolated test environment with its own Postgres schema.
    pub async fn new() -> Self {
        dotenvy::dotenv().ok();

        let database_url =
            std::env::var("DATABASE_URL").expect("DATABASE_URL must be set for integration tests");

        let admin_pool = PgPool::connect(&database_url)
            .await
            .expect("failed to connect to admin database");

        let schema_name = format!("test_{}", Uuid::new_v4().simple());

        // Create isolated schema
        sqlx::query(&format!("CREATE SCHEMA \"{}\"", schema_name))
            .execute(&admin_pool)
            .await
            .expect("failed to create test schema");

        // Connect with search_path set to our test schema
        let test_url = if database_url.contains('?') {
            format!(
                "{}&options=-c search_path={},public",
                database_url, schema_name
            )
        } else {
            format!(
                "{}?options=-c search_path={},public",
                database_url, schema_name
            )
        };

        let pool = PgPool::connect(&test_url)
            .await
            .expect("failed to connect to test database");

        // Run initial schema directly — avoids checksum issues with overlapping
        // migrations (migration 1 already includes columns added by migration 2).
        // Use raw_sql to execute multiple statements in one call.
        sqlx::raw_sql(include_str!(
            "../../../bominal-db/migrations/20260311000001_initial_schema.sql"
        ))
        .execute(&pool)
        .await
        .expect("failed to run initial schema");

        sqlx::raw_sql(include_str!(
            "../../../bominal-db/migrations/20260312000001_add_passkey_and_expiry_fields.sql"
        ))
        .execute(&pool)
        .await
        .expect("failed to run passkey/expiry migration");

        let encryption_key = EncryptionKey::from_hex(TEST_ENCRYPTION_KEY).unwrap();

        let state = SharedState {
            db: pool.clone(),
            start_time: Instant::now(),
            event_bus: EventBus::new(),
            email: bominal_email::EmailClient::new("re_test_dummy", "Test <test@test.com>"),
            encryption_key: encryption_key.clone(),
            evervault: EvervaultConfig::new(
                "team_test_dummy",
                "app_test_dummy",
                "ev:key:test_dummy",
                "srt.test.relay.evervault.app",
                "ktx.test.relay.evervault.app",
            ),
            app_base_url: "http://localhost:3000".to_string(),
            prometheus_handle: metrics_exporter_prometheus::PrometheusBuilder::new()
                .install_recorder()
                .unwrap_or_else(|_| {
                    // Recorder already installed by another test — create a handle-only builder
                    metrics_exporter_prometheus::PrometheusBuilder::new().build_recorder().handle()
                }),
        };

        let router = Router::new()
            .nest("/api", api_routes())
            .with_state(state);

        Self {
            router,
            pool,
            encryption_key,
            schema_name,
            admin_pool,
        }
    }

    // ── Auth helpers ────────────────────────────────────────────────────

    /// Register a user and return the session cookie value.
    pub async fn register_user(
        &self,
        email: &str,
        password: &str,
        display_name: &str,
    ) -> String {
        let body = serde_json::json!({
            "email": email,
            "password": password,
            "display_name": display_name,
        });

        let req = Request::builder()
            .method(Method::POST)
            .uri("/api/auth/register")
            .header("content-type", "application/json")
            .body(Body::from(serde_json::to_vec(&body).unwrap()))
            .unwrap();

        let resp = self
            .router
            .clone()
            .oneshot(req)
            .await
            .expect("register request failed");

        assert_eq!(resp.status(), StatusCode::OK, "register should succeed");

        extract_session_cookie(&resp)
    }

    /// Login and return the session cookie value.
    pub async fn login_user(&self, email: &str, password: &str) -> String {
        let body = serde_json::json!({
            "email": email,
            "password": password,
        });

        let req = Request::builder()
            .method(Method::POST)
            .uri("/api/auth/login")
            .header("content-type", "application/json")
            .body(Body::from(serde_json::to_vec(&body).unwrap()))
            .unwrap();

        let resp = self
            .router
            .clone()
            .oneshot(req)
            .await
            .expect("login request failed");

        assert_eq!(resp.status(), StatusCode::OK, "login should succeed");

        extract_session_cookie(&resp)
    }

    // ── Request builders ────────────────────────────────────────────────

    pub fn authed_get(&self, uri: &str, session: &str) -> Request<Body> {
        Request::builder()
            .method(Method::GET)
            .uri(uri)
            .header("cookie", format!("bominal_session={session}"))
            .body(Body::empty())
            .unwrap()
    }

    pub fn authed_post(
        &self,
        uri: &str,
        session: &str,
        body: &serde_json::Value,
    ) -> Request<Body> {
        Request::builder()
            .method(Method::POST)
            .uri(uri)
            .header("cookie", format!("bominal_session={session}"))
            .header("content-type", "application/json")
            .body(Body::from(serde_json::to_vec(body).unwrap()))
            .unwrap()
    }

    pub fn authed_patch(
        &self,
        uri: &str,
        session: &str,
        body: &serde_json::Value,
    ) -> Request<Body> {
        Request::builder()
            .method(Method::PATCH)
            .uri(uri)
            .header("cookie", format!("bominal_session={session}"))
            .header("content-type", "application/json")
            .body(Body::from(serde_json::to_vec(body).unwrap()))
            .unwrap()
    }

    pub fn authed_delete(&self, uri: &str, session: &str) -> Request<Body> {
        Request::builder()
            .method(Method::DELETE)
            .uri(uri)
            .header("cookie", format!("bominal_session={session}"))
            .body(Body::empty())
            .unwrap()
    }

    // ── Send + parse ────────────────────────────────────────────────────

    pub async fn send(&self, req: Request<Body>) -> (StatusCode, serde_json::Value) {
        let resp = self
            .router
            .clone()
            .oneshot(req)
            .await
            .expect("request failed");

        let status = resp.status();
        let bytes = resp
            .into_body()
            .collect()
            .await
            .expect("failed to read body")
            .to_bytes();

        let json = if bytes.is_empty() {
            serde_json::json!(null)
        } else {
            serde_json::from_slice(&bytes).unwrap_or(serde_json::json!({"raw": String::from_utf8_lossy(&bytes).to_string()}))
        };

        (status, json)
    }

    /// Send a request and also return the raw response headers.
    pub async fn send_with_headers(
        &self,
        req: Request<Body>,
    ) -> (StatusCode, axum::http::HeaderMap, serde_json::Value) {
        let resp = self
            .router
            .clone()
            .oneshot(req)
            .await
            .expect("request failed");

        let status = resp.status();
        let headers = resp.headers().clone();
        let bytes = resp
            .into_body()
            .collect()
            .await
            .expect("failed to read body")
            .to_bytes();

        let json = if bytes.is_empty() {
            serde_json::json!(null)
        } else {
            serde_json::from_slice(&bytes).unwrap_or(serde_json::json!(null))
        };

        (status, headers, json)
    }

    // ── Cleanup ─────────────────────────────────────────────────────────

    pub async fn cleanup(self) {
        self.pool.close().await;
        let _ = sqlx::query(&format!("DROP SCHEMA \"{}\" CASCADE", self.schema_name))
            .execute(&self.admin_pool)
            .await;
        self.admin_pool.close().await;
    }
}

/// Extract the `bominal_session` cookie value from a response.
fn extract_session_cookie<B>(resp: &axum::http::Response<B>) -> String {
    resp.headers()
        .get_all("set-cookie")
        .iter()
        .find_map(|v| {
            let s = v.to_str().ok()?;
            if s.starts_with("bominal_session=") {
                Some(
                    s.split(';')
                        .next()?
                        .strip_prefix("bominal_session=")?
                        .to_string(),
                )
            } else {
                None
            }
        })
        .expect("no bominal_session cookie in response")
}
