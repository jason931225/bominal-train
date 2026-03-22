//! Shared data-transfer types used across service, server, and frontend crates.
//!
//! These are pure serde structs with no server-side dependencies, so they compile
//! for both native and `wasm32` targets.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::task::{
    PassengerList, Provider, SeatPreference, TargetTrainList, TaskStatus,
};

/// Card info (masked — never exposes raw encrypted fields).
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CardInfo {
    pub id: Uuid,
    pub label: String,
    pub last_four: String,
    pub card_type: String,
    pub card_type_name: String,
    pub created_at: DateTime<Utc>,
}

/// Unified train search result.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TrainInfo {
    pub provider: String,
    pub train_type: String,
    pub train_type_name: String,
    pub train_number: String,
    pub dep_station: String,
    pub dep_date: String,
    pub dep_time: String,
    pub arr_station: String,
    pub arr_time: String,
    pub general_available: bool,
    pub special_available: bool,
    pub standby_available: bool,
}

/// Station display entry.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct StationInfo {
    pub name_ko: String,
    pub name_en: String,
    pub name_ja: String,
}

/// Provider credential info (password masked).
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ProviderInfo {
    pub provider: String,
    pub login_id: String,
    pub status: String,
    pub last_verified_at: Option<String>,
}

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

/// Input for creating a new reservation task.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CreateTaskInput {
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
}

/// Input for updating an existing reservation task.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct UpdateTaskInput {
    pub status: Option<TaskStatus>,
    pub notify_enabled: Option<bool>,
    pub auto_retry: Option<bool>,
    pub target_trains: Option<TargetTrainList>,
}
