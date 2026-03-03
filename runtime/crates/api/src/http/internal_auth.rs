use std::sync::Arc;

use axum::{
    extract::{Request, State},
    http::HeaderMap,
    middleware::Next,
    response::{IntoResponse, Response},
};
use bominal_shared::error::{ApiError, ApiErrorCode, ApiErrorStatus};

use super::super::{AppState, request_id_from_headers};

const INTERNAL_SERVICE_TOKEN_HEADER: &str = "x-internal-service-token";
const INTERNAL_SERVICE_AUDIENCE: &str = "internal-api";
const INTERNAL_IDENTITY_SECRET_ENV: &str = "INTERNAL_IDENTITY_SECRET";
const DEFAULT_INTERNAL_ISSUER: &str = "bominal-internal";
const TEST_INTERNAL_IDENTITY_SECRET_FALLBACK: &str = "test-internal-secret";
const MAX_FUTURE_IAT_SKEW_SECONDS: i64 = 60;

#[derive(Debug, serde::Deserialize)]
struct InternalServiceTokenClaims {
    iss: String,
    sub: String,
    aud: String,
    iat: i64,
    exp: i64,
    jti: String,
    role: Option<String>,
    scope: Option<String>,
}

#[derive(Debug, serde::Deserialize)]
struct JwtHeader {
    alg: String,
}

#[derive(Debug)]
enum InternalAuthError {
    MissingServiceToken,
    InvalidServiceToken,
}

pub(super) async fn require_service_jwt(
    State(state): State<Arc<AppState>>,
    request: Request,
    next: Next,
) -> Response {
    let request_id = request_id_from_headers(request.headers());

    match validate_service_token(state.as_ref(), request.headers()) {
        Ok(()) => next.run(request).await,
        Err(InternalAuthError::MissingServiceToken) => ApiError::new(
            ApiErrorStatus::Unauthorized,
            ApiErrorCode::Unauthorized,
            "missing internal service token",
            request_id,
        )
        .into_response(),
        Err(InternalAuthError::InvalidServiceToken) => ApiError::new(
            ApiErrorStatus::Unauthorized,
            ApiErrorCode::Unauthorized,
            "invalid internal service token",
            request_id,
        )
        .into_response(),
    }
}

pub(super) fn compatibility_aliases_enabled(state: &AppState) -> bool {
    let app_env = normalize_app_env(&state.config.app_env);
    let non_production = !matches!(app_env.as_str(), "production" | "prod");
    let internal_debug_mode = internal_debug_mode_enabled(&app_env);

    non_production && internal_debug_mode
}

fn validate_service_token(state: &AppState, headers: &HeaderMap) -> Result<(), InternalAuthError> {
    let token = headers
        .get(INTERNAL_SERVICE_TOKEN_HEADER)
        .and_then(|value| value.to_str().ok())
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .ok_or(InternalAuthError::MissingServiceToken)?;

    let claims = parse_and_verify_claims(state, token)?;
    validate_claims(state, &claims)
}

fn parse_and_verify_claims(
    state: &AppState,
    token: &str,
) -> Result<InternalServiceTokenClaims, InternalAuthError> {
    let mut segments = token.split('.');
    let header_segment = segments
        .next()
        .ok_or(InternalAuthError::InvalidServiceToken)?;
    let payload_segment = segments
        .next()
        .ok_or(InternalAuthError::InvalidServiceToken)?;
    let signature_segment = segments
        .next()
        .ok_or(InternalAuthError::InvalidServiceToken)?;

    if segments.next().is_some() {
        return Err(InternalAuthError::InvalidServiceToken);
    }

    let header_bytes = decode_base64url(header_segment)?;
    let header = serde_json::from_slice::<JwtHeader>(&header_bytes)
        .map_err(|_| InternalAuthError::InvalidServiceToken)?;
    if header.alg.trim() != "HS256" {
        return Err(InternalAuthError::InvalidServiceToken);
    }

    let secret =
        resolve_internal_identity_secret(state).ok_or(InternalAuthError::InvalidServiceToken)?;

    let signature = decode_base64url(signature_segment)?;
    let signing_input = format!("{header_segment}.{payload_segment}");
    let expected_signature = hmac_sha256(secret.as_bytes(), signing_input.as_bytes());
    if !constant_time_equals(signature.as_slice(), expected_signature.as_slice()) {
        return Err(InternalAuthError::InvalidServiceToken);
    }

    let payload_bytes = decode_base64url(payload_segment)?;

    serde_json::from_slice::<InternalServiceTokenClaims>(&payload_bytes)
        .map_err(|_| InternalAuthError::InvalidServiceToken)
}

fn resolve_internal_identity_secret(state: &AppState) -> Option<String> {
    if let Some(secret) = std::env::var(INTERNAL_IDENTITY_SECRET_ENV)
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
    {
        return Some(secret);
    }

    let app_env = normalize_app_env(&state.config.app_env);

    #[cfg(test)]
    let fallback_enabled = true;
    #[cfg(not(test))]
    let fallback_enabled = false;

    if fallback_enabled || app_env == "test" {
        return Some(TEST_INTERNAL_IDENTITY_SECRET_FALLBACK.to_string());
    }

    None
}

fn validate_claims(
    state: &AppState,
    claims: &InternalServiceTokenClaims,
) -> Result<(), InternalAuthError> {
    let expected_issuer = std::env::var("INTERNAL_IDENTITY_ISSUER")
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| DEFAULT_INTERNAL_ISSUER.to_string());

    if claims.iss.trim() != expected_issuer
        || claims.sub.trim().is_empty()
        || claims.aud.trim() != INTERNAL_SERVICE_AUDIENCE
        || claims.jti.trim().is_empty()
    {
        return Err(InternalAuthError::InvalidServiceToken);
    }

    let now = chrono::Utc::now().timestamp();
    if claims.exp <= now
        || claims.exp <= claims.iat
        || claims.iat > now + MAX_FUTURE_IAT_SKEW_SECONDS
    {
        return Err(InternalAuthError::InvalidServiceToken);
    }

    if !role_is_internal(claims) {
        return Err(InternalAuthError::InvalidServiceToken);
    }

    let _ = state;

    Ok(())
}

fn role_is_internal(claims: &InternalServiceTokenClaims) -> bool {
    let role_is_internal = claims
        .role
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(|value| value.contains("internal"))
        .unwrap_or(true);

    let scope_is_internal = claims
        .scope
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(|value| value.contains("internal"))
        .unwrap_or(true);

    role_is_internal && scope_is_internal
}

fn normalize_app_env(app_env: &str) -> String {
    app_env.trim().to_ascii_lowercase()
}

fn internal_debug_mode_enabled(app_env: &str) -> bool {
    let env_debug_enabled = std::env::var("INTERNAL_DEBUG_MODE")
        .ok()
        .and_then(|value| parse_bool(&value))
        .unwrap_or(false);

    env_debug_enabled
        || matches!(
            app_env,
            "debug" | "dev-debug" | "local-debug" | "test-debug"
        )
}

fn parse_bool(raw: &str) -> Option<bool> {
    match raw.trim().to_ascii_lowercase().as_str() {
        "1" | "true" | "yes" | "on" => Some(true),
        "0" | "false" | "no" | "off" => Some(false),
        _ => None,
    }
}

fn decode_base64url(input: &str) -> Result<Vec<u8>, InternalAuthError> {
    let mut out = Vec::with_capacity((input.len() * 3) / 4 + 3);
    let mut buffer = 0u32;
    let mut bits = 0usize;

    for byte in input.bytes() {
        let sextet = match byte {
            b'A'..=b'Z' => (byte - b'A') as u32,
            b'a'..=b'z' => (byte - b'a' + 26) as u32,
            b'0'..=b'9' => (byte - b'0' + 52) as u32,
            b'-' => 62,
            b'_' => 63,
            _ => return Err(InternalAuthError::InvalidServiceToken),
        };

        buffer = (buffer << 6) | sextet;
        bits += 6;

        if bits >= 8 {
            bits -= 8;
            out.push(((buffer >> bits) & 0xff) as u8);
            buffer &= (1u32 << bits) - 1;
        }
    }

    if bits > 0 && (buffer & ((1u32 << bits) - 1)) != 0 {
        return Err(InternalAuthError::InvalidServiceToken);
    }

    Ok(out)
}

fn constant_time_equals(lhs: &[u8], rhs: &[u8]) -> bool {
    if lhs.len() != rhs.len() {
        return false;
    }

    let mut diff = 0u8;
    for (left, right) in lhs.iter().zip(rhs.iter()) {
        diff |= left ^ right;
    }

    diff == 0
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

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use bominal_shared::config::{
        AppConfig, EvervaultConfig, PasskeyConfig, PasskeyProvider, RedisConfig, RuntimeSchedule,
    };
    use serde_json::json;

    use super::*;

    fn test_state(app_env: &str) -> AppState {
        AppState {
            config: AppConfig {
                app_env: app_env.to_string(),
                app_host: "127.0.0.1".to_string(),
                app_port: 8080,
                log_json: false,
                session_cookie_name: "bominal_session".to_string(),
                session_ttl_seconds: 3600,
                session_secret: "test-session-secret".to_string(),
                invite_base_url: "http://127.0.0.1:8000".to_string(),
                database_url: String::new(),
                redis: RedisConfig {
                    url: "redis://127.0.0.1:6379".to_string(),
                    queue_key: "queue".to_string(),
                    queue_dlq_key: "queue:dlq".to_string(),
                    lease_prefix: "lease".to_string(),
                    rate_limit_prefix: "rate".to_string(),
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
            },
            db_pool: None,
            redis_client: None,
            http_client: reqwest::Client::new(),
            webauthn: None,
        }
    }

    fn expected_issuer() -> String {
        std::env::var("INTERNAL_IDENTITY_ISSUER")
            .ok()
            .map(|value| value.trim().to_string())
            .filter(|value| !value.is_empty())
            .unwrap_or_else(|| DEFAULT_INTERNAL_ISSUER.to_string())
    }

    fn build_claims(now: i64) -> InternalServiceTokenClaims {
        InternalServiceTokenClaims {
            iss: expected_issuer(),
            sub: "svc-auth".to_string(),
            aud: INTERNAL_SERVICE_AUDIENCE.to_string(),
            iat: now - 10,
            exp: now + 300,
            jti: "jti-1".to_string(),
            role: Some("service-internal".to_string()),
            scope: Some("read:internal".to_string()),
        }
    }

    fn encode_base64url(bytes: &[u8]) -> String {
        const ALPHABET: &[u8; 64] =
            b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-_";

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

    fn build_token(header_alg: &str, payload: serde_json::Value, secret: &str) -> String {
        let header = json!({ "alg": header_alg });
        let header_segment =
            encode_base64url(&serde_json::to_vec(&header).expect("header serialization"));
        let payload_segment =
            encode_base64url(&serde_json::to_vec(&payload).expect("payload serialization"));

        let signing_input = format!("{header_segment}.{payload_segment}");
        let signature = hmac_sha256(secret.as_bytes(), signing_input.as_bytes());
        let signature_segment = encode_base64url(&signature);

        format!("{header_segment}.{payload_segment}.{signature_segment}")
    }

    #[test]
    fn parse_bool_matrix() {
        let truthy_cases = ["1", "true", "TRUE", " yes ", "On"];
        for raw in truthy_cases {
            assert_eq!(
                parse_bool(raw),
                Some(true),
                "expected truthy parse for {raw:?}"
            );
        }

        let falsy_cases = ["0", "false", "FALSE", " no ", "OFF"];
        for raw in falsy_cases {
            assert_eq!(
                parse_bool(raw),
                Some(false),
                "expected falsy parse for {raw:?}"
            );
        }

        let invalid_cases = ["", "2", "maybe", "enable", "  "];
        for raw in invalid_cases {
            assert_eq!(parse_bool(raw), None, "expected None parse for {raw:?}");
        }
    }

    #[test]
    fn decode_base64url_rejects_invalid_characters() {
        let invalid_inputs = ["Zm9v=", "Zm9v.", "Zm9v+", "abc$"];
        for input in invalid_inputs {
            assert!(
                matches!(
                    decode_base64url(input),
                    Err(InternalAuthError::InvalidServiceToken)
                ),
                "expected invalid token for input {input:?}"
            );
        }
    }

    #[test]
    fn token_validation_rejects_wrong_alg() {
        let state = test_state("test");
        let now = chrono::Utc::now().timestamp();
        let claims = build_claims(now);
        let payload = json!({
            "iss": claims.iss,
            "sub": claims.sub,
            "aud": claims.aud,
            "iat": claims.iat,
            "exp": claims.exp,
            "jti": claims.jti,
            "role": claims.role,
            "scope": claims.scope,
        });

        let token = build_token("HS512", payload, TEST_INTERNAL_IDENTITY_SECRET_FALLBACK);
        assert!(matches!(
            parse_and_verify_claims(&state, &token),
            Err(InternalAuthError::InvalidServiceToken)
        ));
    }

    #[test]
    fn token_validation_rejects_bad_signature() {
        let state = test_state("test");
        let now = chrono::Utc::now().timestamp();
        let claims = build_claims(now);
        let payload = json!({
            "iss": claims.iss,
            "sub": claims.sub,
            "aud": claims.aud,
            "iat": claims.iat,
            "exp": claims.exp,
            "jti": claims.jti,
            "role": claims.role,
            "scope": claims.scope,
        });

        let token = build_token("HS256", payload, "wrong-secret");
        assert!(matches!(
            parse_and_verify_claims(&state, &token),
            Err(InternalAuthError::InvalidServiceToken)
        ));
    }

    #[test]
    fn role_scope_internal_checks_fail_closed() {
        let now = chrono::Utc::now().timestamp();
        let state = test_state("test");

        let mut claims = build_claims(now);
        assert!(role_is_internal(&claims));
        assert!(validate_claims(&state, &claims).is_ok());

        claims.role = Some("customer-facing".to_string());
        assert!(!role_is_internal(&claims));
        assert!(matches!(
            validate_claims(&state, &claims),
            Err(InternalAuthError::InvalidServiceToken)
        ));

        claims = build_claims(now);
        claims.scope = Some("read:public".to_string());
        assert!(!role_is_internal(&claims));
        assert!(matches!(
            validate_claims(&state, &claims),
            Err(InternalAuthError::InvalidServiceToken)
        ));
    }

    #[test]
    fn iat_exp_skew_handling() {
        let now = chrono::Utc::now().timestamp();
        let state = test_state("test");

        let mut expired = build_claims(now);
        expired.exp = now;
        assert!(matches!(
            validate_claims(&state, &expired),
            Err(InternalAuthError::InvalidServiceToken)
        ));

        let mut exp_before_iat = build_claims(now);
        exp_before_iat.iat = now + 10;
        exp_before_iat.exp = now + 10;
        assert!(matches!(
            validate_claims(&state, &exp_before_iat),
            Err(InternalAuthError::InvalidServiceToken)
        ));

        let mut future_iat_beyond_skew = build_claims(now);
        future_iat_beyond_skew.iat = now + MAX_FUTURE_IAT_SKEW_SECONDS + 5;
        future_iat_beyond_skew.exp = future_iat_beyond_skew.iat + 10;
        assert!(matches!(
            validate_claims(&state, &future_iat_beyond_skew),
            Err(InternalAuthError::InvalidServiceToken)
        ));

        let mut future_iat_at_skew_boundary = build_claims(now);
        future_iat_at_skew_boundary.iat = now + MAX_FUTURE_IAT_SKEW_SECONDS;
        future_iat_at_skew_boundary.exp = future_iat_at_skew_boundary.iat + 10;
        assert!(validate_claims(&state, &future_iat_at_skew_boundary).is_ok());
    }

    #[test]
    fn compatibility_aliases_enabled_requires_non_prod_and_debug() {
        let production_debug = test_state("production-debug");
        assert!(!compatibility_aliases_enabled(&production_debug));

        let dev_debug = test_state("dev-debug");
        assert!(compatibility_aliases_enabled(&dev_debug));

        let prod = test_state("prod");
        assert!(!compatibility_aliases_enabled(&prod));
    }
}
