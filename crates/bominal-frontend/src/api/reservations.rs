//! Reservation server functions — list, cancel, pay.
//!
//! Reservations are fetched live from the SRT/KTX provider (not stored in DB).

use leptos::prelude::*;
use serde::{Deserialize, Serialize};

/// Reservation summary.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ReservationInfo {
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

/// List active reservations from the provider.
#[server(prefix = "/sfn")]
pub async fn list_reservations(provider: String) -> Result<Vec<ReservationInfo>, ServerFnError> {
    let (pool, user_id) = super::tasks::require_auth().await?;

    let cred = bominal_db::provider::find_by_user_and_provider(&pool, user_id, &provider)
        .await
        .map_err(|e| ServerFnError::new(format!("Database error: {e}")))?
        .ok_or_else(|| ServerFnError::new(format!("{provider} credentials required")))?;

    if cred.status != "valid" {
        return Err(ServerFnError::new(format!(
            "{provider} credentials are invalid"
        )));
    }

    let key = use_context::<bominal_domain::crypto::encryption::EncryptionKey>()
        .ok_or_else(|| ServerFnError::new("Server misconfigured"))?;

    let password = bominal_domain::crypto::encryption::decrypt(&key, &cred.encrypted_password)
        .map_err(|e| ServerFnError::new(format!("Decryption error: {e}")))?;

    match provider.as_str() {
        "SRT" => list_srt(&cred.login_id, &password).await,
        "KTX" => list_ktx(&cred.login_id, &password).await,
        _ => Err(ServerFnError::new(format!("Invalid provider: {provider}"))),
    }
}

/// Cancel a reservation.
#[server(prefix = "/sfn")]
pub async fn cancel_reservation(
    provider: String,
    reservation_number: String,
) -> Result<(), ServerFnError> {
    let (pool, user_id) = super::tasks::require_auth().await?;

    let cred = bominal_db::provider::find_by_user_and_provider(&pool, user_id, &provider)
        .await
        .map_err(|e| ServerFnError::new(format!("Database error: {e}")))?
        .ok_or_else(|| ServerFnError::new(format!("{provider} credentials required")))?;

    let key = use_context::<bominal_domain::crypto::encryption::EncryptionKey>()
        .ok_or_else(|| ServerFnError::new("Server misconfigured"))?;

    let password = bominal_domain::crypto::encryption::decrypt(&key, &cred.encrypted_password)
        .map_err(|e| ServerFnError::new(format!("Decryption error: {e}")))?;

    match provider.as_str() {
        "SRT" => cancel_srt(&cred.login_id, &password, &reservation_number).await,
        "KTX" => cancel_ktx(&cred.login_id, &password, &reservation_number).await,
        _ => Err(ServerFnError::new(format!("Invalid provider: {provider}"))),
    }
}

/// Pay for a reservation using a stored card.
#[server(prefix = "/sfn")]
pub async fn pay_reservation(
    provider: String,
    reservation_number: String,
    card_id: String,
) -> Result<(), ServerFnError> {
    let (pool, user_id) = super::tasks::require_auth().await?;

    let card_uuid = uuid::Uuid::parse_str(&card_id)
        .map_err(|e| ServerFnError::new(format!("Bad card ID: {e}")))?;

    let cred = bominal_db::provider::find_by_user_and_provider(&pool, user_id, &provider)
        .await
        .map_err(|e| ServerFnError::new(format!("Database error: {e}")))?
        .ok_or_else(|| ServerFnError::new(format!("{provider} credentials required")))?;

    let card = bominal_db::card::find_by_id(&pool, card_uuid, user_id)
        .await
        .map_err(|e| ServerFnError::new(format!("Database error: {e}")))?
        .ok_or_else(|| ServerFnError::new("Card not found"))?;

    let key = use_context::<bominal_domain::crypto::encryption::EncryptionKey>()
        .ok_or_else(|| ServerFnError::new("Server misconfigured"))?;

    let password = bominal_domain::crypto::encryption::decrypt(&key, &cred.encrypted_password)
        .map_err(|e| ServerFnError::new(format!("Decryption error: {e}")))?;

    // Decrypt card fields
    use bominal_domain::crypto::encryption::decrypt;
    let card_number = decrypt(&key, &card.encrypted_number)
        .map_err(|e| ServerFnError::new(format!("Card decryption error: {e}")))?;
    let card_password = decrypt(&key, &card.encrypted_password)
        .map_err(|e| ServerFnError::new(format!("Card decryption error: {e}")))?;
    let birthday = decrypt(&key, &card.encrypted_birthday)
        .map_err(|e| ServerFnError::new(format!("Card decryption error: {e}")))?;
    let expiry = decrypt(&key, &card.encrypted_expiry)
        .map_err(|e| ServerFnError::new(format!("Card decryption error: {e}")))?;

    match provider.as_str() {
        "SRT" => {
            pay_srt(
                &cred.login_id,
                &password,
                &reservation_number,
                &card_number,
                &card_password,
                &birthday,
                &expiry,
                &card.card_type,
            )
            .await
        }
        "KTX" => {
            pay_ktx(
                &cred.login_id,
                &password,
                &reservation_number,
                &card_number,
                &card_password,
                &birthday,
                &expiry,
                &card.card_type,
            )
            .await
        }
        _ => Err(ServerFnError::new(format!("Invalid provider: {provider}"))),
    }
}

// ── SRT ──────────────────────────────────────────────────────────────

async fn list_srt(login_id: &str, password: &str) -> Result<Vec<ReservationInfo>, ServerFnError> {
    let mut client = bominal_provider::srt::SrtClient::new();
    client
        .login(login_id, password)
        .await
        .map_err(|e| ServerFnError::new(format!("SRT login: {e}")))?;

    let reservations = client
        .get_reservations()
        .await
        .map_err(|e| ServerFnError::new(format!("SRT reservations: {e}")))?;

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

async fn cancel_srt(login_id: &str, password: &str, pnr: &str) -> Result<(), ServerFnError> {
    let mut client = bominal_provider::srt::SrtClient::new();
    client
        .login(login_id, password)
        .await
        .map_err(|e| ServerFnError::new(format!("SRT login: {e}")))?;

    client
        .cancel(pnr)
        .await
        .map_err(|e| ServerFnError::new(format!("SRT cancel: {e}")))?;

    Ok(())
}

// ── KTX ──────────────────────────────────────────────────────────────

async fn list_ktx(login_id: &str, password: &str) -> Result<Vec<ReservationInfo>, ServerFnError> {
    let mut client = bominal_provider::ktx::KtxClient::new();
    client
        .login(login_id, password)
        .await
        .map_err(|e| ServerFnError::new(format!("KTX login: {e}")))?;

    let reservations = client
        .get_reservations()
        .await
        .map_err(|e| ServerFnError::new(format!("KTX reservations: {e}")))?;

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

async fn cancel_ktx(login_id: &str, password: &str, pnr: &str) -> Result<(), ServerFnError> {
    let mut client = bominal_provider::ktx::KtxClient::new();
    client
        .login(login_id, password)
        .await
        .map_err(|e| ServerFnError::new(format!("KTX login: {e}")))?;

    let reservations = client
        .get_reservations()
        .await
        .map_err(|e| ServerFnError::new(format!("KTX reservations: {e}")))?;

    let reservation = reservations
        .iter()
        .find(|r| r.rsv_id == pnr)
        .ok_or_else(|| ServerFnError::new(format!("Reservation {pnr} not found")))?;

    client
        .cancel(reservation)
        .await
        .map_err(|e| ServerFnError::new(format!("KTX cancel: {e}")))?;

    Ok(())
}

// ── Payment helpers ──────────────────────────────────────────────────

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
) -> Result<(), ServerFnError> {
    let mut client = bominal_provider::srt::SrtClient::new();
    client
        .login(login_id, password)
        .await
        .map_err(|e| ServerFnError::new(format!("SRT login: {e}")))?;

    let reservations = client
        .get_reservations()
        .await
        .map_err(|e| ServerFnError::new(format!("SRT reservations: {e}")))?;

    let reservation = reservations
        .iter()
        .find(|r| r.reservation_number == pnr)
        .ok_or_else(|| ServerFnError::new(format!("Reservation {pnr} not found")))?;

    client
        .pay_with_card(
            reservation,
            card_number,
            card_password,
            birthday,
            expiry,
            0, // no installment
            card_type,
        )
        .await
        .map_err(|e| ServerFnError::new(format!("SRT payment: {e}")))?;

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
) -> Result<(), ServerFnError> {
    let mut client = bominal_provider::ktx::KtxClient::new();
    client
        .login(login_id, password)
        .await
        .map_err(|e| ServerFnError::new(format!("KTX login: {e}")))?;

    let reservations = client
        .get_reservations()
        .await
        .map_err(|e| ServerFnError::new(format!("KTX reservations: {e}")))?;

    let reservation = reservations
        .iter()
        .find(|r| r.rsv_id == pnr)
        .ok_or_else(|| ServerFnError::new(format!("Reservation {pnr} not found")))?;

    client
        .pay_with_card(
            reservation,
            card_number,
            card_password,
            birthday,
            expiry,
            "0", // no installment
            card_type,
        )
        .await
        .map_err(|e| ServerFnError::new(format!("KTX payment: {e}")))?;

    Ok(())
}
