//! Payment card server functions.

use leptos::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Card info (masked — never exposes raw encrypted fields).
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CardInfo {
    pub id: Uuid,
    pub label: String,
    pub last_four: String,
    pub card_type: String,
    pub card_type_name: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

/// List all payment cards for the current user (masked).
#[server(prefix = "/sfn")]
pub async fn list_cards() -> Result<Vec<CardInfo>, ServerFnError> {
    let (pool, user_id) = super::tasks::require_auth().await?;

    let rows = bominal_db::card::find_by_user(&pool, user_id)
        .await
        .map_err(|e| ServerFnError::new(format!("Database error: {e}")))?;

    Ok(rows.iter().map(row_to_info).collect())
}

/// Delete a payment card.
#[server(prefix = "/sfn")]
pub async fn delete_card(card_id: String) -> Result<(), ServerFnError> {
    let (pool, user_id) = super::tasks::require_auth().await?;
    let id = Uuid::parse_str(&card_id).map_err(|e| ServerFnError::new(format!("Bad ID: {e}")))?;

    let deleted = bominal_db::card::delete_card(&pool, id, user_id)
        .await
        .map_err(|e| ServerFnError::new(format!("Database error: {e}")))?;

    if !deleted {
        return Err(ServerFnError::new("Card not found"));
    }

    Ok(())
}

// ── Helpers ────────────────────────────────────────────────────────

fn row_to_info(row: &bominal_db::card::PaymentCardRow) -> CardInfo {
    CardInfo {
        id: row.id,
        label: row.label.clone(),
        last_four: row.last_four.clone(),
        card_type: row.card_type.clone(),
        card_type_name: card_type_name(&row.card_type).to_string(),
        created_at: row.created_at,
    }
}

fn card_type_name(code: &str) -> &str {
    match code {
        "J" => "신용카드",
        "S" => "체크카드",
        _ => "기타",
    }
}
