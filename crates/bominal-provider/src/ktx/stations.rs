//! KTX/Korail station codes (33 stations).
//! Note: Station names differ from SRT (e.g., "김천구미" vs "김천(구미)").

pub fn station_code(name: &str) -> Option<&'static str> {
    match name {
        "서울" => Some("0001"),
        "용산" => Some("0003"),
        "영등포" => Some("0004"),
        "광명" => Some("0501"),
        "수원" => Some("0005"),
        "천안아산" => Some("0502"),
        "오송" => Some("0297"),
        "대전" => Some("0010"),
        "서대전" => Some("0012"),
        "김천구미" => Some("0507"),
        "동대구" => Some("0015"),
        "포항" => Some("0516"),
        "경주" => Some("0508"),
        "울산" => Some("0509"),
        "부산" => Some("0020"),
        "마산" => Some("0059"),
        "창원중앙" => Some("0512"),
        "경산" => Some("0513"),
        "밀양" => Some("0075"),
        "진영" => Some("0056"),
        "창원" => Some("0515"),
        "진주" => Some("0063"),
        "익산" => Some("0030"),
        "전주" => Some("0045"),
        "정읍" => Some("0510"),
        "광주송정" => Some("0036"),
        "목포" => Some("0041"),
        "나주" => Some("0511"),
        "순천" => Some("0051"),
        "여수EXPO" => Some("0053"),
        "남원" => Some("0049"),
        "공주" => Some("0514"),
        "강릉" => Some("0115"),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn known_stations() {
        assert_eq!(station_code("서울"), Some("0001"));
        assert_eq!(station_code("부산"), Some("0020"));
        assert_eq!(station_code("김천구미"), Some("0507"));
    }

    #[test]
    fn ktx_only_stations() {
        assert_eq!(station_code("강릉"), Some("0115"));
        assert_eq!(station_code("서대전"), Some("0012"));
    }

    #[test]
    fn srt_name_not_in_ktx() {
        assert_eq!(station_code("김천(구미)"), None); // SRT format
        assert_eq!(station_code("수서"), None); // SRT-only station
    }
}
