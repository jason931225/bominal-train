//! Reservation task repository — normalized task storage without JSONB blobs.

use std::collections::HashMap;

use bominal_domain::task::{
    PassengerCount, PassengerKind, PassengerList, Provider, ReservationSnapshot, SeatPreference,
    TargetTrain, TargetTrainList, TaskStatus,
};
use chrono::{DateTime, Utc};
use sqlx::PgPool;
use uuid::Uuid;

#[derive(Debug, Clone, sqlx::FromRow)]
struct TaskCoreRow {
    id: Uuid,
    user_id: Uuid,
    provider: String,
    departure_station: String,
    arrival_station: String,
    travel_date: String,
    departure_time: String,
    seat_preference: String,
    auto_pay: bool,
    payment_card_id: Option<Uuid>,
    notify_enabled: bool,
    auto_retry: bool,
    status: String,
    reservation_number: Option<String>,
    reserved_dep_station: Option<String>,
    reserved_arr_station: Option<String>,
    reserved_dep_date: Option<String>,
    reserved_dep_time: Option<String>,
    reserved_train_number: Option<String>,
    reserved_total_cost: Option<String>,
    reservation_is_waiting: Option<bool>,
    started_at: Option<DateTime<Utc>>,
    last_attempt_at: Option<DateTime<Utc>>,
    attempt_count: i32,
    created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, sqlx::FromRow)]
struct TaskPassengerRow {
    task_id: Uuid,
    passenger_kind: String,
    passenger_count: i16,
}

#[derive(Debug, Clone, sqlx::FromRow)]
struct TaskTargetRow {
    task_id: Uuid,
    ordinal: i16,
    train_number: String,
    dep_time: String,
}

/// Aggregated reservation task row used by callers.
#[derive(Debug, Clone)]
pub struct TaskRow {
    pub id: Uuid,
    pub user_id: Uuid,
    pub provider: Provider,
    pub departure_station: String,
    pub arrival_station: String,
    pub travel_date: String,
    pub departure_time: String,
    pub passengers: PassengerList,
    pub seat_preference: SeatPreference,
    pub target_trains: TargetTrainList,
    pub auto_pay: bool,
    pub payment_card_id: Option<Uuid>,
    pub notify_enabled: bool,
    pub auto_retry: bool,
    pub status: TaskStatus,
    pub reservation_number: Option<String>,
    pub reservation: Option<ReservationSnapshot>,
    pub started_at: Option<DateTime<Utc>>,
    pub last_attempt_at: Option<DateTime<Utc>>,
    pub attempt_count: i32,
    pub created_at: DateTime<Utc>,
}

#[allow(clippy::too_many_arguments)]
pub async fn create_task(
    pool: &PgPool,
    user_id: Uuid,
    provider: Provider,
    departure_station: &str,
    arrival_station: &str,
    travel_date: &str,
    departure_time: &str,
    passengers: &PassengerList,
    seat_preference: SeatPreference,
    target_trains: &TargetTrainList,
    auto_pay: bool,
    payment_card_id: Option<Uuid>,
    notify_enabled: bool,
    auto_retry: bool,
) -> Result<TaskRow, sqlx::Error> {
    let mut tx = pool.begin().await?;

    let core = sqlx::query_as::<_, TaskCoreRow>(
        r#"
        INSERT INTO reservation_tasks (
            user_id, provider, departure_station, arrival_station,
            travel_date, departure_time, seat_preference,
            auto_pay, payment_card_id, notify_enabled, auto_retry
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
        RETURNING *
        "#,
    )
    .bind(user_id)
    .bind(provider.as_str())
    .bind(departure_station)
    .bind(arrival_station)
    .bind(travel_date)
    .bind(departure_time)
    .bind(seat_preference.as_str())
    .bind(auto_pay)
    .bind(payment_card_id)
    .bind(notify_enabled)
    .bind(auto_retry)
    .fetch_one(&mut *tx)
    .await?;

    insert_passengers(&mut tx, core.id, passengers).await?;
    insert_targets(&mut tx, core.id, target_trains).await?;

    tx.commit().await?;
    build_task_row(core, passengers.clone(), target_trains.clone())
}

pub async fn find_by_user(pool: &PgPool, user_id: Uuid) -> Result<Vec<TaskRow>, sqlx::Error> {
    let cores = sqlx::query_as::<_, TaskCoreRow>(
        "SELECT * FROM reservation_tasks WHERE user_id = $1 ORDER BY created_at DESC",
    )
    .bind(user_id)
    .fetch_all(pool)
    .await?;
    hydrate_tasks(pool, cores).await
}

pub async fn find_by_id(
    pool: &PgPool,
    task_id: Uuid,
    user_id: Uuid,
) -> Result<Option<TaskRow>, sqlx::Error> {
    let core =
        sqlx::query_as::<_, TaskCoreRow>("SELECT * FROM reservation_tasks WHERE id = $1 AND user_id = $2")
            .bind(task_id)
            .bind(user_id)
            .fetch_optional(pool)
            .await?;

    match core {
        Some(row) => Ok(hydrate_tasks(pool, vec![row]).await?.into_iter().next()),
        None => Ok(None),
    }
}

pub async fn find_by_status(pool: &PgPool, status: TaskStatus) -> Result<Vec<TaskRow>, sqlx::Error> {
    let cores = sqlx::query_as::<_, TaskCoreRow>(
        "SELECT * FROM reservation_tasks WHERE status = $1 ORDER BY created_at ASC",
    )
    .bind(status.as_str())
    .fetch_all(pool)
    .await?;
    hydrate_tasks(pool, cores).await
}

pub async fn claim_queued_tasks(pool: &PgPool) -> Result<Vec<TaskRow>, sqlx::Error> {
    let cores = sqlx::query_as::<_, TaskCoreRow>(
        r#"
        UPDATE reservation_tasks
        SET status = 'running', started_at = COALESCE(started_at, now())
        WHERE id IN (
            SELECT id FROM reservation_tasks
            WHERE status = 'queued'
            ORDER BY created_at ASC
            FOR UPDATE SKIP LOCKED
        )
        RETURNING *
        "#,
    )
    .fetch_all(pool)
    .await?;
    hydrate_tasks(pool, cores).await
}

pub async fn update_status(pool: &PgPool, task_id: Uuid, status: TaskStatus) -> Result<(), sqlx::Error> {
    let started_clause = if status == TaskStatus::Running {
        ", started_at = COALESCE(started_at, now())"
    } else {
        ""
    };

    let query = format!("UPDATE reservation_tasks SET status = $1{started_clause} WHERE id = $2");

    sqlx::query(&query)
        .bind(status.as_str())
        .bind(task_id)
        .execute(pool)
        .await?;
    Ok(())
}

pub async fn record_attempt(pool: &PgPool, task_id: Uuid) -> Result<(), sqlx::Error> {
    sqlx::query(
        "UPDATE reservation_tasks SET attempt_count = attempt_count + 1, last_attempt_at = now() WHERE id = $1",
    )
    .bind(task_id)
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn mark_confirmed(
    pool: &PgPool,
    task_id: Uuid,
    reservation_number: &str,
    reservation: &ReservationSnapshot,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        UPDATE reservation_tasks
        SET status = 'confirmed',
            reservation_number = $1,
            reserved_dep_station = $2,
            reserved_arr_station = $3,
            reserved_dep_date = $4,
            reserved_dep_time = $5,
            reserved_train_number = $6,
            reserved_total_cost = $7,
            reservation_is_waiting = $8
        WHERE id = $9
        "#,
    )
    .bind(reservation_number)
    .bind(&reservation.dep_station)
    .bind(&reservation.arr_station)
    .bind(&reservation.dep_date)
    .bind(&reservation.dep_time)
    .bind(&reservation.train_number)
    .bind(&reservation.total_cost)
    .bind(reservation.is_waiting)
    .bind(task_id)
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn update_task(
    pool: &PgPool,
    task_id: Uuid,
    user_id: Uuid,
    status: Option<TaskStatus>,
    notify_enabled: Option<bool>,
    auto_retry: Option<bool>,
    target_trains: Option<&TargetTrainList>,
) -> Result<Option<TaskRow>, sqlx::Error> {
    let mut tx = pool.begin().await?;
    let mut sets = Vec::new();
    let mut param_idx = 3u32;

    if status.is_some() {
        sets.push(format!("status = ${param_idx}"));
        param_idx += 1;
    }
    if notify_enabled.is_some() {
        sets.push(format!("notify_enabled = ${param_idx}"));
        param_idx += 1;
    }
    if auto_retry.is_some() {
        sets.push(format!("auto_retry = ${param_idx}"));
    }

    let core = if sets.is_empty() {
        sqlx::query_as::<_, TaskCoreRow>("SELECT * FROM reservation_tasks WHERE id = $1 AND user_id = $2")
            .bind(task_id)
            .bind(user_id)
            .fetch_optional(&mut *tx)
            .await?
    } else {
        let query = format!(
            "UPDATE reservation_tasks SET {} WHERE id = $1 AND user_id = $2 RETURNING *",
            sets.join(", ")
        );

        let mut q = sqlx::query_as::<_, TaskCoreRow>(&query)
            .bind(task_id)
            .bind(user_id);

        if let Some(next) = status {
            q = q.bind(next.as_str());
        }
        if let Some(next) = notify_enabled {
            q = q.bind(next);
        }
        if let Some(next) = auto_retry {
            q = q.bind(next);
        }

        q.fetch_optional(&mut *tx).await?
    };

    let Some(core) = core else {
        tx.rollback().await?;
        return Ok(None);
    };

    if let Some(trains) = target_trains {
        sqlx::query("DELETE FROM reservation_task_targets WHERE task_id = $1")
            .bind(core.id)
            .execute(&mut *tx)
            .await?;
        insert_targets(&mut tx, core.id, trains).await?;
    }

    tx.commit().await?;

    let tasks = hydrate_tasks(pool, vec![core]).await?;
    Ok(tasks.into_iter().next())
}

pub async fn delete_task(pool: &PgPool, task_id: Uuid, user_id: Uuid) -> Result<bool, sqlx::Error> {
    let result = sqlx::query(
        "UPDATE reservation_tasks SET status = 'cancelled' WHERE id = $1 AND user_id = $2 AND status NOT IN ('confirmed', 'cancelled')",
    )
    .bind(task_id)
    .bind(user_id)
    .execute(pool)
    .await?;
    Ok(result.rows_affected() > 0)
}

async fn insert_passengers(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    task_id: Uuid,
    passengers: &PassengerList,
) -> Result<(), sqlx::Error> {
    for passenger in &passengers.0 {
        sqlx::query(
            r#"
            INSERT INTO reservation_task_passengers (task_id, passenger_kind, passenger_count)
            VALUES ($1, $2, $3)
            "#,
        )
        .bind(task_id)
        .bind(passenger_kind_str(passenger.kind))
        .bind(i16::from(passenger.count))
        .execute(&mut **tx)
        .await?;
    }
    Ok(())
}

async fn insert_targets(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    task_id: Uuid,
    targets: &TargetTrainList,
) -> Result<(), sqlx::Error> {
    for (ordinal, target) in targets.0.iter().enumerate() {
        sqlx::query(
            r#"
            INSERT INTO reservation_task_targets (task_id, ordinal, train_number, dep_time)
            VALUES ($1, $2, $3, $4)
            "#,
        )
        .bind(task_id)
        .bind(ordinal as i16)
        .bind(&target.train_number)
        .bind(&target.dep_time)
        .execute(&mut **tx)
        .await?;
    }
    Ok(())
}

async fn hydrate_tasks(pool: &PgPool, cores: Vec<TaskCoreRow>) -> Result<Vec<TaskRow>, sqlx::Error> {
    if cores.is_empty() {
        return Ok(Vec::new());
    }

    let task_ids: Vec<Uuid> = cores.iter().map(|row| row.id).collect();
    let passenger_rows = sqlx::query_as::<_, TaskPassengerRow>(
        r#"
        SELECT task_id, passenger_kind, passenger_count
        FROM reservation_task_passengers
        WHERE task_id = ANY($1)
        ORDER BY task_id, passenger_kind
        "#,
    )
    .bind(&task_ids)
    .fetch_all(pool)
    .await?;

    let target_rows = sqlx::query_as::<_, TaskTargetRow>(
        r#"
        SELECT task_id, ordinal, train_number, dep_time
        FROM reservation_task_targets
        WHERE task_id = ANY($1)
        ORDER BY task_id, ordinal
        "#,
    )
    .bind(&task_ids)
    .fetch_all(pool)
    .await?;

    let mut passengers_by_task: HashMap<Uuid, PassengerList> = HashMap::new();
    for row in passenger_rows {
        passengers_by_task
            .entry(row.task_id)
            .or_default()
            .0
            .push(PassengerCount::new(
                parse_passenger_kind(&row.passenger_kind)?,
                row.passenger_count as u8,
            ));
    }

    let mut targets_by_task: HashMap<Uuid, TargetTrainList> = HashMap::new();
    for row in target_rows {
        let _ = row.ordinal;
        targets_by_task
            .entry(row.task_id)
            .or_default()
            .0
            .push(TargetTrain {
                train_number: row.train_number,
                dep_time: row.dep_time,
            });
    }

    cores.into_iter()
        .map(|core| {
            let passengers = passengers_by_task.remove(&core.id).unwrap_or_default();
            let target_trains = targets_by_task.remove(&core.id).unwrap_or_default();
            build_task_row(core, passengers, target_trains)
        })
        .collect()
}

fn build_task_row(
    core: TaskCoreRow,
    passengers: PassengerList,
    target_trains: TargetTrainList,
) -> Result<TaskRow, sqlx::Error> {
    let reservation = match (
        core.reserved_dep_station,
        core.reserved_arr_station,
        core.reserved_dep_date,
        core.reserved_dep_time,
        core.reserved_train_number,
        core.reserved_total_cost,
        core.reservation_is_waiting,
    ) {
        (
            Some(dep_station),
            Some(arr_station),
            Some(dep_date),
            Some(dep_time),
            Some(train_number),
            Some(total_cost),
            Some(is_waiting),
        ) => Some(ReservationSnapshot {
            dep_station,
            arr_station,
            dep_date,
            dep_time,
            train_number,
            total_cost,
            is_waiting,
        }),
        _ => None,
    };

    Ok(TaskRow {
        id: core.id,
        user_id: core.user_id,
        provider: parse_provider(&core.provider)?,
        departure_station: core.departure_station,
        arrival_station: core.arrival_station,
        travel_date: core.travel_date,
        departure_time: core.departure_time,
        passengers,
        seat_preference: parse_seat_preference(&core.seat_preference)?,
        target_trains,
        auto_pay: core.auto_pay,
        payment_card_id: core.payment_card_id,
        notify_enabled: core.notify_enabled,
        auto_retry: core.auto_retry,
        status: parse_status(&core.status)?,
        reservation_number: core.reservation_number,
        reservation,
        started_at: core.started_at,
        last_attempt_at: core.last_attempt_at,
        attempt_count: core.attempt_count,
        created_at: core.created_at,
    })
}

fn parse_provider(value: &str) -> Result<Provider, sqlx::Error> {
    value
        .parse()
        .map_err(|err: &'static str| sqlx::Error::Protocol(err.into()))
}

fn parse_seat_preference(value: &str) -> Result<SeatPreference, sqlx::Error> {
    value
        .parse()
        .map_err(|err: &'static str| sqlx::Error::Protocol(err.into()))
}

fn parse_status(value: &str) -> Result<TaskStatus, sqlx::Error> {
    value
        .parse()
        .map_err(|err: &'static str| sqlx::Error::Protocol(err.into()))
}

fn passenger_kind_str(kind: PassengerKind) -> &'static str {
    match kind {
        PassengerKind::Adult => "adult",
        PassengerKind::Child => "child",
        PassengerKind::Senior => "senior",
        PassengerKind::Severe => "severe",
        PassengerKind::Mild => "mild",
        PassengerKind::Infant => "infant",
        PassengerKind::Merit => "merit",
    }
}

fn parse_passenger_kind(value: &str) -> Result<PassengerKind, sqlx::Error> {
    match value {
        "adult" => Ok(PassengerKind::Adult),
        "child" => Ok(PassengerKind::Child),
        "senior" => Ok(PassengerKind::Senior),
        "severe" => Ok(PassengerKind::Severe),
        "mild" => Ok(PassengerKind::Mild),
        "infant" => Ok(PassengerKind::Infant),
        "merit" => Ok(PassengerKind::Merit),
        _ => Err(sqlx::Error::Protocol("invalid passenger kind".into())),
    }
}
