//! Card service — list, add, update, delete payment cards.

use uuid::Uuid;

use crate::DbPool;
use crate::error::ServiceError;

pub use bominal_domain::dto::CardInfo;

/// List all payment cards for a user (masked).
pub async fn list(db: &DbPool, user_id: Uuid) -> Result<Vec<CardInfo>, ServiceError> {
    let rows = bominal_db::card::find_by_user(db, user_id).await?;
    Ok(rows.iter().map(row_to_info).collect())
}

/// Add a new payment card. All sensitive fields must be Evervault-encrypted (`ev:` prefix).
#[allow(clippy::too_many_arguments)]
pub async fn add(
    db: &DbPool,
    user_id: Uuid,
    label: &str,
    card_number: &str,
    card_password: &str,
    birthday: &str,
    expire_date: &str,
    expire_date_yymm: Option<&str>,
    card_type: &str,
    last_four: &str,
) -> Result<CardInfo, ServiceError> {
    validate_encrypted_field(card_number, "Card number")?;
    validate_encrypted_field(card_password, "Card password")?;
    validate_encrypted_field(birthday, "Birthday")?;
    validate_encrypted_field(expire_date, "Expire date")?;
    if let Some(yymm) = expire_date_yymm {
        validate_encrypted_field(yymm, "Expire date YYMM")?;
    }
    if last_four.len() != 4 || !last_four.chars().all(|c| c.is_ascii_digit()) {
        return Err(ServiceError::validation(
            "last_four must be exactly 4 digits",
        ));
    }
    validate_card_type(card_type)?;

    let row = bominal_db::card::create_card(
        db,
        user_id,
        label,
        card_number,
        card_password,
        birthday,
        expire_date,
        expire_date_yymm,
        card_type,
        last_four,
    )
    .await?;

    Ok(row_to_info(&row))
}

/// Update a card's label.
pub async fn update_label(
    db: &DbPool,
    card_id: Uuid,
    user_id: Uuid,
    label: &str,
) -> Result<CardInfo, ServiceError> {
    if label.is_empty() {
        return Err(ServiceError::validation("Label cannot be empty"));
    }

    let row = bominal_db::card::update_label(db, card_id, user_id, label)
        .await?
        .ok_or_else(|| ServiceError::not_found("Card not found"))?;

    Ok(row_to_info(&row))
}

/// Delete a payment card.
pub async fn delete(db: &DbPool, card_id: Uuid, user_id: Uuid) -> Result<(), ServiceError> {
    let deleted = bominal_db::card::delete_card(db, card_id, user_id).await?;

    if !deleted {
        return Err(ServiceError::not_found("Card not found"));
    }

    Ok(())
}

// ── Helpers ──────────────────────────────────────────────────────────

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

/// Map card type code to display name.
pub fn card_type_name(code: &str) -> &str {
    match code {
        "J" => "신용카드",
        "S" => "체크카드",
        _ => "기타",
    }
}

fn validate_encrypted_field(value: &str, field_name: &str) -> Result<(), ServiceError> {
    if !value.starts_with("ev:") {
        return Err(ServiceError::validation(format!(
            "{field_name} must be encrypted via Evervault SDK"
        )));
    }
    Ok(())
}

fn validate_card_type(card_type: &str) -> Result<(), ServiceError> {
    match card_type {
        "J" | "S" => Ok(()),
        _ => Err(ServiceError::validation(
            "Card type must be 'J' (credit) or 'S' (debit)",
        )),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_card_type_names() {
        assert_eq!(card_type_name("J"), "신용카드");
        assert_eq!(card_type_name("S"), "체크카드");
        assert_eq!(card_type_name("X"), "기타");
    }

    #[test]
    fn test_validate_encrypted_field() {
        assert!(validate_encrypted_field("ev:abc:data", "test").is_ok());
        assert!(validate_encrypted_field("plaintext", "test").is_err());
    }

    #[test]
    fn test_validate_card_type() {
        assert!(validate_card_type("J").is_ok());
        assert!(validate_card_type("S").is_ok());
        assert!(validate_card_type("X").is_err());
    }
}
