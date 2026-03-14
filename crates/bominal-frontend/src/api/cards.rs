//! Payment card server functions.

use leptos::prelude::*;
use uuid::Uuid;

pub use bominal_service::cards::CardInfo;

/// List all payment cards for the current user (masked).
#[server(prefix = "/sfn")]
pub async fn list_cards() -> Result<Vec<CardInfo>, ServerFnError> {
    let (pool, user_id) = super::tasks::require_auth().await?;
    bominal_service::cards::list(&pool, user_id)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))
}

/// Delete a payment card.
#[server(prefix = "/sfn")]
pub async fn delete_card(card_id: String) -> Result<(), ServerFnError> {
    let (pool, user_id) = super::tasks::require_auth().await?;
    let id = Uuid::parse_str(&card_id).map_err(|e| ServerFnError::new(format!("Bad ID: {e}")))?;

    bominal_service::cards::delete(&pool, id, user_id)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;

    Ok(())
}
