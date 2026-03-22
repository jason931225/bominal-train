//! Reservation service — list, cancel, pay reservations via SRT/KTX providers.
//!
//! Reservations are fetched live from the provider (not stored in DB).

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::error::ServiceError;
use crate::{DbPool, EncryptionKey};

pub use bominal_domain::dto::ReservationInfo;

/// Ticket detail within a reservation.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TicketInfo {
    pub car: String,
    pub seat: String,
    pub seat_type: String,
    pub passenger_type: String,
    pub price: i64,
    pub original_price: i64,
    pub discount: i64,
}

/// List active reservations for a user from the provider.
pub async fn list(
    db: &DbPool,
    user_id: Uuid,
    provider: &str,
    encryption_key: &EncryptionKey,
) -> Result<Vec<ReservationInfo>, ServiceError> {
    let (login_id, password) =
        require_decrypted_credentials(db, user_id, provider, encryption_key).await?;

    match provider {
        "SRT" => list_srt(&login_id, &password).await,
        "KTX" => list_ktx(&login_id, &password).await,
        _ => Err(ServiceError::validation(format!(
            "Invalid provider: {provider}"
        ))),
    }
}

/// Get ticket detail for a reservation.
pub async fn ticket_detail(
    db: &DbPool,
    user_id: Uuid,
    provider: &str,
    pnr: &str,
    encryption_key: &EncryptionKey,
) -> Result<Vec<TicketInfo>, ServiceError> {
    let (login_id, password) =
        require_decrypted_credentials(db, user_id, provider, encryption_key).await?;

    match provider {
        "SRT" => srt_ticket_detail(&login_id, &password, pnr).await,
        "KTX" => ktx_ticket_detail(&login_id, &password, pnr).await,
        _ => Err(ServiceError::validation(format!(
            "Invalid provider: {provider}"
        ))),
    }
}

/// Cancel a reservation.
pub async fn cancel(
    db: &DbPool,
    user_id: Uuid,
    provider: &str,
    pnr: &str,
    encryption_key: &EncryptionKey,
) -> Result<(), ServiceError> {
    let (login_id, password) =
        require_decrypted_credentials(db, user_id, provider, encryption_key).await?;

    match provider {
        "SRT" => cancel_srt(&login_id, &password, pnr).await,
        "KTX" => cancel_ktx(&login_id, &password, pnr).await,
        _ => Err(ServiceError::validation(format!(
            "Invalid provider: {provider}"
        ))),
    }
}

/// Pay for a reservation using a stored card (via Evervault Outbound Relay).
#[allow(clippy::too_many_arguments)]
pub async fn pay_with_stored_card(
    db: &DbPool,
    user_id: Uuid,
    provider: &str,
    pnr: &str,
    card_id: Uuid,
    encryption_key: &EncryptionKey,
    relay_domain: &str,
) -> Result<(), ServiceError> {
    let (login_id, password) =
        require_decrypted_credentials(db, user_id, provider, encryption_key).await?;

    let card = bominal_db::card::find_by_id(db, card_id, user_id)
        .await?
        .ok_or_else(|| ServiceError::not_found("Card not found"))?;

    match provider {
        "SRT" => {
            let srt_expiry = card
                .encrypted_expiry_yymm
                .as_deref()
                .unwrap_or(&card.encrypted_expiry);
            pay_srt(
                &login_id,
                &password,
                pnr,
                &card.encrypted_number,
                &card.encrypted_password,
                &card.encrypted_birthday,
                srt_expiry,
                &card.card_type,
                relay_domain,
            )
            .await
        }
        "KTX" => {
            pay_ktx(
                &login_id,
                &password,
                pnr,
                &card.encrypted_number,
                &card.encrypted_password,
                &card.encrypted_birthday,
                &card.encrypted_expiry,
                &card.card_type,
                relay_domain,
            )
            .await
        }
        _ => Err(ServiceError::validation(format!(
            "Invalid provider: {provider}"
        ))),
    }
}

/// Pay for a reservation using raw encrypted card fields (REST API path).
#[allow(clippy::too_many_arguments)]
pub async fn pay_with_raw_card(
    db: &DbPool,
    user_id: Uuid,
    provider: &str,
    pnr: &str,
    card_number: &str,
    card_password: &str,
    validation_number: &str,
    expire_date: &str,
    installment: u8,
    card_type: &str,
    encryption_key: &EncryptionKey,
    relay_domain: &str,
) -> Result<(), ServiceError> {
    let (login_id, password) =
        require_decrypted_credentials(db, user_id, provider, encryption_key).await?;

    match provider {
        "SRT" => {
            let mut client = bominal_provider::srt::SrtClient::with_relay(relay_domain);
            client.login(&login_id, &password).await?;

            let reservations = client.get_reservations().await?;
            let reservation = reservations
                .iter()
                .find(|r| r.reservation_number == pnr)
                .ok_or_else(|| ServiceError::not_found(format!("Reservation {pnr} not found")))?;

            client
                .pay_with_card(
                    reservation,
                    card_number,
                    card_password,
                    validation_number,
                    expire_date,
                    installment,
                    card_type,
                )
                .await?;
        }
        "KTX" => {
            let mut client = bominal_provider::ktx::KtxClient::with_relay(relay_domain);
            client.login(&login_id, &password).await?;

            let reservations = client.get_reservations().await?;
            let reservation = reservations
                .iter()
                .find(|r| r.rsv_id == pnr)
                .ok_or_else(|| ServiceError::not_found(format!("Reservation {pnr} not found")))?;

            client
                .pay_with_card(
                    reservation,
                    card_number,
                    card_password,
                    validation_number,
                    expire_date,
                    &installment.to_string(),
                    card_type,
                )
                .await?;
        }
        _ => {
            return Err(ServiceError::validation(format!(
                "Invalid provider: {provider}"
            )));
        }
    }

    Ok(())
}

/// Refund a paid reservation (KTX only).
pub async fn refund(
    db: &DbPool,
    user_id: Uuid,
    provider: &str,
    pnr: &str,
    encryption_key: &EncryptionKey,
) -> Result<(), ServiceError> {
    let (login_id, password) =
        require_decrypted_credentials(db, user_id, provider, encryption_key).await?;

    match provider {
        "SRT" => Err(ServiceError::validation(
            "Refund is not yet supported for SRT reservations",
        )),
        "KTX" => refund_ktx(&login_id, &password, pnr).await,
        _ => Err(ServiceError::validation(format!(
            "Invalid provider: {provider}"
        ))),
    }
}

// ── Shared credential helper ─────────────────────────────────────────

async fn require_decrypted_credentials(
    db: &DbPool,
    user_id: Uuid,
    provider: &str,
    encryption_key: &EncryptionKey,
) -> Result<(String, String), ServiceError> {
    let cred = bominal_db::provider::find_by_user_and_provider(db, user_id, provider)
        .await?
        .ok_or_else(|| {
            ServiceError::validation(format!(
                "{provider} credentials required. Please add in settings."
            ))
        })?;

    if cred.status != "valid" {
        return Err(ServiceError::validation(format!(
            "{provider} credentials are invalid. Please update in settings."
        )));
    }

    let password =
        bominal_domain::crypto::encryption::decrypt(encryption_key, &cred.encrypted_password)
            .map_err(|e| ServiceError::Crypto(e.to_string()))?;

    Ok((cred.login_id, password))
}

// ── SRT implementation ──────────────────────────────────────────────

async fn list_srt(login_id: &str, password: &str) -> Result<Vec<ReservationInfo>, ServiceError> {
    let mut client = bominal_provider::srt::SrtClient::new();
    client.login(login_id, password).await?;

    let reservations = client.get_reservations().await?;

    Ok(reservations
        .iter()
        .map(|r| ReservationInfo {
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
        .collect())
}

async fn srt_ticket_detail(
    login_id: &str,
    password: &str,
    pnr: &str,
) -> Result<Vec<TicketInfo>, ServiceError> {
    let mut client = bominal_provider::srt::SrtClient::new();
    client.login(login_id, password).await?;

    let tickets = client.ticket_info(pnr).await?;

    Ok(tickets
        .iter()
        .map(|t| TicketInfo {
            car: t.car.clone(),
            seat: t.seat.clone(),
            seat_type: t.seat_type.clone(),
            passenger_type: t.passenger_type.clone(),
            price: t.price,
            original_price: t.original_price,
            discount: t.discount,
        })
        .collect())
}

async fn cancel_srt(login_id: &str, password: &str, pnr: &str) -> Result<(), ServiceError> {
    let mut client = bominal_provider::srt::SrtClient::new();
    client.login(login_id, password).await?;
    client.cancel(pnr).await?;
    Ok(())
}

#[allow(clippy::too_many_arguments)]
async fn pay_srt(
    login_id: &str,
    password: &str,
    pnr: &str,
    card_number: &str,
    card_password: &str,
    birthday: &str,
    expiry: &str,
    card_type: &str,
    relay_domain: &str,
) -> Result<(), ServiceError> {
    let mut client = bominal_provider::srt::SrtClient::with_relay(relay_domain);
    client.login(login_id, password).await?;

    let reservations = client.get_reservations().await?;
    let reservation = reservations
        .iter()
        .find(|r| r.reservation_number == pnr)
        .ok_or_else(|| ServiceError::not_found(format!("Reservation {pnr} not found")))?;

    client
        .pay_with_card(
            reservation,
            card_number,
            card_password,
            birthday,
            expiry,
            0,
            card_type,
        )
        .await?;

    Ok(())
}

// ── KTX implementation ──────────────────────────────────────────────

async fn list_ktx(login_id: &str, password: &str) -> Result<Vec<ReservationInfo>, ServiceError> {
    let mut client = bominal_provider::ktx::KtxClient::new();
    client.login(login_id, password).await?;

    let reservations = client.get_reservations().await?;

    Ok(reservations
        .iter()
        .map(|r| ReservationInfo {
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
        .collect())
}

async fn ktx_ticket_detail(
    login_id: &str,
    password: &str,
    pnr: &str,
) -> Result<Vec<TicketInfo>, ServiceError> {
    let mut client = bominal_provider::ktx::KtxClient::new();
    client.login(login_id, password).await?;

    let tickets = client.ticket_info(pnr).await?;

    Ok(tickets
        .iter()
        .map(|t| {
            let price = t.price.parse::<i64>().unwrap_or(0);
            TicketInfo {
                car: t.car.clone(),
                seat: t.seat.clone(),
                seat_type: t.seat_type.clone(),
                passenger_type: String::new(),
                price,
                original_price: price,
                discount: 0,
            }
        })
        .collect())
}

async fn cancel_ktx(login_id: &str, password: &str, pnr: &str) -> Result<(), ServiceError> {
    let mut client = bominal_provider::ktx::KtxClient::new();
    client.login(login_id, password).await?;

    let reservations = client.get_reservations().await?;
    let reservation = reservations
        .iter()
        .find(|r| r.rsv_id == pnr)
        .ok_or_else(|| ServiceError::not_found(format!("Reservation {pnr} not found")))?;

    client.cancel(reservation).await?;
    Ok(())
}

#[allow(clippy::too_many_arguments)]
async fn pay_ktx(
    login_id: &str,
    password: &str,
    pnr: &str,
    card_number: &str,
    card_password: &str,
    birthday: &str,
    expiry: &str,
    card_type: &str,
    relay_domain: &str,
) -> Result<(), ServiceError> {
    let mut client = bominal_provider::ktx::KtxClient::with_relay(relay_domain);
    client.login(login_id, password).await?;

    let reservations = client.get_reservations().await?;
    let reservation = reservations
        .iter()
        .find(|r| r.rsv_id == pnr)
        .ok_or_else(|| ServiceError::not_found(format!("Reservation {pnr} not found")))?;

    client
        .pay_with_card(
            reservation,
            card_number,
            card_password,
            birthday,
            expiry,
            "0",
            card_type,
        )
        .await?;

    Ok(())
}

async fn refund_ktx(login_id: &str, password: &str, pnr: &str) -> Result<(), ServiceError> {
    let mut client = bominal_provider::ktx::KtxClient::new();
    client.login(login_id, password).await?;

    let tickets = client.ticket_info(pnr).await?;

    if tickets.is_empty() {
        return Err(ServiceError::validation("No tickets found for refund"));
    }

    for ticket in &tickets {
        client.refund(ticket).await?;
    }

    Ok(())
}
