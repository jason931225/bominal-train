//! Payment card route handlers.
//!
//! - GET    /api/cards       — list cards (masked)
//! - POST   /api/cards       — add card
//! - PATCH  /api/cards/:id   — update card label
//! - DELETE /api/cards/:id   — delete card

use axum::Json;
use axum::extract::{Path, State};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::error::AppError;
use crate::extractors::AuthUser;
use crate::state::SharedState;

/// Card response (masked — never exposes raw encrypted fields).
#[derive(Debug, Serialize)]
pub struct CardResponse {
    pub id: Uuid,
    pub label: String,
    pub last_four: String,
    pub card_type: String,
    pub card_type_name: String,
    pub created_at: DateTime<Utc>,
}

/// Add card request.
///
/// All sensitive fields (`card_number`, `card_password`, `birthday`,
/// `expire_date`) must be pre-encrypted by the Evervault JS SDK on the
/// frontend. The server stores these `ev:`-prefixed strings directly
/// and never sees plaintext card data.
#[derive(Debug, Deserialize)]
pub struct AddCardRequest {
    pub label: Option<String>,
    /// Evervault-encrypted card number (ev:... format).
    pub card_number: String,
    /// Evervault-encrypted card password (ev:... format).
    pub card_password: String,
    /// Evervault-encrypted birthday (ev:... format).
    pub birthday: String,
    /// Evervault-encrypted expiry date (ev:... format).
    pub expire_date: String,
    /// Evervault-encrypted expiry date in YYMM format (ev:... format).
    pub expire_date_yymm: Option<String>,
    /// Last 4 digits of card (plaintext, for display).
    pub last_four: String,
    pub card_type: Option<String>,
}

/// Update card request.
#[derive(Debug, Deserialize)]
pub struct UpdateCardRequest {
    pub label: String,
}

/// GET /api/cards — list all cards (masked).
pub async fn list_cards(
    user: AuthUser,
    State(state): State<SharedState>,
) -> Result<Json<Vec<CardResponse>>, AppError> {
    let rows = bominal_db::card::find_by_user(&state.db, user.user_id)
        .await
        .map_err(|e| AppError::Internal(e.into()))?;

    Ok(Json(rows.iter().map(card_to_response).collect()))
}

/// POST /api/cards — add a payment card.
pub async fn add_card(
    user: AuthUser,
    State(state): State<SharedState>,
    Json(req): Json<AddCardRequest>,
) -> Result<Json<CardResponse>, AppError> {
    validate_card_request(&req)?;

    let last_four = &req.last_four;
    let card_type = req.card_type.as_deref().unwrap_or("J");
    let label = req.label.as_deref().unwrap_or("My Card");

    // Card fields arrive pre-encrypted by the Evervault JS SDK (ev: prefix).
    // The server never sees plaintext card data — store as-is.
    let row = bominal_db::card::create_card(
        &state.db,
        user.user_id,
        label,
        &req.card_number,
        &req.card_password,
        &req.birthday,
        &req.expire_date,
        req.expire_date_yymm.as_deref(),
        card_type,
        last_four,
    )
    .await
    .map_err(|e| AppError::Internal(e.into()))?;

    Ok(Json(card_to_response(&row)))
}

/// PATCH /api/cards/:id — update card label.
pub async fn update_card(
    user: AuthUser,
    State(state): State<SharedState>,
    Path(card_id): Path<Uuid>,
    Json(req): Json<UpdateCardRequest>,
) -> Result<Json<CardResponse>, AppError> {
    if req.label.is_empty() {
        return Err(AppError::BadRequest("Label cannot be empty".to_string()));
    }

    let row = bominal_db::card::update_label(&state.db, card_id, user.user_id, &req.label)
        .await
        .map_err(|e| AppError::Internal(e.into()))?
        .ok_or_else(|| AppError::NotFound("Card not found".to_string()))?;

    Ok(Json(card_to_response(&row)))
}

/// DELETE /api/cards/:id — delete a card.
pub async fn delete_card(
    user: AuthUser,
    State(state): State<SharedState>,
    Path(card_id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, AppError> {
    let deleted = bominal_db::card::delete_card(&state.db, card_id, user.user_id)
        .await
        .map_err(|e| AppError::Internal(e.into()))?;

    if !deleted {
        return Err(AppError::NotFound("Card not found".to_string()));
    }

    Ok(Json(serde_json::json!({ "deleted": true })))
}

// ── Helpers ──────────────────────────────────────────────────────────

fn card_to_response(row: &bominal_db::card::PaymentCardRow) -> CardResponse {
    CardResponse {
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

fn validate_card_request(req: &AddCardRequest) -> Result<(), AppError> {
    // Sensitive fields must be Evervault-encrypted (ev: prefix).
    if !req.card_number.starts_with("ev:") {
        return Err(AppError::BadRequest(
            "Card number must be encrypted via Evervault SDK".to_string(),
        ));
    }
    if !req.card_password.starts_with("ev:") {
        return Err(AppError::BadRequest(
            "Card password must be encrypted via Evervault SDK".to_string(),
        ));
    }
    if !req.birthday.starts_with("ev:") {
        return Err(AppError::BadRequest(
            "Birthday must be encrypted via Evervault SDK".to_string(),
        ));
    }
    if !req.expire_date.starts_with("ev:") {
        return Err(AppError::BadRequest(
            "Expire date must be encrypted via Evervault SDK".to_string(),
        ));
    }
    if let Some(ref yymm) = req.expire_date_yymm {
        if !yymm.starts_with("ev:") {
            return Err(AppError::BadRequest(
                "Expire date YYMM must be encrypted via Evervault SDK".to_string(),
            ));
        }
    }
    // last_four is plaintext for display — validate it.
    if req.last_four.len() != 4 || !req.last_four.chars().all(|c| c.is_ascii_digit()) {
        return Err(AppError::BadRequest(
            "last_four must be exactly 4 digits".to_string(),
        ));
    }
    if let Some(ct) = &req.card_type {
        if ct != "J" && ct != "S" {
            return Err(AppError::BadRequest(
                "Card type must be 'J' (credit) or 'S' (debit)".to_string(),
            ));
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ev_encrypted(val: &str) -> String {
        format!("ev:abc123:{val}")
    }

    fn valid_request() -> AddCardRequest {
        AddCardRequest {
            label: None,
            card_number: ev_encrypted("card"),
            card_password: ev_encrypted("pw"),
            birthday: ev_encrypted("bday"),
            expire_date: ev_encrypted("exp"),
            expire_date_yymm: Some(ev_encrypted("yymm")),
            last_four: "3456".to_string(),
            card_type: Some("J".to_string()),
        }
    }

    #[test]
    fn validate_valid_card() {
        assert!(validate_card_request(&valid_request()).is_ok());
    }

    #[test]
    fn validate_rejects_plaintext_card_number() {
        let mut req = valid_request();
        req.card_number = "1234567890123456".to_string();
        assert!(validate_card_request(&req).is_err());
    }

    #[test]
    fn validate_rejects_plaintext_password() {
        let mut req = valid_request();
        req.card_password = "12".to_string();
        assert!(validate_card_request(&req).is_err());
    }

    #[test]
    fn validate_rejects_bad_last_four() {
        let mut req = valid_request();
        req.last_four = "abc".to_string();
        assert!(validate_card_request(&req).is_err());
    }

    #[test]
    fn validate_rejects_bad_card_type() {
        let mut req = valid_request();
        req.card_type = Some("X".to_string());
        assert!(validate_card_request(&req).is_err());
    }

    #[test]
    fn card_type_names() {
        assert_eq!(card_type_name("J"), "신용카드");
        assert_eq!(card_type_name("S"), "체크카드");
        assert_eq!(card_type_name("X"), "기타");
    }
}
