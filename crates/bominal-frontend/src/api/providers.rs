//! Provider credential server functions.

use leptos::prelude::*;
use serde::{Deserialize, Serialize};

/// Provider credential info (password masked).
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ProviderInfo {
    pub provider: String,
    pub login_id: String,
    pub status: String,
    pub last_verified_at: Option<String>,
}

/// List all provider credentials for the current user.
#[server(prefix = "/sfn")]
pub async fn list_providers() -> Result<Vec<ProviderInfo>, ServerFnError> {
    let (pool, user_id) = super::tasks::require_auth().await?;

    let creds = bominal_db::provider::find_by_user(&pool, user_id)
        .await
        .map_err(|e| ServerFnError::new(format!("Database error: {e}")))?;

    Ok(creds
        .iter()
        .map(|c| ProviderInfo {
            provider: c.provider.clone(),
            login_id: mask_login_id(&c.login_id),
            status: c.status.clone(),
            last_verified_at: c.last_verified_at.map(|t| t.to_rfc3339()),
        })
        .collect())
}

/// Add or update provider credentials (verifies via login first).
#[server(prefix = "/sfn")]
pub async fn add_provider(
    provider: String,
    login_id: String,
    password: String,
) -> Result<ProviderInfo, ServerFnError> {
    let (pool, user_id) = super::tasks::require_auth().await?;

    if provider != "SRT" && provider != "KTX" {
        return Err(ServerFnError::new(format!("Invalid provider: {provider}")));
    }
    if login_id.is_empty() || password.is_empty() {
        return Err(ServerFnError::new("Login ID and password are required"));
    }

    // Verify credentials by attempting login
    verify_login(&provider, &login_id, &password).await?;

    // Encrypt password for storage
    let encryption_key = use_context::<bominal_domain::crypto::encryption::EncryptionKey>()
        .ok_or_else(|| ServerFnError::new("Server misconfigured"))?;

    let encrypted_password =
        bominal_domain::crypto::encryption::encrypt(&encryption_key, &password)
            .map_err(|e| ServerFnError::new(format!("Encryption error: {e}")))?;

    let row = bominal_db::provider::upsert_credential(
        &pool,
        user_id,
        &provider,
        &login_id,
        &encrypted_password,
        "valid",
    )
    .await
    .map_err(|e| ServerFnError::new(format!("Database error: {e}")))?;

    Ok(ProviderInfo {
        provider: row.provider,
        login_id: mask_login_id(&row.login_id),
        status: row.status,
        last_verified_at: row.last_verified_at.map(|t| t.to_rfc3339()),
    })
}

/// Delete provider credentials.
#[server(prefix = "/sfn")]
pub async fn delete_provider(provider: String) -> Result<(), ServerFnError> {
    let (pool, user_id) = super::tasks::require_auth().await?;

    let deleted = bominal_db::provider::delete_credential(&pool, user_id, &provider)
        .await
        .map_err(|e| ServerFnError::new(format!("Database error: {e}")))?;

    if !deleted {
        return Err(ServerFnError::new(format!(
            "No {provider} credentials found"
        )));
    }

    Ok(())
}

// ── Helpers ────────────────────────────────────────────────────────

fn mask_login_id(login_id: &str) -> String {
    if login_id.len() <= 5 {
        return "*".repeat(login_id.len());
    }
    let first = &login_id[..3];
    let last = &login_id[login_id.len() - 2..];
    let masked_len = login_id.len() - 5;
    format!("{first}{}{last}", "*".repeat(masked_len))
}

async fn verify_login(provider: &str, login_id: &str, password: &str) -> Result<(), ServerFnError> {
    match provider {
        "SRT" => {
            let mut client = bominal_provider::srt::SrtClient::new();
            client
                .login(login_id, password)
                .await
                .map_err(|e| ServerFnError::new(format!("SRT login failed: {e}")))?;
            let _ = client.logout().await;
        }
        "KTX" => {
            let mut client = bominal_provider::ktx::KtxClient::new();
            client
                .login(login_id, password)
                .await
                .map_err(|e| ServerFnError::new(format!("KTX login failed: {e}")))?;
            let _ = client.logout().await;
        }
        _ => return Err(ServerFnError::new(format!("Invalid provider: {provider}"))),
    }
    Ok(())
}
