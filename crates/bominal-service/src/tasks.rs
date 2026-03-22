//! Task service — create, list, update, delete reservation tasks.

use uuid::Uuid;

use crate::DbPool;
use crate::error::ServiceError;

pub use bominal_domain::dto::{CreateTaskInput, UpdateTaskInput};
pub use bominal_domain::task::{
    PassengerCount, PassengerKind, PassengerList, Provider, ReservationTask as TaskInfo,
    SeatPreference, TargetTrain, TargetTrainList, TaskStatus,
};

pub async fn list(db: &DbPool, user_id: Uuid) -> Result<Vec<TaskInfo>, ServiceError> {
    let rows = bominal_db::task::find_by_user(db, user_id).await?;
    Ok(rows.into_iter().map(row_to_info).collect())
}

pub async fn get(db: &DbPool, task_id: Uuid, user_id: Uuid) -> Result<TaskInfo, ServiceError> {
    let row = bominal_db::task::find_by_id(db, task_id, user_id)
        .await?
        .ok_or_else(|| ServiceError::not_found("Task not found"))?;
    Ok(row_to_info(row))
}

pub async fn create(
    db: &DbPool,
    user_id: Uuid,
    input: &CreateTaskInput,
) -> Result<TaskInfo, ServiceError> {
    validate_create(input)?;

    let cred =
        bominal_db::provider::find_by_user_and_provider(db, user_id, input.provider.as_str())
            .await?;

    match cred {
        Some(c) if c.status == "valid" => {}
        _ => {
            return Err(ServiceError::validation(format!(
                "Valid {} credentials required to create a task",
                input.provider
            )));
        }
    }

    let row = bominal_db::task::create_task(
        db,
        user_id,
        input.provider,
        &input.departure_station,
        &input.arrival_station,
        &input.travel_date,
        &input.departure_time,
        &input.passengers,
        input.seat_preference,
        &input.target_trains,
        input.auto_pay,
        input.payment_card_id,
        input.notify_enabled,
        input.auto_retry,
    )
    .await?;

    Ok(row_to_info(row))
}

pub async fn update(
    db: &DbPool,
    task_id: Uuid,
    user_id: Uuid,
    input: &UpdateTaskInput,
) -> Result<TaskInfo, ServiceError> {
    if let Some(status) = input.status {
        validate_status_update(status)?;
    }

    if let Some(target_trains) = &input.target_trains {
        validate_target_trains(target_trains)?;
    }

    let row = bominal_db::task::update_task(
        db,
        task_id,
        user_id,
        input.status,
        input.notify_enabled,
        input.auto_retry,
        input.target_trains.as_ref(),
    )
    .await?
    .ok_or_else(|| ServiceError::not_found("Task not found"))?;

    Ok(row_to_info(row))
}

pub async fn delete(db: &DbPool, task_id: Uuid, user_id: Uuid) -> Result<(), ServiceError> {
    let cancelled = bominal_db::task::delete_task(db, task_id, user_id).await?;

    if !cancelled {
        return Err(ServiceError::not_found(
            "Task not found or already in terminal state",
        ));
    }

    Ok(())
}

fn row_to_info(row: bominal_db::task::TaskRow) -> TaskInfo {
    TaskInfo {
        id: row.id,
        user_id: row.user_id,
        provider: row.provider,
        departure_station: row.departure_station,
        arrival_station: row.arrival_station,
        travel_date: row.travel_date,
        departure_time: row.departure_time,
        passengers: row.passengers,
        seat_preference: row.seat_preference,
        target_trains: row.target_trains,
        auto_pay: row.auto_pay,
        payment_card_id: row.payment_card_id,
        notify_enabled: row.notify_enabled,
        auto_retry: row.auto_retry,
        status: row.status,
        reservation_number: row.reservation_number,
        reservation: row.reservation,
        started_at: row.started_at,
        last_attempt_at: row.last_attempt_at,
        attempt_count: row.attempt_count,
        created_at: row.created_at,
    }
}

fn validate_create(input: &CreateTaskInput) -> Result<(), ServiceError> {
    if input.departure_station.is_empty() || input.arrival_station.is_empty() {
        return Err(ServiceError::validation(
            "Departure and arrival stations are required",
        ));
    }
    if input.departure_station == input.arrival_station {
        return Err(ServiceError::validation(
            "Departure and arrival stations must be different",
        ));
    }
    if input.travel_date.len() != 8 || !input.travel_date.chars().all(|c| c.is_ascii_digit()) {
        return Err(ServiceError::validation(
            "Travel date must be YYYYMMDD format",
        ));
    }
    if input.departure_time.len() != 6 || !input.departure_time.chars().all(|c| c.is_ascii_digit())
    {
        return Err(ServiceError::validation(
            "Departure time must be HHMMSS format",
        ));
    }

    validate_passengers(&input.passengers)?;
    validate_target_trains(&input.target_trains)?;

    if input.auto_pay && input.payment_card_id.is_none() {
        return Err(ServiceError::validation(
            "Payment card required for auto-pay tasks",
        ));
    }

    Ok(())
}

fn validate_passengers(passengers: &PassengerList) -> Result<(), ServiceError> {
    if passengers.is_empty() {
        return Err(ServiceError::validation(
            "At least one passenger is required",
        ));
    }

    if passengers.total_count() > 9 {
        return Err(ServiceError::validation("Passenger count cannot exceed 9"));
    }

    for passenger in &passengers.0 {
        if passenger.count == 0 {
            return Err(ServiceError::validation(
                "Passenger groups must have a positive count",
            ));
        }
    }

    Ok(())
}

fn validate_target_trains(target_trains: &TargetTrainList) -> Result<(), ServiceError> {
    if target_trains.is_empty() {
        return Err(ServiceError::validation(
            "At least one target train is required",
        ));
    }

    if target_trains.0.iter().any(TargetTrain::is_blank) {
        return Err(ServiceError::validation(
            "Target trains must include a train number",
        ));
    }

    Ok(())
}

pub fn validate_status_update(status: TaskStatus) -> Result<(), ServiceError> {
    match status {
        TaskStatus::Queued | TaskStatus::Idle => Ok(()),
        _ => Err(ServiceError::validation(format!(
            "Cannot manually set status to '{status}'. Use DELETE to cancel."
        ))),
    }
}

pub fn validate_provider(provider: &str) -> Result<Provider, ServiceError> {
    provider
        .parse()
        .map_err(|_| ServiceError::validation(format!("Invalid provider: {provider}")))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_input() -> CreateTaskInput {
        CreateTaskInput {
            provider: Provider::Srt,
            departure_station: "Seoul".to_string(),
            arrival_station: "Busan".to_string(),
            travel_date: "20260314".to_string(),
            departure_time: "090000".to_string(),
            passengers: PassengerList(vec![PassengerCount::new(PassengerKind::Adult, 1)]),
            seat_preference: SeatPreference::GeneralFirst,
            target_trains: TargetTrainList(vec![TargetTrain {
                train_number: "305".to_string(),
                dep_time: "090000".to_string(),
            }]),
            auto_pay: false,
            payment_card_id: None,
            notify_enabled: true,
            auto_retry: true,
        }
    }

    #[test]
    fn validates_status_update() {
        assert!(validate_status_update(TaskStatus::Queued).is_ok());
        assert!(validate_status_update(TaskStatus::Idle).is_ok());
        assert!(validate_status_update(TaskStatus::Running).is_err());
    }

    #[test]
    fn rejects_empty_target_trains() {
        let mut input = sample_input();
        input.target_trains = TargetTrainList::default();
        assert!(validate_create(&input).is_err());
    }

    #[test]
    fn rejects_empty_passengers() {
        let mut input = sample_input();
        input.passengers = PassengerList::default();
        assert!(validate_create(&input).is_err());
    }

    #[test]
    fn rejects_auto_pay_without_card() {
        let mut input = sample_input();
        input.auto_pay = true;
        assert!(validate_create(&input).is_err());
    }
}
