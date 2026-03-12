//! Reservation management route handlers.
//!
//! - GET    /api/reservations                — list active reservations
//! - GET    /api/reservations/:pnr/tickets   — ticket detail for a reservation
//! - POST   /api/reservations/:pnr/cancel    — cancel a reservation
//! - POST   /api/reservations/:pnr/pay       — pay with card
//! - POST   /api/reservations/:pnr/refund    — refund a paid reservation

use axum::extract::{Path, State};
use axum::Json;
use serde::{Deserialize, Serialize};

use crate::error::AppError;
use crate::extractors::AuthUser;
use crate::search::map_provider_error;
use crate::state::SharedState;

/// Reservation summary returned from list endpoint.
#[derive(Debug, Serialize)]
pub struct ReservationResponse {
    pub provider: String,
    pub reservation_number: String,
    pub train_number: String,
    pub train_name: String,
    pub dep_station: String,
    pub arr_station: String,
    pub dep_date: String,
    pub dep_time: String,
    pub arr_time: String,
    pub total_cost: String,
    pub seat_count: String,
    pub paid: bool,
    pub is_waiting: bool,
    pub payment_deadline_date: String,
    pub payment_deadline_time: String,
}

/// Ticket detail within a reservation.
#[derive(Debug, Serialize)]
pub struct TicketResponse {
    pub car: String,
    pub seat: String,
    pub seat_type: String,
    pub passenger_type: String,
    pub price: i64,
    pub original_price: i64,
    pub discount: i64,
}

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

/// GET /api/reservations — list active reservations from the provider.
pub async fn list_reservations(
    user: AuthUser,
    State(state): State<SharedState>,
    axum::extract::Query(params): axum::extract::Query<ProviderQuery>,
) -> Result<Json<Vec<ReservationResponse>>, AppError> {
    let provider = params.provider.as_deref().unwrap_or("SRT");
    let cred = require_credentials(&state, user.user_id, provider).await?;

    let key = &state.encryption_key;
    match provider {
        "SRT" => list_srt_reservations(&cred, key).await,
        "KTX" => list_ktx_reservations(&cred, key).await,
        _ => Err(AppError::BadRequest(format!("Invalid provider: {provider}"))),
    }
}

/// GET /api/reservations/:pnr/tickets — ticket detail.
pub async fn ticket_detail(
    user: AuthUser,
    State(state): State<SharedState>,
    Path(pnr): Path<String>,
    axum::extract::Query(params): axum::extract::Query<ProviderQuery>,
) -> Result<Json<Vec<TicketResponse>>, AppError> {
    let provider = params.provider.as_deref().unwrap_or("SRT");
    let cred = require_credentials(&state, user.user_id, provider).await?;

    let key = &state.encryption_key;
    match provider {
        "SRT" => srt_ticket_detail(&cred, &pnr, key).await,
        "KTX" => ktx_ticket_detail(&cred, &pnr, key).await,
        _ => Err(AppError::BadRequest(format!("Invalid provider: {provider}"))),
    }
}

/// POST /api/reservations/:pnr/cancel — cancel a reservation.
pub async fn cancel_reservation(
    user: AuthUser,
    State(state): State<SharedState>,
    Path(pnr): Path<String>,
    axum::extract::Query(params): axum::extract::Query<ProviderQuery>,
) -> Result<Json<serde_json::Value>, AppError> {
    let provider = params.provider.as_deref().unwrap_or("SRT");
    let cred = require_credentials(&state, user.user_id, provider).await?;

    let key = &state.encryption_key;
    match provider {
        "SRT" => cancel_srt(&cred, &pnr, key).await,
        "KTX" => cancel_ktx(&cred, &pnr, key).await,
        _ => Err(AppError::BadRequest(format!("Invalid provider: {provider}"))),
    }
}

/// POST /api/reservations/:pnr/refund — refund a paid reservation.
pub async fn refund_reservation(
    user: AuthUser,
    State(state): State<SharedState>,
    Path(pnr): Path<String>,
    axum::extract::Query(params): axum::extract::Query<ProviderQuery>,
) -> Result<Json<serde_json::Value>, AppError> {
    let provider = params.provider.as_deref().unwrap_or("SRT");
    let cred = require_credentials(&state, user.user_id, provider).await?;

    let key = &state.encryption_key;
    match provider {
        "SRT" => {
            // TODO: SRT client does not yet support refund — implement when available
            Err(AppError::BadRequest(
                "Refund is not yet supported for SRT reservations".to_string(),
            ))
        }
        "KTX" => refund_ktx(&cred, &pnr, key).await,
        _ => Err(AppError::BadRequest(format!("Invalid provider: {provider}"))),
    }
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
    let cred = require_credentials(&state, user.user_id, provider).await?;

    let key = &state.encryption_key;
    let ev = &state.evervault;
    match provider {
        "SRT" => pay_srt(&cred, &pnr, &req, key, &ev.srt_relay_domain).await,
        "KTX" => pay_ktx(&cred, &pnr, &req, key, &ev.ktx_relay_domain).await,
        _ => Err(AppError::BadRequest(format!("Invalid provider: {provider}"))),
    }
}

// ── Query params ────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct ProviderQuery {
    pub provider: Option<String>,
}

// ── Helpers ─────────────────────────────────────────────────────────

async fn require_credentials(
    state: &SharedState,
    user_id: uuid::Uuid,
    provider: &str,
) -> Result<bominal_db::provider::ProviderCredentialRow, AppError> {
    let cred = bominal_db::provider::find_by_user_and_provider(&state.db, user_id, provider)
        .await
        .map_err(|e| AppError::Internal(e.into()))?
        .ok_or_else(|| {
            AppError::BadRequest(format!(
                "{provider} credentials required. Please add in settings."
            ))
        })?;

    if cred.status != "valid" {
        return Err(AppError::BadRequest(format!(
            "{provider} credentials are invalid. Please update in settings."
        )));
    }

    Ok(cred)
}

// ── SRT implementation ──────────────────────────────────────────────

async fn login_srt(
    cred: &bominal_db::provider::ProviderCredentialRow,
    encryption_key: &bominal_domain::crypto::encryption::EncryptionKey,
) -> Result<bominal_provider::srt::SrtClient, AppError> {
    let password = bominal_domain::crypto::encryption::decrypt(
        encryption_key,
        &cred.encrypted_password,
    )
    .map_err(|e| AppError::Internal(e.into()))?;

    let mut client = bominal_provider::srt::SrtClient::new();
    client
        .login(&cred.login_id, &password)
        .await
        .map_err(map_provider_error)?;
    Ok(client)
}

async fn login_srt_with_relay(
    cred: &bominal_db::provider::ProviderCredentialRow,
    encryption_key: &bominal_domain::crypto::encryption::EncryptionKey,
    relay_domain: &str,
) -> Result<bominal_provider::srt::SrtClient, AppError> {
    let password = bominal_domain::crypto::encryption::decrypt(
        encryption_key,
        &cred.encrypted_password,
    )
    .map_err(|e| AppError::Internal(e.into()))?;

    let mut client = bominal_provider::srt::SrtClient::with_relay(relay_domain);
    client
        .login(&cred.login_id, &password)
        .await
        .map_err(map_provider_error)?;
    Ok(client)
}

async fn list_srt_reservations(
    cred: &bominal_db::provider::ProviderCredentialRow,
    encryption_key: &bominal_domain::crypto::encryption::EncryptionKey,
) -> Result<Json<Vec<ReservationResponse>>, AppError> {
    let client = login_srt(cred, encryption_key).await?;

    let reservations = client
        .get_reservations()
        .await
        .map_err(map_provider_error)?;

    let results: Vec<ReservationResponse> = reservations
        .iter()
        .map(|r| ReservationResponse {
            provider: "SRT".to_string(),
            reservation_number: r.reservation_number.clone(),
            train_number: r.train_number.clone(),
            train_name: r.display_name().to_string(),
            dep_station: r.dep_station_name.clone(),
            arr_station: r.arr_station_name.clone(),
            dep_date: r.dep_date.clone(),
            dep_time: r.dep_time.clone(),
            arr_time: r.arr_time.clone(),
            total_cost: r.total_cost.clone(),
            seat_count: r.seat_count.clone(),
            paid: r.paid,
            is_waiting: r.is_waiting,
            payment_deadline_date: r.payment_date.clone(),
            payment_deadline_time: r.payment_time.clone(),
        })
        .collect();

    Ok(Json(results))
}

async fn srt_ticket_detail(
    cred: &bominal_db::provider::ProviderCredentialRow,
    pnr: &str,
    encryption_key: &bominal_domain::crypto::encryption::EncryptionKey,
) -> Result<Json<Vec<TicketResponse>>, AppError> {
    let client = login_srt(cred, encryption_key).await?;

    let tickets = client
        .ticket_info(pnr)
        .await
        .map_err(map_provider_error)?;

    let results: Vec<TicketResponse> = tickets
        .iter()
        .map(|t| TicketResponse {
            car: t.car.clone(),
            seat: t.seat.clone(),
            seat_type: t.seat_type.clone(),
            passenger_type: t.passenger_type.clone(),
            price: t.price,
            original_price: t.original_price,
            discount: t.discount,
        })
        .collect();

    Ok(Json(results))
}

async fn cancel_srt(
    cred: &bominal_db::provider::ProviderCredentialRow,
    pnr: &str,
    encryption_key: &bominal_domain::crypto::encryption::EncryptionKey,
) -> Result<Json<serde_json::Value>, AppError> {
    let client = login_srt(cred, encryption_key).await?;

    client.cancel(pnr).await.map_err(map_provider_error)?;

    Ok(Json(serde_json::json!({ "cancelled": true, "pnr": pnr })))
}

async fn pay_srt(
    cred: &bominal_db::provider::ProviderCredentialRow,
    pnr: &str,
    req: &PayRequest,
    encryption_key: &bominal_domain::crypto::encryption::EncryptionKey,
    relay_domain: &str,
) -> Result<Json<serde_json::Value>, AppError> {
    let client = login_srt_with_relay(cred, encryption_key, relay_domain).await?;

    // Find the reservation to pass to pay_with_card
    let reservations = client
        .get_reservations()
        .await
        .map_err(map_provider_error)?;

    let reservation = reservations
        .iter()
        .find(|r| r.reservation_number == pnr)
        .ok_or_else(|| AppError::NotFound(format!("Reservation {pnr} not found")))?;

    let installment = req.installment.unwrap_or(0);
    let card_type = req.card_type.as_deref().unwrap_or("J");

    client
        .pay_with_card(
            reservation,
            &req.card_number,
            &req.card_password,
            &req.validation_number,
            &req.expire_date,
            installment,
            card_type,
        )
        .await
        .map_err(map_provider_error)?;

    Ok(Json(serde_json::json!({ "paid": true, "pnr": pnr })))
}

// ── KTX implementation ──────────────────────────────────────────────

async fn login_ktx(
    cred: &bominal_db::provider::ProviderCredentialRow,
    encryption_key: &bominal_domain::crypto::encryption::EncryptionKey,
) -> Result<bominal_provider::ktx::KtxClient, AppError> {
    let password = bominal_domain::crypto::encryption::decrypt(
        encryption_key,
        &cred.encrypted_password,
    )
    .map_err(|e| AppError::Internal(e.into()))?;

    let mut client = bominal_provider::ktx::KtxClient::new();
    client
        .login(&cred.login_id, &password)
        .await
        .map_err(map_provider_error)?;
    Ok(client)
}

async fn login_ktx_with_relay(
    cred: &bominal_db::provider::ProviderCredentialRow,
    encryption_key: &bominal_domain::crypto::encryption::EncryptionKey,
    relay_domain: &str,
) -> Result<bominal_provider::ktx::KtxClient, AppError> {
    let password = bominal_domain::crypto::encryption::decrypt(
        encryption_key,
        &cred.encrypted_password,
    )
    .map_err(|e| AppError::Internal(e.into()))?;

    let mut client = bominal_provider::ktx::KtxClient::with_relay(relay_domain);
    client
        .login(&cred.login_id, &password)
        .await
        .map_err(map_provider_error)?;
    Ok(client)
}

async fn list_ktx_reservations(
    cred: &bominal_db::provider::ProviderCredentialRow,
    encryption_key: &bominal_domain::crypto::encryption::EncryptionKey,
) -> Result<Json<Vec<ReservationResponse>>, AppError> {
    let client = login_ktx(cred, encryption_key).await?;

    let reservations = client
        .get_reservations()
        .await
        .map_err(map_provider_error)?;

    let results: Vec<ReservationResponse> = reservations
        .iter()
        .map(|r| ReservationResponse {
            provider: "KTX".to_string(),
            reservation_number: r.rsv_id.clone(),
            train_number: r.train_no.clone(),
            train_name: r.train_type_name.clone(),
            dep_station: r.dep_name.clone(),
            arr_station: r.arr_name.clone(),
            dep_date: r.dep_date.clone(),
            dep_time: r.dep_time.clone(),
            arr_time: r.arr_time.clone(),
            total_cost: r.price.clone(),
            seat_count: r.tickets.len().to_string(),
            paid: r.paid,
            is_waiting: r.is_waiting,
            payment_deadline_date: String::new(),
            payment_deadline_time: String::new(),
        })
        .collect();

    Ok(Json(results))
}

async fn ktx_ticket_detail(
    cred: &bominal_db::provider::ProviderCredentialRow,
    pnr: &str,
    encryption_key: &bominal_domain::crypto::encryption::EncryptionKey,
) -> Result<Json<Vec<TicketResponse>>, AppError> {
    let client = login_ktx(cred, encryption_key).await?;

    let tickets = client
        .ticket_info(pnr)
        .await
        .map_err(map_provider_error)?;

    let results: Vec<TicketResponse> = tickets
        .iter()
        .map(|t| {
            let price = t.price.parse::<i64>().unwrap_or(0);
            TicketResponse {
                car: t.car.clone(),
                seat: t.seat.clone(),
                seat_type: t.seat_type.clone(),
                passenger_type: String::new(),
                price,
                original_price: price,
                discount: 0,
            }
        })
        .collect();

    Ok(Json(results))
}

async fn cancel_ktx(
    cred: &bominal_db::provider::ProviderCredentialRow,
    pnr: &str,
    encryption_key: &bominal_domain::crypto::encryption::EncryptionKey,
) -> Result<Json<serde_json::Value>, AppError> {
    let client = login_ktx(cred, encryption_key).await?;

    let reservations = client
        .get_reservations()
        .await
        .map_err(map_provider_error)?;

    let reservation = reservations
        .iter()
        .find(|r| r.rsv_id == pnr)
        .ok_or_else(|| AppError::NotFound(format!("Reservation {pnr} not found")))?;

    client.cancel(reservation).await.map_err(map_provider_error)?;

    Ok(Json(serde_json::json!({ "cancelled": true, "pnr": pnr })))
}

async fn pay_ktx(
    cred: &bominal_db::provider::ProviderCredentialRow,
    pnr: &str,
    req: &PayRequest,
    encryption_key: &bominal_domain::crypto::encryption::EncryptionKey,
    relay_domain: &str,
) -> Result<Json<serde_json::Value>, AppError> {
    let client = login_ktx_with_relay(cred, encryption_key, relay_domain).await?;

    let reservations = client
        .get_reservations()
        .await
        .map_err(map_provider_error)?;

    let reservation = reservations
        .iter()
        .find(|r| r.rsv_id == pnr)
        .ok_or_else(|| AppError::NotFound(format!("Reservation {pnr} not found")))?;

    let installment = req.installment.unwrap_or(0).to_string();
    let card_type = req.card_type.as_deref().unwrap_or("J");

    client
        .pay_with_card(
            reservation,
            &req.card_number,
            &req.card_password,
            &req.validation_number,
            &req.expire_date,
            &installment,
            card_type,
        )
        .await
        .map_err(map_provider_error)?;

    Ok(Json(serde_json::json!({ "paid": true, "pnr": pnr })))
}

async fn refund_ktx(
    cred: &bominal_db::provider::ProviderCredentialRow,
    pnr: &str,
    encryption_key: &bominal_domain::crypto::encryption::EncryptionKey,
) -> Result<Json<serde_json::Value>, AppError> {
    let client = login_ktx(cred, encryption_key).await?;

    let tickets = client
        .ticket_info(pnr)
        .await
        .map_err(map_provider_error)?;

    if tickets.is_empty() {
        return Err(AppError::BadRequest(
            "No tickets found for refund".to_string(),
        ));
    }

    for ticket in &tickets {
        client.refund(ticket).await.map_err(map_provider_error)?;
    }

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
        let req: PayRequest = serde_json::from_str(r#"{
            "card_number": "ev:abc:num",
            "card_password": "ev:abc:pw",
            "validation_number": "ev:abc:val",
            "expire_date": "ev:abc:exp"
        }"#).unwrap();
        assert!(req.installment.is_none());
        assert!(req.card_type.is_none());
    }

    #[test]
    fn pay_request_full() {
        let req: PayRequest = serde_json::from_str(r#"{
            "card_number": "ev:abc:num",
            "card_password": "ev:abc:pw",
            "validation_number": "ev:abc:val",
            "expire_date": "ev:abc:exp",
            "installment": 3,
            "card_type": "J"
        }"#).unwrap();
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
