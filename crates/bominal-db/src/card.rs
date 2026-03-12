//! Payment card repository — CRUD for the payment_cards table.

use chrono::{DateTime, Utc};
use sqlx::PgPool;
use uuid::Uuid;

/// Row returned from the payment_cards table.
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct PaymentCardRow {
    pub id: Uuid,
    pub user_id: Uuid,
    pub label: String,
    pub encrypted_number: String,
    pub encrypted_password: String,
    pub encrypted_birthday: String,
    pub encrypted_expiry: String,
    pub encrypted_expiry_yymm: Option<String>,
    pub card_type: String,
    pub last_four: String,
    pub created_at: DateTime<Utc>,
}

/// Add a payment card.
pub async fn create_card(
    pool: &PgPool,
    user_id: Uuid,
    label: &str,
    encrypted_number: &str,
    encrypted_password: &str,
    encrypted_birthday: &str,
    encrypted_expiry: &str,
    encrypted_expiry_yymm: Option<&str>,
    card_type: &str,
    last_four: &str,
) -> Result<PaymentCardRow, sqlx::Error> {
    sqlx::query_as::<_, PaymentCardRow>(
        r#"
        INSERT INTO payment_cards (
            user_id, label, encrypted_number, encrypted_password,
            encrypted_birthday, encrypted_expiry, encrypted_expiry_yymm,
            card_type, last_four
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
        RETURNING *
        "#,
    )
    .bind(user_id)
    .bind(label)
    .bind(encrypted_number)
    .bind(encrypted_password)
    .bind(encrypted_birthday)
    .bind(encrypted_expiry)
    .bind(encrypted_expiry_yymm)
    .bind(card_type)
    .bind(last_four)
    .fetch_one(pool)
    .await
}

/// Find all cards for a user.
pub async fn find_by_user(
    pool: &PgPool,
    user_id: Uuid,
) -> Result<Vec<PaymentCardRow>, sqlx::Error> {
    sqlx::query_as::<_, PaymentCardRow>(
        "SELECT * FROM payment_cards WHERE user_id = $1 ORDER BY created_at DESC",
    )
    .bind(user_id)
    .fetch_all(pool)
    .await
}

/// Find a specific card by ID (with user ownership check).
pub async fn find_by_id(
    pool: &PgPool,
    card_id: Uuid,
    user_id: Uuid,
) -> Result<Option<PaymentCardRow>, sqlx::Error> {
    sqlx::query_as::<_, PaymentCardRow>(
        "SELECT * FROM payment_cards WHERE id = $1 AND user_id = $2",
    )
    .bind(card_id)
    .bind(user_id)
    .fetch_optional(pool)
    .await
}

/// Update card label.
pub async fn update_label(
    pool: &PgPool,
    card_id: Uuid,
    user_id: Uuid,
    label: &str,
) -> Result<Option<PaymentCardRow>, sqlx::Error> {
    sqlx::query_as::<_, PaymentCardRow>(
        "UPDATE payment_cards SET label = $1 WHERE id = $2 AND user_id = $3 RETURNING *",
    )
    .bind(label)
    .bind(card_id)
    .bind(user_id)
    .fetch_optional(pool)
    .await
}

/// Delete a card.
pub async fn delete_card(pool: &PgPool, card_id: Uuid, user_id: Uuid) -> Result<bool, sqlx::Error> {
    let result = sqlx::query("DELETE FROM payment_cards WHERE id = $1 AND user_id = $2")
        .bind(card_id)
        .bind(user_id)
        .execute(pool)
        .await?;
    Ok(result.rows_affected() > 0)
}
