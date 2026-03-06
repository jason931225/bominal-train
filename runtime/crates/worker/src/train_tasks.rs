use anyhow::Result;
use chrono::{DateTime, Datelike, Duration, TimeZone, Utc};
use serde_json::{Value, json};
use sqlx::{PgPool, Row};
use tracing::{info, warn};

use crate::runtime::{
    RuntimeExecutionConfig,
    executor::{ClaimedRuntimeJob, ExecutionErrorKind, ProviderExecutor},
};

const STATE_RUNNING: i16 = 1;
const STATE_PAUSED: i16 = 2;
const STATE_AWAITING_PAYMENT: i16 = 3;
const STATE_COMPLETED: i16 = 4;
const STATE_FAILED: i16 = 5;
const STATE_CANCELLED: i16 = 6;
const STATE_EXPIRED: i16 = 7;

const REASON_NONPAYMENT: i16 = 1;
const REASON_EOL: i16 = 2;

#[derive(Debug)]
struct TrainTaskRow {
    task_id: String,
    user_id: String,
    provider: String,
    state_code: i16,
    dep_station_code: String,
    arr_station_code: String,
    dep_date: String,
    dep_time: String,
    departure_at: DateTime<Utc>,
    passengers_json: Value,
    seat_preference: String,
    auto_pay: bool,
    retry_on_expiry: bool,
    retry_count: i32,
    max_retry_count: i32,
    payment_method_ref: Option<String>,
    pay_by_at: Option<DateTime<Utc>>,
}

#[derive(Debug)]
struct CandidateRow {
    candidate_json: Value,
}

pub async fn process_due_train_task(
    pool: &PgPool,
    config: &RuntimeExecutionConfig,
) -> Result<Option<String>> {
    let now = Utc::now();
    let task = claim_due_task(pool, now).await?;
    let Some(task) = task else {
        return Ok(None);
    };

    if task.state_code == STATE_AWAITING_PAYMENT {
        handle_awaiting_payment(pool, &task, now).await?;
        return Ok(Some(task.task_id));
    }

    if now >= task.departure_at {
        set_terminal_state(
            pool,
            &task.task_id,
            STATE_EXPIRED,
            Some(REASON_EOL),
            Some("eol"),
            json!({"reason": "departure_passed"}),
            now,
        )
        .await?;
        return Ok(Some(task.task_id));
    }

    let candidates = load_candidates(pool, &task.task_id).await?;
    if candidates.is_empty() {
        schedule_next_poll(pool, &task.task_id, now + Duration::seconds(15), now).await?;
        insert_task_event(
            pool,
            &task.task_id,
            "tick_skipped",
            json!({"reason": "no_candidates"}),
        )
        .await?;
        return Ok(Some(task.task_id));
    }

    let search_payload = json!({
        "provider": task.provider,
        "operation": "search_train",
        "subject_ref": task.user_id,
        "refs": {"subject_ref": task.user_id},
        "request": {
            "dep_station_code": task.dep_station_code,
            "arr_station_code": task.arr_station_code,
            "dep_date": task.dep_date,
            "dep_time": task.dep_time,
            "passengers": task.passengers_json,
            "available_only": true
        }
    });

    let search_job = synthetic_job(
        format!("train-task:{}:search", task.task_id),
        search_payload,
    );
    let search_result = ProviderExecutor
        .execute(pool, &search_job, &config.payment_policy)
        .await;

    let tried_now = Utc::now();
    if let Err(error) = search_result {
        let class = error
            .context
            .get("class")
            .and_then(Value::as_str)
            .unwrap_or_default()
            .to_ascii_lowercase();
        if error.kind == ExecutionErrorKind::Fatal
            && (class.contains("auth") || class.contains("missing_session"))
        {
            pause_task_for_auth(pool, &task.task_id, tried_now).await?;
            insert_task_event(
                pool,
                &task.task_id,
                "state_changed",
                json!({"state_code": STATE_PAUSED, "state_name": "paused", "reason": "auth_invalid"}),
            )
            .await?;
            return Ok(Some(task.task_id));
        }

        let next_poll =
            tried_now + Duration::milliseconds(dynamic_poll_millis(task.departure_at, tried_now));
        update_attempt_times(pool, &task.task_id, tried_now, Some(tried_now), None, None).await?;
        schedule_next_poll(pool, &task.task_id, next_poll, tried_now).await?;
        insert_task_event(
            pool,
            &task.task_id,
            "search_failed",
            json!({
                "error_kind": error.kind.as_str(),
                "message": error.safe_message(),
            }),
        )
        .await?;
        return Ok(Some(task.task_id));
    }

    let search_success = search_result.expect("search_result is ok");
    let trains = search_success
        .result_redacted
        .pointer("/response/payload/trains")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();

    let selected = select_candidate_train(&candidates, &trains);
    if selected.is_none() {
        let next_poll =
            tried_now + Duration::milliseconds(dynamic_poll_millis(task.departure_at, tried_now));
        update_attempt_times(pool, &task.task_id, tried_now, Some(tried_now), None, None).await?;
        schedule_next_poll(pool, &task.task_id, next_poll, tried_now).await?;
        insert_task_event(
            pool,
            &task.task_id,
            "search_checked",
            json!({"matched": false}),
        )
        .await?;
        return Ok(Some(task.task_id));
    }

    let selected_train = selected.expect("selected train exists");
    let reserve_payload = json!({
        "provider": task.provider,
        "operation": "reserve",
        "subject_ref": task.user_id,
        "refs": {"subject_ref": task.user_id},
        "request": {
            "train": selected_train,
            "passengers": task.passengers_json,
            "seat_preference": normalize_seat_preference(task.seat_preference.as_str()),
            "window_seat": false
        }
    });
    let reserve_job = synthetic_job(
        format!("train-task:{}:reserve", task.task_id),
        reserve_payload,
    );
    let reserve_result = ProviderExecutor
        .execute(pool, &reserve_job, &config.payment_policy)
        .await;

    let reserve_now = Utc::now();
    if let Err(error) = reserve_result {
        let next_poll = reserve_now
            + Duration::milliseconds(dynamic_poll_millis(task.departure_at, reserve_now));
        update_attempt_times(
            pool,
            &task.task_id,
            reserve_now,
            Some(tried_now),
            Some(reserve_now),
            None,
        )
        .await?;
        schedule_next_poll(pool, &task.task_id, next_poll, reserve_now).await?;
        insert_task_event(
            pool,
            &task.task_id,
            "reserve_failed",
            json!({"error_kind": error.kind.as_str(), "message": error.safe_message()}),
        )
        .await?;
        return Ok(Some(task.task_id));
    }

    let reserve_success = reserve_result.expect("reserve result ok");
    let reservation = reserve_success
        .result_redacted
        .pointer("/response/payload/reservation")
        .cloned()
        .unwrap_or_else(|| json!({}));
    let reservation_id = reservation
        .get("reservation_id")
        .and_then(Value::as_str)
        .map(str::to_string)
        .unwrap_or_default();

    update_after_reservation(
        pool,
        &task.task_id,
        reserve_now,
        tried_now,
        reserve_now,
        selected_train.clone(),
        reservation.clone(),
    )
    .await?;

    if task.auto_pay {
        let Some(payment_method_ref) = task.payment_method_ref.as_deref() else {
            set_awaiting_payment(
                pool,
                &task.task_id,
                reserve_now,
                reservation.clone(),
                Some("payment_method_missing"),
            )
            .await?;
            return Ok(Some(task.task_id));
        };
        let pay_payload = json!({
            "provider": task.provider,
            "operation": "pay_with_card",
            "subject_ref": task.user_id,
            "owner_ref": task.user_id,
            "payment_method_ref": payment_method_ref,
            "refs": {
                "subject_ref": task.user_id,
                "owner_ref": task.user_id,
                "payment_method_ref": payment_method_ref
            },
            "request": {
                "reservation_id": reservation_id,
                "card_identity_type": "personal",
                "installment_months": 0
            }
        });
        let pay_job = synthetic_job(format!("train-task:{}:pay", task.task_id), pay_payload);
        let pay_result = ProviderExecutor
            .execute(pool, &pay_job, &config.payment_policy)
            .await;
        let pay_now = Utc::now();
        if let Ok(success) = pay_result {
            let paid = success
                .result_redacted
                .pointer("/response/payload/paid")
                .and_then(Value::as_bool)
                .unwrap_or(false);
            if paid {
                set_terminal_state(
                    pool,
                    &task.task_id,
                    STATE_COMPLETED,
                    None,
                    None,
                    json!({"reservation": reservation, "payment": success.result_redacted}),
                    pay_now,
                )
                .await?;
                return Ok(Some(task.task_id));
            }
        }

        set_awaiting_payment(
            pool,
            &task.task_id,
            pay_now,
            reservation,
            Some("autopay_failed"),
        )
        .await?;
        update_attempt_times(
            pool,
            &task.task_id,
            pay_now,
            Some(tried_now),
            Some(reserve_now),
            Some(pay_now),
        )
        .await?;
        return Ok(Some(task.task_id));
    }

    set_awaiting_payment(
        pool,
        &task.task_id,
        reserve_now,
        reservation,
        Some("autopay_disabled"),
    )
    .await?;
    Ok(Some(task.task_id))
}

pub async fn process_scheduled_tasks(
    pool: &PgPool,
    config: &RuntimeExecutionConfig,
) -> Result<Option<String>> {
    let now = Utc::now();
    let row = claim_due_scheduler_task(pool, now).await?;
    let Some((id, task_key, task_type)) = row else {
        return Ok(None);
    };

    let execution = match task_type.as_str() {
        "provider_auth_recheck_daily" => run_provider_auth_recheck(pool, config).await,
        _ => Ok(()),
    };

    match execution {
        Ok(()) => {
            let next_midnight = next_midnight_utc(now);
            sqlx::query(
                "update worker_scheduled_tasks
                 set status = 'queued',
                     run_at = $2,
                     last_run_at = $3,
                     last_error = null,
                     updated_at = $3
                 where id = $1",
            )
            .bind(id)
            .bind(next_midnight)
            .bind(now)
            .execute(pool)
            .await?;
            info!(task_key = %task_key, task_type = %task_type, "scheduled worker task completed");
        }
        Err(error) => {
            let retry_at = now + Duration::minutes(5);
            sqlx::query(
                "update worker_scheduled_tasks
                 set status = 'queued',
                     run_at = $2,
                     last_run_at = $3,
                     last_error = $4,
                     updated_at = $3
                 where id = $1",
            )
            .bind(id)
            .bind(retry_at)
            .bind(now)
            .bind(error.to_string())
            .execute(pool)
            .await?;
            warn!(task_key = %task_key, task_type = %task_type, error = %error, "scheduled worker task failed");
        }
    }

    Ok(Some(task_key))
}

async fn claim_due_task(pool: &PgPool, now: DateTime<Utc>) -> Result<Option<TrainTaskRow>> {
    let row = sqlx::query(
        "with candidate as (
            select task_id
            from train_tasks
            where state_code in (0, 1, 3)
              and next_poll_at <= $1
            order by next_poll_at asc
            limit 1
            for update skip locked
        )
        update train_tasks t
           set state_code = case when t.state_code = 0 then 1 else t.state_code end,
               state_name = case when t.state_code = 0 then 'running' else t.state_name end,
               updated_at = $1
          from candidate c
         where t.task_id = c.task_id
         returning
             t.task_id,
             t.user_id,
             t.provider,
             t.state_code,
             t.dep_station_code,
             t.arr_station_code,
             t.dep_date,
             t.dep_time,
             t.departure_at,
             t.passengers_json,
             t.seat_preference,
             t.auto_pay,
             t.retry_on_expiry,
             t.retry_count,
             t.max_retry_count,
             t.payment_method_ref,
             t.pay_by_at",
    )
    .bind(now)
    .fetch_optional(pool)
    .await?;

    let Some(row) = row else {
        return Ok(None);
    };

    Ok(Some(TrainTaskRow {
        task_id: row.try_get("task_id")?,
        user_id: row.try_get("user_id")?,
        provider: row.try_get("provider")?,
        state_code: row.try_get("state_code")?,
        dep_station_code: row.try_get("dep_station_code")?,
        arr_station_code: row.try_get("arr_station_code")?,
        dep_date: row.try_get("dep_date")?,
        dep_time: row.try_get("dep_time")?,
        departure_at: row.try_get("departure_at")?,
        passengers_json: row.try_get("passengers_json")?,
        seat_preference: row.try_get("seat_preference")?,
        auto_pay: row.try_get("auto_pay")?,
        retry_on_expiry: row.try_get("retry_on_expiry")?,
        retry_count: row.try_get("retry_count")?,
        max_retry_count: row.try_get("max_retry_count")?,
        payment_method_ref: row.try_get("payment_method_ref")?,
        pay_by_at: row.try_get("pay_by_at")?,
    }))
}

async fn load_candidates(pool: &PgPool, task_id: &str) -> Result<Vec<CandidateRow>> {
    let rows = sqlx::query(
        "select candidate_json
         from train_task_candidates
         where task_id = $1
         order by priority_index asc",
    )
    .bind(task_id)
    .fetch_all(pool)
    .await?;
    let mut out = Vec::with_capacity(rows.len());
    for row in rows {
        out.push(CandidateRow {
            candidate_json: row.try_get("candidate_json")?,
        });
    }
    Ok(out)
}

fn select_candidate_train(candidates: &[CandidateRow], trains: &[Value]) -> Option<Value> {
    for candidate in candidates {
        let c_train_number = candidate
            .candidate_json
            .get("train_number")
            .and_then(Value::as_str)
            .unwrap_or_default();
        let c_dep_date = candidate
            .candidate_json
            .get("dep_date")
            .and_then(Value::as_str)
            .unwrap_or_default();
        let c_dep_time = candidate
            .candidate_json
            .get("dep_time")
            .and_then(Value::as_str)
            .unwrap_or_default();
        for train in trains {
            let train_number = train
                .get("train_number")
                .and_then(Value::as_str)
                .unwrap_or_default();
            let dep_date = train
                .get("dep_date")
                .and_then(Value::as_str)
                .unwrap_or_default();
            let dep_time = train
                .get("dep_time")
                .and_then(Value::as_str)
                .unwrap_or_default();
            if train_number == c_train_number && dep_date == c_dep_date && dep_time == c_dep_time {
                let general = train
                    .get("general_seat_available")
                    .and_then(Value::as_bool)
                    .unwrap_or(false);
                let special = train
                    .get("special_seat_available")
                    .and_then(Value::as_bool)
                    .unwrap_or(false);
                let standby = train
                    .get("standby_available")
                    .and_then(Value::as_bool)
                    .unwrap_or(false);
                if general || special || standby {
                    return Some(train.clone());
                }
            }
        }
    }
    None
}

fn synthetic_job(job_id: String, payload: Value) -> ClaimedRuntimeJob {
    ClaimedRuntimeJob {
        job_id,
        kind: "runtime.synthetic".to_string(),
        user_id: payload
            .get("subject_ref")
            .and_then(Value::as_str)
            .map(str::to_string),
        payload,
        persisted_payload: json!({}),
        attempt_count: 0,
        max_attempts: 1,
        idempotency_scope: None,
        idempotency_key: None,
    }
}

fn normalize_seat_preference(raw: &str) -> &'static str {
    match raw.trim().to_ascii_lowercase().as_str() {
        "general_only" => "general_only",
        "special_first" => "special_first",
        "special_only" => "special_only",
        _ => "general_first",
    }
}

async fn handle_awaiting_payment(
    pool: &PgPool,
    task: &TrainTaskRow,
    now: DateTime<Utc>,
) -> Result<()> {
    let Some(pay_by_at) = task.pay_by_at else {
        schedule_next_poll(pool, &task.task_id, now + Duration::seconds(30), now).await?;
        return Ok(());
    };
    if now < pay_by_at {
        schedule_next_poll(pool, &task.task_id, now + Duration::seconds(30), now).await?;
        return Ok(());
    }

    if task.retry_on_expiry && task.retry_count < task.max_retry_count && now < task.departure_at {
        sqlx::query(
            "update train_tasks
             set state_code = $2,
                 state_name = 'running',
                 state_reason_code = null,
                 state_reason_name = null,
                 retry_count = retry_count + 1,
                 pay_by_at = null,
                 next_poll_at = $3,
                 updated_at = $3
             where task_id = $1",
        )
        .bind(&task.task_id)
        .bind(STATE_RUNNING)
        .bind(now)
        .execute(pool)
        .await?;
        insert_task_event(
            pool,
            &task.task_id,
            "state_changed",
            json!({"state_code": STATE_RUNNING, "state_name": "running", "reason": "retry_on_expiry"}),
        )
        .await?;
        return Ok(());
    }

    set_terminal_state(
        pool,
        &task.task_id,
        STATE_EXPIRED,
        Some(REASON_NONPAYMENT),
        Some("nonpayment"),
        json!({"reason": "pay_by_elapsed"}),
        now,
    )
    .await
}

async fn pause_task_for_auth(pool: &PgPool, task_id: &str, now: DateTime<Utc>) -> Result<()> {
    sqlx::query(
        "update train_tasks
         set state_code = $2,
             state_name = 'paused',
             state_reason_code = null,
             state_reason_name = 'auth_invalid',
             updated_at = $3,
             next_poll_at = $3
         where task_id = $1",
    )
    .bind(task_id)
    .bind(STATE_PAUSED)
    .bind(now)
    .execute(pool)
    .await?;
    Ok(())
}

async fn schedule_next_poll(
    pool: &PgPool,
    task_id: &str,
    next_poll_at: DateTime<Utc>,
    now: DateTime<Utc>,
) -> Result<()> {
    sqlx::query(
        "update train_tasks
         set next_poll_at = $2,
             updated_at = $3
         where task_id = $1",
    )
    .bind(task_id)
    .bind(next_poll_at)
    .bind(now)
    .execute(pool)
    .await?;
    Ok(())
}

async fn update_attempt_times(
    pool: &PgPool,
    task_id: &str,
    last_tried: DateTime<Utc>,
    last_search: Option<DateTime<Utc>>,
    last_reservation: Option<DateTime<Utc>>,
    last_payment: Option<DateTime<Utc>>,
) -> Result<()> {
    sqlx::query(
        "update train_tasks
         set last_tried_at = $2,
             last_search_tried_at = coalesce($3, last_search_tried_at),
             last_reservation_tried_at = coalesce($4, last_reservation_tried_at),
             last_payment_tried_at = coalesce($5, last_payment_tried_at),
             updated_at = $2
         where task_id = $1",
    )
    .bind(task_id)
    .bind(last_tried)
    .bind(last_search)
    .bind(last_reservation)
    .bind(last_payment)
    .execute(pool)
    .await?;
    Ok(())
}

async fn update_after_reservation(
    pool: &PgPool,
    task_id: &str,
    now: DateTime<Utc>,
    search_tried_at: DateTime<Utc>,
    reservation_tried_at: DateTime<Utc>,
    selected_train: Value,
    reservation: Value,
) -> Result<()> {
    sqlx::query(
        "update train_tasks
         set state_code = $2,
             state_name = 'running',
             selected_train_json = cast($3 as jsonb),
             reservation_json = cast($4 as jsonb),
             last_tried_at = $5,
             last_search_tried_at = $6,
             last_reservation_tried_at = $7,
             updated_at = $5
         where task_id = $1",
    )
    .bind(task_id)
    .bind(STATE_RUNNING)
    .bind(selected_train)
    .bind(reservation)
    .bind(now)
    .bind(search_tried_at)
    .bind(reservation_tried_at)
    .execute(pool)
    .await?;
    insert_task_event(
        pool,
        task_id,
        "reserved",
        json!({"state_code": STATE_RUNNING, "state_name": "running"}),
    )
    .await?;
    Ok(())
}

async fn set_awaiting_payment(
    pool: &PgPool,
    task_id: &str,
    now: DateTime<Utc>,
    reservation: Value,
    reason: Option<&str>,
) -> Result<()> {
    let pay_by = now + Duration::minutes(15);
    sqlx::query(
        "update train_tasks
         set state_code = $2,
             state_name = 'awaiting_payment',
             pay_by_at = $3,
             reservation_json = cast($4 as jsonb),
             next_poll_at = $5,
             updated_at = $5
         where task_id = $1",
    )
    .bind(task_id)
    .bind(STATE_AWAITING_PAYMENT)
    .bind(pay_by)
    .bind(reservation)
    .bind(now)
    .execute(pool)
    .await?;
    insert_task_event(
        pool,
        task_id,
        "state_changed",
        json!({"state_code": STATE_AWAITING_PAYMENT, "state_name": "awaiting_payment", "reason": reason}),
    )
    .await?;
    Ok(())
}

async fn set_terminal_state(
    pool: &PgPool,
    task_id: &str,
    state_code: i16,
    reason_code: Option<i16>,
    reason_name: Option<&str>,
    terminal_payload: Value,
    now: DateTime<Utc>,
) -> Result<()> {
    let state_name = match state_code {
        STATE_COMPLETED => "completed",
        STATE_FAILED => "failed",
        STATE_CANCELLED => "cancelled",
        STATE_EXPIRED => "expired",
        _ => "failed",
    };
    sqlx::query(
        "update train_tasks
         set state_code = $2,
             state_name = $3,
             state_reason_code = $4,
             state_reason_name = $5,
             terminal_payload = cast($6 as jsonb),
             completed_at = $7,
             updated_at = $7,
             next_poll_at = $7
         where task_id = $1",
    )
    .bind(task_id)
    .bind(state_code)
    .bind(state_name)
    .bind(reason_code)
    .bind(reason_name)
    .bind(terminal_payload)
    .bind(now)
    .execute(pool)
    .await?;

    insert_task_event(
        pool,
        task_id,
        "state_changed",
        json!({
            "state_code": state_code,
            "state_name": state_name,
            "state_reason_code": reason_code,
            "state_reason_name": reason_name,
        }),
    )
    .await?;
    Ok(())
}

async fn insert_task_event(
    pool: &PgPool,
    task_id: &str,
    event_type: &str,
    payload: Value,
) -> Result<()> {
    sqlx::query(
        "insert into train_task_events (task_id, event_type, event_payload, created_at)
         values ($1, $2, cast($3 as jsonb), now())",
    )
    .bind(task_id)
    .bind(event_type)
    .bind(payload)
    .execute(pool)
    .await?;
    Ok(())
}

fn dynamic_poll_millis(departure_at: DateTime<Utc>, now: DateTime<Utc>) -> i64 {
    let seconds = (departure_at - now).num_seconds().max(0) as f64;
    let hours = seconds / 3600.0;
    let base = if hours >= 168.0 {
        6.0
    } else if hours >= 72.0 {
        2.0 + (hours - 72.0) * (6.0 - 2.0) / (168.0 - 72.0)
    } else if hours >= 48.0 {
        1.5 + (hours - 48.0) * (2.0 - 1.5) / (72.0 - 48.0)
    } else if hours >= 24.0 {
        1.25 + (hours - 24.0) * (1.5 - 1.25) / (48.0 - 24.0)
    } else {
        1.25
    };

    let jitter_seed = (now.timestamp_subsec_nanos() as f64) / 1_000_000_000.0;
    let jitter = -0.15 + (0.35 * jitter_seed);
    let effective = (base * (1.0 + jitter)).max(1.25);
    (effective * 1000.0) as i64
}

async fn claim_due_scheduler_task(
    pool: &PgPool,
    now: DateTime<Utc>,
) -> Result<Option<(i64, String, String)>> {
    let row = sqlx::query(
        "with candidate as (
            select id
            from worker_scheduled_tasks
            where status = 'queued'
              and run_at <= $1
            order by run_at asc
            limit 1
            for update skip locked
        )
        update worker_scheduled_tasks t
           set status = 'running',
               updated_at = $1
          from candidate c
         where t.id = c.id
         returning t.id, t.task_key, t.task_type",
    )
    .bind(now)
    .fetch_optional(pool)
    .await?;

    let Some(row) = row else {
        return Ok(None);
    };
    Ok(Some((
        row.try_get("id")?,
        row.try_get("task_key")?,
        row.try_get("task_type")?,
    )))
}

async fn run_provider_auth_recheck(pool: &PgPool, config: &RuntimeExecutionConfig) -> Result<()> {
    let rows = sqlx::query(
        "select provider, subject_ref
         from provider_auth_secrets
         where credential_kind = 'login' and revoked_at is null",
    )
    .fetch_all(pool)
    .await?;

    for row in rows {
        let provider: String = row.try_get("provider")?;
        let subject_ref: String = row.try_get("subject_ref")?;
        let login_payload = json!({
            "provider": provider,
            "operation": "login",
            "subject_ref": subject_ref,
            "refs": { "subject_ref": subject_ref },
            "request": {}
        });
        let login_job = synthetic_job(
            format!("provider-auth-recheck:{}:{}", provider, subject_ref),
            login_payload,
        );
        let probe = ProviderExecutor
            .execute(pool, &login_job, &config.payment_policy)
            .await;

        let checked_at = Utc::now();
        let (status, message) = match probe {
            Ok(_) => (
                "success",
                format!(
                    "{}: Successfully authenticated.",
                    provider.to_ascii_uppercase()
                ),
            ),
            Err(error) => (
                "error",
                format!(
                    "{}: {}",
                    provider.to_ascii_uppercase(),
                    error.safe_message()
                ),
            ),
        };
        let metadata = json!({
            "auth_probe_status": status,
            "auth_probe_message": message,
            "auth_probe_checked_at": checked_at,
        });
        sqlx::query(
            "update provider_auth_secrets
             set redacted_metadata = coalesce(redacted_metadata, '{}'::jsonb) || cast($3 as jsonb),
                 updated_at = $4
             where provider = $1 and subject_ref = $2 and credential_kind = 'login' and revoked_at is null",
        )
        .bind(&provider)
        .bind(&subject_ref)
        .bind(metadata)
        .bind(checked_at)
        .execute(pool)
        .await?;
    }

    Ok(())
}

fn next_midnight_utc(now: DateTime<Utc>) -> DateTime<Utc> {
    let date = now.date_naive();
    let next = date.succ_opt().unwrap_or(date);
    Utc.with_ymd_and_hms(next.year(), next.month(), next.day(), 0, 0, 0)
        .single()
        .unwrap_or_else(Utc::now)
}
