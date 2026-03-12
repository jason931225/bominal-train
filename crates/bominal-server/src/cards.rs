//! Payment card route handlers.
//!
//! - GET    /api/cards       — list cards (masked)
//! - POST   /api/cards       — add card
//! - PATCH  /api/cards/:id   — update card label
//! - DELETE /api/cards/:id   — delete card

use axum::extract::{Path, State};
use axum::Json;
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
#[derive(Debug, Deserialize)]
pub struct AddCardRequest {
    pub label: Option<String>,
    pub card_number: String,
    pub card_password: String,
    pub birthday: String,
    pub expire_date: String,
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

    let last_four = &req.card_number[req.card_number.len() - 4..];
    let card_type = req.card_type.as_deref().unwrap_or("J");
    let label = req.label.as_deref().unwrap_or("My Card");

    // Encrypt sensitive card fields with AES-256-GCM before storage
    use bominal_domain::crypto::encryption::encrypt;
    let key = &state.encryption_key;
    let enc_number = encrypt(key, &req.card_number).map_err(|e| AppError::Internal(e.into()))?;
    let enc_password =
        encrypt(key, &req.card_password).map_err(|e| AppError::Internal(e.into()))?;
    let enc_birthday = encrypt(key, &req.birthday).map_err(|e| AppError::Internal(e.into()))?;
    let enc_expiry =
        encrypt(key, &req.expire_date).map_err(|e| AppError::Internal(e.into()))?;

    let row = bominal_db::card::create_card(
        &state.db,
        user.user_id,
        label,
        &enc_number,
        &enc_password,
        &enc_birthday,
        &enc_expiry,
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
    if req.card_number.len() < 15 || req.card_number.len() > 16 {
        return Err(AppError::BadRequest(
            "Card number must be 15-16 digits".to_string(),
        ));
    }
    if !req.card_number.chars().all(|c| c.is_ascii_digit()) {
        return Err(AppError::BadRequest(
            "Card number must contain only digits".to_string(),
        ));
    }
    if req.card_password.len() != 2 || !req.card_password.chars().all(|c| c.is_ascii_digit()) {
        return Err(AppError::BadRequest(
            "Card password must be 2 digits".to_string(),
        ));
    }
    if req.birthday.len() != 6 || !req.birthday.chars().all(|c| c.is_ascii_digit()) {
        return Err(AppError::BadRequest(
            "Birthday must be YYMMDD format (6 digits)".to_string(),
        ));
    }
    if req.expire_date.len() != 4 || !req.expire_date.chars().all(|c| c.is_ascii_digit()) {
        return Err(AppError::BadRequest(
            "Expire date must be MMYY or YYMM format (4 digits)".to_string(),
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

    #[test]
    fn validate_valid_card() {
        let req = AddCardRequest {
            label: None,
            card_number: "1234567890123456".to_string(),
            card_password: "12".to_string(),
            birthday: "900101".to_string(),
            expire_date: "1228".to_string(),
            card_type: Some("J".to_string()),
        };
        assert!(validate_card_request(&req).is_ok());
    }

    #[test]
    fn validate_short_card_number() {
        let req = AddCardRequest {
            label: None,
            card_number: "1234".to_string(),
            card_password: "12".to_string(),
            birthday: "900101".to_string(),
            expire_date: "1228".to_string(),
            card_type: None,
        };
        assert!(validate_card_request(&req).is_err());
    }

    #[test]
    fn validate_bad_password() {
        let req = AddCardRequest {
            label: None,
            card_number: "1234567890123456".to_string(),
            card_password: "abc".to_string(),
            birthday: "900101".to_string(),
            expire_date: "1228".to_string(),
            card_type: None,
        };
        assert!(validate_card_request(&req).is_err());
    }

    #[test]
    fn validate_bad_birthday() {
        let req = AddCardRequest {
            label: None,
            card_number: "1234567890123456".to_string(),
            card_password: "12".to_string(),
            birthday: "19900101".to_string(),
            expire_date: "1228".to_string(),
            card_type: None,
        };
        assert!(validate_card_request(&req).is_err());
    }

    #[test]
    fn validate_bad_card_type() {
        let req = AddCardRequest {
            label: None,
            card_number: "1234567890123456".to_string(),
            card_password: "12".to_string(),
            birthday: "900101".to_string(),
            expire_date: "1228".to_string(),
            card_type: Some("X".to_string()),
        };
        assert!(validate_card_request(&req).is_err());
    }

    #[test]
    fn card_type_names() {
        assert_eq!(card_type_name("J"), "신용카드");
        assert_eq!(card_type_name("S"), "체크카드");
        assert_eq!(card_type_name("X"), "기타");
    }
}
