mod common;

use axum::body::Body;
use axum::http::{Method, Request, StatusCode};

use common::TestApp;

#[tokio::test]
async fn login_start_returns_challenge() {
    let app = TestApp::new().await;

    let req = Request::builder()
        .method(Method::POST)
        .uri("/api/auth/passkey/login/start")
        .body(Body::empty())
        .unwrap();

    let (status, json) = app.send(req).await;
    assert_eq!(status, StatusCode::OK);
    assert!(
        json["challenge_id"].is_string(),
        "should contain challenge_id"
    );
    assert!(json["options"].is_object(), "should contain options");

    app.cleanup().await;
}

#[tokio::test]
async fn login_finish_invalid_challenge() {
    let app = TestApp::new().await;

    let body = serde_json::json!({
        "challenge_id": "nonexistent-challenge",
        "credential": {
            "id": "dGVzdA",
            "rawId": "dGVzdA",
            "response": {
                "authenticatorData": "dGVzdA",
                "clientDataJSON": "dGVzdA",
                "signature": "dGVzdA"
            },
            "type": "public-key"
        }
    });

    let req = Request::builder()
        .method(Method::POST)
        .uri("/api/auth/passkey/login/finish")
        .header("content-type", "application/json")
        .body(Body::from(serde_json::to_vec(&body).unwrap()))
        .unwrap();

    let (status, _) = app.send(req).await;
    // 400 (invalid challenge) or 422 (body parse) — either rejects the request
    assert!(
        status == StatusCode::BAD_REQUEST || status == StatusCode::UNPROCESSABLE_ENTITY,
        "invalid challenge should be rejected, got {status}"
    );

    app.cleanup().await;
}

#[tokio::test]
async fn login_finish_expired_challenge() {
    let app = TestApp::new().await;

    // Insert an expired challenge directly (created 10 minutes ago)
    sqlx::query(
        "INSERT INTO passkey_challenges (challenge_id, state, created_at) \
         VALUES ($1, $2, NOW() - INTERVAL '10 minutes')",
    )
    .bind("expired-challenge-id")
    .bind("{}")
    .execute(&app.pool)
    .await
    .expect("failed to insert expired challenge");

    let body = serde_json::json!({
        "challenge_id": "expired-challenge-id",
        "credential": {
            "id": "dGVzdA",
            "rawId": "dGVzdA",
            "response": {
                "authenticatorData": "dGVzdA",
                "clientDataJSON": "dGVzdA",
                "signature": "dGVzdA"
            },
            "type": "public-key"
        }
    });

    let req = Request::builder()
        .method(Method::POST)
        .uri("/api/auth/passkey/login/finish")
        .header("content-type", "application/json")
        .body(Body::from(serde_json::to_vec(&body).unwrap()))
        .unwrap();

    let (status, _) = app.send(req).await;
    // 400 (expired challenge) or 422 (body parse) — either rejects the request
    assert!(
        status == StatusCode::BAD_REQUEST || status == StatusCode::UNPROCESSABLE_ENTITY,
        "expired challenge should be rejected, got {status}"
    );

    app.cleanup().await;
}

#[tokio::test]
async fn register_start_requires_auth() {
    let app = TestApp::new().await;

    let req = Request::builder()
        .method(Method::POST)
        .uri("/api/auth/passkey/register/start")
        .body(Body::empty())
        .unwrap();

    let (status, _) = app.send(req).await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);

    app.cleanup().await;
}

#[tokio::test]
async fn register_start_returns_options() {
    let app = TestApp::new().await;

    let session = app
        .register_user("passkey@example.com", "password123", "Passkey User")
        .await;

    let req = Request::builder()
        .method(Method::POST)
        .uri("/api/auth/passkey/register/start")
        .header("cookie", format!("bominal_session={session}"))
        .body(Body::empty())
        .unwrap();

    let (status, json) = app.send(req).await;
    assert_eq!(status, StatusCode::OK);
    assert!(
        json["challenge_id"].is_string(),
        "should contain challenge_id"
    );
    assert!(json["options"].is_object(), "should contain options");

    app.cleanup().await;
}

#[tokio::test]
async fn update_credential_key_updates_public_key() {
    let app = TestApp::new().await;

    let session = app
        .register_user("cred@example.com", "password123", "Cred User")
        .await;

    // Get the user_id from /me
    let req = app.authed_get("/api/auth/me", &session);
    let (_, json) = app.send(req).await;
    let user_id: uuid::Uuid = json["user_id"].as_str().unwrap().parse().unwrap();

    // Store a test credential
    let cred = bominal_db::passkey::store_credential(
        &app.pool,
        user_id,
        "test-cred-id",
        r#"{"original": true}"#,
        "Test Key",
    )
    .await
    .expect("failed to store credential");

    assert_eq!(cred.public_key, r#"{"original": true}"#);

    // Update the credential key
    bominal_db::passkey::update_credential_key(
        &app.pool,
        "test-cred-id",
        r#"{"updated": true, "counter": 1}"#,
    )
    .await
    .expect("failed to update credential key");

    // Verify the update
    let updated = bominal_db::passkey::find_credential_by_id(&app.pool, "test-cred-id")
        .await
        .expect("failed to find credential")
        .expect("credential not found");

    assert_eq!(updated.public_key, r#"{"updated": true, "counter": 1}"#);

    app.cleanup().await;
}
