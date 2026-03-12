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

/// Add a payment card.
#[server(prefix = "/sfn")]
pub async fn add_card(
    card_number: String,
    card_password: String,
    birthday: String,
    expire_date: String,
    label: Option<String>,
    card_type: Option<String>,
) -> Result<CardInfo, ServerFnError> {
    let (pool, user_id) = super::tasks::require_auth().await?;

    // Validate
    if card_number.len() < 15
        || card_number.len() > 16
        || !card_number.chars().all(|c| c.is_ascii_digit())
    {
        return Err(ServerFnError::new("Card number must be 15-16 digits"));
    }
    if card_password.len() != 2 || !card_password.chars().all(|c| c.is_ascii_digit()) {
        return Err(ServerFnError::new("Card password must be 2 digits"));
    }
    if birthday.len() != 6 || !birthday.chars().all(|c| c.is_ascii_digit()) {
        return Err(ServerFnError::new("Birthday must be YYMMDD (6 digits)"));
    }
    if expire_date.len() != 4 || !expire_date.chars().all(|c| c.is_ascii_digit()) {
        return Err(ServerFnError::new("Expiry must be MMYY (4 digits)"));
    }

    let last_four = &card_number[card_number.len() - 4..];
    let ct = card_type.as_deref().unwrap_or("J");
    let lbl = label.as_deref().unwrap_or("My Card");

    // Encrypt sensitive fields
    let key = use_context::<bominal_domain::crypto::encryption::EncryptionKey>()
        .ok_or_else(|| ServerFnError::new("Server misconfigured"))?;

    use bominal_domain::crypto::encryption::encrypt;
    let enc_number = encrypt(&key, &card_number).map_err(|e| ServerFnError::new(format!("{e}")))?;
    let enc_password =
        encrypt(&key, &card_password).map_err(|e| ServerFnError::new(format!("{e}")))?;
    let enc_birthday =
        encrypt(&key, &birthday).map_err(|e| ServerFnError::new(format!("{e}")))?;
    let enc_expiry =
        encrypt(&key, &expire_date).map_err(|e| ServerFnError::new(format!("{e}")))?;

    let row = bominal_db::card::create_card(
        &pool, user_id, lbl, &enc_number, &enc_password, &enc_birthday, &enc_expiry, ct, last_four,
    )
    .await
    .map_err(|e| ServerFnError::new(format!("Database error: {e}")))?;

    Ok(row_to_info(&row))
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
