mod common;

use axum::body::Body;
use axum::http::{Method, Request, StatusCode};

use common::TestApp;

#[tokio::test]
async fn register_success() {
    let app = TestApp::new().await;
    let body = serde_json::json!({
        "email": "alice@example.com",
        "password": "password123",
        "display_name": "Alice",
    });

    let req = Request::builder()
        .method(Method::POST)
        .uri("/api/auth/register")
        .header("content-type", "application/json")
        .body(Body::from(serde_json::to_vec(&body).unwrap()))
        .unwrap();

    let (status, headers, json) = app.send_with_headers(req).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(json["email"], "alice@example.com");
    assert_eq!(json["display_name"], "Alice");
    assert!(json["user_id"].is_string());

    // Should have Set-Cookie header
    let cookie = headers
        .get("set-cookie")
        .expect("missing set-cookie")
        .to_str()
        .unwrap();
    assert!(cookie.contains("bominal_session="));

    app.cleanup().await;
}

#[tokio::test]
async fn register_duplicate_email() {
    let app = TestApp::new().await;

    app.register_user("dup@example.com", "password123", "First")
        .await;

    let body = serde_json::json!({
        "email": "dup@example.com",
        "password": "password456",
        "display_name": "Second",
    });

    let req = Request::builder()
        .method(Method::POST)
        .uri("/api/auth/register")
        .header("content-type", "application/json")
        .body(Body::from(serde_json::to_vec(&body).unwrap()))
        .unwrap();

    let (status, _) = app.send(req).await;
    assert_eq!(status, StatusCode::BAD_REQUEST);

    app.cleanup().await;
}

#[tokio::test]
async fn register_invalid_email() {
    let app = TestApp::new().await;

    let body = serde_json::json!({
        "email": "not-an-email",
        "password": "password123",
        "display_name": "Bob",
    });

    let req = Request::builder()
        .method(Method::POST)
        .uri("/api/auth/register")
        .header("content-type", "application/json")
        .body(Body::from(serde_json::to_vec(&body).unwrap()))
        .unwrap();

    let (status, _) = app.send(req).await;
    assert_eq!(status, StatusCode::BAD_REQUEST);

    app.cleanup().await;
}

#[tokio::test]
async fn register_short_password() {
    let app = TestApp::new().await;

    let body = serde_json::json!({
        "email": "short@example.com",
        "password": "short",
        "display_name": "Bob",
    });

    let req = Request::builder()
        .method(Method::POST)
        .uri("/api/auth/register")
        .header("content-type", "application/json")
        .body(Body::from(serde_json::to_vec(&body).unwrap()))
        .unwrap();

    let (status, _) = app.send(req).await;
    assert_eq!(status, StatusCode::BAD_REQUEST);

    app.cleanup().await;
}

#[tokio::test]
async fn register_empty_name() {
    let app = TestApp::new().await;

    let body = serde_json::json!({
        "email": "noname@example.com",
        "password": "password123",
        "display_name": "   ",
    });

    let req = Request::builder()
        .method(Method::POST)
        .uri("/api/auth/register")
        .header("content-type", "application/json")
        .body(Body::from(serde_json::to_vec(&body).unwrap()))
        .unwrap();

    let (status, _) = app.send(req).await;
    assert_eq!(status, StatusCode::BAD_REQUEST);

    app.cleanup().await;
}

#[tokio::test]
async fn login_success() {
    let app = TestApp::new().await;

    app.register_user("login@example.com", "password123", "Login User")
        .await;

    let body = serde_json::json!({
        "email": "login@example.com",
        "password": "password123",
    });

    let req = Request::builder()
        .method(Method::POST)
        .uri("/api/auth/login")
        .header("content-type", "application/json")
        .body(Body::from(serde_json::to_vec(&body).unwrap()))
        .unwrap();

    let (status, headers, json) = app.send_with_headers(req).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(json["email"], "login@example.com");

    let cookie = headers.get("set-cookie").expect("missing set-cookie");
    assert!(cookie.to_str().unwrap().contains("bominal_session="));

    app.cleanup().await;
}

#[tokio::test]
async fn login_wrong_password() {
    let app = TestApp::new().await;

    app.register_user("wrongpw@example.com", "password123", "User")
        .await;

    let body = serde_json::json!({
        "email": "wrongpw@example.com",
        "password": "wrongpassword",
    });

    let req = Request::builder()
        .method(Method::POST)
        .uri("/api/auth/login")
        .header("content-type", "application/json")
        .body(Body::from(serde_json::to_vec(&body).unwrap()))
        .unwrap();

    let (status, _) = app.send(req).await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);

    app.cleanup().await;
}

#[tokio::test]
async fn login_nonexistent() {
    let app = TestApp::new().await;

    let body = serde_json::json!({
        "email": "noexist@example.com",
        "password": "password123",
    });

    let req = Request::builder()
        .method(Method::POST)
        .uri("/api/auth/login")
        .header("content-type", "application/json")
        .body(Body::from(serde_json::to_vec(&body).unwrap()))
        .unwrap();

    let (status, _) = app.send(req).await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);

    app.cleanup().await;
}

#[tokio::test]
async fn me_valid_session() {
    let app = TestApp::new().await;

    let session = app
        .register_user("me@example.com", "password123", "Me User")
        .await;

    let req = app.authed_get("/api/auth/me", &session);
    let (status, json) = app.send(req).await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(json["email"], "me@example.com");
    assert_eq!(json["display_name"], "Me User");

    app.cleanup().await;
}

#[tokio::test]
async fn me_no_session() {
    let app = TestApp::new().await;

    let req = Request::builder()
        .method(Method::GET)
        .uri("/api/auth/me")
        .body(Body::empty())
        .unwrap();

    let (status, _) = app.send(req).await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);

    app.cleanup().await;
}

#[tokio::test]
async fn logout_clears_session() {
    let app = TestApp::new().await;

    let session = app
        .register_user("logout@example.com", "password123", "Logout")
        .await;

    // Logout
    let req = Request::builder()
        .method(Method::POST)
        .uri("/api/auth/logout")
        .header("cookie", format!("bominal_session={session}"))
        .body(Body::empty())
        .unwrap();

    let (status, _) = app.send(req).await;
    assert_eq!(status, StatusCode::OK);

    // /me should now be 401
    let req = app.authed_get("/api/auth/me", &session);
    let (status, _) = app.send(req).await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);

    app.cleanup().await;
}
