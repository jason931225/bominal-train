//! WebAuthn passkey handlers — register and login ceremonies.

use axum::Json;
use axum::extract::State;
use axum::http::HeaderMap;
use axum::http::header::SET_COOKIE;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::error::AppError;
use crate::extractors::AuthUser;
use crate::state::SharedState;

// ── Registration ─────────────────────────────────────────────────

#[derive(Debug, Serialize)]
pub struct RegisterStartResponse {
    pub challenge: serde_json::Value,
}

/// POST /api/auth/passkey/register/start
pub async fn register_start(
    user: AuthUser,
    State(state): State<SharedState>,
) -> Result<Json<RegisterStartResponse>, AppError> {
    // For now, return a placeholder that the frontend can use to start the ceremony.
    // Full WebAuthn integration requires webauthn-rs Webauthn instance initialization.
    let challenge_id = Uuid::new_v4().to_string();

    // Store challenge in DB for verification in register/finish
    bominal_db::passkey::store_challenge(&state.db, user.user_id, &challenge_id)
        .await
        .map_err(|e| AppError::Internal(e.into()))?;

    Ok(Json(RegisterStartResponse {
        challenge: serde_json::json!({
            "challenge_id": challenge_id,
            "rp": { "name": "Bominal", "id": "bominal.com" },
            "user": {
                "id": user.user_id.to_string(),
                "name": user.email,
                "displayName": user.display_name,
            },
            "pubKeyCredParams": [
                { "type": "public-key", "alg": -7 },
                { "type": "public-key", "alg": -257 },
            ],
            "timeout": 60000,
            "attestation": "none",
            "authenticatorSelection": {
                "residentKey": "preferred",
                "userVerification": "preferred",
            },
        }),
    }))
}

#[derive(Debug, Deserialize)]
pub struct RegisterFinishRequest {
    pub challenge_id: String,
    pub credential: serde_json::Value,
    pub label: Option<String>,
}

/// POST /api/auth/passkey/register/finish
pub async fn register_finish(
    user: AuthUser,
    State(state): State<SharedState>,
    Json(req): Json<RegisterFinishRequest>,
) -> Result<Json<serde_json::Value>, AppError> {
    // Verify challenge exists and is not expired
    let _challenge =
        bominal_db::passkey::verify_challenge(&state.db, user.user_id, &req.challenge_id)
            .await
            .map_err(|e| AppError::Internal(e.into()))?
            .ok_or_else(|| AppError::BadRequest("Invalid or expired challenge".to_string()))?;

    // Extract credential id from the client response
    let credential_id = req
        .credential
        .get("id")
        .and_then(|v| v.as_str())
        .ok_or_else(|| AppError::BadRequest("Missing credential id".to_string()))?;

    let public_key = serde_json::to_string(&req.credential)
        .map_err(|e| AppError::Internal(anyhow::anyhow!("{e}")))?;

    let label = req.label.as_deref().unwrap_or("My Passkey");

    bominal_db::passkey::store_credential(
        &state.db,
        user.user_id,
        credential_id,
        &public_key,
        label,
    )
    .await
    .map_err(|e| AppError::Internal(e.into()))?;

    Ok(Json(serde_json::json!({ "registered": true })))
}

// ── Login ────────────────────────────────────────────────────────

#[derive(Debug, Serialize)]
pub struct LoginStartResponse {
    pub challenge: serde_json::Value,
}

/// POST /api/auth/passkey/login/start
pub async fn login_start(
    State(state): State<SharedState>,
) -> Result<Json<LoginStartResponse>, AppError> {
    let challenge_id = Uuid::new_v4().to_string();

    // For login start we don't know the user yet, so we use an empty allow list
    // (discoverable credentials / resident keys).
    bominal_db::passkey::store_login_challenge(&state.db, &challenge_id)
        .await
        .map_err(|e| AppError::Internal(e.into()))?;

    Ok(Json(LoginStartResponse {
        challenge: serde_json::json!({
            "challenge_id": challenge_id,
            "rpId": "bominal.com",
            "timeout": 60000,
            "userVerification": "preferred",
        }),
    }))
}

#[derive(Debug, Deserialize)]
pub struct LoginFinishRequest {
    pub challenge_id: String,
    pub credential: serde_json::Value,
}

/// POST /api/auth/passkey/login/finish
pub async fn login_finish(
    State(state): State<SharedState>,
    Json(req): Json<LoginFinishRequest>,
) -> Result<(HeaderMap, Json<serde_json::Value>), AppError> {
    // Verify login challenge exists and is not expired
    let _challenge = bominal_db::passkey::verify_login_challenge(&state.db, &req.challenge_id)
        .await
        .map_err(|e| AppError::Internal(e.into()))?
        .ok_or_else(|| AppError::BadRequest("Invalid or expired challenge".to_string()))?;

    // Look up the credential by its id
    let credential_id = req
        .credential
        .get("id")
        .and_then(|v| v.as_str())
        .ok_or_else(|| AppError::BadRequest("Missing credential id".to_string()))?;

    let cred = bominal_db::passkey::find_credential_by_id(&state.db, credential_id)
        .await
        .map_err(|e| AppError::Internal(e.into()))?
        .ok_or(AppError::Unauthorized)?;

    // Resolve the owning user
    let user = bominal_db::user::find_by_id(&state.db, cred.user_id)
        .await
        .map_err(|e| AppError::Internal(e.into()))?
        .ok_or(AppError::Unauthorized)?;

    // Create a new session for the authenticated user
    let session_id = Uuid::new_v4().to_string();
    let expires_at = chrono::Utc::now() + chrono::Duration::hours(24);

    bominal_db::session::create_session(&state.db, &session_id, user.id, expires_at)
        .await
        .map_err(|e| AppError::Internal(e.into()))?;

    let domain_attr = crate::auth::cookie_domain_attr(&state.app_base_url);
    let mut headers = HeaderMap::new();
    headers.insert(
        SET_COOKIE,
        format!(
            "bominal_session={}; Path=/; HttpOnly; SameSite=Lax; Secure; Max-Age={}{}",
            session_id,
            24 * 3600,
            domain_attr
        )
        .parse()
        .unwrap(),
    );

    Ok((
        headers,
        Json(serde_json::json!({
            "user_id": user.id,
            "email": user.email,
            "display_name": user.display_name,
        })),
    ))
}
