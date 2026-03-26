//! SRT train schedule from search results.

use serde_json::Value;

use super::stations::{station_name, train_name};

/// A train schedule returned from SRT search.
#[derive(Debug, Clone)]
pub struct SrtTrain {
    pub train_code: String,
    pub train_number: String,
    pub dep_date: String,
    pub dep_time: String,
    pub dep_station_code: String,
    pub dep_station_name: String,
    pub arr_date: String,
    pub arr_time: String,
    pub arr_station_code: String,
    pub arr_station_name: String,
    pub general_seat_state: String,
    pub special_seat_state: String,
    pub reserve_wait_possible_code: String,
    pub dep_station_run_order: String,
    pub dep_station_constitution_order: String,
    pub arr_station_run_order: String,
    pub arr_station_constitution_order: String,
}

impl SrtTrain {
    /// Parse a train from SRT search response JSON.
    pub fn from_json(data: &Value) -> Option<Self> {
        let s = |key: &str| data.get(key)?.as_str().map(String::from);

        let train_code = s("stlbTrnClsfCd")?;
        let dep_station_code = s("dptRsStnCd")?;
        let arr_station_code = s("arvRsStnCd")?;

        Some(Self {
            dep_station_name: station_name(&dep_station_code)
                .unwrap_or("알 수 없는 역")
                .to_string(),
            arr_station_name: station_name(&arr_station_code)
                .unwrap_or("알 수 없는 역")
                .to_string(),
            train_code,
            train_number: s("trnNo")?,
            dep_date: s("dptDt")?,
            dep_time: s("dptTm")?,
            dep_station_code,
            arr_date: s("arvDt")?,
            arr_time: s("arvTm")?,
            arr_station_code,
            general_seat_state: s("gnrmRsvPsbStr")?,
            special_seat_state: s("sprmRsvPsbStr")?,
            reserve_wait_possible_code: s("rsvWaitPsbCd")?,
            dep_station_run_order: s("dptStnRunOrdr")?,
            dep_station_constitution_order: s("dptStnConsOrdr")?,
            arr_station_run_order: s("arvStnRunOrdr")?,
            arr_station_constitution_order: s("arvStnConsOrdr")?,
        })
    }

    /// Display name for this train's type (e.g. "SRT", "KTX").
    pub fn display_name(&self) -> &str {
        train_name(&self.train_code).unwrap_or("알 수 없는 열차")
    }

    /// Whether this is an SRT train (code "17").
    pub fn is_srt(&self) -> bool {
        self.train_code == "17"
    }

    /// Whether general (standard) seats are available.
    pub fn general_seat_available(&self) -> bool {
        self.general_seat_state.contains("예약가능")
    }

    /// Whether special (first class) seats are available.
    pub fn special_seat_available(&self) -> bool {
        self.special_seat_state.contains("예약가능")
    }

    /// Whether standby reservation is available (code contains "9").
    pub fn reserve_standby_available(&self) -> bool {
        self.reserve_wait_possible_code.contains('9')
    }

    /// Whether any seat (general or special) is available.
    pub fn seat_available(&self) -> bool {
        self.general_seat_available() || self.special_seat_available()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn sample_train_json() -> Value {
        json!({
            "stlbTrnClsfCd": "17",
            "trnNo": "305",
            "dptDt": "20260315",
            "dptTm": "090000",
            "dptRsStnCd": "0551",
            "arvDt": "20260315",
            "arvTm": "113600",
            "arvRsStnCd": "0020",
            "gnrmRsvPsbStr": "예약가능",
            "sprmRsvPsbStr": "매진",
            "rsvWaitPsbCd": "9",
            "dptStnRunOrdr": "000001",
            "dptStnConsOrdr": "000001",
            "arvStnRunOrdr": "000011",
            "arvStnConsOrdr": "000011"
        })
    }

    #[test]
    fn parse_train_from_json() {
        let train = SrtTrain::from_json(&sample_train_json()).unwrap();
        assert_eq!(train.train_number, "305");
        assert_eq!(train.dep_station_name, "수서");
        assert_eq!(train.arr_station_name, "부산");
        assert!(train.is_srt());
    }

    #[test]
    fn seat_availability() {
        let train = SrtTrain::from_json(&sample_train_json()).unwrap();
        assert!(train.general_seat_available());
        assert!(!train.special_seat_available());
        assert!(train.seat_available());
        assert!(train.reserve_standby_available());
    }

    #[test]
    fn sold_out_train() {
        let mut data = sample_train_json();
        data["gnrmRsvPsbStr"] = json!("매진");
        let train = SrtTrain::from_json(&data).unwrap();
        assert!(!train.general_seat_available());
        assert!(!train.seat_available());
    }

    #[test]
    fn no_standby() {
        let mut data = sample_train_json();
        data["rsvWaitPsbCd"] = json!("-1");
        let train = SrtTrain::from_json(&data).unwrap();
        assert!(!train.reserve_standby_available());
    }

    #[test]
    fn non_srt_train() {
        let mut data = sample_train_json();
        data["stlbTrnClsfCd"] = json!("00");
        let train = SrtTrain::from_json(&data).unwrap();
        assert!(!train.is_srt());
        assert_eq!(train.display_name(), "KTX");
    }
}
