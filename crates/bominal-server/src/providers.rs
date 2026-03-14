//! Provider credential route handlers.
//!
//! - GET  /api/providers           — list credentials (masked)
//! - POST /api/providers           — add/update + verify credentials
//! - DELETE /api/providers/:provider — remove credentials

use axum::Json;
use axum::extract::{Path, State};
use serde::Deserialize;

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

/// GET /api/providers — list all provider credentials for the user.
pub async fn list_providers(
    user: AuthUser,
    State(state): State<SharedState>,
) -> Result<Json<Vec<bominal_service::providers::ProviderInfo>>, AppError> {
    let result = bominal_service::providers::list(&state.db, user.user_id).await?;
    Ok(Json(result))
}

/// POST /api/providers — add or update provider credentials.
///
/// Validates by attempting a provider login. Only saves if login succeeds.
pub async fn add_provider(
    user: AuthUser,
    State(state): State<SharedState>,
    Json(req): Json<AddProviderRequest>,
) -> Result<Json<bominal_service::providers::ProviderInfo>, AppError> {
    let result = bominal_service::providers::add(
        &state.db,
        user.user_id,
        &req.provider,
        &req.login_id,
        &req.password,
        &state.encryption_key,
    )
    .await?;

    Ok(Json(result))
}

/// DELETE /api/providers/:provider — remove provider credentials.
pub async fn delete_provider(
    user: AuthUser,
    State(state): State<SharedState>,
    Path(provider): Path<String>,
) -> Result<Json<serde_json::Value>, AppError> {
    bominal_service::providers::delete(&state.db, user.user_id, &provider).await?;
    Ok(Json(serde_json::json!({ "deleted": true })))
}

#[cfg(test)]
mod tests {
    #[test]
    fn mask_short_login() {
        assert_eq!(bominal_service::providers::mask_login_id("abc"), "***");
    }

    #[test]
    fn mask_email() {
        assert_eq!(
            bominal_service::providers::mask_login_id("user@example.com"),
            "use***********om"
        );
    }

    #[test]
    fn mask_phone() {
        assert_eq!(
            bominal_service::providers::mask_login_id("01012345678"),
            "010******78"
        );
    }
}
