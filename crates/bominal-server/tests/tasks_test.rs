mod common;

use axum::body::Body;
use axum::http::{Method, Request, StatusCode};
use uuid::Uuid;

use common::TestApp;

/// Insert a fake provider credential directly into the DB so task creation works.
async fn insert_provider_credential(app: &TestApp, user_id: Uuid, provider: &str) {
    let enc_pw = bominal_domain::crypto::encryption::encrypt(&app.encryption_key, "fake_password")
        .expect("encryption failed");

    bominal_db::provider::upsert_credential(
        &app.pool,
        user_id,
        provider,
        "test_login_id",
        &enc_pw,
        "valid",
    )
    .await
    .expect("failed to insert provider credential");
}

/// Helper: register + insert SRT creds, return (session, user_id).
async fn setup_user_with_creds(app: &TestApp) -> (String, Uuid) {
    let session = app
        .register_user("tasks@example.com", "password123", "TaskUser")
        .await;

    // Get user_id from /me
    let req = app.authed_get("/api/auth/me", &session);
    let (_, json) = app.send(req).await;
    let user_id: Uuid = json["user_id"].as_str().unwrap().parse().unwrap();

    insert_provider_credential(app, user_id, "SRT").await;
    (session, user_id)
}

fn valid_task_body() -> serde_json::Value {
    serde_json::json!({
        "provider": "SRT",
        "departure_station": "수서",
        "arrival_station": "부산",
        "travel_date": "20260315",
        "departure_time": "090000",
        "passengers": [{"type": "adult", "count": 1}],
        "seat_preference": "GeneralFirst",
        "target_trains": [{"train_number": "305", "dep_time": "090000"}],
        "auto_pay": false,
        "notify_enabled": false,
        "auto_retry": true,
    })
}

#[tokio::test]
async fn create_task_success() {
    let app = TestApp::new().await;
    let (session, _) = setup_user_with_creds(&app).await;

    let req = app.authed_post("/api/tasks", &session, &valid_task_body());
    let (status, json) = app.send(req).await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(json["status"], "queued");
    assert_eq!(json["provider"], "SRT");
    assert!(json["id"].is_string());

    app.cleanup().await;
}

#[tokio::test]
async fn create_task_no_credentials() {
    let app = TestApp::new().await;
    let session = app
        .register_user("nocred@example.com", "password123", "NoCred")
        .await;

    let req = app.authed_post("/api/tasks", &session, &valid_task_body());
    let (status, _) = app.send(req).await;

    assert_eq!(status, StatusCode::BAD_REQUEST);

    app.cleanup().await;
}

#[tokio::test]
async fn create_task_invalid_provider() {
    let app = TestApp::new().await;
    let (session, _) = setup_user_with_creds(&app).await;

    let mut body = valid_task_body();
    body["provider"] = serde_json::json!("KORAIL");

    let req = app.authed_post("/api/tasks", &session, &body);
    let (status, _) = app.send(req).await;

    assert_eq!(status, StatusCode::BAD_REQUEST);

    app.cleanup().await;
}

#[tokio::test]
async fn create_task_empty_trains() {
    let app = TestApp::new().await;
    let (session, _) = setup_user_with_creds(&app).await;

    let mut body = valid_task_body();
    body["target_trains"] = serde_json::json!([]);

    let req = app.authed_post("/api/tasks", &session, &body);
    let (status, _) = app.send(req).await;

    assert_eq!(status, StatusCode::BAD_REQUEST);

    app.cleanup().await;
}

#[tokio::test]
async fn list_tasks_empty() {
    let app = TestApp::new().await;
    let session = app
        .register_user("empty@example.com", "password123", "Empty")
        .await;

    let req = app.authed_get("/api/tasks", &session);
    let (status, json) = app.send(req).await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(json.as_array().unwrap().len(), 0);

    app.cleanup().await;
}

#[tokio::test]
async fn list_tasks_own_only() {
    let app = TestApp::new().await;

    // User A creates a task
    let session_a = app
        .register_user("usera@example.com", "password123", "UserA")
        .await;
    let req = app.authed_get("/api/auth/me", &session_a);
    let (_, json) = app.send(req).await;
    let user_a_id: Uuid = json["user_id"].as_str().unwrap().parse().unwrap();
    insert_provider_credential(&app, user_a_id, "SRT").await;

    let req = app.authed_post("/api/tasks", &session_a, &valid_task_body());
    let (status, _) = app.send(req).await;
    assert_eq!(status, StatusCode::OK);

    // User B should see empty list
    let session_b = app
        .register_user("userb@example.com", "password123", "UserB")
        .await;
    let req = app.authed_get("/api/tasks", &session_b);
    let (status, json) = app.send(req).await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(json.as_array().unwrap().len(), 0);

    app.cleanup().await;
}

#[tokio::test]
async fn get_task_success() {
    let app = TestApp::new().await;
    let (session, _) = setup_user_with_creds(&app).await;

    // Create task
    let req = app.authed_post("/api/tasks", &session, &valid_task_body());
    let (_, created) = app.send(req).await;
    let task_id = created["id"].as_str().unwrap();

    // Get task
    let req = app.authed_get(&format!("/api/tasks/{task_id}"), &session);
    let (status, json) = app.send(req).await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(json["id"], task_id);
    assert_eq!(json["departure_station"], "수서");

    app.cleanup().await;
}

#[tokio::test]
async fn get_task_not_found() {
    let app = TestApp::new().await;
    let session = app
        .register_user("nf@example.com", "password123", "NF")
        .await;

    let fake_id = Uuid::new_v4();
    let req = app.authed_get(&format!("/api/tasks/{fake_id}"), &session);
    let (status, _) = app.send(req).await;

    assert_eq!(status, StatusCode::NOT_FOUND);

    app.cleanup().await;
}

#[tokio::test]
async fn get_task_other_user() {
    let app = TestApp::new().await;
    let (session_a, _) = setup_user_with_creds(&app).await;

    // User A creates task
    let req = app.authed_post("/api/tasks", &session_a, &valid_task_body());
    let (_, created) = app.send(req).await;
    let task_id = created["id"].as_str().unwrap();

    // User B tries to get it
    let session_b = app
        .register_user("other@example.com", "password123", "Other")
        .await;
    let req = app.authed_get(&format!("/api/tasks/{task_id}"), &session_b);
    let (status, _) = app.send(req).await;

    assert_eq!(status, StatusCode::NOT_FOUND);

    app.cleanup().await;
}

#[tokio::test]
async fn update_task_status() {
    let app = TestApp::new().await;
    let (session, _) = setup_user_with_creds(&app).await;

    let req = app.authed_post("/api/tasks", &session, &valid_task_body());
    let (_, created) = app.send(req).await;
    let task_id = created["id"].as_str().unwrap();

    let body = serde_json::json!({ "status": "idle" });
    let req = app.authed_patch(&format!("/api/tasks/{task_id}"), &session, &body);
    let (status, json) = app.send(req).await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(json["status"], "idle");

    app.cleanup().await;
}

#[tokio::test]
async fn update_task_invalid_status() {
    let app = TestApp::new().await;
    let (session, _) = setup_user_with_creds(&app).await;

    let req = app.authed_post("/api/tasks", &session, &valid_task_body());
    let (_, created) = app.send(req).await;
    let task_id = created["id"].as_str().unwrap();

    let body = serde_json::json!({ "status": "confirmed" });
    let req = app.authed_patch(&format!("/api/tasks/{task_id}"), &session, &body);
    let (status, _) = app.send(req).await;

    assert_eq!(status, StatusCode::BAD_REQUEST);

    app.cleanup().await;
}

#[tokio::test]
async fn delete_task_success() {
    let app = TestApp::new().await;
    let (session, _) = setup_user_with_creds(&app).await;

    let req = app.authed_post("/api/tasks", &session, &valid_task_body());
    let (_, created) = app.send(req).await;
    let task_id = created["id"].as_str().unwrap();

    let req = app.authed_delete(&format!("/api/tasks/{task_id}"), &session);
    let (status, json) = app.send(req).await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(json["cancelled"], true);

    app.cleanup().await;
}

#[tokio::test]
async fn delete_task_other_user() {
    let app = TestApp::new().await;
    let (session_a, _) = setup_user_with_creds(&app).await;

    let req = app.authed_post("/api/tasks", &session_a, &valid_task_body());
    let (_, created) = app.send(req).await;
    let task_id = created["id"].as_str().unwrap();

    let session_b = app
        .register_user("del_other@example.com", "password123", "DelOther")
        .await;
    let req = app.authed_delete(&format!("/api/tasks/{task_id}"), &session_b);
    let (status, _) = app.send(req).await;

    assert_eq!(status, StatusCode::NOT_FOUND);

    app.cleanup().await;
}

#[tokio::test]
async fn unauthenticated_access() {
    let app = TestApp::new().await;

    let req = Request::builder()
        .method(Method::GET)
        .uri("/api/tasks")
        .body(Body::empty())
        .unwrap();

    let (status, _) = app.send(req).await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);

    app.cleanup().await;
}
