//! Provider credential route handlers.
//!
//! - GET  /api/providers           — list credentials (masked)
//! - POST /api/providers           — add/update + verify credentials
//! - DELETE /api/providers/:provider — remove credentials

use axum::extract::{Path, State};
use axum::Json;
use serde::{Deserialize, Serialize};

use crate::error::AppError;
use crate::extractors::AuthUser;
use crate::state::SharedState;

/// Request to add/update provider credentials.
#[derive(Debug, Deserialize)]
pub struct AddProviderRequest {
    pub provider: String,
    pub login_id: String,
    pub password: String,
}

/// Public credential info (password masked).
#[derive(Debug, Serialize)]
pub struct ProviderCredentialResponse {
    pub provider: String,
    pub login_id: String,
    pub status: String,
    pub last_verified_at: Option<String>,
}

/// GET /api/providers — list all provider credentials for the user.
pub async fn list_providers(
    user: AuthUser,
    State(state): State<SharedState>,
) -> Result<Json<Vec<ProviderCredentialResponse>>, AppError> {
    let creds = bominal_db::provider::find_by_user(&state.db, user.user_id)
        .await
        .map_err(|e| AppError::Internal(e.into()))?;

    let response: Vec<ProviderCredentialResponse> = creds
        .iter()
        .map(|c| ProviderCredentialResponse {
            provider: c.provider.clone(),
            login_id: mask_login_id(&c.login_id),
            status: c.status.clone(),
            last_verified_at: c.last_verified_at.map(|t| t.to_rfc3339()),
        })
        .collect();

    Ok(Json(response))
}

/// POST /api/providers — add or update provider credentials.
///
/// Validates by attempting a provider login. Only saves if login succeeds.
pub async fn add_provider(
    user: AuthUser,
    State(state): State<SharedState>,
    Json(req): Json<AddProviderRequest>,
) -> Result<Json<ProviderCredentialResponse>, AppError> {
    let provider = validate_provider(&req.provider)?;

    if req.login_id.is_empty() || req.password.is_empty() {
        return Err(AppError::BadRequest(
            "Login ID and password are required".to_string(),
        ));
    }

    // Attempt provider login to verify credentials
    let verify_result = verify_provider_login(provider, &req.login_id, &req.password).await;

    let status = match &verify_result {
        Ok(()) => "valid",
        Err(_) => {
            // Do NOT store invalid credentials
            return Err(AppError::BadRequest(
                verify_result
                    .unwrap_err()
                    .to_string(),
            ));
        }
    };

    // Encrypt password for storage with AES-256-GCM
    let encrypted_password = bominal_domain::crypto::encryption::encrypt(
        &state.encryption_key,
        &req.password,
    )
    .map_err(|e| AppError::Internal(e.into()))?;

    let row = bominal_db::provider::upsert_credential(
        &state.db,
        user.user_id,
        provider,
        &req.login_id,
        &encrypted_password,
        status,
    )
    .await
    .map_err(|e| AppError::Internal(e.into()))?;

    Ok(Json(ProviderCredentialResponse {
        provider: row.provider,
        login_id: mask_login_id(&row.login_id),
        status: row.status,
        last_verified_at: row.last_verified_at.map(|t| t.to_rfc3339()),
    }))
}

/// DELETE /api/providers/:provider — remove provider credentials.
pub async fn delete_provider(
    user: AuthUser,
    State(state): State<SharedState>,
    Path(provider): Path<String>,
) -> Result<Json<serde_json::Value>, AppError> {
    let provider = validate_provider(&provider)?;

    let deleted = bominal_db::provider::delete_credential(&state.db, user.user_id, provider)
        .await
        .map_err(|e| AppError::Internal(e.into()))?;

    if !deleted {
        return Err(AppError::NotFound(format!(
            "No {provider} credentials found"
        )));
    }

    Ok(Json(serde_json::json!({ "deleted": true })))
}

// ── Helpers ──────────────────────────────────────────────────────────

/// Validate provider name is "SRT" or "KTX".
fn validate_provider(provider: &str) -> Result<&str, AppError> {
    match provider {
        "SRT" | "KTX" => Ok(provider),
        _ => Err(AppError::BadRequest(format!(
            "Invalid provider: {provider}. Must be SRT or KTX"
        ))),
    }
}

/// Mask login ID for display (show first 3 + last 2 chars).
fn mask_login_id(login_id: &str) -> String {
    if login_id.len() <= 5 {
        return "*".repeat(login_id.len());
    }
    let first = &login_id[..3];
    let last = &login_id[login_id.len() - 2..];
    let masked_len = login_id.len() - 5;
    format!("{first}{}{last}", "*".repeat(masked_len))
}

/// Attempt provider login to verify credentials.
async fn verify_provider_login(
    provider: &str,
    login_id: &str,
    password: &str,
) -> Result<(), bominal_provider::types::ProviderError> {
    match provider {
        "SRT" => {
            let mut client = bominal_provider::srt::SrtClient::new();
            client.login(login_id, password).await?;
            // Immediately logout to free the session
            let _ = client.logout().await;
            Ok(())
        }
        "KTX" => {
            let mut client = bominal_provider::ktx::KtxClient::new();
            client.login(login_id, password).await?;
            let _ = client.logout().await;
            Ok(())
        }
        _ => unreachable!("provider validated before this call"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mask_short_login() {
        assert_eq!(mask_login_id("abc"), "***");
    }

    #[test]
    fn mask_email() {
        // "user@example.com" = 16 chars, first 3 + (16-5=11 stars) + last 2
        assert_eq!(mask_login_id("user@example.com"), "use***********om");
    }

    #[test]
    fn mask_phone() {
        // "01012345678" = 11 chars, first 3 + (11-5=6 stars) + last 2
        assert_eq!(mask_login_id("01012345678"), "010******78");
    }

    #[test]
    fn validate_provider_valid() {
        assert!(validate_provider("SRT").is_ok());
        assert!(validate_provider("KTX").is_ok());
    }

    #[test]
    fn validate_provider_invalid() {
        assert!(validate_provider("KORAIL").is_err());
    }
}
