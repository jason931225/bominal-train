use std::{sync::Arc, time::Duration};

use axum::{
    Router,
    body::to_bytes,
    http::{StatusCode, header, request::Builder as RequestBuilder},
};
use bominal_shared::config::{
    AppConfig, EvervaultConfig, PasskeyConfig, PasskeyProvider, RedisConfig, RuntimeSchedule,
};
use tower::util::ServiceExt;

#[allow(dead_code)]
#[path = "../src/main.rs"]
mod api_main;

const INTERNAL_SERVICE_TOKEN_HEADER: &str = "x-internal-service-token";
const TEST_INTERNAL_IDENTITY_SECRET_FALLBACK: &str = "test-internal-secret";
const DEFAULT_INTERNAL_ISSUER: &str = "bominal-internal";
const INTERNAL_IDENTITY_SECRET_ENV: &str = "INTERNAL_IDENTITY_SECRET";

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
        database_url: "".to_string(),
        redis: RedisConfig {
            url: "redis://127.0.0.1:6379".to_string(),
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
        Err(err) => panic!("failed to construct test AppState: {err}"),
    };

    api_main::build_router(state)
}

fn request_builder(path: &str) -> RequestBuilder {
    axum::http::Request::builder()
        .method("PUT")
        .uri(path)
        .header(header::CONTENT_TYPE, "application/json")
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

fn assert_envelope_fields(body: &serde_json::Value, expected_code: &str) {
    assert_eq!(body["code"], expected_code);
    assert!(body["message"].is_string());
    assert!(body["request_id"].is_string());
}

fn expected_issuer() -> String {
    std::env::var("INTERNAL_IDENTITY_ISSUER")
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| DEFAULT_INTERNAL_ISSUER.to_string())
}

fn expected_internal_identity_secret() -> String {
    std::env::var(INTERNAL_IDENTITY_SECRET_ENV)
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| TEST_INTERNAL_IDENTITY_SECRET_FALLBACK.to_string())
}

fn build_valid_internal_service_token() -> String {
    let now = chrono::Utc::now().timestamp();
    let payload = serde_json::json!({
        "iss": expected_issuer(),
        "sub": "svc-auth",
        "aud": "internal-api",
        "iat": now - 10,
        "exp": now + 300,
        "jti": "internal-auth-contract-test",
        "role": "service-internal",
        "scope": "read:internal"
    });

    build_hs256_jwt(payload, &expected_internal_identity_secret())
}

fn build_hs256_jwt(payload: serde_json::Value, secret: &str) -> String {
    let header = serde_json::json!({ "alg": "HS256", "typ": "JWT" });
    let header_segment = encode_base64url(
        &serde_json::to_vec(&header).expect("failed to serialize internal token header"),
    );
    let payload_segment = encode_base64url(
        &serde_json::to_vec(&payload).expect("failed to serialize internal token payload"),
    );

    let signing_input = format!("{header_segment}.{payload_segment}");
    let signature = hmac_sha256(secret.as_bytes(), signing_input.as_bytes());
    let signature_segment = encode_base64url(&signature);

    format!("{header_segment}.{payload_segment}.{signature_segment}")
}

fn encode_base64url(bytes: &[u8]) -> String {
    const ALPHABET: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-_";

    let mut out = String::new();
    let mut index = 0;

    while index + 3 <= bytes.len() {
        let chunk = ((bytes[index] as u32) << 16)
            | ((bytes[index + 1] as u32) << 8)
            | (bytes[index + 2] as u32);
        out.push(ALPHABET[((chunk >> 18) & 0x3f) as usize] as char);
        out.push(ALPHABET[((chunk >> 12) & 0x3f) as usize] as char);
        out.push(ALPHABET[((chunk >> 6) & 0x3f) as usize] as char);
        out.push(ALPHABET[(chunk & 0x3f) as usize] as char);
        index += 3;
    }

    let rem = bytes.len() - index;
    if rem == 1 {
        let chunk = (bytes[index] as u32) << 16;
        out.push(ALPHABET[((chunk >> 18) & 0x3f) as usize] as char);
        out.push(ALPHABET[((chunk >> 12) & 0x3f) as usize] as char);
    } else if rem == 2 {
        let chunk = ((bytes[index] as u32) << 16) | ((bytes[index + 1] as u32) << 8);
        out.push(ALPHABET[((chunk >> 18) & 0x3f) as usize] as char);
        out.push(ALPHABET[((chunk >> 12) & 0x3f) as usize] as char);
        out.push(ALPHABET[((chunk >> 6) & 0x3f) as usize] as char);
    }

    out
}

fn hmac_sha256(key: &[u8], message: &[u8]) -> [u8; 32] {
    const BLOCK_SIZE: usize = 64;

    let mut key_block = [0u8; BLOCK_SIZE];
    if key.len() > BLOCK_SIZE {
        let hashed_key = sha256(key);
        key_block[..hashed_key.len()].copy_from_slice(&hashed_key);
    } else {
        key_block[..key.len()].copy_from_slice(key);
    }

    let mut inner_pad = [0x36u8; BLOCK_SIZE];
    let mut outer_pad = [0x5cu8; BLOCK_SIZE];
    for index in 0..BLOCK_SIZE {
        inner_pad[index] ^= key_block[index];
        outer_pad[index] ^= key_block[index];
    }

    let mut inner = Vec::with_capacity(BLOCK_SIZE + message.len());
    inner.extend_from_slice(&inner_pad);
    inner.extend_from_slice(message);
    let inner_hash = sha256(&inner);

    let mut outer = Vec::with_capacity(BLOCK_SIZE + inner_hash.len());
    outer.extend_from_slice(&outer_pad);
    outer.extend_from_slice(&inner_hash);

    sha256(&outer)
}

fn sha256(input: &[u8]) -> [u8; 32] {
    const INITIAL_STATE: [u32; 8] = [
        0x6a09e667, 0xbb67ae85, 0x3c6ef372, 0xa54ff53a, 0x510e527f, 0x9b05688c, 0x1f83d9ab,
        0x5be0cd19,
    ];
    const ROUND_CONSTANTS: [u32; 64] = [
        0x428a2f98, 0x71374491, 0xb5c0fbcf, 0xe9b5dba5, 0x3956c25b, 0x59f111f1, 0x923f82a4,
        0xab1c5ed5, 0xd807aa98, 0x12835b01, 0x243185be, 0x550c7dc3, 0x72be5d74, 0x80deb1fe,
        0x9bdc06a7, 0xc19bf174, 0xe49b69c1, 0xefbe4786, 0x0fc19dc6, 0x240ca1cc, 0x2de92c6f,
        0x4a7484aa, 0x5cb0a9dc, 0x76f988da, 0x983e5152, 0xa831c66d, 0xb00327c8, 0xbf597fc7,
        0xc6e00bf3, 0xd5a79147, 0x06ca6351, 0x14292967, 0x27b70a85, 0x2e1b2138, 0x4d2c6dfc,
        0x53380d13, 0x650a7354, 0x766a0abb, 0x81c2c92e, 0x92722c85, 0xa2bfe8a1, 0xa81a664b,
        0xc24b8b70, 0xc76c51a3, 0xd192e819, 0xd6990624, 0xf40e3585, 0x106aa070, 0x19a4c116,
        0x1e376c08, 0x2748774c, 0x34b0bcb5, 0x391c0cb3, 0x4ed8aa4a, 0x5b9cca4f, 0x682e6ff3,
        0x748f82ee, 0x78a5636f, 0x84c87814, 0x8cc70208, 0x90befffa, 0xa4506ceb, 0xbef9a3f7,
        0xc67178f2,
    ];

    let mut state = INITIAL_STATE;
    let mut padded = Vec::with_capacity(((input.len() + 9).div_ceil(64)) * 64);
    padded.extend_from_slice(input);
    padded.push(0x80);
    while (padded.len() + 8) % 64 != 0 {
        padded.push(0);
    }
    padded.extend_from_slice(&((input.len() as u64) * 8).to_be_bytes());

    for chunk in padded.chunks_exact(64) {
        let mut message_schedule = [0u32; 64];
        for (index, word) in message_schedule.iter_mut().take(16).enumerate() {
            let offset = index * 4;
            *word = u32::from_be_bytes([
                chunk[offset],
                chunk[offset + 1],
                chunk[offset + 2],
                chunk[offset + 3],
            ]);
        }

        for index in 16..64 {
            let s0 = message_schedule[index - 15].rotate_right(7)
                ^ message_schedule[index - 15].rotate_right(18)
                ^ (message_schedule[index - 15] >> 3);
            let s1 = message_schedule[index - 2].rotate_right(17)
                ^ message_schedule[index - 2].rotate_right(19)
                ^ (message_schedule[index - 2] >> 10);
            message_schedule[index] = message_schedule[index - 16]
                .wrapping_add(s0)
                .wrapping_add(message_schedule[index - 7])
                .wrapping_add(s1);
        }

        let mut a = state[0];
        let mut b = state[1];
        let mut c = state[2];
        let mut d = state[3];
        let mut e = state[4];
        let mut f = state[5];
        let mut g = state[6];
        let mut h = state[7];

        for index in 0..64 {
            let sum1 = e.rotate_right(6) ^ e.rotate_right(11) ^ e.rotate_right(25);
            let choice = (e & f) ^ ((!e) & g);
            let temp1 = h
                .wrapping_add(sum1)
                .wrapping_add(choice)
                .wrapping_add(ROUND_CONSTANTS[index])
                .wrapping_add(message_schedule[index]);
            let sum0 = a.rotate_right(2) ^ a.rotate_right(13) ^ a.rotate_right(22);
            let majority = (a & b) ^ (a & c) ^ (b & c);
            let temp2 = sum0.wrapping_add(majority);

            h = g;
            g = f;
            f = e;
            e = d.wrapping_add(temp1);
            d = c;
            c = b;
            b = a;
            a = temp1.wrapping_add(temp2);
        }

        state[0] = state[0].wrapping_add(a);
        state[1] = state[1].wrapping_add(b);
        state[2] = state[2].wrapping_add(c);
        state[3] = state[3].wrapping_add(d);
        state[4] = state[4].wrapping_add(e);
        state[5] = state[5].wrapping_add(f);
        state[6] = state[6].wrapping_add(g);
        state[7] = state[7].wrapping_add(h);
    }

    let mut digest = [0u8; 32];
    for (index, value) in state.iter().enumerate() {
        let offset = index * 4;
        digest[offset..offset + 4].copy_from_slice(&value.to_be_bytes());
    }

    digest
}

#[tokio::test]
async fn missing_internal_service_token_returns_unauthorized_envelope() {
    let app = build_test_app().await;

    let request = match request_builder("/internal/v1/providers/srt/credentials")
        .header("x-request-id", "missing-service-token")
        .body(axum::body::Body::from(
            serde_json::json!({
                "identity_ciphertext": "identity",
                "password_ciphertext": "password"
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
    assert_envelope_fields(&body, "unauthorized");
    assert_eq!(body["message"], "missing internal service token");
    assert_eq!(body["request_id"], "missing-service-token");
}

#[tokio::test]
async fn malformed_internal_service_token_returns_unauthorized_envelope() {
    let app = build_test_app().await;

    let request = match request_builder("/internal/v1/providers/srt/credentials")
        .header(INTERNAL_SERVICE_TOKEN_HEADER, "not-a-jwt")
        .header("x-request-id", "malformed-service-token")
        .body(axum::body::Body::from(
            serde_json::json!({
                "identity_ciphertext": "identity",
                "password_ciphertext": "password"
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
    assert_envelope_fields(&body, "unauthorized");
    assert_eq!(body["message"], "invalid internal service token");
    assert_eq!(body["request_id"], "malformed-service-token");
}

#[tokio::test]
async fn valid_internal_service_token_reaches_handler_and_returns_invalid_request() {
    let app = build_test_app().await;
    let service_token = build_valid_internal_service_token();

    let request = match request_builder("/internal/v1/providers/srt/credentials")
        .header(INTERNAL_SERVICE_TOKEN_HEADER, service_token)
        .header("x-request-id", "downstream-validation")
        .body(axum::body::Body::from(
            serde_json::json!({
                "identity_ciphertext": "",
                "password_ciphertext": ""
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
    assert_envelope_fields(&body, "invalid_request");
    assert_eq!(body["message"], "invalid provider credentials payload");
    assert_eq!(body["request_id"], "downstream-validation");
}

#[tokio::test]
async fn valid_internal_service_token_supports_ktx_credentials_route() {
    let app = build_test_app().await;
    let service_token = build_valid_internal_service_token();

    let request = match request_builder("/internal/v1/providers/ktx/credentials")
        .header(INTERNAL_SERVICE_TOKEN_HEADER, service_token)
        .header("x-request-id", "ktx-credentials-validation")
        .body(axum::body::Body::from(
            serde_json::json!({
                "identity_ciphertext": "",
                "password_ciphertext": ""
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
    assert_envelope_fields(&body, "invalid_request");
    assert_eq!(body["message"], "invalid provider credentials payload");
    assert_eq!(body["request_id"], "ktx-credentials-validation");
}

#[tokio::test]
async fn valid_internal_service_token_supports_ktx_payment_method_route() {
    let app = build_test_app().await;
    let service_token = build_valid_internal_service_token();

    let request = match request_builder("/internal/v1/providers/ktx/payment-method")
        .header(INTERNAL_SERVICE_TOKEN_HEADER, service_token)
        .header("x-request-id", "ktx-payment-validation")
        .body(axum::body::Body::from(
            serde_json::json!({
                "pan_ciphertext": "",
                "expiry_month_ciphertext": "",
                "expiry_year_ciphertext": "",
                "birth_or_business_number_ciphertext": "",
                "card_password_two_digits_ciphertext": ""
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
    assert_envelope_fields(&body, "invalid_request");
    assert_eq!(body["message"], "invalid payment payload");
    assert_eq!(body["request_id"], "ktx-payment-validation");
}

#[tokio::test]
async fn provider_job_events_route_rejects_unknown_and_invalid_query_params() {
    let app = build_test_app().await;
    let service_token = build_valid_internal_service_token();

    let since_id_request = match axum::http::Request::builder()
        .method("GET")
        .uri("/internal/v1/provider-jobs/job-1/events?since_id=10")
        .header(INTERNAL_SERVICE_TOKEN_HEADER, service_token.clone())
        .header("x-request-id", "provider-events-since-id")
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
    assert_envelope_fields(&since_id_body, "invalid_request");
    assert_eq!(since_id_body["request_id"], "provider-events-since-id");

    let limit_request = match axum::http::Request::builder()
        .method("GET")
        .uri("/internal/v1/provider-jobs/job-1/events?limit=abc")
        .header(INTERNAL_SERVICE_TOKEN_HEADER, service_token)
        .header("x-request-id", "provider-events-invalid-limit")
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
    assert_envelope_fields(&limit_body, "invalid_request");
    assert_eq!(limit_body["request_id"], "provider-events-invalid-limit");
}

#[tokio::test]
async fn compatibility_alias_route_is_disabled_without_internal_debug_mode() {
    let app = build_test_app().await;
    let service_token = build_valid_internal_service_token();

    let request = match request_builder("/api/internal/providers/ktx/credentials")
        .header(INTERNAL_SERVICE_TOKEN_HEADER, service_token)
        .body(axum::body::Body::from(
            serde_json::json!({
                "identity_ciphertext": "identity",
                "password_ciphertext": "password"
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

    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}
