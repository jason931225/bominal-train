//! Reservation server functions — list, cancel, pay.
//!
//! Reservations are fetched live from the SRT/KTX provider (not stored in DB).

use leptos::prelude::*;

pub use bominal_domain::dto::{ReservationInfo, TicketInfo};

/// List active reservations from the provider.
#[server(prefix = "/sfn")]
pub async fn list_reservations(provider: String) -> Result<Vec<ReservationInfo>, ServerFnError> {
    let (pool, user_id) = super::tasks::require_auth().await?;

    let key = use_context::<bominal_service::EncryptionKey>()
        .ok_or_else(|| ServerFnError::new("Server misconfigured"))?;

    bominal_service::reservations::list(&pool, user_id, &provider, &key)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))
}

/// Cancel a reservation.
#[server(prefix = "/sfn")]
pub async fn cancel_reservation(
    provider: String,
    reservation_number: String,
) -> Result<(), ServerFnError> {
    let (pool, user_id) = super::tasks::require_auth().await?;

    let key = use_context::<bominal_service::EncryptionKey>()
        .ok_or_else(|| ServerFnError::new("Server misconfigured"))?;

    bominal_service::reservations::cancel(&pool, user_id, &provider, &reservation_number, &key)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))
}

/// Get ticket details for a reservation.
#[server(prefix = "/sfn")]
pub async fn ticket_detail(
    provider: String,
    reservation_number: String,
) -> Result<Vec<TicketInfo>, ServerFnError> {
    let (pool, user_id) = super::tasks::require_auth().await?;

    let key = use_context::<bominal_service::EncryptionKey>()
        .ok_or_else(|| ServerFnError::new("Server misconfigured"))?;

    bominal_service::reservations::ticket_detail(
        &pool,
        user_id,
        &provider,
        &reservation_number,
        &key,
    )
    .await
    .map_err(|e| ServerFnError::new(e.to_string()))
}

/// Refund a paid reservation.
#[server(prefix = "/sfn")]
pub async fn refund_reservation(
    provider: String,
    reservation_number: String,
) -> Result<(), ServerFnError> {
    let (pool, user_id) = super::tasks::require_auth().await?;

    let key = use_context::<bominal_service::EncryptionKey>()
        .ok_or_else(|| ServerFnError::new("Server misconfigured"))?;

    bominal_service::reservations::refund(&pool, user_id, &provider, &reservation_number, &key)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))
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

    let key = use_context::<bominal_service::EncryptionKey>()
        .ok_or_else(|| ServerFnError::new("Server misconfigured"))?;

    let relay = use_context::<crate::EvervaultRelay>()
        .ok_or_else(|| ServerFnError::new("Server misconfigured"))?;

    let relay_domain = match provider.as_str() {
        "SRT" => &relay.srt_domain,
        "KTX" => &relay.ktx_domain,
        _ => return Err(ServerFnError::new(format!("Invalid provider: {provider}"))),
    };

    bominal_service::reservations::pay_with_stored_card(
        &pool,
        user_id,
        &provider,
        &reservation_number,
        card_uuid,
        &key,
        relay_domain,
    )
    .await
    .map_err(|e| ServerFnError::new(e.to_string()))
}
