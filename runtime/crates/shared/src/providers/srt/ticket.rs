use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TicketInfoRequest {
    pub reservation_id: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SrtTicket {
    pub reservation_id: String,
    pub car: Option<String>,
    pub seat: Option<String>,
    pub seat_class: String,
    pub passenger_type: String,
    pub price: i64,
    pub discount: i64,
    pub waiting: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TicketInfoResponse {
    pub tickets: Vec<SrtTicket>,
}
