//! WebAuthn passkey handlers — register and login ceremonies.
//!
//! Uses `webauthn-rs` for proper challenge generation and signature verification.

use axum::Json;
use axum::extract::State;
use axum::http::HeaderMap;
use axum::http::header::SET_COOKIE;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use webauthn_rs::prelude::*;

use crate::error::AppError;
use crate::extractors::AuthUser;
use crate::state::SharedState;

// ── Registration ─────────────────────────────────────────────────

#[derive(Debug, Serialize)]
pub struct RegisterStartResponse {
    pub challenge_id: String,
    pub options: CreationChallengeResponse,
}

/// POST /api/auth/passkey/register/start
///
/// Initiates a WebAuthn passkey registration ceremony. The caller must be
/// authenticated (adding a passkey to an existing account).
pub async fn register_start(
    user: AuthUser,
    State(state): State<SharedState>,
) -> Result<Json<RegisterStartResponse>, AppError> {
    // Load any existing passkeys so the authenticator can exclude them.
    let existing_rows = bominal_db::passkey::find_credentials_by_user(&state.db, user.user_id)
        .await
        .map_err(|e| AppError::Internal(e.into()))?;

    let existing_passkeys: Vec<Passkey> = existing_rows
        .iter()
        .filter_map(|r| serde_json::from_str(&r.public_key).ok())
        .collect();

    let exclude_creds = if existing_passkeys.is_empty() {
        None
    } else {
        Some(
            existing_passkeys
                .iter()
                .map(|pk| pk.cred_id().clone())
                .collect(),
        )
    };

    // Use webauthn-rs Uuid (same underlying crate)
    let wa_user_id = user.user_id;

    let (ccr, reg_state) = state
        .webauthn
        .start_passkey_registration(wa_user_id, &user.email, &user.display_name, exclude_creds)
        .map_err(|e| AppError::Internal(anyhow::anyhow!("WebAuthn register start: {e}")))?;

    // Serialize the ceremony state for storage
    let state_json = serde_json::to_string(&reg_state)
        .map_err(|e| AppError::Internal(anyhow::anyhow!("{e}")))?;

    let challenge_id = Uuid::new_v4().to_string();

    bominal_db::passkey::store_challenge(&state.db, user.user_id, &challenge_id, &state_json)
        .await
        .map_err(|e| AppError::Internal(e.into()))?;

    Ok(Json(RegisterStartResponse {
        challenge_id,
        options: ccr,
    }))
}

#[derive(Debug, Deserialize)]
pub struct RegisterFinishRequest {
    pub challenge_id: String,
    pub credential: RegisterPublicKeyCredential,
    pub label: Option<String>,
}

/// POST /api/auth/passkey/register/finish
///
/// Completes the registration ceremony by verifying the attestation.
pub async fn register_finish(
    user: AuthUser,
    State(state): State<SharedState>,
    Json(req): Json<RegisterFinishRequest>,
) -> Result<Json<serde_json::Value>, AppError> {
    // Load and consume the stored ceremony state
    let challenge =
        bominal_db::passkey::verify_challenge(&state.db, user.user_id, &req.challenge_id)
            .await
            .map_err(|e| AppError::Internal(e.into()))?
            .ok_or_else(|| AppError::BadRequest("Invalid or expired challenge".to_string()))?;

    let reg_state: PasskeyRegistration = serde_json::from_str(
        challenge
            .state
            .as_deref()
            .ok_or_else(|| AppError::Internal(anyhow::anyhow!("Missing ceremony state")))?,
    )
    .map_err(|e| AppError::Internal(anyhow::anyhow!("Bad ceremony state: {e}")))?;

    // Verify the attestation and extract the passkey
    let passkey = state
        .webauthn
        .finish_passkey_registration(&req.credential, &reg_state)
        .map_err(|e| AppError::BadRequest(format!("Registration verification failed: {e}")))?;

    // Serialize the Passkey for DB storage.
    // cred_id() returns Base64UrlSafeData — serialize via serde to get base64url string.
    let cred_id_value = serde_json::to_value(passkey.cred_id())
        .map_err(|e| AppError::Internal(anyhow::anyhow!("{e}")))?;
    let credential_id = cred_id_value
        .as_str()
        .ok_or_else(|| AppError::Internal(anyhow::anyhow!("cred_id not a string")))?
        .to_string();
    let public_key =
        serde_json::to_string(&passkey).map_err(|e| AppError::Internal(anyhow::anyhow!("{e}")))?;

    let label = req.label.as_deref().unwrap_or("My Passkey");

    bominal_db::passkey::store_credential(
        &state.db,
        user.user_id,
        &credential_id,
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
    pub challenge_id: String,
    pub options: RequestChallengeResponse,
}

/// POST /api/auth/passkey/login/start
///
/// Initiates a discoverable authentication ceremony (no user identity required).
pub async fn login_start(
    State(state): State<SharedState>,
) -> Result<Json<LoginStartResponse>, AppError> {
    let (rcr, auth_state) = state
        .webauthn
        .start_discoverable_authentication()
        .map_err(|e| AppError::Internal(anyhow::anyhow!("WebAuthn login start: {e}")))?;

    let state_json = serde_json::to_string(&auth_state)
        .map_err(|e| AppError::Internal(anyhow::anyhow!("{e}")))?;

    let challenge_id = Uuid::new_v4().to_string();

    bominal_db::passkey::store_login_challenge(&state.db, &challenge_id, &state_json)
        .await
        .map_err(|e| AppError::Internal(e.into()))?;

    Ok(Json(LoginStartResponse {
        challenge_id,
        options: rcr,
    }))
}

#[derive(Debug, Deserialize)]
pub struct LoginFinishRequest {
    pub challenge_id: String,
    pub credential: PublicKeyCredential,
}

/// POST /api/auth/passkey/login/finish
///
/// Completes the discoverable authentication ceremony by verifying the
/// assertion signature, then creates a session for the authenticated user.
pub async fn login_finish(
    State(state): State<SharedState>,
    Json(req): Json<LoginFinishRequest>,
) -> Result<(HeaderMap, Json<serde_json::Value>), AppError> {
    // Load and consume the stored ceremony state
    let challenge = bominal_db::passkey::verify_login_challenge(&state.db, &req.challenge_id)
        .await
        .map_err(|e| AppError::Internal(e.into()))?
        .ok_or_else(|| AppError::BadRequest("Invalid or expired challenge".to_string()))?;

    let auth_state: DiscoverableAuthentication = serde_json::from_str(
        challenge
            .state
            .as_deref()
            .ok_or_else(|| AppError::Internal(anyhow::anyhow!("Missing ceremony state")))?,
    )
    .map_err(|e| AppError::Internal(anyhow::anyhow!("Bad ceremony state: {e}")))?;

    // Extract the user handle from the assertion to identify the user.
    // The browser sends back the user ID that was stored during registration.
    let user_handle = req
        .credential
        .response
        .user_handle
        .as_ref()
        .ok_or_else(|| AppError::BadRequest("Missing userHandle in assertion".to_string()))?;

    let user_id = Uuid::from_slice(&user_handle[..])
        .map_err(|_| AppError::BadRequest("Invalid userHandle".to_string()))?;

    // Load the user's stored passkeys
    let cred_rows = bominal_db::passkey::find_credentials_by_user(&state.db, user_id)
        .await
        .map_err(|e| AppError::Internal(e.into()))?;

    if cred_rows.is_empty() {
        return Err(AppError::Unauthorized);
    }

    let passkeys: Vec<Passkey> = cred_rows
        .iter()
        .filter_map(|r| serde_json::from_str(&r.public_key).ok())
        .collect();

    // Build DiscoverableKey list for verification
    let creds: Vec<DiscoverableKey> = passkeys.into_iter().map(DiscoverableKey::from).collect();

    // Verify the assertion signature
    let _auth_result = state
        .webauthn
        .finish_discoverable_authentication(&req.credential, auth_state, &creds)
        .map_err(|e| AppError::BadRequest(format!("Authentication verification failed: {e}")))?;

    // Resolve the user
    let user = bominal_db::user::find_by_id(&state.db, user_id)
        .await
        .map_err(|e| AppError::Internal(e.into()))?
        .ok_or(AppError::Unauthorized)?;

    // Create a new session
    let session_id = Uuid::new_v4().to_string();
    let expires_at = chrono::Utc::now() + chrono::Duration::hours(24);

    bominal_db::session::create_session(&state.db, &session_id, user.id, expires_at)
        .await
        .map_err(|e| AppError::Internal(e.into()))?;

    let domain_attr = crate::auth::cookie_domain_attr(&state.app_base_url);
    let secure_attr = crate::auth::cookie_secure_attr(&state.app_base_url);
    let mut headers = HeaderMap::new();
    headers.insert(
        SET_COOKIE,
        format!(
            "bominal_session={}; Path=/; HttpOnly; SameSite=Lax{}; Max-Age={}{}",
            session_id,
            secure_attr,
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
