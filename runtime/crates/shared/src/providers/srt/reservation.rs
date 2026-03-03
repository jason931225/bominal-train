use serde::{Deserialize, Serialize};

use super::{
    search::SrtTrain,
    types::{Passenger, SeatClassPreference},
};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SrtReservation {
    pub reservation_id: String,
    pub train_number: String,
    pub dep_station_code: String,
    pub arr_station_code: String,
    pub dep_date: String,
    pub dep_time: String,
    pub arr_time: String,
    pub seat_count: u8,
    pub total_cost: i64,
    pub paid: bool,
    pub waiting: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReserveRequest {
    pub train: SrtTrain,
    pub passengers: Vec<Passenger>,
    pub seat_preference: SeatClassPreference,
    pub window_seat: Option<bool>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReserveResponse {
    pub reservation: SrtReservation,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReserveStandbyRequest {
    pub train: SrtTrain,
    pub passengers: Vec<Passenger>,
    pub seat_preference: SeatClassPreference,
    pub notification_phone: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReserveStandbyResponse {
    pub reservation: SrtReservation,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReserveStandbyOptionSettingsRequest {
    pub reservation_id: String,
    pub agree_sms: bool,
    pub agree_class_change: bool,
    pub phone_number: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReserveStandbyOptionSettingsResponse {
    pub updated: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GetReservationsRequest {
    pub paid_only: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GetReservationsResponse {
    pub reservations: Vec<SrtReservation>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CancelRequest {
    pub reservation_id: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CancelResponse {
    pub canceled: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReserveInfoRequest {
    pub reservation_id: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReserveInfoResponse {
    pub reservation: Option<SrtReservation>,
    pub refundable: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RefundRequest {
    pub reservation_id: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RefundResponse {
    pub refunded: bool,
}
