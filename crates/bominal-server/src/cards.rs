//! Payment card route handlers.
//!
//! - GET    /api/cards       — list cards (masked)
//! - POST   /api/cards       — add card
//! - PATCH  /api/cards/:id   — update card label
//! - DELETE /api/cards/:id   — delete card

use axum::Json;
use axum::extract::{Path, State};
use serde::Deserialize;
use uuid::Uuid;

use crate::error::AppError;
use crate::extractors::AuthUser;
use crate::state::SharedState;

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
) -> Result<Json<Vec<bominal_service::cards::CardInfo>>, AppError> {
    let result = bominal_service::cards::list(&state.db, user.user_id).await?;
    Ok(Json(result))
}

/// POST /api/cards — add a payment card.
pub async fn add_card(
    user: AuthUser,
    State(state): State<SharedState>,
    Json(req): Json<AddCardRequest>,
) -> Result<Json<bominal_service::cards::CardInfo>, AppError> {
    let label = req.label.as_deref().unwrap_or("My Card");
    let card_type = req.card_type.as_deref().unwrap_or("J");

    let result = bominal_service::cards::add(
        &state.db,
        user.user_id,
        label,
        &req.card_number,
        &req.card_password,
        &req.birthday,
        &req.expire_date,
        req.expire_date_yymm.as_deref(),
        card_type,
        &req.last_four,
    )
    .await?;

    Ok(Json(result))
}

/// PATCH /api/cards/:id — update card label.
pub async fn update_card(
    user: AuthUser,
    State(state): State<SharedState>,
    Path(card_id): Path<Uuid>,
    Json(req): Json<UpdateCardRequest>,
) -> Result<Json<bominal_service::cards::CardInfo>, AppError> {
    let result =
        bominal_service::cards::update_label(&state.db, card_id, user.user_id, &req.label).await?;
    Ok(Json(result))
}

/// DELETE /api/cards/:id — delete a card.
pub async fn delete_card(
    user: AuthUser,
    State(state): State<SharedState>,
    Path(card_id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, AppError> {
    bominal_service::cards::delete(&state.db, card_id, user.user_id).await?;
    Ok(Json(serde_json::json!({ "deleted": true })))
}

#[cfg(test)]
mod tests {
    fn ev_encrypted(val: &str) -> String {
        format!("ev:abc123:{val}")
    }

    #[test]
    fn card_type_names() {
        assert_eq!(
            bominal_service::cards::card_type_name("J"),
            "\u{c2e0}\u{c6a9}\u{ce74}\u{b4dc}"
        );
        assert_eq!(
            bominal_service::cards::card_type_name("S"),
            "\u{ccb4}\u{d06c}\u{ce74}\u{b4dc}"
        );
        assert_eq!(
            bominal_service::cards::card_type_name("X"),
            "\u{ae30}\u{d0c0}"
        );
    }

    #[test]
    fn ev_encrypted_format() {
        let val = ev_encrypted("card");
        assert!(val.starts_with("ev:"));
    }
}
