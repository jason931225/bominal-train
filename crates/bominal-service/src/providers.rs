//! Provider service — manage SRT/KTX credentials (add, list, delete, verify).

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::error::ServiceError;
use crate::{DbPool, EncryptionKey};

/// Provider credential info (password masked).
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ProviderInfo {
    pub provider: String,
    pub login_id: String,
    pub status: String,
    pub last_verified_at: Option<String>,
}

/// List all provider credentials for a user.
pub async fn list(db: &DbPool, user_id: Uuid) -> Result<Vec<ProviderInfo>, ServiceError> {
    let creds = bominal_db::provider::find_by_user(db, user_id).await?;

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
pub async fn add(
    db: &DbPool,
    user_id: Uuid,
    provider: &str,
    login_id: &str,
    password: &str,
    encryption_key: &EncryptionKey,
) -> Result<ProviderInfo, ServiceError> {
    crate::tasks::validate_provider(provider)?;

    if login_id.is_empty() || password.is_empty() {
        return Err(ServiceError::validation(
            "Login ID and password are required",
        ));
    }

    // Verify credentials by attempting login
    verify_login(provider, login_id, password).await?;

    // Encrypt password for storage with AES-256-GCM
    let encrypted_password = bominal_domain::crypto::encryption::encrypt(encryption_key, password)
        .map_err(|e| ServiceError::Crypto(e.to_string()))?;

    let row = bominal_db::provider::upsert_credential(
        db,
        user_id,
        provider,
        login_id,
        &encrypted_password,
        "valid",
    )
    .await?;

    Ok(ProviderInfo {
        provider: row.provider,
        login_id: mask_login_id(&row.login_id),
        status: row.status,
        last_verified_at: row.last_verified_at.map(|t| t.to_rfc3339()),
    })
}

/// Delete provider credentials.
pub async fn delete(
    db: &DbPool,
    user_id: Uuid,
    provider: &str,
) -> Result<(), ServiceError> {
    crate::tasks::validate_provider(provider)?;

    let deleted = bominal_db::provider::delete_credential(db, user_id, provider).await?;

    if !deleted {
        return Err(ServiceError::not_found(format!(
            "No {provider} credentials found"
        )));
    }

    Ok(())
}

/// Verify provider credentials by attempting a login/logout cycle.
pub async fn verify_login(
    provider: &str,
    login_id: &str,
    password: &str,
) -> Result<(), ServiceError> {
    match provider {
        "SRT" => {
            let mut client = bominal_provider::srt::SrtClient::new();
            client.login(login_id, password).await?;
            let _ = client.logout().await;
        }
        "KTX" => {
            let mut client = bominal_provider::ktx::KtxClient::new();
            client.login(login_id, password).await?;
            let _ = client.logout().await;
        }
        _ => return Err(ServiceError::validation(format!("Invalid provider: {provider}"))),
    }
    Ok(())
}

/// Mask login ID for display (show first 3 + last 2 chars).
pub fn mask_login_id(login_id: &str) -> String {
    if login_id.len() <= 5 {
        return "*".repeat(login_id.len());
    }
    let first = &login_id[..3];
    let last = &login_id[login_id.len() - 2..];
    let masked_len = login_id.len() - 5;
    format!("{first}{}{last}", "*".repeat(masked_len))
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
        assert_eq!(mask_login_id("user@example.com"), "use***********om");
    }

    #[test]
    fn mask_phone() {
        assert_eq!(mask_login_id("01012345678"), "010******78");
    }
}
