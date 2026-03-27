//! SRT station codes.
//!
//! Codes sourced from the Python `srt` library's `constants.py`.
//! Note: "경주" is an alias for "신경주" (same code 0508).

/// SRT station name (Korean) to code mapping.
pub fn station_code(name: &str) -> Option<&'static str> {
    match name {
        "수서" => Some("0551"),
        "동탄" => Some("0552"),
        "평택지제" => Some("0553"),
        "곡성" => Some("0049"),
        "공주" => Some("0514"),
        "광주송정" => Some("0036"),
        "구례구" => Some("0050"),
        "김천(구미)" => Some("0507"),
        "나주" => Some("0037"),
        "남원" => Some("0048"),
        "대전" => Some("0010"),
        "동대구" => Some("0015"),
        "마산" => Some("0059"),
        "목포" => Some("0041"),
        "밀양" => Some("0017"),
        "부산" => Some("0020"),
        "서대구" => Some("0506"),
        "순천" => Some("0051"),
        "신경주" | "경주" => Some("0508"),
        "여수EXPO" => Some("0053"),
        "여천" => Some("0139"),
        "오송" => Some("0297"),
        "울산(통도사)" => Some("0509"),
        "익산" => Some("0030"),
        "전주" => Some("0045"),
        "정읍" => Some("0033"),
        "진영" => Some("0056"),
        "진주" => Some("0063"),
        "창원" => Some("0057"),
        "창원중앙" => Some("0512"),
        "천안아산" => Some("0502"),
        "포항" => Some("0515"),
        _ => None,
    }
}

/// SRT station code to name mapping.
pub fn station_name(code: &str) -> Option<&'static str> {
    match code {
        "0551" => Some("수서"),
        "0552" => Some("동탄"),
        "0553" => Some("평택지제"),
        "0049" => Some("곡성"),
        "0514" => Some("공주"),
        "0036" => Some("광주송정"),
        "0050" => Some("구례구"),
        "0507" => Some("김천(구미)"),
        "0037" => Some("나주"),
        "0048" => Some("남원"),
        "0010" => Some("대전"),
        "0015" => Some("동대구"),
        "0059" => Some("마산"),
        "0041" => Some("목포"),
        "0017" => Some("밀양"),
        "0020" => Some("부산"),
        "0506" => Some("서대구"),
        "0051" => Some("순천"),
        "0508" => Some("신경주"),
        "0053" => Some("여수EXPO"),
        "0139" => Some("여천"),
        "0297" => Some("오송"),
        "0509" => Some("울산(통도사)"),
        "0030" => Some("익산"),
        "0045" => Some("전주"),
        "0033" => Some("정읍"),
        "0056" => Some("진영"),
        "0063" => Some("진주"),
        "0057" => Some("창원"),
        "0512" => Some("창원중앙"),
        "0502" => Some("천안아산"),
        "0515" => Some("포항"),
        _ => None,
    }
}

/// Train code to name mapping.
pub fn train_name(code: &str) -> Option<&'static str> {
    match code {
        "00" => Some("KTX"),
        "02" => Some("무궁화"),
        "03" => Some("통근열차"),
        "04" => Some("누리로"),
        "05" => Some("전체"),
        "07" => Some("KTX-산천"),
        "08" => Some("ITX-새마을"),
        "09" => Some("ITX-청춘"),
        "10" => Some("KTX-산천"),
        "17" => Some("SRT"),
        "18" => Some("ITX-마음"),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn known_stations() {
        assert_eq!(station_code("수서"), Some("0551"));
        assert_eq!(station_code("부산"), Some("0020"));
        assert_eq!(station_code("김천(구미)"), Some("0507"));
        assert_eq!(station_code("밀양"), Some("0017"));
        assert_eq!(station_code("서대구"), Some("0506"));
    }

    #[test]
    fn gyeongju_alias() {
        assert_eq!(station_code("신경주"), Some("0508"));
        assert_eq!(station_code("경주"), Some("0508"));
    }

    #[test]
    fn unknown_station() {
        assert_eq!(station_code("서울"), None);
    }

    #[test]
    fn reverse_lookup() {
        assert_eq!(station_name("0551"), Some("수서"));
        assert_eq!(station_name("0020"), Some("부산"));
        assert_eq!(station_name("9999"), None);
    }

    #[test]
    fn train_names() {
        assert_eq!(train_name("17"), Some("SRT"));
        assert_eq!(train_name("00"), Some("KTX"));
        assert_eq!(train_name("99"), None);
    }
}
