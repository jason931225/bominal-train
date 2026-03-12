//! Station name registry with Korean, English, and Japanese.
//!
//! Station names differ between SRT and KTX for some stations.
//! The Korean name is always the canonical key (used in provider API calls).

use super::Locale;

/// A station name entry with all three locale representations.
#[derive(Debug, Clone, Copy)]
pub struct StationEntry {
    pub korean: &'static str,
    pub english: &'static str,
    pub japanese: &'static str,
}

/// SRT station registry (32 stations).
pub const SRT_STATIONS: &[StationEntry] = &[
    StationEntry { korean: "수서", english: "Suseo", japanese: "スソ" },
    StationEntry { korean: "동탄", english: "Dongtan", japanese: "トンタン" },
    StationEntry { korean: "평택지제", english: "Pyeongtaek-Jije", japanese: "ピョンテクチジェ" },
    StationEntry { korean: "천안아산", english: "Cheonan-Asan", japanese: "チョナンアサン" },
    StationEntry { korean: "오송", english: "Osong", japanese: "オソン" },
    StationEntry { korean: "대전", english: "Daejeon", japanese: "テジョン" },
    StationEntry { korean: "김천(구미)", english: "Gimcheon-Gumi", japanese: "キムチョン（クミ）" },
    StationEntry { korean: "동대구", english: "Dong-Daegu", japanese: "トンテグ" },
    StationEntry { korean: "신경주", english: "Sin-Gyeongju", japanese: "シンギョンジュ" },
    StationEntry { korean: "울산(통도사)", english: "Ulsan (Tongdosa)", japanese: "ウルサン（トンドサ）" },
    StationEntry { korean: "부산", english: "Busan", japanese: "プサン" },
    StationEntry { korean: "공주", english: "Gongju", japanese: "コンジュ" },
    StationEntry { korean: "익산", english: "Iksan", japanese: "イクサン" },
    StationEntry { korean: "정읍", english: "Jeongeup", japanese: "チョンウプ" },
    StationEntry { korean: "광주송정", english: "Gwangju-Songjeong", japanese: "クァンジュソンジョン" },
    StationEntry { korean: "나주", english: "Naju", japanese: "ナジュ" },
    StationEntry { korean: "목포", english: "Mokpo", japanese: "モクポ" },
    StationEntry { korean: "전주", english: "Jeonju", japanese: "チョンジュ" },
    StationEntry { korean: "남원", english: "Namwon", japanese: "ナムウォン" },
    StationEntry { korean: "순천", english: "Suncheon", japanese: "スンチョン" },
    StationEntry { korean: "여수EXPO", english: "Yeosu-EXPO", japanese: "ヨスEXPO" },
    StationEntry { korean: "여천", english: "Yeocheon", japanese: "ヨチョン" },
    StationEntry { korean: "구례구", english: "Gurye-gu", japanese: "クレグ" },
    StationEntry { korean: "밀양", english: "Miryang", japanese: "ミリャン" },
    StationEntry { korean: "진영", english: "Jinyeong", japanese: "チニョン" },
    StationEntry { korean: "창원중앙", english: "Changwon-Jungang", japanese: "チャンウォンチュンアン" },
    StationEntry { korean: "경산", english: "Gyeongsan", japanese: "キョンサン" },
    StationEntry { korean: "마산", english: "Masan", japanese: "マサン" },
    StationEntry { korean: "창원", english: "Changwon", japanese: "チャンウォン" },
    StationEntry { korean: "진주", english: "Jinju", japanese: "チンジュ" },
    StationEntry { korean: "포항", english: "Pohang", japanese: "ポハン" },
    StationEntry { korean: "영천", english: "Yeongcheon", japanese: "ヨンチョン" },
];

/// KTX/Korail station registry (33 stations).
/// Note: Some station names differ from SRT (e.g., "김천구미" vs "김천(구미)").
pub const KTX_STATIONS: &[StationEntry] = &[
    StationEntry { korean: "서울", english: "Seoul", japanese: "ソウル" },
    StationEntry { korean: "용산", english: "Yongsan", japanese: "ヨンサン" },
    StationEntry { korean: "영등포", english: "Yeongdeungpo", japanese: "ヨンドゥンポ" },
    StationEntry { korean: "광명", english: "Gwangmyeong", japanese: "クァンミョン" },
    StationEntry { korean: "수원", english: "Suwon", japanese: "スウォン" },
    StationEntry { korean: "천안아산", english: "Cheonan-Asan", japanese: "チョナンアサン" },
    StationEntry { korean: "오송", english: "Osong", japanese: "オソン" },
    StationEntry { korean: "대전", english: "Daejeon", japanese: "テジョン" },
    StationEntry { korean: "서대전", english: "Seo-Daejeon", japanese: "ソデジョン" },
    StationEntry { korean: "김천구미", english: "Gimcheon-Gumi", japanese: "キムチョングミ" },
    StationEntry { korean: "동대구", english: "Dong-Daegu", japanese: "トンテグ" },
    StationEntry { korean: "포항", english: "Pohang", japanese: "ポハン" },
    StationEntry { korean: "경주", english: "Gyeongju", japanese: "キョンジュ" },
    StationEntry { korean: "울산", english: "Ulsan", japanese: "ウルサン" },
    StationEntry { korean: "부산", english: "Busan", japanese: "プサン" },
    StationEntry { korean: "마산", english: "Masan", japanese: "マサン" },
    StationEntry { korean: "창원중앙", english: "Changwon-Jungang", japanese: "チャンウォンチュンアン" },
    StationEntry { korean: "경산", english: "Gyeongsan", japanese: "キョンサン" },
    StationEntry { korean: "밀양", english: "Miryang", japanese: "ミリャン" },
    StationEntry { korean: "진영", english: "Jinyeong", japanese: "チニョン" },
    StationEntry { korean: "창원", english: "Changwon", japanese: "チャンウォン" },
    StationEntry { korean: "진주", english: "Jinju", japanese: "チンジュ" },
    StationEntry { korean: "익산", english: "Iksan", japanese: "イクサン" },
    StationEntry { korean: "전주", english: "Jeonju", japanese: "チョンジュ" },
    StationEntry { korean: "정읍", english: "Jeongeup", japanese: "チョンウプ" },
    StationEntry { korean: "광주송정", english: "Gwangju-Songjeong", japanese: "クァンジュソンジョン" },
    StationEntry { korean: "목포", english: "Mokpo", japanese: "モクポ" },
    StationEntry { korean: "나주", english: "Naju", japanese: "ナジュ" },
    StationEntry { korean: "순천", english: "Suncheon", japanese: "スンチョン" },
    StationEntry { korean: "여수EXPO", english: "Yeosu-EXPO", japanese: "ヨスEXPO" },
    StationEntry { korean: "남원", english: "Namwon", japanese: "ナムウォン" },
    StationEntry { korean: "공주", english: "Gongju", japanese: "コンジュ" },
    StationEntry { korean: "강릉", english: "Gangneung", japanese: "カンヌン" },
];

/// Get the display name for a station in the given locale.
///
/// # Arguments
/// * `locale` - Target locale
/// * `korean_name` - The Korean station name (canonical key)
/// * `provider` - "SRT" or "KTX" (determines which registry to search)
///
/// # Examples
///
/// ```
/// use bominal_domain::i18n::{Locale, stations::display_name};
/// assert_eq!(display_name(Locale::Ko, "부산", "SRT"), "부산");
/// assert_eq!(display_name(Locale::En, "부산", "SRT"), "Busan");
/// assert_eq!(display_name(Locale::Ja, "부산", "SRT"), "プサン");
/// ```
pub fn display_name(locale: Locale, korean_name: &str, provider: &str) -> &'static str {
    let stations = match provider {
        "SRT" => SRT_STATIONS,
        _ => KTX_STATIONS,
    };

    for entry in stations {
        if entry.korean == korean_name {
            return match locale {
                Locale::Ko => entry.korean,
                Locale::En => entry.english,
                Locale::Ja => entry.japanese,
            };
        }
    }

    // Station not found in registry; return empty (caller should fallback to korean_name)
    ""
}

/// Get all station entries for a provider.
pub fn stations_for_provider(provider: &str) -> &'static [StationEntry] {
    match provider {
        "SRT" => SRT_STATIONS,
        _ => KTX_STATIONS,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn srt_station_count() {
        assert_eq!(SRT_STATIONS.len(), 32);
    }

    #[test]
    fn ktx_station_count() {
        assert_eq!(KTX_STATIONS.len(), 33);
    }

    #[test]
    fn display_name_all_locales() {
        assert_eq!(display_name(Locale::Ko, "서울", "KTX"), "서울");
        assert_eq!(display_name(Locale::En, "서울", "KTX"), "Seoul");
        assert_eq!(display_name(Locale::Ja, "서울", "KTX"), "ソウル");
    }

    #[test]
    fn station_name_divergence() {
        // SRT uses "김천(구미)", KTX uses "김천구미"
        assert_eq!(display_name(Locale::En, "김천(구미)", "SRT"), "Gimcheon-Gumi");
        assert_eq!(display_name(Locale::En, "김천구미", "KTX"), "Gimcheon-Gumi");
        // Cross-lookup should return empty (different name in different registry)
        assert_eq!(display_name(Locale::En, "김천(구미)", "KTX"), "");
    }

    #[test]
    fn unknown_station_returns_empty() {
        assert_eq!(display_name(Locale::En, "nonexistent", "SRT"), "");
    }
}
