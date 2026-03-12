//! KTX/Korail reservation and ticket types.
//!
//! Ported from `third_party/srtgo/srtgo/ktx.py` (Reservation/Ticket classes).

use serde_json::Value;

/// A KTX ticket (seat within a reservation).
#[derive(Debug, Clone)]
pub struct KtxTicket {
    pub car: String,
    pub seat: String,
    pub seat_type: String,
    pub price: String,
    pub pnr_no: String,
    pub train_no: String,
    /// Sale info fields required for refund.
    pub sale_info1: String,
    pub sale_info2: String,
    pub sale_info3: String,
    pub sale_info4: String,
}

impl KtxTicket {
    /// Parse a ticket from KTX reservation list response.
    pub fn from_json(data: &Value) -> Option<Self> {
        let s = |key: &str| data.get(key)?.as_str().map(String::from);

        Some(Self {
            car: s("h_srcar_no").unwrap_or_default(),
            seat: s("h_seat_no").unwrap_or_default(),
            seat_type: s("h_psrm_cl_nm").unwrap_or_default(),
            price: s("h_rcvd_amt").unwrap_or_default(),
            pnr_no: s("h_pnr_no").unwrap_or_default(),
            train_no: s("h_trn_no").unwrap_or_default(),
            sale_info1: s("h_orgtk_sale_wct_no").unwrap_or_default(),
            sale_info2: s("h_orgtk_sale_dt").unwrap_or_default(),
            sale_info3: s("h_orgtk_sale_sqno").unwrap_or_default(),
            sale_info4: s("h_orgtk_ret_pwd").unwrap_or_default(),
        })
    }
}

/// A KTX reservation.
#[derive(Debug, Clone)]
pub struct KtxReservation {
    pub rsv_id: String,
    pub journey_no: String,
    pub journey_cnt: String,
    pub rsv_chg_no: String,
    pub train_type: String,
    pub train_type_name: String,
    pub train_no: String,
    pub dep_name: String,
    pub dep_code: String,
    pub dep_date: String,
    pub dep_time: String,
    pub arr_name: String,
    pub arr_code: String,
    pub arr_time: String,
    pub price: String,
    pub wct_no: String,
    pub paid: bool,
    pub is_waiting: bool,
    pub tickets: Vec<KtxTicket>,
}

impl KtxReservation {
    /// Parse a reservation from KTX reservation view response.
    pub fn from_json(data: &Value, tickets: Vec<KtxTicket>) -> Option<Self> {
        let s = |key: &str| data.get(key)?.as_str().map(String::from);

        let paid = data
            .get("h_stl_flg")
            .and_then(|v| v.as_str())
            .map(|s| s == "Y")
            .unwrap_or(false);

        Some(Self {
            rsv_id: s("h_pnr_no")?,
            journey_no: s("h_jrny_sqno").unwrap_or_else(|| "001".to_string()),
            journey_cnt: s("h_jrny_cnt").unwrap_or_else(|| "01".to_string()),
            rsv_chg_no: s("h_rsv_chg_no").unwrap_or_else(|| "00000".to_string()),
            train_type: s("h_trn_clsf_cd").unwrap_or_default(),
            train_type_name: s("h_trn_clsf_nm").unwrap_or_default(),
            train_no: s("h_trn_no").unwrap_or_default(),
            dep_name: s("h_dpt_rs_stn_nm").unwrap_or_default(),
            dep_code: s("h_dpt_rs_stn_cd").unwrap_or_default(),
            dep_date: s("h_run_dt").unwrap_or_default(),
            dep_time: s("h_dpt_tm").unwrap_or_default(),
            arr_name: s("h_arv_rs_stn_nm").unwrap_or_default(),
            arr_code: s("h_arv_rs_stn_cd").unwrap_or_default(),
            arr_time: s("h_arv_tm").unwrap_or_default(),
            price: s("h_tot_rcvd_amt").unwrap_or_else(|| "0".to_string()),
            wct_no: s("h_wct_no").unwrap_or_default(),
            paid,
            is_waiting: !tickets.is_empty() && tickets.iter().all(|t| t.seat.is_empty()),
            tickets,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn parse_ticket() {
        let data = json!({
            "h_srcar_no": "5",
            "h_seat_no": "12A",
            "h_psrm_cl_nm": "일반실",
            "h_rcvd_amt": "59800",
            "h_pnr_no": "1234567890",
            "h_trn_no": "101",
            "h_orgtk_sale_wct_no": "W001",
            "h_orgtk_sale_dt": "20260315",
            "h_orgtk_sale_sqno": "001",
            "h_orgtk_ret_pwd": "XXXX"
        });
        let ticket = KtxTicket::from_json(&data).unwrap();
        assert_eq!(ticket.car, "5");
        assert_eq!(ticket.seat, "12A");
        assert_eq!(ticket.price, "59800");
    }

    #[test]
    fn parse_reservation() {
        let data = json!({
            "h_pnr_no": "1234567890",
            "h_jrny_sqno": "001",
            "h_jrny_cnt": "01",
            "h_rsv_chg_no": "00000",
            "h_trn_clsf_cd": "100",
            "h_trn_clsf_nm": "KTX",
            "h_trn_no": "101",
            "h_dpt_rs_stn_nm": "서울",
            "h_dpt_rs_stn_cd": "0001",
            "h_run_dt": "20260315",
            "h_dpt_tm": "060000",
            "h_arv_rs_stn_nm": "부산",
            "h_arv_rs_stn_cd": "0020",
            "h_arv_tm": "083500",
            "h_tot_rcvd_amt": "59800",
            "h_wct_no": "W001",
            "h_stl_flg": "N"
        });
        let tickets = vec![KtxTicket::from_json(&json!({
            "h_srcar_no": "5",
            "h_seat_no": "12A",
            "h_psrm_cl_nm": "일반실",
            "h_rcvd_amt": "59800",
            "h_pnr_no": "1234567890",
            "h_trn_no": "101",
            "h_orgtk_sale_wct_no": "W001",
            "h_orgtk_sale_dt": "20260315",
            "h_orgtk_sale_sqno": "001",
            "h_orgtk_ret_pwd": "XXXX"
        }))
        .unwrap()];

        let rsv = KtxReservation::from_json(&data, tickets).unwrap();
        assert_eq!(rsv.rsv_id, "1234567890");
        assert_eq!(rsv.dep_name, "서울");
        assert_eq!(rsv.arr_name, "부산");
        assert!(!rsv.paid);
        assert!(!rsv.is_waiting);
    }

    #[test]
    fn waiting_reservation() {
        let data = json!({
            "h_pnr_no": "9999999999",
            "h_trn_clsf_cd": "100",
            "h_trn_no": "101",
            "h_dpt_rs_stn_nm": "서울",
            "h_dpt_rs_stn_cd": "0001",
            "h_run_dt": "20260315",
            "h_dpt_tm": "060000",
            "h_arv_rs_stn_nm": "부산",
            "h_arv_rs_stn_cd": "0020",
            "h_arv_tm": "083500",
            "h_stl_flg": "N"
        });
        let tickets = vec![KtxTicket::from_json(&json!({
            "h_srcar_no": "5",
            "h_seat_no": "",
            "h_psrm_cl_nm": "일반실",
            "h_rcvd_amt": "0",
            "h_pnr_no": "9999999999",
            "h_trn_no": "101"
        }))
        .unwrap()];

        let rsv = KtxReservation::from_json(&data, tickets).unwrap();
        assert!(rsv.is_waiting);
    }

    #[test]
    fn empty_tickets_not_waiting() {
        let data = json!({
            "h_pnr_no": "0000000000",
            "h_trn_clsf_cd": "100",
            "h_trn_no": "101",
            "h_dpt_rs_stn_nm": "서울",
            "h_dpt_rs_stn_cd": "0001",
            "h_run_dt": "20260315",
            "h_dpt_tm": "060000",
            "h_arv_rs_stn_nm": "부산",
            "h_arv_rs_stn_cd": "0020",
            "h_arv_tm": "083500",
            "h_stl_flg": "N"
        });
        let rsv = KtxReservation::from_json(&data, vec![]).unwrap();
        assert!(!rsv.is_waiting);
    }
}
