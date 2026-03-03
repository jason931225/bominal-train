use serde::{Deserialize, Serialize};

use super::{netfunnel::NetfunnelStatus, types::Passenger};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SearchTrainRequest {
    pub dep_station_code: String,
    pub arr_station_code: String,
    pub dep_date: String,
    pub dep_time: String,
    pub time_limit: Option<String>,
    pub passengers: Vec<Passenger>,
    pub available_only: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SrtTrain {
    pub train_code: String,
    pub train_number: String,
    pub dep_station_code: String,
    pub arr_station_code: String,
    pub dep_date: String,
    pub dep_time: String,
    pub arr_date: String,
    pub arr_time: String,
    pub general_seat_available: bool,
    pub special_seat_available: bool,
    pub standby_available: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SearchTrainResponse {
    pub trains: Vec<SrtTrain>,
    pub netfunnel_status: NetfunnelStatus,
}
