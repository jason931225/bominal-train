//! Reservation management route handlers.
//!
//! - GET    /api/reservations                — list active reservations
//! - GET    /api/reservations/:pnr/tickets   — ticket detail for a reservation
//! - POST   /api/reservations/:pnr/cancel    — cancel a reservation
//! - POST   /api/reservations/:pnr/pay       — pay with card
//! - POST   /api/reservations/:pnr/refund    — refund a paid reservation

use axum::Json;
use axum::extract::{Path, State};
use serde::Deserialize;

use crate::error::AppError;
use crate::extractors::AuthUser;
use crate::state::SharedState;

/// Payment request body.
///
/// All sensitive fields must be pre-encrypted by the Evervault JS SDK
/// (`ev:` prefix). The server never sees plaintext card data.
#[derive(Debug, Deserialize)]
pub struct PayRequest {
    /// Evervault-encrypted card number (ev:... format).
    pub card_number: String,
    /// Evervault-encrypted card password (ev:... format).
    pub card_password: String,
    /// Evervault-encrypted validation number (ev:... format).
    pub validation_number: String,
    /// Evervault-encrypted expire date (ev:... format).
    pub expire_date: String,
    pub installment: Option<u8>,
    pub card_type: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ProviderQuery {
    pub provider: Option<String>,
}

/// GET /api/reservations — list active reservations from the provider.
pub async fn list_reservations(
    user: AuthUser,
    State(state): State<SharedState>,
    axum::extract::Query(params): axum::extract::Query<ProviderQuery>,
) -> Result<Json<Vec<bominal_service::reservations::ReservationInfo>>, AppError> {
    let provider = params.provider.as_deref().unwrap_or("SRT");

    let result = bominal_service::reservations::list(
        &state.db,
        user.user_id,
        provider,
        &state.encryption_key,
    )
    .await?;

    Ok(Json(result))
}

/// GET /api/reservations/:pnr/tickets — ticket detail.
pub async fn ticket_detail(
    user: AuthUser,
    State(state): State<SharedState>,
    Path(pnr): Path<String>,
    axum::extract::Query(params): axum::extract::Query<ProviderQuery>,
) -> Result<Json<Vec<bominal_service::reservations::TicketInfo>>, AppError> {
    let provider = params.provider.as_deref().unwrap_or("SRT");

    let result = bominal_service::reservations::ticket_detail(
        &state.db,
        user.user_id,
        provider,
        &pnr,
        &state.encryption_key,
    )
    .await?;

    Ok(Json(result))
}

/// POST /api/reservations/:pnr/cancel — cancel a reservation.
pub async fn cancel_reservation(
    user: AuthUser,
    State(state): State<SharedState>,
    Path(pnr): Path<String>,
    axum::extract::Query(params): axum::extract::Query<ProviderQuery>,
) -> Result<Json<serde_json::Value>, AppError> {
    let provider = params.provider.as_deref().unwrap_or("SRT");

    bominal_service::reservations::cancel(
        &state.db,
        user.user_id,
        provider,
        &pnr,
        &state.encryption_key,
    )
    .await?;

    Ok(Json(serde_json::json!({ "cancelled": true, "pnr": pnr })))
}

/// POST /api/reservations/:pnr/pay — pay with card.
pub async fn pay_reservation(
    user: AuthUser,
    State(state): State<SharedState>,
    Path(pnr): Path<String>,
    axum::extract::Query(params): axum::extract::Query<ProviderQuery>,
    Json(req): Json<PayRequest>,
) -> Result<Json<serde_json::Value>, AppError> {
    validate_pay_request(&req)?;

    let provider = params.provider.as_deref().unwrap_or("SRT");
    let installment = req.installment.unwrap_or(0);
    let card_type = req.card_type.as_deref().unwrap_or("J");

    let relay_domain = match provider {
        "SRT" => &state.evervault.srt_relay_domain,
        "KTX" => &state.evervault.ktx_relay_domain,
        _ => return Err(AppError::BadRequest(format!("Invalid provider: {provider}"))),
    };

    bominal_service::reservations::pay_with_raw_card(
        &state.db,
        user.user_id,
        provider,
        &pnr,
        &req.card_number,
        &req.card_password,
        &req.validation_number,
        &req.expire_date,
        installment,
        card_type,
        &state.encryption_key,
        relay_domain,
    )
    .await?;

    Ok(Json(serde_json::json!({ "paid": true, "pnr": pnr })))
}

/// POST /api/reservations/:pnr/refund — refund a paid reservation.
pub async fn refund_reservation(
    user: AuthUser,
    State(state): State<SharedState>,
    Path(pnr): Path<String>,
    axum::extract::Query(params): axum::extract::Query<ProviderQuery>,
) -> Result<Json<serde_json::Value>, AppError> {
    let provider = params.provider.as_deref().unwrap_or("SRT");

    bominal_service::reservations::refund(
        &state.db,
        user.user_id,
        provider,
        &pnr,
        &state.encryption_key,
    )
    .await?;

    Ok(Json(serde_json::json!({ "refunded": true, "pnr": pnr })))
}

fn validate_pay_request(req: &PayRequest) -> Result<(), AppError> {
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
    if !req.validation_number.starts_with("ev:") {
        return Err(AppError::BadRequest(
            "Validation number must be encrypted via Evervault SDK".to_string(),
        ));
    }
    if !req.expire_date.starts_with("ev:") {
        return Err(AppError::BadRequest(
            "Expire date must be encrypted via Evervault SDK".to_string(),
        ));
    }
    if let Some(ct) = &req.card_type
        && ct != "J"
        && ct != "S"
    {
        return Err(AppError::BadRequest(
            "Card type must be 'J' (credit) or 'S' (debit)".to_string(),
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn provider_query_default() {
        let q: ProviderQuery = serde_json::from_str("{}").unwrap();
        assert!(q.provider.is_none());
    }

    #[test]
    fn provider_query_with_provider() {
        let q: ProviderQuery = serde_json::from_str(r#"{"provider":"KTX"}"#).unwrap();
        assert_eq!(q.provider.as_deref(), Some("KTX"));
    }

    #[test]
    fn pay_request_defaults() {
        let req: PayRequest = serde_json::from_str(
            r#"{
            "card_number": "ev:abc:num",
            "card_password": "ev:abc:pw",
            "validation_number": "ev:abc:val",
            "expire_date": "ev:abc:exp"
        }"#,
        )
        .unwrap();
        assert!(req.installment.is_none());
        assert!(req.card_type.is_none());
    }

    #[test]
    fn pay_request_full() {
        let req: PayRequest = serde_json::from_str(
            r#"{
            "card_number": "ev:abc:num",
            "card_password": "ev:abc:pw",
            "validation_number": "ev:abc:val",
            "expire_date": "ev:abc:exp",
            "installment": 3,
            "card_type": "J"
        }"#,
        )
        .unwrap();
        assert_eq!(req.installment, Some(3));
        assert_eq!(req.card_type.as_deref(), Some("J"));
    }

    #[test]
    fn validate_pay_request_valid() {
        let req = PayRequest {
            card_number: "ev:abc:num".to_string(),
            card_password: "ev:abc:pw".to_string(),
            validation_number: "ev:abc:val".to_string(),
            expire_date: "ev:abc:exp".to_string(),
            installment: None,
            card_type: None,
        };
        assert!(validate_pay_request(&req).is_ok());
    }

    #[test]
    fn validate_pay_request_rejects_plaintext() {
        let req = PayRequest {
            card_number: "1234567890123456".to_string(),
            card_password: "ev:abc:pw".to_string(),
            validation_number: "ev:abc:val".to_string(),
            expire_date: "ev:abc:exp".to_string(),
            installment: None,
            card_type: None,
        };
        assert!(validate_pay_request(&req).is_err());
    }

    #[test]
    fn validate_pay_request_rejects_bad_card_type() {
        let req = PayRequest {
            card_number: "ev:abc:num".to_string(),
            card_password: "ev:abc:pw".to_string(),
            validation_number: "ev:abc:val".to_string(),
            expire_date: "ev:abc:exp".to_string(),
            installment: None,
            card_type: Some("X".to_string()),
        };
        assert!(validate_pay_request(&req).is_err());
    }
}
