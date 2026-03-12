//! Background reservation task runner.
//!
//! Spawns a tokio task per active reservation that loops:
//! 1. Search for target trains
//! 2. If seat available → attempt reserve
//! 3. Sleep with gamma-distributed delay
//! 4. Handle errors (re-login, retry, etc.)
//!
//! Ported from `srtgo.py:698-783`.

use rand_distr::{Distribution, Gamma};
use tracing::{debug, error, info, warn};

const MAX_ATTEMPTS: i32 = 10_000;

use bominal_db::DbPool;
use bominal_domain::crypto::encryption::{self, EncryptionKey};
use bominal_email::EmailClient;
use bominal_email::templates::reservation::{AlertKind, ReservationDetails};
use bominal_provider::ktx::KtxClient;
use bominal_provider::srt::SrtClient;
use bominal_provider::srt::passenger::{PassengerGroup, PassengerType, WindowSeat, total_count};
use bominal_provider::types::{ProviderError, SeatPreference};

use crate::evervault::EvervaultConfig;
use crate::sse::{EventBus, TaskEvent};

/// Start the task runner background loop.
///
/// Polls for queued tasks every 5 seconds and spawns a worker for each.
pub fn spawn_runner(
    db: DbPool,
    event_bus: EventBus,
    email: EmailClient,
    encryption_key: EncryptionKey,
    evervault: EvervaultConfig,
    app_base_url: String,
) {
    tokio::spawn(async move {
        info!("Task runner started");
        loop {
            if let Err(e) = poll_and_dispatch(&db, &event_bus, &email, &encryption_key, &evervault, &app_base_url).await {
                error!(error = %e, "Task runner poll error");
            }
            tokio::time::sleep(std::time::Duration::from_secs(5)).await;
        }
    });
}

/// Atomically claim queued tasks and spawn workers.
async fn poll_and_dispatch(
    db: &DbPool,
    event_bus: &EventBus,
    email: &EmailClient,
    encryption_key: &EncryptionKey,
    evervault: &EvervaultConfig,
    app_base_url: &str,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let tasks = bominal_db::task::claim_queued_tasks(db).await?;

    for task in tasks {
        let db = db.clone();
        let bus = event_bus.clone();
        let email = email.clone();
        let key = encryption_key.clone();
        let ev = evervault.clone();
        let base_url = app_base_url.to_string();

        tokio::spawn(async move {
            info!(task_id = %task.id, provider = %task.provider, "Starting reservation worker");

            bus.publish(task.user_id, TaskEvent {
                task_id: task.id,
                status: "running".to_string(),
                message: "Reservation worker started".to_string(),
                attempt_count: 0,
                reservation_number: None,
            }).await;

            let result = match task.provider.as_str() {
                "SRT" => run_srt_task(&db, &task, &bus, &email, &key, &ev, &base_url).await,
                "KTX" => run_ktx_task(&db, &task, &bus, &email, &key, &ev, &base_url).await,
                _ => {
                    error!(task_id = %task.id, provider = %task.provider, "Unknown provider");
                    Err("Unknown provider".into())
                }
            };

            if let Err(e) = result {
                error!(task_id = %task.id, error = %e, "Task failed");
                if let Err(db_err) = bominal_db::task::update_status(&db, task.id, "failed").await {
                    error!(task_id = %task.id, error = %db_err, "Failed to mark task as failed");
                }
                bus.publish(task.user_id, TaskEvent {
                    task_id: task.id,
                    status: "failed".to_string(),
                    message: format!("Task failed: {e}"),
                    attempt_count: 0,
                    reservation_number: None,
                }).await;

                // Send failure email if notifications enabled
                if task.notify_enabled {
                    send_alert_email(&db, &email, &task, AlertKind::Failed, None, &base_url).await;
                }
            }
        });
    }

    Ok(())
}

/// Run the SRT reservation loop for a single task.
async fn run_srt_task(
    db: &DbPool,
    task: &bominal_db::task::TaskRow,
    event_bus: &EventBus,
    email: &EmailClient,
    encryption_key: &EncryptionKey,
    evervault: &EvervaultConfig,
    app_base_url: &str,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // Fetch stored credentials and decrypt password
    let cred = bominal_db::provider::find_by_user_and_provider(db, task.user_id, "SRT")
        .await?
        .ok_or("No SRT credentials found")?;
    let password = encryption::decrypt(encryption_key, &cred.encrypted_password)?;

    // Create client with Evervault Relay proxy (decrypts ev: card fields in-flight).
    let mut client = SrtClient::with_relay(&evervault.srt_relay_domain);
    client.login(&cred.login_id, &password).await?;

    let seat_pref = parse_seat_preference(&task.seat_preference);
    let target_train_numbers = extract_train_numbers(&task.target_trains);
    let passengers = parse_passengers(&task.passengers);

    // Gamma distribution: shape=4, scale=0.25 → mean ~1.0s + 0.25s base
    let gamma = Gamma::new(4.0, 0.25).expect("Invalid gamma params");

    info!(
        task_id = %task.id,
        trains = ?target_train_numbers,
        passengers = ?passengers,
        "SRT reservation loop starting"
    );

    loop {
        // Check if task was cancelled or max attempts reached
        let current = bominal_db::task::find_by_id(db, task.id, task.user_id).await?;
        match current.as_ref().map(|t| t.status.as_str()) {
            Some("cancelled") | Some("confirmed") | Some("failed") | None => {
                info!(task_id = %task.id, "Task terminated");
                return Ok(());
            }
            _ => {}
        }
        if let Some(ref t) = current {
            if t.attempt_count >= MAX_ATTEMPTS {
                warn!(task_id = %task.id, attempts = t.attempt_count, "Max attempts reached");
                bominal_db::task::update_status(db, task.id, "failed").await?;
                event_bus.publish(task.user_id, TaskEvent {
                    task_id: task.id,
                    status: "failed".to_string(),
                    message: format!("Max attempts ({MAX_ATTEMPTS}) reached"),
                    attempt_count: t.attempt_count,
                    reservation_number: None,
                }).await;
                return Ok(());
            }
        }

        // Record attempt
        bominal_db::task::record_attempt(db, task.id).await?;

        // Search for trains
        let search_result = client
            .search_train(
                &task.departure_station,
                &task.arrival_station,
                Some(&task.travel_date),
                Some(&task.departure_time),
                false,
            )
            .await;

        match search_result {
            Ok(trains) => {
                // Find first matching target train with available seats
                for target_no in &target_train_numbers {
                    let matching = trains.iter().find(|t| {
                        t.train_number == *target_no
                            && (t.seat_available() || t.reserve_standby_available())
                    });

                    if let Some(train) = matching {
                        info!(
                            task_id = %task.id,
                            train_number = %train.train_number,
                            "Found available train, attempting reservation"
                        );

                        let reserve_result = if train.seat_available() {
                            client
                                .reserve(
                                    train,
                                    &passengers,
                                    seat_pref,
                                    WindowSeat::None,
                                )
                                .await
                        } else {
                            client
                                .reserve_standby(
                                    train,
                                    &passengers,
                                    seat_pref,
                                    None,
                                )
                                .await
                        };

                        match reserve_result {
                            Ok(reservation) => {
                                info!(
                                    task_id = %task.id,
                                    pnr = %reservation.reservation_number,
                                    "Reservation confirmed!"
                                );
                                bominal_db::task::mark_confirmed(
                                    db,
                                    task.id,
                                    &reservation.reservation_number,
                                    &serde_json::json!({
                                        "dep_station": reservation.dep_station_name,
                                        "arr_station": reservation.arr_station_name,
                                        "dep_date": reservation.dep_date,
                                        "dep_time": reservation.dep_time,
                                        "train_number": reservation.train_number,
                                        "total_cost": reservation.total_cost,
                                        "is_waiting": reservation.is_waiting,
                                    }),
                                )
                                .await?;
                                event_bus.publish(task.user_id, TaskEvent {
                                    task_id: task.id,
                                    status: "confirmed".to_string(),
                                    message: format!(
                                        "Reservation confirmed! Train {} — PNR {}",
                                        reservation.train_number, reservation.reservation_number
                                    ),
                                    attempt_count: task.attempt_count,
                                    reservation_number: Some(reservation.reservation_number.clone()),
                                }).await;

                                // Send confirmation email
                                if task.notify_enabled {
                                    let kind = if reservation.is_waiting {
                                        AlertKind::WaitingConfirmed
                                    } else {
                                        AlertKind::Confirmed
                                    };
                                    send_alert_email(db, email, task, kind, Some(&reservation.reservation_number), app_base_url).await;
                                }

                                // Auto-pay if configured (skip for standby/waiting reservations)
                                if task.auto_pay && !reservation.is_waiting {
                                    let pnr = reservation.reservation_number.clone();
                                    try_auto_pay_srt(
                                        db, &client, task, &reservation, &pnr, event_bus, email,
                                        evervault, app_base_url,
                                    ).await;
                                }

                                return Ok(());
                            }
                            Err(ProviderError::DuplicateReservation) => {
                                warn!(task_id = %task.id, "Duplicate reservation");
                                bominal_db::task::update_status(db, task.id, "failed").await?;
                                event_bus.publish(task.user_id, TaskEvent {
                                    task_id: task.id,
                                    status: "failed".to_string(),
                                    message: "Duplicate reservation detected".to_string(),
                                    attempt_count: task.attempt_count,
                                    reservation_number: None,
                                }).await;
                                return Ok(());
                            }
                            Err(e) => {
                                warn!(task_id = %task.id, error = %e, "Reserve failed, continuing");
                            }
                        }
                    }
                }

                debug!(task_id = %task.id, "No matching trains available, retrying");
            }
            Err(ProviderError::NoResults) => {
                debug!(task_id = %task.id, "No results, continuing");
            }
            Err(ProviderError::SessionExpired) => {
                warn!(task_id = %task.id, "Session expired, re-logging in");
                if let Err(e) = client.login(&cred.login_id, &password).await {
                    error!(task_id = %task.id, error = %e, "Re-login failed");
                    bominal_db::task::update_status(db, task.id, "failed").await?;
                    return Ok(());
                }
            }
            Err(ProviderError::NetFunnelBlocked) => {
                debug!(task_id = %task.id, "NetFunnel blocked, retrying");
            }
            Err(ProviderError::NetworkError(e)) => {
                warn!(task_id = %task.id, error = %e, "Network error, retrying");
            }
            Err(e) => {
                warn!(task_id = %task.id, error = %e, "Search error, retrying");
            }
        }

        // Gamma-distributed sleep
        let delay_ms = {
            let mut rng = rand::rng();
            let delay = gamma.sample(&mut rng) + 0.25;
            (delay * 1000.0) as u64
        };
        tokio::time::sleep(std::time::Duration::from_millis(delay_ms)).await;
    }
}

/// Run the KTX reservation loop for a single task.
async fn run_ktx_task(
    db: &DbPool,
    task: &bominal_db::task::TaskRow,
    event_bus: &EventBus,
    email: &EmailClient,
    encryption_key: &EncryptionKey,
    evervault: &EvervaultConfig,
    app_base_url: &str,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let cred = bominal_db::provider::find_by_user_and_provider(db, task.user_id, "KTX")
        .await?
        .ok_or("No KTX credentials found")?;
    let password = encryption::decrypt(encryption_key, &cred.encrypted_password)?;

    let mut client = KtxClient::with_relay(&evervault.ktx_relay_domain);
    client.login(&cred.login_id, &password).await?;

    let seat_pref = parse_seat_preference(&task.seat_preference);
    let target_train_numbers = extract_train_numbers(&task.target_trains);
    let passengers = parse_passengers(&task.passengers);
    let psg_count = total_count(&passengers);

    let gamma = Gamma::new(4.0, 0.25).expect("Invalid gamma params");

    info!(
        task_id = %task.id,
        trains = ?target_train_numbers,
        passengers = psg_count,
        "KTX reservation loop starting"
    );

    loop {
        // Check if task was cancelled or max attempts reached
        let current = bominal_db::task::find_by_id(db, task.id, task.user_id).await?;
        match current.as_ref().map(|t| t.status.as_str()) {
            Some("cancelled") | Some("confirmed") | Some("failed") | None => {
                info!(task_id = %task.id, "Task terminated");
                return Ok(());
            }
            _ => {}
        }
        if let Some(ref t) = current {
            if t.attempt_count >= MAX_ATTEMPTS {
                warn!(task_id = %task.id, attempts = t.attempt_count, "Max attempts reached");
                bominal_db::task::update_status(db, task.id, "failed").await?;
                event_bus.publish(task.user_id, TaskEvent {
                    task_id: task.id,
                    status: "failed".to_string(),
                    message: format!("Max attempts ({MAX_ATTEMPTS}) reached"),
                    attempt_count: t.attempt_count,
                    reservation_number: None,
                }).await;
                return Ok(());
            }
        }

        bominal_db::task::record_attempt(db, task.id).await?;

        let search_result = client
            .search_train(
                &task.departure_station,
                &task.arrival_station,
                Some(&task.travel_date),
                Some(&task.departure_time),
                false,
            )
            .await;

        match search_result {
            Ok(trains) => {
                for target_no in &target_train_numbers {
                    let matching = trains.iter().find(|t| {
                        t.train_no == *target_no
                            && (t.seat_available() || t.waiting_available())
                    });

                    if let Some(train) = matching {
                        info!(
                            task_id = %task.id,
                            train_number = %train.train_no,
                            "Found available KTX train, attempting reservation"
                        );

                        let reserve_result = client
                            .reserve(train, seat_pref, psg_count)
                            .await;

                        match reserve_result {
                            Ok(reservation) => {
                                info!(
                                    task_id = %task.id,
                                    pnr = %reservation.rsv_id,
                                    "KTX reservation confirmed!"
                                );
                                bominal_db::task::mark_confirmed(
                                    db,
                                    task.id,
                                    &reservation.rsv_id,
                                    &serde_json::json!({
                                        "dep_station": reservation.dep_name,
                                        "arr_station": reservation.arr_name,
                                        "dep_date": reservation.dep_date,
                                        "dep_time": reservation.dep_time,
                                        "train_number": reservation.train_no,
                                        "total_cost": reservation.price,
                                        "is_waiting": reservation.is_waiting,
                                    }),
                                )
                                .await?;
                                event_bus.publish(task.user_id, TaskEvent {
                                    task_id: task.id,
                                    status: "confirmed".to_string(),
                                    message: format!(
                                        "KTX reservation confirmed! Train {} — PNR {}",
                                        reservation.train_no, reservation.rsv_id
                                    ),
                                    attempt_count: task.attempt_count,
                                    reservation_number: Some(reservation.rsv_id.clone()),
                                }).await;

                                // Send confirmation email
                                if task.notify_enabled {
                                    let kind = if reservation.is_waiting {
                                        AlertKind::WaitingConfirmed
                                    } else {
                                        AlertKind::Confirmed
                                    };
                                    send_alert_email(db, email, task, kind, Some(&reservation.rsv_id), app_base_url).await;
                                }

                                // Auto-pay if configured (skip for standby/waiting reservations)
                                if task.auto_pay && !reservation.is_waiting {
                                    let pnr = reservation.rsv_id.clone();
                                    try_auto_pay_ktx(
                                        db, &client, task, &reservation, &pnr, event_bus, email,
                                        evervault, app_base_url,
                                    ).await;
                                }

                                return Ok(());
                            }
                            Err(ProviderError::DuplicateReservation) => {
                                warn!(task_id = %task.id, "Duplicate reservation");
                                bominal_db::task::update_status(db, task.id, "failed").await?;
                                event_bus.publish(task.user_id, TaskEvent {
                                    task_id: task.id,
                                    status: "failed".to_string(),
                                    message: "Duplicate reservation detected".to_string(),
                                    attempt_count: task.attempt_count,
                                    reservation_number: None,
                                }).await;
                                return Ok(());
                            }
                            Err(e) => {
                                warn!(task_id = %task.id, error = %e, "KTX reserve failed, continuing");
                            }
                        }
                    }
                }

                debug!(task_id = %task.id, "No matching KTX trains available, retrying");
            }
            Err(ProviderError::NoResults) => {
                debug!(task_id = %task.id, "No results, continuing");
            }
            Err(ProviderError::SessionExpired) => {
                warn!(task_id = %task.id, "Session expired, re-logging in");
                if let Err(e) = client.login(&cred.login_id, &password).await {
                    error!(task_id = %task.id, error = %e, "Re-login failed");
                    bominal_db::task::update_status(db, task.id, "failed").await?;
                    return Ok(());
                }
            }
            Err(ProviderError::NetworkError(e)) => {
                warn!(task_id = %task.id, error = %e, "Network error, retrying");
            }
            Err(e) => {
                warn!(task_id = %task.id, error = %e, "Search error, retrying");
            }
        }

        let delay_ms = {
            let mut rng = rand::rng();
            let delay = gamma.sample(&mut rng) + 0.25;
            (delay * 1000.0) as u64
        };
        tokio::time::sleep(std::time::Duration::from_millis(delay_ms)).await;
    }
}

// ── Auto-pay ─────────────────────────────────────────────────────────

/// Attempt automatic payment for an SRT reservation.
///
/// Logs warnings on failure but never fails the task — the reservation
/// is still valid even if payment doesn't go through.
async fn try_auto_pay_srt(
    db: &DbPool,
    client: &SrtClient,
    task: &bominal_db::task::TaskRow,
    reservation: &bominal_provider::srt::reservation::SrtReservation,
    pnr: &str,
    event_bus: &EventBus,
    email: &EmailClient,
    _evervault: &EvervaultConfig,
    app_base_url: &str,
) {
    // Card fields are Evervault-encrypted; the Relay (configured on the client) decrypts in-flight.
    let card_id = match task.payment_card_id {
        Some(id) => id,
        None => {
            warn!(task_id = %task.id, "auto_pay enabled but no payment_card_id set");
            return;
        }
    };

    let card = match bominal_db::card::find_by_id(db, card_id, task.user_id).await {
        Ok(Some(c)) => c,
        Ok(None) => {
            warn!(task_id = %task.id, card_id = %card_id, "Payment card not found for auto-pay");
            event_bus.publish(task.user_id, TaskEvent {
                task_id: task.id,
                status: "pay_failed".to_string(),
                message: "Auto-pay failed: payment card not found".to_string(),
                attempt_count: task.attempt_count,
                reservation_number: Some(pnr.to_string()),
            }).await;
            return;
        }
        Err(e) => {
            error!(task_id = %task.id, error = %e, "DB error fetching card for auto-pay");
            return;
        }
    };

    // Card fields are Evervault-encrypted (ev: prefix). The Evervault Outbound
    // Relay decrypts them in-flight when the provider payment request passes
    // through the relay proxy. We pass the encrypted values directly.
    match client
        .pay_with_card(
            reservation,
            &card.encrypted_number,
            &card.encrypted_password,
            &card.encrypted_birthday,
            &card.encrypted_expiry,
            0, // 일시불 (lump-sum)
            &card.card_type,
        )
        .await
    {
        Ok(()) => {
            info!(task_id = %task.id, "SRT auto-pay successful");
            event_bus.publish(task.user_id, TaskEvent {
                task_id: task.id,
                status: "paid".to_string(),
                message: "Auto-pay completed successfully".to_string(),
                attempt_count: task.attempt_count,
                reservation_number: Some(pnr.to_string()),
            }).await;
            if task.notify_enabled {
                send_alert_email(db, email, task, AlertKind::Paid, Some(pnr), app_base_url).await;
            }
        }
        Err(e) => {
            warn!(task_id = %task.id, error = %e, "SRT auto-pay failed");
            event_bus.publish(task.user_id, TaskEvent {
                task_id: task.id,
                status: "pay_failed".to_string(),
                message: format!("Auto-pay failed: {e}. Please pay manually."),
                attempt_count: task.attempt_count,
                reservation_number: Some(pnr.to_string()),
            }).await;
            if task.notify_enabled {
                send_alert_email(db, email, task, AlertKind::PayFailed, Some(pnr), app_base_url).await;
            }
        }
    }
}

/// Attempt automatic payment for a KTX reservation.
async fn try_auto_pay_ktx(
    db: &DbPool,
    client: &KtxClient,
    task: &bominal_db::task::TaskRow,
    reservation: &bominal_provider::ktx::reservation::KtxReservation,
    pnr: &str,
    event_bus: &EventBus,
    email: &EmailClient,
    _evervault: &EvervaultConfig,
    app_base_url: &str,
) {
    // Card fields are Evervault-encrypted; the Relay (configured on the client) decrypts in-flight.
    let card_id = match task.payment_card_id {
        Some(id) => id,
        None => {
            warn!(task_id = %task.id, "auto_pay enabled but no payment_card_id set");
            return;
        }
    };

    let card = match bominal_db::card::find_by_id(db, card_id, task.user_id).await {
        Ok(Some(c)) => c,
        Ok(None) => {
            warn!(task_id = %task.id, card_id = %card_id, "Payment card not found for auto-pay");
            event_bus.publish(task.user_id, TaskEvent {
                task_id: task.id,
                status: "pay_failed".to_string(),
                message: "Auto-pay failed: payment card not found".to_string(),
                attempt_count: task.attempt_count,
                reservation_number: Some(pnr.to_string()),
            }).await;
            return;
        }
        Err(e) => {
            error!(task_id = %task.id, error = %e, "DB error fetching card for auto-pay");
            return;
        }
    };

    // Card fields are Evervault-encrypted. Passed through Evervault Relay.
    match client
        .pay_with_card(
            reservation,
            &card.encrypted_number,
            &card.encrypted_password,
            &card.encrypted_birthday,
            &card.encrypted_expiry,
            "00", // 일시불 (lump-sum)
            &card.card_type,
        )
        .await
    {
        Ok(()) => {
            info!(task_id = %task.id, "KTX auto-pay successful");
            event_bus.publish(task.user_id, TaskEvent {
                task_id: task.id,
                status: "paid".to_string(),
                message: "Auto-pay completed successfully".to_string(),
                attempt_count: task.attempt_count,
                reservation_number: Some(pnr.to_string()),
            }).await;
            if task.notify_enabled {
                send_alert_email(db, email, task, AlertKind::Paid, Some(pnr), app_base_url).await;
            }
        }
        Err(e) => {
            warn!(task_id = %task.id, error = %e, "KTX auto-pay failed");
            event_bus.publish(task.user_id, TaskEvent {
                task_id: task.id,
                status: "pay_failed".to_string(),
                message: format!("Auto-pay failed: {e}. Please pay manually."),
                attempt_count: task.attempt_count,
                reservation_number: Some(pnr.to_string()),
            }).await;
            if task.notify_enabled {
                send_alert_email(db, email, task, AlertKind::PayFailed, Some(pnr), app_base_url).await;
            }
        }
    }
}

// ── Email alerts ─────────────────────────────────────────────────────

/// Send a reservation alert email (best-effort, never fails the task).
async fn send_alert_email(
    db: &DbPool,
    email: &EmailClient,
    task: &bominal_db::task::TaskRow,
    kind: AlertKind,
    pnr: Option<&str>,
    app_base_url: &str,
) {
    let user = match bominal_db::user::find_by_id(db, task.user_id).await {
        Ok(Some(u)) => u,
        _ => return,
    };

    let details = ReservationDetails {
        provider: &task.provider,
        train_number: &extract_train_numbers(&task.target_trains)
            .first()
            .cloned()
            .unwrap_or_default(),
        dep_station: &task.departure_station,
        arr_station: &task.arrival_station,
        dep_date: &task.travel_date,
        dep_time: &task.departure_time,
        pnr,
        total_cost: None,
    };

    let app_url = app_base_url;
    let (subject, html) = bominal_email::templates::reservation::render(
        &user.display_name,
        kind,
        &details,
        app_url,
    );

    email.send_best_effort(&user.email, &subject, &html).await;
}

// ── Helpers ──────────────────────────────────────────────────────────

fn parse_seat_preference(s: &str) -> SeatPreference {
    match s {
        "GeneralOnly" => SeatPreference::GeneralOnly,
        "SpecialOnly" => SeatPreference::SpecialOnly,
        "SpecialFirst" => SeatPreference::SpecialFirst,
        _ => SeatPreference::GeneralFirst,
    }
}

fn extract_train_numbers(trains: &serde_json::Value) -> Vec<String> {
    trains
        .as_array()
        .map(|arr| {
            arr.iter()
                .filter_map(|v| {
                    v.get("train_number")
                        .and_then(|n| n.as_str())
                        .map(String::from)
                })
                .collect()
        })
        .unwrap_or_default()
}

/// Parse the passengers JSON array into `PassengerGroup` list.
///
/// Expected format: `[{"type": "adult", "count": 1}, {"type": "child", "count": 2}]`
/// Falls back to 1 adult if parsing fails.
fn parse_passengers(passengers: &serde_json::Value) -> Vec<PassengerGroup> {
    let groups: Vec<PassengerGroup> = passengers
        .as_array()
        .map(|arr| {
            arr.iter()
                .filter_map(|v| {
                    let ptype = v.get("type").and_then(|t| t.as_str())?;
                    let count = v.get("count").and_then(|c| c.as_u64())? as u8;
                    if count == 0 {
                        return None;
                    }
                    let passenger_type = match ptype {
                        "adult" => PassengerType::Adult,
                        "child" => PassengerType::Child,
                        "senior" => PassengerType::Senior,
                        "disability1to3" => PassengerType::Disability1To3,
                        "disability4to6" => PassengerType::Disability4To6,
                        _ => return None,
                    };
                    Some(PassengerGroup::new(passenger_type, count))
                })
                .collect()
        })
        .unwrap_or_default();

    if groups.is_empty() {
        vec![PassengerGroup::adults(1)]
    } else {
        groups
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_seat_preferences() {
        assert_eq!(parse_seat_preference("GeneralFirst"), SeatPreference::GeneralFirst);
        assert_eq!(parse_seat_preference("SpecialOnly"), SeatPreference::SpecialOnly);
        assert_eq!(parse_seat_preference("unknown"), SeatPreference::GeneralFirst);
    }

    #[test]
    fn extract_trains() {
        let trains = serde_json::json!([
            {"train_number": "305", "dep_time": "090000"},
            {"train_number": "307", "dep_time": "100000"},
        ]);
        let numbers = extract_train_numbers(&trains);
        assert_eq!(numbers, vec!["305", "307"]);
    }

    #[test]
    fn extract_trains_empty() {
        let trains = serde_json::json!([]);
        assert!(extract_train_numbers(&trains).is_empty());
    }

    #[test]
    fn extract_trains_null() {
        let trains = serde_json::json!(null);
        assert!(extract_train_numbers(&trains).is_empty());
    }

    #[test]
    fn parse_passengers_single_adult() {
        let json = serde_json::json!([{"type": "adult", "count": 1}]);
        let groups = parse_passengers(&json);
        assert_eq!(groups.len(), 1);
        assert_eq!(groups[0].passenger_type, PassengerType::Adult);
        assert_eq!(groups[0].count, 1);
    }

    #[test]
    fn parse_passengers_mixed() {
        let json = serde_json::json!([
            {"type": "adult", "count": 2},
            {"type": "child", "count": 1},
            {"type": "senior", "count": 1}
        ]);
        let groups = parse_passengers(&json);
        assert_eq!(groups.len(), 3);
        assert_eq!(groups[0].count, 2);
        assert_eq!(groups[1].passenger_type, PassengerType::Child);
        assert_eq!(groups[2].passenger_type, PassengerType::Senior);
    }

    #[test]
    fn parse_passengers_fallback_on_empty() {
        let json = serde_json::json!([]);
        let groups = parse_passengers(&json);
        assert_eq!(groups.len(), 1);
        assert_eq!(groups[0].passenger_type, PassengerType::Adult);
        assert_eq!(groups[0].count, 1);
    }

    #[test]
    fn parse_passengers_fallback_on_null() {
        let json = serde_json::json!(null);
        let groups = parse_passengers(&json);
        assert_eq!(groups.len(), 1);
        assert_eq!(groups[0].passenger_type, PassengerType::Adult);
    }

    #[test]
    fn parse_passengers_skips_unknown_type() {
        let json = serde_json::json!([
            {"type": "vip", "count": 1},
            {"type": "adult", "count": 1}
        ]);
        let groups = parse_passengers(&json);
        assert_eq!(groups.len(), 1);
        assert_eq!(groups[0].passenger_type, PassengerType::Adult);
    }

    #[test]
    fn parse_passengers_skips_zero_count() {
        let json = serde_json::json!([
            {"type": "adult", "count": 0},
            {"type": "child", "count": 2}
        ]);
        let groups = parse_passengers(&json);
        assert_eq!(groups.len(), 1);
        assert_eq!(groups[0].passenger_type, PassengerType::Child);
    }
}
