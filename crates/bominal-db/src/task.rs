//! Reservation task repository — CRUD for the reservation_tasks table.

use chrono::{DateTime, Utc};
use sqlx::PgPool;
use uuid::Uuid;

/// Row returned from the reservation_tasks table.
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct TaskRow {
    pub id: Uuid,
    pub user_id: Uuid,
    pub provider: String,
    pub departure_station: String,
    pub arrival_station: String,
    pub travel_date: String,
    pub departure_time: String,
    pub passengers: serde_json::Value,
    pub seat_preference: String,
    pub target_trains: serde_json::Value,
    pub auto_pay: bool,
    pub payment_card_id: Option<Uuid>,
    pub notify_enabled: bool,
    pub auto_retry: bool,
    pub status: String,
    pub reservation_number: Option<String>,
    pub reservation_data: Option<serde_json::Value>,
    pub started_at: Option<DateTime<Utc>>,
    pub last_attempt_at: Option<DateTime<Utc>>,
    pub attempt_count: i32,
    pub created_at: DateTime<Utc>,
}

/// Create a new reservation task.
pub async fn create_task(
    pool: &PgPool,
    user_id: Uuid,
    provider: &str,
    departure_station: &str,
    arrival_station: &str,
    travel_date: &str,
    departure_time: &str,
    passengers: &serde_json::Value,
    seat_preference: &str,
    target_trains: &serde_json::Value,
    auto_pay: bool,
    payment_card_id: Option<Uuid>,
    notify_enabled: bool,
) -> Result<TaskRow, sqlx::Error> {
    sqlx::query_as::<_, TaskRow>(
        r#"
        INSERT INTO reservation_tasks (
            user_id, provider, departure_station, arrival_station,
            travel_date, departure_time, passengers, seat_preference,
            target_trains, auto_pay, payment_card_id, notify_enabled
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)
        RETURNING *
        "#,
    )
    .bind(user_id)
    .bind(provider)
    .bind(departure_station)
    .bind(arrival_station)
    .bind(travel_date)
    .bind(departure_time)
    .bind(passengers)
    .bind(seat_preference)
    .bind(target_trains)
    .bind(auto_pay)
    .bind(payment_card_id)
    .bind(notify_enabled)
    .fetch_one(pool)
    .await
}

/// Find all tasks for a user.
pub async fn find_by_user(pool: &PgPool, user_id: Uuid) -> Result<Vec<TaskRow>, sqlx::Error> {
    sqlx::query_as::<_, TaskRow>(
        "SELECT * FROM reservation_tasks WHERE user_id = $1 ORDER BY created_at DESC",
    )
    .bind(user_id)
    .fetch_all(pool)
    .await
}

/// Find a specific task by ID (with user ownership check).
pub async fn find_by_id(
    pool: &PgPool,
    task_id: Uuid,
    user_id: Uuid,
) -> Result<Option<TaskRow>, sqlx::Error> {
    sqlx::query_as::<_, TaskRow>(
        "SELECT * FROM reservation_tasks WHERE id = $1 AND user_id = $2",
    )
    .bind(task_id)
    .bind(user_id)
    .fetch_optional(pool)
    .await
}

/// Find all tasks with a specific status (for the task runner).
pub async fn find_by_status(pool: &PgPool, status: &str) -> Result<Vec<TaskRow>, sqlx::Error> {
    sqlx::query_as::<_, TaskRow>(
        "SELECT * FROM reservation_tasks WHERE status = $1 ORDER BY created_at ASC",
    )
    .bind(status)
    .fetch_all(pool)
    .await
}

/// Atomically claim queued tasks by setting status to 'running'.
///
/// Uses `UPDATE ... RETURNING` to avoid race conditions where multiple
/// poll cycles could pick up the same task.
pub async fn claim_queued_tasks(pool: &PgPool) -> Result<Vec<TaskRow>, sqlx::Error> {
    sqlx::query_as::<_, TaskRow>(
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
    .await
}

/// Update task status.
pub async fn update_status(
    pool: &PgPool,
    task_id: Uuid,
    status: &str,
) -> Result<(), sqlx::Error> {
    let started_clause = if status == "running" {
        ", started_at = COALESCE(started_at, now())"
    } else {
        ""
    };

    let query = format!(
        "UPDATE reservation_tasks SET status = $1{started_clause} WHERE id = $2"
    );

    sqlx::query(&query)
        .bind(status)
        .bind(task_id)
        .execute(pool)
        .await?;
    Ok(())
}

/// Record a reservation attempt.
pub async fn record_attempt(pool: &PgPool, task_id: Uuid) -> Result<(), sqlx::Error> {
    sqlx::query(
        "UPDATE reservation_tasks SET attempt_count = attempt_count + 1, last_attempt_at = now() WHERE id = $1",
    )
    .bind(task_id)
    .execute(pool)
    .await?;
    Ok(())
}

/// Mark task as confirmed with reservation details.
pub async fn mark_confirmed(
    pool: &PgPool,
    task_id: Uuid,
    reservation_number: &str,
    reservation_data: &serde_json::Value,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        UPDATE reservation_tasks
        SET status = 'confirmed',
            reservation_number = $1,
            reservation_data = $2
        WHERE id = $3
        "#,
    )
    .bind(reservation_number)
    .bind(reservation_data)
    .bind(task_id)
    .execute(pool)
    .await?;
    Ok(())
}

/// Update task fields (for PATCH).
pub async fn update_task(
    pool: &PgPool,
    task_id: Uuid,
    user_id: Uuid,
    status: Option<&str>,
    notify_enabled: Option<bool>,
    target_trains: Option<&serde_json::Value>,
) -> Result<Option<TaskRow>, sqlx::Error> {
    // Build dynamic SET clause
    let mut sets = Vec::new();
    let mut param_idx = 3u32; // $1 = task_id, $2 = user_id

    if status.is_some() {
        sets.push(format!("status = ${param_idx}"));
        param_idx += 1;
    }
    if notify_enabled.is_some() {
        sets.push(format!("notify_enabled = ${param_idx}"));
        param_idx += 1;
    }
    if target_trains.is_some() {
        sets.push(format!("target_trains = ${param_idx}"));
    }

    if sets.is_empty() {
        return find_by_id(pool, task_id, user_id).await;
    }

    let query = format!(
        "UPDATE reservation_tasks SET {} WHERE id = $1 AND user_id = $2 RETURNING *",
        sets.join(", ")
    );

    let mut q = sqlx::query_as::<_, TaskRow>(&query)
        .bind(task_id)
        .bind(user_id);

    if let Some(s) = status {
        q = q.bind(s);
    }
    if let Some(n) = notify_enabled {
        q = q.bind(n);
    }
    if let Some(t) = target_trains {
        q = q.bind(t);
    }

    q.fetch_optional(pool).await
}

/// Delete (cancel) a task.
pub async fn delete_task(
    pool: &PgPool,
    task_id: Uuid,
    user_id: Uuid,
) -> Result<bool, sqlx::Error> {
    let result = sqlx::query(
        "UPDATE reservation_tasks SET status = 'cancelled' WHERE id = $1 AND user_id = $2 AND status NOT IN ('confirmed', 'cancelled')",
    )
    .bind(task_id)
    .bind(user_id)
    .execute(pool)
    .await?;
    Ok(result.rows_affected() > 0)
}
