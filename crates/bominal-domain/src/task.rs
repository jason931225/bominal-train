//! Reservation task domain model and typed task payloads.

use std::fmt;
use std::str::FromStr;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Provider {
    #[serde(rename = "SRT")]
    Srt,
    #[serde(rename = "KTX")]
    Ktx,
}

impl Provider {
    pub const fn as_str(self) -> &'static str {
        match self {
            Provider::Srt => "SRT",
            Provider::Ktx => "KTX",
        }
    }
}

impl fmt::Display for Provider {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl FromStr for Provider {
    type Err = &'static str;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "SRT" => Ok(Self::Srt),
            "KTX" => Ok(Self::Ktx),
            _ => Err("provider must be SRT or KTX"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SeatPreference {
    #[serde(rename = "GeneralFirst")]
    GeneralFirst,
    #[serde(rename = "SpecialFirst")]
    SpecialFirst,
    #[serde(rename = "GeneralOnly")]
    GeneralOnly,
    #[serde(rename = "SpecialOnly")]
    SpecialOnly,
}

impl SeatPreference {
    pub const fn as_str(self) -> &'static str {
        match self {
            SeatPreference::GeneralFirst => "GeneralFirst",
            SeatPreference::SpecialFirst => "SpecialFirst",
            SeatPreference::GeneralOnly => "GeneralOnly",
            SeatPreference::SpecialOnly => "SpecialOnly",
        }
    }
}

impl fmt::Display for SeatPreference {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl FromStr for SeatPreference {
    type Err = &'static str;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "GeneralFirst" => Ok(Self::GeneralFirst),
            "SpecialFirst" => Ok(Self::SpecialFirst),
            "GeneralOnly" => Ok(Self::GeneralOnly),
            "SpecialOnly" => Ok(Self::SpecialOnly),
            _ => Err("invalid seat preference"),
        }
    }
}

/// Task status with valid state transitions enforced.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TaskStatus {
    Queued,
    Running,
    Idle,
    AwaitingPayment,
    Confirmed,
    Failed,
    Cancelled,
}

impl TaskStatus {
    pub fn can_transition_to(self, next: TaskStatus) -> bool {
        matches!(
            (self, next),
            (TaskStatus::Queued, TaskStatus::Running)
                | (TaskStatus::Queued, TaskStatus::Cancelled)
                | (TaskStatus::Running, TaskStatus::Idle)
                | (TaskStatus::Running, TaskStatus::AwaitingPayment)
                | (TaskStatus::Running, TaskStatus::Confirmed)
                | (TaskStatus::Running, TaskStatus::Failed)
                | (TaskStatus::Running, TaskStatus::Cancelled)
                | (TaskStatus::Idle, TaskStatus::Running)
                | (TaskStatus::Idle, TaskStatus::Cancelled)
                | (TaskStatus::AwaitingPayment, TaskStatus::Confirmed)
                | (TaskStatus::AwaitingPayment, TaskStatus::Failed)
                | (TaskStatus::AwaitingPayment, TaskStatus::Cancelled)
        )
    }

    pub const fn as_str(self) -> &'static str {
        match self {
            TaskStatus::Queued => "queued",
            TaskStatus::Running => "running",
            TaskStatus::Idle => "idle",
            TaskStatus::AwaitingPayment => "awaiting_payment",
            TaskStatus::Confirmed => "confirmed",
            TaskStatus::Failed => "failed",
            TaskStatus::Cancelled => "cancelled",
        }
    }

    pub const fn i18n_key(self) -> &'static str {
        match self {
            TaskStatus::Queued => "task.queued",
            TaskStatus::Running => "task.running",
            TaskStatus::Idle => "task.idle",
            TaskStatus::AwaitingPayment => "task.awaiting_payment",
            TaskStatus::Confirmed => "task.confirmed",
            TaskStatus::Failed => "task.failed",
            TaskStatus::Cancelled => "task.cancelled",
        }
    }
}

impl fmt::Display for TaskStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl FromStr for TaskStatus {
    type Err = &'static str;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "queued" => Ok(Self::Queued),
            "running" => Ok(Self::Running),
            "idle" => Ok(Self::Idle),
            "awaiting_payment" => Ok(Self::AwaitingPayment),
            "confirmed" => Ok(Self::Confirmed),
            "failed" => Ok(Self::Failed),
            "cancelled" => Ok(Self::Cancelled),
            _ => Err("invalid task status"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PassengerKind {
    Adult,
    Child,
    Senior,
    Severe,
    Mild,
    Infant,
    Merit,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PassengerCount {
    #[serde(rename = "type")]
    pub kind: PassengerKind,
    pub count: u8,
}

impl PassengerCount {
    pub const fn new(kind: PassengerKind, count: u8) -> Self {
        Self { kind, count }
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct PassengerList(pub Vec<PassengerCount>);

impl PassengerList {
    pub fn total_count(&self) -> u8 {
        self.0.iter().map(|p| p.count).sum()
    }

    pub fn is_empty(&self) -> bool {
        self.0.iter().all(|p| p.count == 0)
    }
}

impl fmt::Display for PassengerList {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let json = serde_json::to_string(&self.0).map_err(|_| fmt::Error)?;
        f.write_str(&json)
    }
}

impl FromStr for PassengerList {
    type Err = serde_json::Error;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        serde_json::from_str(value).map(Self)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TargetTrain {
    pub train_number: String,
    pub dep_time: String,
}

impl TargetTrain {
    pub fn is_blank(&self) -> bool {
        self.train_number.trim().is_empty()
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct TargetTrainList(pub Vec<TargetTrain>);

impl TargetTrainList {
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

impl fmt::Display for TargetTrainList {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let json = serde_json::to_string(&self.0).map_err(|_| fmt::Error)?;
        f.write_str(&json)
    }
}

impl FromStr for TargetTrainList {
    type Err = serde_json::Error;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        serde_json::from_str(value).map(Self)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReservationSnapshot {
    pub dep_station: String,
    pub arr_station: String,
    pub dep_date: String,
    pub dep_time: String,
    pub train_number: String,
    pub total_cost: String,
    pub is_waiting: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReservationTask {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn provider_round_trip() {
        assert_eq!(Provider::from_str("SRT"), Ok(Provider::Srt));
        assert_eq!(Provider::Ktx.to_string(), "KTX");
    }

    #[test]
    fn seat_preference_round_trip() {
        assert_eq!(
            SeatPreference::from_str("GeneralFirst"),
            Ok(SeatPreference::GeneralFirst)
        );
        assert_eq!(SeatPreference::SpecialOnly.to_string(), "SpecialOnly");
    }

    #[test]
    fn valid_transitions() {
        assert!(TaskStatus::Queued.can_transition_to(TaskStatus::Running));
        assert!(TaskStatus::Running.can_transition_to(TaskStatus::Confirmed));
        assert!(TaskStatus::Running.can_transition_to(TaskStatus::AwaitingPayment));
        assert!(TaskStatus::Idle.can_transition_to(TaskStatus::Running));
        assert!(TaskStatus::AwaitingPayment.can_transition_to(TaskStatus::Confirmed));
    }

    #[test]
    fn invalid_transitions() {
        assert!(!TaskStatus::Confirmed.can_transition_to(TaskStatus::Running));
        assert!(!TaskStatus::Failed.can_transition_to(TaskStatus::Running));
        assert!(!TaskStatus::Cancelled.can_transition_to(TaskStatus::Running));
        assert!(!TaskStatus::Queued.can_transition_to(TaskStatus::Confirmed));
    }

    #[test]
    fn passenger_list_json_round_trip() {
        let value = PassengerList(vec![PassengerCount::new(PassengerKind::Adult, 1)]);
        assert_eq!(PassengerList::from_str(&value.to_string()).unwrap(), value);
    }

    #[test]
    fn target_train_list_json_round_trip() {
        let value = TargetTrainList(vec![TargetTrain {
            train_number: "305".to_string(),
            dep_time: "090000".to_string(),
        }]);
        assert_eq!(
            TargetTrainList::from_str(&value.to_string()).unwrap(),
            value
        );
    }
}
