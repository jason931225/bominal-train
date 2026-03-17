//! SRT reservation and ticket types.
//!
//! Ported from `third_party/srt/SRT/reservation.py`.

use serde_json::Value;

use super::stations::{station_name, train_name};

/// Seat type classification.
const SEAT_TYPE: &[(&str, &str)] = &[("1", "일반실"), ("2", "특실")];

/// Passenger type classification.
const PASSENGER_TYPE: &[(&str, &str)] = &[
    ("1", "어른/청소년"),
    ("2", "장애 1~3급"),
    ("3", "장애 4~6급"),
    ("4", "경로"),
    ("5", "어린이"),
];

/// A single ticket within a reservation.
#[derive(Debug, Clone)]
pub struct SrtTicket {
    pub car: String,
    pub seat: String,
    pub seat_type_code: String,
    pub seat_type: String,
    pub passenger_type_code: String,
    pub passenger_type: String,
    pub price: i64,
    pub original_price: i64,
    pub discount: i64,
}

impl SrtTicket {
    /// Parse a ticket from SRT ticket_info response JSON.
    pub fn from_json(data: &Value) -> Option<Self> {
        let s = |key: &str| data.get(key)?.as_str().map(String::from);
        let seat_type_code = s("psrmClCd")?;
        let passenger_type_code = s("psgTpCd")?;

        let seat_type = SEAT_TYPE
            .iter()
            .find(|(k, _)| *k == seat_type_code)
            .map(|(_, v)| v.to_string())
            .unwrap_or_else(|| seat_type_code.clone());

        let passenger_type = PASSENGER_TYPE
            .iter()
            .find(|(k, _)| *k == passenger_type_code)
            .map(|(_, v)| v.to_string())
            .unwrap_or_else(|| passenger_type_code.clone());

        Some(Self {
            car: s("scarNo")?,
            seat: s("seatNo")?,
            seat_type_code,
            seat_type,
            passenger_type_code,
            passenger_type,
            price: data.get("rcvdAmt")?.as_str()?.parse().ok()?,
            original_price: data.get("stdrPrc")?.as_str()?.parse().ok()?,
            discount: data.get("dcntPrc")?.as_str()?.parse().ok()?,
        })
    }
}

/// An SRT reservation (combines train info, payment info, and tickets).
#[derive(Debug, Clone)]
pub struct SrtReservation {
    pub reservation_number: String,
    pub total_cost: String,
    pub seat_count: String,
    pub train_code: String,
    pub train_number: String,
    pub dep_date: String,
    pub dep_time: String,
    pub dep_station_code: String,
    pub dep_station_name: String,
    pub arr_time: String,
    pub arr_station_code: String,
    pub arr_station_name: String,
    pub payment_date: String,
    pub payment_time: String,
    pub paid: bool,
    pub tickets: Vec<SrtTicket>,
    /// True if this is a standby (waiting) reservation (seat is empty string).
    pub is_waiting: bool,
}

impl SrtReservation {
    /// Parse a reservation from the zipped train + pay data from get_reservations.
    pub fn from_json(train: &Value, pay: &Value, tickets: Vec<SrtTicket>) -> Option<Self> {
        let ts = |v: &Value, key: &str| v.get(key)?.as_str().map(String::from);

        let dep_station_code = ts(pay, "dptRsStnCd")?;
        let arr_station_code = ts(pay, "arvRsStnCd")?;

        let paid = pay
            .get("stlFlg")
            .and_then(|v| v.as_str())
            .map(|s| s == "Y")
            .unwrap_or(false);

        Some(Self {
            reservation_number: ts(train, "pnrNo")?,
            total_cost: ts(train, "rcvdAmt")?,
            seat_count: ts(train, "tkSpecNum")?,
            train_code: ts(pay, "stlbTrnClsfCd")?,
            train_number: ts(pay, "trnNo")?,
            dep_date: ts(pay, "dptDt")?,
            dep_time: ts(pay, "dptTm")?,
            dep_station_name: station_name(&dep_station_code)
                .unwrap_or("알 수 없는 역")
                .to_string(),
            dep_station_code,
            arr_time: ts(pay, "arvTm")?,
            arr_station_name: station_name(&arr_station_code)
                .unwrap_or("알 수 없는 역")
                .to_string(),
            arr_station_code,
            payment_date: ts(pay, "iseLmtDt")?,
            payment_time: ts(pay, "iseLmtTm")?,
            paid,
            is_waiting: !tickets.is_empty() && tickets.iter().all(|t| t.seat.is_empty()),
            tickets,
        })
    }

    /// Display name for this train's type.
    pub fn display_name(&self) -> &str {
        train_name(&self.train_code).unwrap_or("알 수 없는 열차")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn parse_ticket() {
        let data = json!({
            "scarNo": "18",
            "seatNo": "9C",
            "psrmClCd": "1",
            "psgTpCd": "1",
            "rcvdAmt": "52300",
            "stdrPrc": "52900",
            "dcntPrc": "600"
        });
        let ticket = SrtTicket::from_json(&data).unwrap();
        assert_eq!(ticket.car, "18");
        assert_eq!(ticket.seat, "9C");
        assert_eq!(ticket.seat_type, "일반실");
        assert_eq!(ticket.passenger_type, "어른/청소년");
        assert_eq!(ticket.price, 52300);
        assert_eq!(ticket.discount, 600);
    }

    #[test]
    fn parse_reservation() {
        let train = json!({
            "pnrNo": "1234567890",
            "rcvdAmt": "52300",
            "tkSpecNum": "1"
        });
        let pay = json!({
            "stlbTrnClsfCd": "17",
            "trnNo": "305",
            "dptDt": "20260315",
            "dptTm": "090000",
            "dptRsStnCd": "0551",
            "arvTm": "113600",
            "arvRsStnCd": "0020",
            "iseLmtDt": "20260315",
            "iseLmtTm": "100000",
            "stlFlg": "N"
        });
        let tickets = vec![
            SrtTicket::from_json(&json!({
                "scarNo": "18",
                "seatNo": "9C",
                "psrmClCd": "1",
                "psgTpCd": "1",
                "rcvdAmt": "52300",
                "stdrPrc": "52900",
                "dcntPrc": "600"
            }))
            .unwrap(),
        ];

        let reservation = SrtReservation::from_json(&train, &pay, tickets).unwrap();
        assert_eq!(reservation.reservation_number, "1234567890");
        assert_eq!(reservation.dep_station_name, "수서");
        assert_eq!(reservation.arr_station_name, "부산");
        assert!(!reservation.paid);
        assert!(!reservation.is_waiting);
    }

    #[test]
    fn empty_tickets_not_waiting() {
        let train = json!({
            "pnrNo": "0000000000",
            "rcvdAmt": "0",
            "tkSpecNum": "0"
        });
        let pay = json!({
            "stlbTrnClsfCd": "17",
            "trnNo": "305",
            "dptDt": "20260315",
            "dptTm": "090000",
            "dptRsStnCd": "0551",
            "arvTm": "113600",
            "arvRsStnCd": "0020",
            "iseLmtDt": "20260315",
            "iseLmtTm": "100000",
            "stlFlg": "N"
        });
        let tickets: Vec<SrtTicket> = vec![];
        let reservation = SrtReservation::from_json(&train, &pay, tickets).unwrap();
        assert!(!reservation.is_waiting);
    }

    #[test]
    fn waiting_reservation() {
        let train = json!({
            "pnrNo": "9999999999",
            "rcvdAmt": "52300",
            "tkSpecNum": "1"
        });
        let pay = json!({
            "stlbTrnClsfCd": "17",
            "trnNo": "305",
            "dptDt": "20260315",
            "dptTm": "090000",
            "dptRsStnCd": "0551",
            "arvTm": "113600",
            "arvRsStnCd": "0020",
            "iseLmtDt": "20260315",
            "iseLmtTm": "100000",
            "stlFlg": "N"
        });
        let tickets = vec![
            SrtTicket::from_json(&json!({
                "scarNo": "18",
                "seatNo": "",
                "psrmClCd": "1",
                "psgTpCd": "1",
                "rcvdAmt": "0",
                "stdrPrc": "0",
                "dcntPrc": "0"
            }))
            .unwrap(),
        ];

        let reservation = SrtReservation::from_json(&train, &pay, tickets).unwrap();
        assert!(reservation.is_waiting);
    }
}
