//! Provider credential server functions.

use leptos::prelude::*;

pub use bominal_domain::dto::ProviderInfo;

/// List all provider credentials for the current user.
#[server(prefix = "/sfn")]
pub async fn list_providers() -> Result<Vec<ProviderInfo>, ServerFnError> {
    let (pool, user_id) = super::tasks::require_auth().await?;
    bominal_service::providers::list(&pool, user_id)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))
}

/// Add or update provider credentials (verifies via login first).
#[server(prefix = "/sfn")]
pub async fn add_provider(
    provider: String,
    login_id: String,
    password: String,
) -> Result<ProviderInfo, ServerFnError> {
    let (pool, user_id) = super::tasks::require_auth().await?;

    let key = use_context::<bominal_service::EncryptionKey>()
        .ok_or_else(|| ServerFnError::new("Server misconfigured"))?;

    bominal_service::providers::add(&pool, user_id, &provider, &login_id, &password, &key)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))
}

/// Delete provider credentials.
#[server(prefix = "/sfn")]
pub async fn delete_provider(provider: String) -> Result<(), ServerFnError> {
    let (pool, user_id) = super::tasks::require_auth().await?;

    bominal_service::providers::delete(&pool, user_id, &provider)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))
}
