mod common;

use axum::body::Body;
use axum::http::{Method, Request, StatusCode};
use uuid::Uuid;

use common::TestApp;

/// Insert a provider credential directly in the DB (bypasses real login).
async fn insert_cred(app: &TestApp, user_id: Uuid, provider: &str) {
    let enc_pw = bominal_domain::crypto::encryption::encrypt(&app.encryption_key, "test_pw")
        .expect("encrypt");

    bominal_db::provider::upsert_credential(
        &app.pool,
        user_id,
        provider,
        "test_login@example.com",
        &enc_pw,
        "valid",
    )
    .await
    .expect("upsert credential");
}

async fn get_user_id(app: &TestApp, session: &str) -> Uuid {
    let req = app.authed_get("/api/auth/me", session);
    let (_, json) = app.send(req).await;
    json["user_id"].as_str().unwrap().parse().unwrap()
}

#[tokio::test]
async fn list_providers_empty() {
    let app = TestApp::new().await;
    let session = app
        .register_user("prov0@example.com", "password123", "P0")
        .await;

    let req = app.authed_get("/api/providers", &session);
    let (status, json) = app.send(req).await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(json.as_array().unwrap().len(), 0);

    app.cleanup().await;
}

#[tokio::test]
async fn list_providers_with_data() {
    let app = TestApp::new().await;
    let session = app
        .register_user("prov1@example.com", "password123", "P1")
        .await;
    let user_id = get_user_id(&app, &session).await;

    insert_cred(&app, user_id, "SRT").await;

    let req = app.authed_get("/api/providers", &session);
    let (status, json) = app.send(req).await;

    assert_eq!(status, StatusCode::OK);
    let providers = json.as_array().unwrap();
    assert_eq!(providers.len(), 1);
    assert_eq!(providers[0]["provider"], "SRT");
    assert_eq!(providers[0]["status"], "valid");
    // login_id should be masked
    let login = providers[0]["login_id"].as_str().unwrap();
    assert!(login.contains("*"), "login_id should be masked: {login}");

    app.cleanup().await;
}

#[tokio::test]
async fn list_providers_own_only() {
    let app = TestApp::new().await;

    // User A has a credential
    let session_a = app
        .register_user("prova@example.com", "password123", "PA")
        .await;
    let user_a = get_user_id(&app, &session_a).await;
    insert_cred(&app, user_a, "SRT").await;

    // User B should see empty
    let session_b = app
        .register_user("provb@example.com", "password123", "PB")
        .await;
    let req = app.authed_get("/api/providers", &session_b);
    let (status, json) = app.send(req).await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(json.as_array().unwrap().len(), 0);

    app.cleanup().await;
}

#[tokio::test]
async fn delete_provider_success() {
    let app = TestApp::new().await;
    let session = app
        .register_user("del@example.com", "password123", "Del")
        .await;
    let user_id = get_user_id(&app, &session).await;

    insert_cred(&app, user_id, "SRT").await;

    let req = app.authed_delete("/api/providers/SRT", &session);
    let (status, json) = app.send(req).await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(json["deleted"], true);

    // Verify it's gone
    let req = app.authed_get("/api/providers", &session);
    let (_, json) = app.send(req).await;
    assert_eq!(json.as_array().unwrap().len(), 0);

    app.cleanup().await;
}

#[tokio::test]
async fn delete_provider_not_found() {
    let app = TestApp::new().await;
    let session = app
        .register_user("delnf@example.com", "password123", "DelNF")
        .await;

    let req = app.authed_delete("/api/providers/SRT", &session);
    let (status, _) = app.send(req).await;

    assert_eq!(status, StatusCode::NOT_FOUND);

    app.cleanup().await;
}

#[tokio::test]
async fn delete_provider_other_user() {
    let app = TestApp::new().await;

    let session_a = app
        .register_user("owna@example.com", "password123", "OwnA")
        .await;
    let user_a = get_user_id(&app, &session_a).await;
    insert_cred(&app, user_a, "SRT").await;

    // User B tries to delete A's provider
    let session_b = app
        .register_user("ownb@example.com", "password123", "OwnB")
        .await;
    let req = app.authed_delete("/api/providers/SRT", &session_b);
    let (status, _) = app.send(req).await;

    assert_eq!(status, StatusCode::NOT_FOUND);

    app.cleanup().await;
}

#[tokio::test]
async fn unauthenticated_access() {
    let app = TestApp::new().await;

    let req = Request::builder()
        .method(Method::GET)
        .uri("/api/providers")
        .body(Body::empty())
        .unwrap();

    let (status, _) = app.send(req).await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);

    app.cleanup().await;
}
