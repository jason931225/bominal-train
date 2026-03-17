mod common;

use axum::body::Body;
use axum::http::{Method, Request, StatusCode};

use common::TestApp;

#[tokio::test]
async fn list_srt_stations() {
    let app = TestApp::new().await;

    let req = Request::builder()
        .method(Method::GET)
        .uri("/api/stations/SRT")
        .body(Body::empty())
        .unwrap();

    let (status, json) = app.send(req).await;

    assert_eq!(status, StatusCode::OK);
    let stations = json.as_array().unwrap();
    assert!(!stations.is_empty(), "SRT stations should not be empty");
    assert!(stations[0]["name_ko"].is_string());
    assert!(stations[0]["name_en"].is_string());
    assert!(stations[0]["name_ja"].is_string());

    app.cleanup().await;
}

#[tokio::test]
async fn list_ktx_stations() {
    let app = TestApp::new().await;

    let req = Request::builder()
        .method(Method::GET)
        .uri("/api/stations/KTX")
        .body(Body::empty())
        .unwrap();

    let (status, json) = app.send(req).await;

    assert_eq!(status, StatusCode::OK);
    let stations = json.as_array().unwrap();
    assert!(!stations.is_empty(), "KTX stations should not be empty");

    app.cleanup().await;
}

// ── Station suggest endpoint ────────────────────────────────────────

#[tokio::test]
async fn suggest_korean_exact() {
    let app = TestApp::new().await;

    let req = Request::builder()
        .method(Method::GET)
        .uri("/api/stations/SRT/suggest?q=%EB%B6%80%EC%82%B0") // 부산
        .body(Body::empty())
        .unwrap();

    let (status, json) = app.send(req).await;

    assert_eq!(status, StatusCode::OK);
    let matches = json["matches"].as_array().unwrap();
    assert!(!matches.is_empty());
    assert_eq!(matches[0]["name_ko"].as_str().unwrap(), "부산");

    app.cleanup().await;
}

#[tokio::test]
async fn suggest_english_alias() {
    let app = TestApp::new().await;

    let req = Request::builder()
        .method(Method::GET)
        .uri("/api/stations/KTX/suggest?q=busan")
        .body(Body::empty())
        .unwrap();

    let (status, json) = app.send(req).await;

    assert_eq!(status, StatusCode::OK);
    let matches = json["matches"].as_array().unwrap();
    assert!(!matches.is_empty());
    assert_eq!(matches[0]["name_ko"].as_str().unwrap(), "부산");

    app.cleanup().await;
}

#[tokio::test]
async fn suggest_katakana() {
    let app = TestApp::new().await;

    // プサン = %E3%83%97%E3%82%B5%E3%83%B3
    let req = Request::builder()
        .method(Method::GET)
        .uri("/api/stations/SRT/suggest?q=%E3%83%97%E3%82%B5%E3%83%B3")
        .body(Body::empty())
        .unwrap();

    let (status, json) = app.send(req).await;

    assert_eq!(status, StatusCode::OK);
    let matches = json["matches"].as_array().unwrap();
    assert!(!matches.is_empty());
    assert_eq!(matches[0]["name_ko"].as_str().unwrap(), "부산");

    app.cleanup().await;
}

#[tokio::test]
async fn suggest_qwerty_keyboard() {
    let app = TestApp::new().await;

    // tntj → 수서 (QWERTY 2-set decoding)
    let req = Request::builder()
        .method(Method::GET)
        .uri("/api/stations/SRT/suggest?q=tntj")
        .body(Body::empty())
        .unwrap();

    let (status, json) = app.send(req).await;

    assert_eq!(status, StatusCode::OK);
    let matches = json["matches"].as_array().unwrap();
    assert!(!matches.is_empty());
    assert_eq!(matches[0]["name_ko"].as_str().unwrap(), "수서");

    app.cleanup().await;
}

#[tokio::test]
async fn suggest_chosung() {
    let app = TestApp::new().await;

    // ㅂㅅ = %E3%85%82%E3%85%85 → 부산
    let req = Request::builder()
        .method(Method::GET)
        .uri("/api/stations/SRT/suggest?q=%E3%85%82%E3%85%85")
        .body(Body::empty())
        .unwrap();

    let (status, json) = app.send(req).await;

    assert_eq!(status, StatusCode::OK);
    let matches = json["matches"].as_array().unwrap();
    assert!(!matches.is_empty());
    assert_eq!(matches[0]["name_ko"].as_str().unwrap(), "부산");

    app.cleanup().await;
}

#[tokio::test]
async fn suggest_empty_query() {
    let app = TestApp::new().await;

    let req = Request::builder()
        .method(Method::GET)
        .uri("/api/stations/SRT/suggest?q=")
        .body(Body::empty())
        .unwrap();

    let (status, json) = app.send(req).await;

    assert_eq!(status, StatusCode::OK);
    let matches = json["matches"].as_array().unwrap();
    assert!(matches.is_empty());

    app.cleanup().await;
}

#[tokio::test]
async fn suggest_invalid_provider() {
    let app = TestApp::new().await;

    let req = Request::builder()
        .method(Method::GET)
        .uri("/api/stations/KORAIL/suggest?q=test")
        .body(Body::empty())
        .unwrap();

    let (status, _) = app.send(req).await;

    assert_eq!(status, StatusCode::BAD_REQUEST);

    app.cleanup().await;
}

#[tokio::test]
async fn suggest_submit_mode_autocorrect() {
    let app = TestApp::new().await;

    // tntj in submit mode → should autocorrect to 수서
    let req = Request::builder()
        .method(Method::GET)
        .uri("/api/stations/SRT/suggest?q=tntj&mode=submit")
        .body(Body::empty())
        .unwrap();

    let (status, json) = app.send(req).await;

    assert_eq!(status, StatusCode::OK);
    assert!(json["autocorrect_applied"].as_bool().unwrap_or(false));
    assert_eq!(json["corrected_query"].as_str().unwrap_or(""), "수서");

    app.cleanup().await;
}

#[tokio::test]
async fn suggest_response_shape() {
    let app = TestApp::new().await;

    let req = Request::builder()
        .method(Method::GET)
        .uri("/api/stations/SRT/suggest?q=busan&limit=3")
        .body(Body::empty())
        .unwrap();

    let (status, json) = app.send(req).await;

    assert_eq!(status, StatusCode::OK);
    assert!(json["matches"].is_array());
    assert!(json.get("corrected_query").is_some());
    assert!(json.get("autocorrect_applied").is_some());

    let first = &json["matches"][0];
    assert!(first["name_ko"].is_string());
    assert!(first["name_en"].is_string());
    assert!(first["name_ja"].is_string());
    assert!(first["score"].is_number());
    assert!(first["confidence"].is_number());
    assert!(first["source"].is_string());

    app.cleanup().await;
}

#[tokio::test]
async fn list_stations_invalid_provider() {
    let app = TestApp::new().await;

    let req = Request::builder()
        .method(Method::GET)
        .uri("/api/stations/KORAIL")
        .body(Body::empty())
        .unwrap();

    let (status, _) = app.send(req).await;

    assert_eq!(status, StatusCode::BAD_REQUEST);

    app.cleanup().await;
}
