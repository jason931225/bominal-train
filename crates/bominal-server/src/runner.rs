//! Background reservation task runner — unified multi-provider loop.
//!
//! Spawns a tokio task per active reservation that loops:
//! 1. Search for target trains across all referenced providers
//! 2. Walk targets in priority order — first hit with available seats wins
//! 3. If seat available -> reserve -> mark confirmed -> optional auto-pay
//! 4. Sleep with gamma-distributed delay, retry

use std::collections::HashMap;

use rand_distr::{Distribution, Gamma};
use tracing::{debug, error, info, warn};

const MAX_ATTEMPTS: i32 = 10_000;

use bominal_db::DbPool;
use bominal_domain::crypto::encryption::{self, EncryptionKey};
use bominal_domain::task::{
    PassengerKind, PassengerList, Provider as TaskProvider, ReservationSnapshot,
    SeatPreference as DomainSeatPreference, TargetTrainList, TaskStatus,
};
use bominal_email::EmailClient;
use bominal_email::templates::reservation::{AlertKind, ReservationDetails};
use bominal_provider::ktx::KtxClient;
use bominal_provider::srt::SrtClient;
use bominal_provider::srt::passenger::{PassengerGroup, PassengerType, WindowSeat, total_count};
use bominal_provider::types::{ProviderError, SeatPreference as ProviderSeatPreference};

use bominal_domain::evervault::EvervaultConfig;

use crate::sse::{EventBus, TaskEvent};

// ── Provider session cache ──────────────────────────────────────────

/// Holds an authenticated provider client for the lifetime of a task worker.
enum ProviderSession {
    Srt {
        client: SrtClient,
        login_id: String,
        password: String,
    },
    Ktx {
        client: KtxClient,
        login_id: String,
        password: String,
    },
}

/// Build authenticated sessions for every unique provider in the target list.
async fn build_sessions(
    db: &DbPool,
    user_id: uuid::Uuid,
    target_trains: &TargetTrainList,
    encryption_key: &EncryptionKey,
    evervault: &EvervaultConfig,
) -> Result<HashMap<TaskProvider, ProviderSession>, Box<dyn std::error::Error + Send + Sync>> {
    let mut sessions = HashMap::new();

    for provider in target_trains.unique_providers() {
        let cred =
            bominal_db::provider::find_by_user_and_provider(db, user_id, provider.as_str())
                .await?
                .ok_or_else(|| format!("No {} credentials found", provider))?;
        let password = encryption::decrypt(encryption_key, &cred.encrypted_password)?;

        match provider {
            TaskProvider::Srt => {
                let mut client = SrtClient::with_relay(&evervault.srt_relay_domain);
                client.login(&cred.login_id, &password).await?;
                sessions.insert(
                    provider,
                    ProviderSession::Srt {
                        client,
                        login_id: cred.login_id,
                        password,
                    },
                );
            }
            TaskProvider::Ktx => {
                let mut client = KtxClient::with_relay(&evervault.ktx_relay_domain);
                client.login(&cred.login_id, &password).await?;
                sessions.insert(
                    provider,
                    ProviderSession::Ktx {
                        client,
                        login_id: cred.login_id,
                        password,
                    },
                );
            }
        }
    }

    Ok(sessions)
}

// ── Runner entry point ──────────────────────────────────────────────

/// Start the task runner as a background tokio task.
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
            if let Err(e) = poll_and_dispatch(
                &db,
                &event_bus,
                &email,
                &encryption_key,
                &evervault,
                &app_base_url,
            )
            .await
            {
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

            bus.publish(
                task.user_id,
                TaskEvent {
                    task_id: task.id,
                    status: "running".to_string(),
                    message: "Reservation worker started".to_string(),
                    attempt_count: 0,
                    reservation_number: None,
                },
            )
            .await;

            let result = run_task(&db, &task, &bus, &email, &key, &ev, &base_url).await;

            if let Err(e) = result {
                error!(task_id = %task.id, error = %e, "Task failed");
                if let Err(db_err) =
                    bominal_db::task::update_status(&db, task.id, TaskStatus::Failed).await
                {
                    error!(task_id = %task.id, error = %db_err, "Failed to mark task as failed");
                }
                bus.publish(
                    task.user_id,
                    TaskEvent {
                        task_id: task.id,
                        status: "failed".to_string(),
                        message: format!("Task failed: {e}"),
                        attempt_count: 0,
                        reservation_number: None,
                    },
                )
                .await;

                if task.notify_enabled {
                    send_alert_email(&db, &email, &task, AlertKind::Failed, None, &base_url).await;
                }
            }
        });
    }

    Ok(())
}

// ── Unified task loop ───────────────────────────────────────────────

/// Run the reservation loop for a single task, regardless of provider mix.
///
/// Targets are walked in priority (ordinal) order. Each target carries its own
/// provider, so a single task can search SRT and KTX simultaneously.
async fn run_task(
    db: &DbPool,
    task: &bominal_db::task::TaskRow,
    event_bus: &EventBus,
    email: &EmailClient,
    encryption_key: &EncryptionKey,
    evervault: &EvervaultConfig,
    app_base_url: &str,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let mut sessions =
        build_sessions(db, task.user_id, &task.target_trains, encryption_key, evervault).await?;

    let seat_pref = parse_seat_preference(task.seat_preference);
    let passengers = parse_passengers(&task.passengers);
    let psg_count = total_count(&passengers);

    let gamma = Gamma::new(4.0, 0.25).expect("Invalid gamma params");

    info!(
        task_id = %task.id,
        providers = ?task.target_trains.unique_providers(),
        target_count = task.target_trains.0.len(),
        "Unified reservation loop starting"
    );

    loop {
        // ── Check termination conditions ────────────────────────
        let current = bominal_db::task::find_by_id(db, task.id, task.user_id).await?;
        match current.as_ref().map(|t| t.status.as_str()) {
            Some("cancelled") | Some("confirmed") | Some("failed") | None => {
                info!(task_id = %task.id, "Task terminated");
                return Ok(());
            }
            _ => {}
        }
        if let Some(ref t) = current
            && t.attempt_count >= MAX_ATTEMPTS
        {
            warn!(task_id = %task.id, attempts = t.attempt_count, "Max attempts reached");
            bominal_db::task::update_status(db, task.id, TaskStatus::Failed).await?;
            event_bus
                .publish(
                    task.user_id,
                    TaskEvent {
                        task_id: task.id,
                        status: "failed".to_string(),
                        message: format!("Max attempts ({MAX_ATTEMPTS}) reached"),
                        attempt_count: t.attempt_count,
                        reservation_number: None,
                    },
                )
                .await;
            return Ok(());
        }

        bominal_db::task::record_attempt(db, task.id).await?;

        // ── Search once per unique provider ─────────────────────
        let mut search_cache: HashMap<TaskProvider, SearchOutcome> = HashMap::new();

        for provider in task.target_trains.unique_providers() {
            let outcome = search_provider(
                &mut sessions,
                provider,
                &task.departure_station,
                &task.arrival_station,
                &task.travel_date,
                &task.departure_time,
            )
            .await;
            search_cache.insert(provider, outcome);
        }

        // ── Walk targets in priority order ──────────────────────
        let mut should_fail = false;
        let mut fail_message = String::new();

        for target in &task.target_trains.0 {
            let outcome = match search_cache.get(&target.provider) {
                Some(o) => o,
                None => continue,
            };

            let trains = match outcome {
                SearchOutcome::Ok(trains) => trains,
                SearchOutcome::NoResults | SearchOutcome::NetFunnelBlocked => continue,
                SearchOutcome::SessionExpired => {
                    // Re-login this provider
                    if let Err(e) = relogin_session(&mut sessions, target.provider).await {
                        error!(task_id = %task.id, provider = %target.provider, error = %e, "Re-login failed");
                        bominal_db::task::update_status(db, task.id, TaskStatus::Failed).await?;
                        return Ok(());
                    }
                    continue;
                }
                SearchOutcome::Error(msg) => {
                    if !task.auto_retry {
                        should_fail = true;
                        fail_message = format!("Search error (auto-retry disabled): {msg}");
                    }
                    continue;
                }
            };

            // Try to reserve this specific target
            let reserve_result = try_reserve_target(
                &mut sessions,
                target.provider,
                &target.train_number,
                trains,
                &passengers,
                seat_pref,
                psg_count,
            )
            .await;

            match reserve_result {
                ReserveOutcome::Confirmed {
                    pnr,
                    snapshot,
                    is_waiting,
                } => {
                    info!(
                        task_id = %task.id,
                        provider = %target.provider,
                        pnr = %pnr,
                        "Reservation confirmed!"
                    );
                    bominal_db::task::mark_confirmed(
                        db,
                        task.id,
                        &pnr,
                        &snapshot,
                        target.provider,
                    )
                    .await?;
                    event_bus
                        .publish(
                            task.user_id,
                            TaskEvent {
                                task_id: task.id,
                                status: "confirmed".to_string(),
                                message: format!(
                                    "Reservation confirmed! {} Train {} — PNR {}",
                                    target.provider, target.train_number, pnr
                                ),
                                attempt_count: task.attempt_count,
                                reservation_number: Some(pnr.clone()),
                            },
                        )
                        .await;

                    if task.notify_enabled {
                        let kind = if is_waiting {
                            AlertKind::WaitingConfirmed
                        } else {
                            AlertKind::Confirmed
                        };
                        send_alert_email(db, email, task, kind, Some(&pnr), app_base_url).await;
                    }

                    if task.auto_pay && !is_waiting {
                        try_auto_pay(
                            db,
                            &mut sessions,
                            target.provider,
                            task,
                            &pnr,
                            event_bus,
                            email,
                            app_base_url,
                        )
                        .await;
                    }

                    return Ok(());
                }
                ReserveOutcome::Duplicate => {
                    warn!(task_id = %task.id, "Duplicate reservation");
                    bominal_db::task::update_status(db, task.id, TaskStatus::Failed).await?;
                    event_bus
                        .publish(
                            task.user_id,
                            TaskEvent {
                                task_id: task.id,
                                status: "failed".to_string(),
                                message: "Duplicate reservation detected".to_string(),
                                attempt_count: task.attempt_count,
                                reservation_number: None,
                            },
                        )
                        .await;
                    return Ok(());
                }
                ReserveOutcome::NoMatch | ReserveOutcome::ReserveFailed => {
                    // Continue to next target
                }
            }
        }

        // If any provider had a fatal error and auto-retry is off, fail now
        if should_fail {
            bominal_db::task::update_status(db, task.id, TaskStatus::Failed).await?;
            event_bus
                .publish(
                    task.user_id,
                    TaskEvent {
                        task_id: task.id,
                        status: "failed".to_string(),
                        message: fail_message,
                        attempt_count: task.attempt_count,
                        reservation_number: None,
                    },
                )
                .await;
            return Ok(());
        }

        debug!(task_id = %task.id, "No matching trains available, retrying");

        // Gamma-distributed sleep
        let delay_ms = {
            let mut rng = rand::rng();
            let delay = gamma.sample(&mut rng) + 0.25;
            (delay * 1000.0) as u64
        };
        tokio::time::sleep(std::time::Duration::from_millis(delay_ms)).await;
    }
}

// ── Search abstraction ──────────────────────────────────────────────

enum SearchOutcome {
    Ok(Vec<SearchTrain>),
    NoResults,
    SessionExpired,
    NetFunnelBlocked,
    Error(String),
}

/// Unified search result that normalizes SRT/KTX train data.
struct SearchTrain {
    train_number: String,
    seat_available: bool,
    standby_available: bool,
    /// SRT-specific: the raw train for reservation calls
    srt_train: Option<bominal_provider::srt::train::SrtTrain>,
    /// KTX-specific: the raw train for reservation calls
    ktx_train: Option<bominal_provider::ktx::train::KtxTrain>,
}

async fn search_provider(
    sessions: &mut HashMap<TaskProvider, ProviderSession>,
    provider: TaskProvider,
    dep_station: &str,
    arr_station: &str,
    travel_date: &str,
    departure_time: &str,
) -> SearchOutcome {
    match sessions.get_mut(&provider) {
        Some(ProviderSession::Srt { client, .. }) => {
            let result = bominal_provider::retry_with_backoff!(
                3,
                client
                    .search_train(dep_station, arr_station, Some(travel_date), Some(departure_time), false)
                    .await
            );
            match result {
                Ok(trains) => SearchOutcome::Ok(
                    trains
                        .into_iter()
                        .map(|t| SearchTrain {
                            train_number: t.train_number.clone(),
                            seat_available: t.seat_available(),
                            standby_available: t.reserve_standby_available(),
                            srt_train: Some(t),
                            ktx_train: None,
                        })
                        .collect(),
                ),
                Err(ProviderError::NoResults) => SearchOutcome::NoResults,
                Err(ProviderError::SessionExpired) => SearchOutcome::SessionExpired,
                Err(ProviderError::NetFunnelBlocked) => SearchOutcome::NetFunnelBlocked,
                Err(e) => SearchOutcome::Error(e.to_string()),
            }
        }
        Some(ProviderSession::Ktx { client, .. }) => {
            let result = bominal_provider::retry_with_backoff!(
                3,
                client
                    .search_train(dep_station, arr_station, Some(travel_date), Some(departure_time), false)
                    .await
            );
            match result {
                Ok(trains) => SearchOutcome::Ok(
                    trains
                        .into_iter()
                        .map(|t| SearchTrain {
                            train_number: t.train_no.clone(),
                            seat_available: t.seat_available(),
                            standby_available: t.waiting_available(),
                            srt_train: None,
                            ktx_train: Some(t),
                        })
                        .collect(),
                ),
                Err(ProviderError::NoResults) => SearchOutcome::NoResults,
                Err(ProviderError::SessionExpired) => SearchOutcome::SessionExpired,
                Err(e) => SearchOutcome::Error(e.to_string()),
            }
        }
        None => SearchOutcome::Error(format!("No session for {provider}")),
    }
}

async fn relogin_session(
    sessions: &mut HashMap<TaskProvider, ProviderSession>,
    provider: TaskProvider,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    match sessions.get_mut(&provider) {
        Some(ProviderSession::Srt {
            client,
            login_id,
            password,
        }) => {
            client.login(login_id, password).await?;
        }
        Some(ProviderSession::Ktx {
            client,
            login_id,
            password,
        }) => {
            client.login(login_id, password).await?;
        }
        None => return Err(format!("No session for {provider}").into()),
    }
    Ok(())
}

// ── Reserve abstraction ─────────────────────────────────────────────

enum ReserveOutcome {
    Confirmed {
        pnr: String,
        snapshot: ReservationSnapshot,
        is_waiting: bool,
    },
    Duplicate,
    NoMatch,
    ReserveFailed,
}

async fn try_reserve_target(
    sessions: &mut HashMap<TaskProvider, ProviderSession>,
    provider: TaskProvider,
    target_train_number: &str,
    trains: &[SearchTrain],
    passengers: &[PassengerGroup],
    seat_pref: ProviderSeatPreference,
    psg_count: u8,
) -> ReserveOutcome {
    let matching = trains.iter().find(|t| {
        t.train_number == target_train_number && (t.seat_available || t.standby_available)
    });

    let train = match matching {
        Some(t) => t,
        None => return ReserveOutcome::NoMatch,
    };

    match provider {
        TaskProvider::Srt => {
            let session = match sessions.get_mut(&provider) {
                Some(ProviderSession::Srt { client, .. }) => client,
                _ => return ReserveOutcome::ReserveFailed,
            };
            let srt_train = match &train.srt_train {
                Some(t) => t,
                None => return ReserveOutcome::ReserveFailed,
            };

            let reserve_result = if train.seat_available {
                bominal_provider::retry_with_backoff!(
                    3,
                    session
                        .reserve(srt_train, passengers, seat_pref, WindowSeat::None)
                        .await
                )
            } else {
                bominal_provider::retry_with_backoff!(
                    3,
                    session
                        .reserve_standby(srt_train, passengers, seat_pref, None)
                        .await
                )
            };

            match reserve_result {
                Ok(reservation) => ReserveOutcome::Confirmed {
                    pnr: reservation.reservation_number.clone(),
                    snapshot: ReservationSnapshot {
                        dep_station: reservation.dep_station_name.clone(),
                        arr_station: reservation.arr_station_name.clone(),
                        dep_date: reservation.dep_date.clone(),
                        dep_time: reservation.dep_time.clone(),
                        train_number: reservation.train_number.clone(),
                        total_cost: reservation.total_cost.clone(),
                        is_waiting: reservation.is_waiting,
                    },
                    is_waiting: reservation.is_waiting,
                },
                Err(ProviderError::DuplicateReservation) => ReserveOutcome::Duplicate,
                Err(e) => {
                    warn!(error = %e, "SRT reserve failed, continuing");
                    ReserveOutcome::ReserveFailed
                }
            }
        }
        TaskProvider::Ktx => {
            let session = match sessions.get_mut(&provider) {
                Some(ProviderSession::Ktx { client, .. }) => client,
                _ => return ReserveOutcome::ReserveFailed,
            };
            let ktx_train = match &train.ktx_train {
                Some(t) => t,
                None => return ReserveOutcome::ReserveFailed,
            };

            let reserve_result = bominal_provider::retry_with_backoff!(
                3,
                session.reserve(ktx_train, seat_pref, psg_count).await
            );

            match reserve_result {
                Ok(reservation) => ReserveOutcome::Confirmed {
                    pnr: reservation.rsv_id.clone(),
                    snapshot: ReservationSnapshot {
                        dep_station: reservation.dep_name.clone(),
                        arr_station: reservation.arr_name.clone(),
                        dep_date: reservation.dep_date.clone(),
                        dep_time: reservation.dep_time.clone(),
                        train_number: reservation.train_no.clone(),
                        total_cost: reservation.price.clone(),
                        is_waiting: reservation.is_waiting,
                    },
                    is_waiting: reservation.is_waiting,
                },
                Err(ProviderError::DuplicateReservation) => ReserveOutcome::Duplicate,
                Err(e) => {
                    warn!(error = %e, "KTX reserve failed, continuing");
                    ReserveOutcome::ReserveFailed
                }
            }
        }
    }
}

// ── Auto-pay (unified) ─────────────────────────────────────────────

/// Attempt automatic payment for a confirmed reservation.
///
/// Logs warnings on failure but never fails the task — the reservation
/// is still valid even if payment doesn't go through.
#[allow(clippy::too_many_arguments)]
async fn try_auto_pay(
    db: &DbPool,
    sessions: &mut HashMap<TaskProvider, ProviderSession>,
    provider: TaskProvider,
    task: &bominal_db::task::TaskRow,
    pnr: &str,
    event_bus: &EventBus,
    email: &EmailClient,
    app_base_url: &str,
) {
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
            set_awaiting_payment(db, task, pnr, event_bus, "Auto-pay failed: payment card not found. Please pay manually.").await;
            return;
        }
        Err(e) => {
            error!(task_id = %task.id, error = %e, "DB error fetching card for auto-pay");
            return;
        }
    };

    let pay_result = match provider {
        TaskProvider::Srt => {
            if let Some(ProviderSession::Srt { client, .. }) = sessions.get_mut(&provider) {
                // Fetch the reservation list to get the SRT reservation object for payment
                let reservations = match client.get_reservations().await {
                    Ok(r) => r,
                    Err(e) => {
                        warn!(task_id = %task.id, error = %e, "Failed to list reservations for payment");
                        set_awaiting_payment(db, task, pnr, event_bus, &format!("Auto-pay failed: {e}. Please pay manually.")).await;
                        return;
                    }
                };
                let reservation = match reservations.iter().find(|r| r.reservation_number == pnr) {
                    Some(r) => r,
                    None => {
                        warn!(task_id = %task.id, "Reservation not found in SRT list for payment");
                        set_awaiting_payment(db, task, pnr, event_bus, "Auto-pay failed: reservation not found. Please pay manually.").await;
                        return;
                    }
                };
                let srt_expiry = card
                    .encrypted_expiry_yymm
                    .as_deref()
                    .unwrap_or(&card.encrypted_expiry);
                client
                    .pay_with_card(
                        reservation,
                        &card.encrypted_number,
                        &card.encrypted_password,
                        &card.encrypted_birthday,
                        srt_expiry,
                        0,
                        &card.card_type,
                    )
                    .await
            } else {
                return;
            }
        }
        TaskProvider::Ktx => {
            if let Some(ProviderSession::Ktx { client, .. }) = sessions.get_mut(&provider) {
                let reservations = match client.get_reservations().await {
                    Ok(r) => r,
                    Err(e) => {
                        warn!(task_id = %task.id, error = %e, "Failed to list reservations for payment");
                        set_awaiting_payment(db, task, pnr, event_bus, &format!("Auto-pay failed: {e}. Please pay manually.")).await;
                        return;
                    }
                };
                let reservation = match reservations.iter().find(|r| r.rsv_id == pnr) {
                    Some(r) => r,
                    None => {
                        warn!(task_id = %task.id, "Reservation not found in KTX list for payment");
                        set_awaiting_payment(db, task, pnr, event_bus, "Auto-pay failed: reservation not found. Please pay manually.").await;
                        return;
                    }
                };
                client
                    .pay_with_card(
                        reservation,
                        &card.encrypted_number,
                        &card.encrypted_password,
                        &card.encrypted_birthday,
                        &card.encrypted_expiry,
                        "00",
                        &card.card_type,
                    )
                    .await
            } else {
                return;
            }
        }
    };

    match pay_result {
        Ok(()) => {
            info!(task_id = %task.id, provider = %provider, "Auto-pay successful");
            if task.notify_enabled {
                send_alert_email(db, email, task, AlertKind::Paid, Some(pnr), app_base_url).await;
            }
        }
        Err(e) => {
            warn!(task_id = %task.id, provider = %provider, error = %e, "Auto-pay failed");
            set_awaiting_payment(
                db,
                task,
                pnr,
                event_bus,
                &format!("Auto-pay failed: {e}. Please pay manually."),
            )
            .await;
            if task.notify_enabled {
                send_alert_email(db, email, task, AlertKind::PayFailed, Some(pnr), app_base_url)
                    .await;
            }
        }
    }
}

/// Helper: set task to AwaitingPayment and publish event.
async fn set_awaiting_payment(
    db: &DbPool,
    task: &bominal_db::task::TaskRow,
    pnr: &str,
    event_bus: &EventBus,
    message: &str,
) {
    if let Err(e) = bominal_db::task::update_status(db, task.id, TaskStatus::AwaitingPayment).await
    {
        error!(task_id = %task.id, error = %e, "Failed to update task status to AwaitingPayment");
    }
    event_bus
        .publish(
            task.user_id,
            TaskEvent {
                task_id: task.id,
                status: "awaiting_payment".to_string(),
                message: message.to_string(),
                attempt_count: task.attempt_count,
                reservation_number: Some(pnr.to_string()),
            },
        )
        .await;
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
        provider: task.provider.as_str(),
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
    let (subject, html) =
        bominal_email::templates::reservation::render(&user.display_name, kind, &details, app_url);

    email.send_best_effort(&user.email, &subject, &html).await;
}

// ── Helpers ──────────────────────────────────────────────────────────

fn parse_seat_preference(preference: DomainSeatPreference) -> ProviderSeatPreference {
    match preference {
        DomainSeatPreference::GeneralFirst => ProviderSeatPreference::GeneralFirst,
        DomainSeatPreference::SpecialFirst => ProviderSeatPreference::SpecialFirst,
        DomainSeatPreference::GeneralOnly => ProviderSeatPreference::GeneralOnly,
        DomainSeatPreference::SpecialOnly => ProviderSeatPreference::SpecialOnly,
    }
}

fn extract_train_numbers(trains: &TargetTrainList) -> Vec<String> {
    trains
        .0
        .iter()
        .map(|train| train.train_number.clone())
        .collect()
}

/// Convert typed passengers into provider groups.
///
/// Unsupported types are currently filtered at this boundary until the provider
/// layer grows explicit mappings for them.
fn parse_passengers(passengers: &PassengerList) -> Vec<PassengerGroup> {
    let groups: Vec<PassengerGroup> = passengers
        .0
        .iter()
        .filter_map(|passenger| {
            if passenger.count == 0 {
                return None;
            }

            let passenger_type = match passenger.kind {
                PassengerKind::Adult => PassengerType::Adult,
                PassengerKind::Child => PassengerType::Child,
                PassengerKind::Senior => PassengerType::Senior,
                PassengerKind::Severe => PassengerType::SevereDisability,
                PassengerKind::Mild => PassengerType::MildDisability,
                PassengerKind::Infant | PassengerKind::Merit => return None,
            };
            Some(PassengerGroup::new(passenger_type, passenger.count))
        })
        .collect();

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
        assert_eq!(
            parse_seat_preference(DomainSeatPreference::GeneralFirst),
            ProviderSeatPreference::GeneralFirst
        );
        assert_eq!(
            parse_seat_preference(DomainSeatPreference::SpecialOnly),
            ProviderSeatPreference::SpecialOnly
        );
        assert_eq!(
            parse_seat_preference(DomainSeatPreference::GeneralOnly),
            ProviderSeatPreference::GeneralOnly
        );
    }

    #[test]
    fn extract_trains() {
        let trains = TargetTrainList(vec![
            bominal_domain::task::TargetTrain {
                provider: TaskProvider::Srt,
                train_number: "305".to_string(),
                dep_time: "090000".to_string(),
            },
            bominal_domain::task::TargetTrain {
                provider: TaskProvider::Ktx,
                train_number: "101".to_string(),
                dep_time: "100000".to_string(),
            },
        ]);
        let numbers = extract_train_numbers(&trains);
        assert_eq!(numbers, vec!["305", "101"]);
    }

    #[test]
    fn extract_trains_empty() {
        let trains = TargetTrainList(vec![]);
        assert!(extract_train_numbers(&trains).is_empty());
    }

    #[test]
    fn parse_passengers_single_adult() {
        let passengers = PassengerList(vec![bominal_domain::task::PassengerCount::new(
            PassengerKind::Adult,
            1,
        )]);
        let groups = parse_passengers(&passengers);
        assert_eq!(groups.len(), 1);
        assert_eq!(groups[0].passenger_type, PassengerType::Adult);
        assert_eq!(groups[0].count, 1);
    }

    #[test]
    fn parse_passengers_mixed() {
        let passengers = PassengerList(vec![
            bominal_domain::task::PassengerCount::new(PassengerKind::Adult, 2),
            bominal_domain::task::PassengerCount::new(PassengerKind::Child, 1),
            bominal_domain::task::PassengerCount::new(PassengerKind::Senior, 1),
        ]);
        let groups = parse_passengers(&passengers);
        assert_eq!(groups.len(), 3);
        assert_eq!(groups[0].count, 2);
        assert_eq!(groups[1].passenger_type, PassengerType::Child);
        assert_eq!(groups[2].passenger_type, PassengerType::Senior);
    }

    #[test]
    fn parse_passengers_fallback_on_empty() {
        let passengers = PassengerList(vec![]);
        let groups = parse_passengers(&passengers);
        assert_eq!(groups.len(), 1);
        assert_eq!(groups[0].passenger_type, PassengerType::Adult);
        assert_eq!(groups[0].count, 1);
    }

    #[test]
    fn parse_passengers_skips_zero_count() {
        let passengers = PassengerList(vec![
            bominal_domain::task::PassengerCount::new(PassengerKind::Adult, 0),
            bominal_domain::task::PassengerCount::new(PassengerKind::Child, 2),
        ]);
        let groups = parse_passengers(&passengers);
        assert_eq!(groups.len(), 1);
        assert_eq!(groups[0].passenger_type, PassengerType::Child);
    }

    #[test]
    fn parse_passengers_disability_types() {
        let passengers = PassengerList(vec![
            bominal_domain::task::PassengerCount::new(PassengerKind::Adult, 1),
            bominal_domain::task::PassengerCount::new(PassengerKind::Severe, 1),
            bominal_domain::task::PassengerCount::new(PassengerKind::Mild, 1),
        ]);
        let groups = parse_passengers(&passengers);
        assert_eq!(groups.len(), 3);
        assert_eq!(groups[1].passenger_type, PassengerType::SevereDisability);
        assert_eq!(groups[2].passenger_type, PassengerType::MildDisability);
    }

    #[test]
    fn parse_passengers_skips_infant_and_merit() {
        let passengers = PassengerList(vec![
            bominal_domain::task::PassengerCount::new(PassengerKind::Adult, 1),
            bominal_domain::task::PassengerCount::new(PassengerKind::Infant, 1),
            bominal_domain::task::PassengerCount::new(PassengerKind::Merit, 1),
        ]);
        let groups = parse_passengers(&passengers);
        assert_eq!(groups.len(), 1);
        assert_eq!(groups[0].passenger_type, PassengerType::Adult);
    }
}
