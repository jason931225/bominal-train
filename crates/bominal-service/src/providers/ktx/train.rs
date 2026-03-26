//! KTX/Korail train types parsed from search responses.
//!
//! Ported from `third_party/srtgo/srtgo/ktx.py` (Train class).

use serde_json::Value;

/// SRT train type code (used to filter out SRT trains from KTX search results).
const TRAIN_TYPE_SRT: &str = "17";

/// An individual KTX train from search results.
#[derive(Debug, Clone)]
pub struct KtxTrain {
    pub train_type: String,
    pub train_type_name: String,
    pub train_group: String,
    pub train_no: String,
    pub dep_name: String,
    pub dep_code: String,
    pub dep_date: String,
    pub dep_time: String,
    pub arr_name: String,
    pub arr_code: String,
    pub arr_date: String,
    pub arr_time: String,
    pub run_date: String,
    /// Overall reservation possibility code.
    pub rsv_psb_cd: String,
    /// General seat reservation code (e.g., "11" = available).
    pub gen_rsv_cd: String,
    /// Special seat reservation code.
    pub spe_rsv_cd: String,
    /// Waiting list flag ("Y" or "N").
    pub wait_rsv_flg: String,
}

impl KtxTrain {
    /// Parse a train from KTX search response JSON.
    pub fn from_json(data: &Value) -> Option<Self> {
        let s = |key: &str| data.get(key)?.as_str().map(String::from);

        Some(Self {
            train_type: s("h_trn_clsf_cd")?,
            train_type_name: s("h_trn_clsf_nm").unwrap_or_default(),
            train_group: s("h_trn_gp_cd").unwrap_or_default(),
            train_no: s("h_trn_no")?,
            dep_name: s("h_dpt_rs_stn_nm")?,
            dep_code: s("h_dpt_rs_stn_cd")?,
            dep_date: s("h_dpt_dt")?,
            dep_time: s("h_dpt_tm")?,
            arr_name: s("h_arv_rs_stn_nm")?,
            arr_code: s("h_arv_rs_stn_cd")?,
            arr_date: s("h_arv_dt").unwrap_or_default(),
            arr_time: s("h_arv_tm")?,
            run_date: s("h_run_dt").unwrap_or_default(),
            rsv_psb_cd: s("h_rsv_psb_cd").unwrap_or_default(),
            gen_rsv_cd: s("h_gen_rsv_cd").unwrap_or_default(),
            spe_rsv_cd: s("h_spe_rsv_cd").unwrap_or_default(),
            wait_rsv_flg: s("h_wait_rsv_flg").unwrap_or_default(),
        })
    }

    /// Whether this train has general seats available.
    pub fn general_seat_available(&self) -> bool {
        // "11" means available in KTX system
        self.gen_rsv_cd == "11"
    }

    /// Whether this train has special (first class) seats available.
    pub fn special_seat_available(&self) -> bool {
        self.spe_rsv_cd == "11"
    }

    /// Whether any seat is available (general or special).
    pub fn seat_available(&self) -> bool {
        self.general_seat_available() || self.special_seat_available()
    }

    /// Whether waiting list reservation is available.
    pub fn waiting_available(&self) -> bool {
        self.wait_rsv_flg == "Y"
    }

    /// Whether this is a KTX train (not SRT).
    pub fn is_ktx(&self) -> bool {
        self.train_type != TRAIN_TYPE_SRT
    }

    /// Display name for the train.
    pub fn display_name(&self) -> &str {
        if self.train_type_name.is_empty() {
            &self.train_type
        } else {
            &self.train_type_name
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn sample_train() -> Value {
        json!({
            "h_trn_clsf_cd": "100",
            "h_trn_clsf_nm": "KTX",
            "h_trn_gp_cd": "100",
            "h_trn_no": "101",
            "h_dpt_rs_stn_nm": "서울",
            "h_dpt_rs_stn_cd": "0001",
            "h_dpt_dt": "20260315",
            "h_dpt_tm": "060000",
            "h_arv_rs_stn_nm": "부산",
            "h_arv_rs_stn_cd": "0020",
            "h_arv_dt": "20260315",
            "h_arv_tm": "083500",
            "h_run_dt": "20260315",
            "h_rsv_psb_cd": "Y",
            "h_gen_rsv_cd": "11",
            "h_spe_rsv_cd": "11",
            "h_wait_rsv_flg": "N"
        })
    }

    #[test]
    fn parse_train() {
        let train = KtxTrain::from_json(&sample_train()).unwrap();
        assert_eq!(train.train_no, "101");
        assert_eq!(train.dep_name, "서울");
        assert_eq!(train.arr_name, "부산");
        assert_eq!(train.dep_time, "060000");
        assert!(train.is_ktx());
    }

    #[test]
    fn seat_availability() {
        let train = KtxTrain::from_json(&sample_train()).unwrap();
        assert!(train.general_seat_available());
        assert!(train.special_seat_available());
        assert!(train.seat_available());
    }

    #[test]
    fn sold_out_train() {
        let mut data = sample_train();
        data["h_gen_rsv_cd"] = json!("13");
        data["h_spe_rsv_cd"] = json!("13");
        let train = KtxTrain::from_json(&data).unwrap();
        assert!(!train.general_seat_available());
        assert!(!train.special_seat_available());
        assert!(!train.seat_available());
    }

    #[test]
    fn waiting_available() {
        let mut data = sample_train();
        data["h_wait_rsv_flg"] = json!("Y");
        let train = KtxTrain::from_json(&data).unwrap();
        assert!(train.waiting_available());
    }

    #[test]
    fn srt_train_detection() {
        let mut data = sample_train();
        data["h_trn_clsf_cd"] = json!("17");
        let train = KtxTrain::from_json(&data).unwrap();
        assert!(!train.is_ktx());
    }

    #[test]
    fn display_name() {
        let train = KtxTrain::from_json(&sample_train()).unwrap();
        assert_eq!(train.display_name(), "KTX");
    }
}
