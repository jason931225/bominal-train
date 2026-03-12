//! Reservation domain types.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrainSchedule {
    pub train_no: String,
    pub train_type: String,
    pub departure_station: String,
    pub arrival_station: String,
    pub departure_time: String,
    pub arrival_time: String,
    pub date: String,
    pub general_seat_available: bool,
    pub special_seat_available: bool,
    pub standby_available: bool,
}
