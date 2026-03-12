mod common;

use axum::body::Body;
use axum::http::{Method, Request, StatusCode};
use uuid::Uuid;

use common::TestApp;

fn valid_card_body() -> serde_json::Value {
    serde_json::json!({
        "card_number": "ev:abc123:card_number_encrypted",
        "card_password": "ev:abc123:card_password_encrypted",
        "birthday": "ev:abc123:birthday_encrypted",
        "expire_date": "ev:abc123:expire_date_encrypted",
        "last_four": "3456",
        "card_type": "J",
        "label": "Test Card",
    })
}

#[tokio::test]
async fn add_card_success() {
    let app = TestApp::new().await;
    let session = app
        .register_user("card@example.com", "password123", "CardUser")
        .await;

    let req = app.authed_post("/api/cards", &session, &valid_card_body());
    let (status, json) = app.send(req).await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(json["last_four"], "3456");
    assert_eq!(json["card_type"], "J");
    assert_eq!(json["card_type_name"], "신용카드");
    assert_eq!(json["label"], "Test Card");
    assert!(json["id"].is_string());

    app.cleanup().await;
}

#[tokio::test]
async fn add_card_rejects_plaintext_card_number() {
    let app = TestApp::new().await;
    let session = app
        .register_user("badnum@example.com", "password123", "BadNum")
        .await;

    let mut body = valid_card_body();
    body["card_number"] = serde_json::json!("1234567890123456");

    let req = app.authed_post("/api/cards", &session, &body);
    let (status, json) = app.send(req).await;

    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert!(json["error"].as_str().unwrap().contains("encrypted"));

    app.cleanup().await;
}

#[tokio::test]
async fn add_card_rejects_plaintext_password() {
    let app = TestApp::new().await;
    let session = app
        .register_user("badpw@example.com", "password123", "BadPW")
        .await;

    let mut body = valid_card_body();
    body["card_password"] = serde_json::json!("12");

    let req = app.authed_post("/api/cards", &session, &body);
    let (status, json) = app.send(req).await;

    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert!(json["error"].as_str().unwrap().contains("encrypted"));

    app.cleanup().await;
}

#[tokio::test]
async fn add_card_rejects_plaintext_birthday() {
    let app = TestApp::new().await;
    let session = app
        .register_user("badbday@example.com", "password123", "BadBday")
        .await;

    let mut body = valid_card_body();
    body["birthday"] = serde_json::json!("900101");

    let req = app.authed_post("/api/cards", &session, &body);
    let (status, json) = app.send(req).await;

    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert!(json["error"].as_str().unwrap().contains("encrypted"));

    app.cleanup().await;
}

#[tokio::test]
async fn add_card_rejects_plaintext_expire_date() {
    let app = TestApp::new().await;
    let session = app
        .register_user("badexp@example.com", "password123", "BadExp")
        .await;

    let mut body = valid_card_body();
    body["expire_date"] = serde_json::json!("1228");

    let req = app.authed_post("/api/cards", &session, &body);
    let (status, json) = app.send(req).await;

    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert!(json["error"].as_str().unwrap().contains("encrypted"));

    app.cleanup().await;
}

#[tokio::test]
async fn add_card_rejects_bad_last_four() {
    let app = TestApp::new().await;
    let session = app
        .register_user("bad4@example.com", "password123", "Bad4")
        .await;

    let mut body = valid_card_body();
    body["last_four"] = serde_json::json!("abc");

    let req = app.authed_post("/api/cards", &session, &body);
    let (status, _) = app.send(req).await;

    assert_eq!(status, StatusCode::BAD_REQUEST);

    app.cleanup().await;
}

#[tokio::test]
async fn add_card_rejects_invalid_card_type() {
    let app = TestApp::new().await;
    let session = app
        .register_user("badct@example.com", "password123", "BadCT")
        .await;

    let mut body = valid_card_body();
    body["card_type"] = serde_json::json!("X");

    let req = app.authed_post("/api/cards", &session, &body);
    let (status, _) = app.send(req).await;

    assert_eq!(status, StatusCode::BAD_REQUEST);

    app.cleanup().await;
}

#[tokio::test]
async fn list_cards_empty() {
    let app = TestApp::new().await;
    let session = app
        .register_user("nocards@example.com", "password123", "NoCards")
        .await;

    let req = app.authed_get("/api/cards", &session);
    let (status, json) = app.send(req).await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(json.as_array().unwrap().len(), 0);

    app.cleanup().await;
}

#[tokio::test]
async fn list_cards_own_only() {
    let app = TestApp::new().await;

    // User A adds a card
    let session_a = app
        .register_user("carda@example.com", "password123", "A")
        .await;
    let req = app.authed_post("/api/cards", &session_a, &valid_card_body());
    let (status, _) = app.send(req).await;
    assert_eq!(status, StatusCode::OK);

    // User B should see empty
    let session_b = app
        .register_user("cardb@example.com", "password123", "B")
        .await;
    let req = app.authed_get("/api/cards", &session_b);
    let (status, json) = app.send(req).await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(json.as_array().unwrap().len(), 0);

    app.cleanup().await;
}

#[tokio::test]
async fn list_cards_no_encrypted_fields() {
    let app = TestApp::new().await;
    let session = app
        .register_user("noenc@example.com", "password123", "NoEnc")
        .await;

    let req = app.authed_post("/api/cards", &session, &valid_card_body());
    let (status, _) = app.send(req).await;
    assert_eq!(status, StatusCode::OK);

    let req = app.authed_get("/api/cards", &session);
    let (_, json) = app.send(req).await;

    let card = &json[0];
    assert!(card.get("encrypted_number").is_none());
    assert!(card.get("encrypted_password").is_none());
    assert!(card.get("encrypted_birthday").is_none());
    assert!(card.get("encrypted_expiry").is_none());
    assert!(card.get("card_number").is_none());
    assert!(card.get("card_password").is_none());

    app.cleanup().await;
}

#[tokio::test]
async fn update_card_label() {
    let app = TestApp::new().await;
    let session = app
        .register_user("upd@example.com", "password123", "Upd")
        .await;

    let req = app.authed_post("/api/cards", &session, &valid_card_body());
    let (_, created) = app.send(req).await;
    let card_id = created["id"].as_str().unwrap();

    let body = serde_json::json!({ "label": "Updated Label" });
    let req = app.authed_patch(&format!("/api/cards/{card_id}"), &session, &body);
    let (status, json) = app.send(req).await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(json["label"], "Updated Label");

    app.cleanup().await;
}

#[tokio::test]
async fn update_card_not_found() {
    let app = TestApp::new().await;
    let session = app
        .register_user("updnf@example.com", "password123", "UpdNF")
        .await;

    let fake_id = Uuid::new_v4();
    let body = serde_json::json!({ "label": "X" });
    let req = app.authed_patch(&format!("/api/cards/{fake_id}"), &session, &body);
    let (status, _) = app.send(req).await;

    assert_eq!(status, StatusCode::NOT_FOUND);

    app.cleanup().await;
}

#[tokio::test]
async fn delete_card_success() {
    let app = TestApp::new().await;
    let session = app
        .register_user("delcard@example.com", "password123", "Del")
        .await;

    let req = app.authed_post("/api/cards", &session, &valid_card_body());
    let (_, created) = app.send(req).await;
    let card_id = created["id"].as_str().unwrap();

    let req = app.authed_delete(&format!("/api/cards/{card_id}"), &session);
    let (status, json) = app.send(req).await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(json["deleted"], true);

    app.cleanup().await;
}

#[tokio::test]
async fn delete_card_other_user() {
    let app = TestApp::new().await;

    let session_a = app
        .register_user("owncard@example.com", "password123", "Own")
        .await;
    let req = app.authed_post("/api/cards", &session_a, &valid_card_body());
    let (_, created) = app.send(req).await;
    let card_id = created["id"].as_str().unwrap();

    let session_b = app
        .register_user("thief@example.com", "password123", "Thief")
        .await;
    let req = app.authed_delete(&format!("/api/cards/{card_id}"), &session_b);
    let (status, _) = app.send(req).await;

    assert_eq!(status, StatusCode::NOT_FOUND);

    app.cleanup().await;
}

#[tokio::test]
async fn unauthenticated_access() {
    let app = TestApp::new().await;

    let req = Request::builder()
        .method(Method::GET)
        .uri("/api/cards")
        .body(Body::empty())
        .unwrap();

    let (status, _) = app.send(req).await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);

    app.cleanup().await;
}
