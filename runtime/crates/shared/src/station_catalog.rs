use std::{
    collections::{HashMap, HashSet},
    fs, io,
    path::Path,
};

use chrono::{DateTime, Utc};
use encoding_rs::EUC_KR;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

pub const STATION_SOURCE_URL: &str = "https://app.srail.or.kr/js/stationInfo.js";
pub const STATION_SOURCE_REFERER: &str = "https://app.srail.or.kr/";
pub const STATION_SOURCE_USER_AGENT: &str = "bominal-runtime/1.0";
pub const STATION_CATALOG_SCHEMA_VERSION: i32 = 1;

#[derive(Debug)]
pub enum StationCatalogError {
    Io(io::Error),
    Json(serde_json::Error),
    Http(reqwest::Error),
    InvalidSource(String),
    InvalidSnapshot(String),
}

impl std::fmt::Display for StationCatalogError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Io(err) => write!(f, "io error: {err}"),
            Self::Json(err) => write!(f, "json error: {err}"),
            Self::Http(err) => write!(f, "http error: {err}"),
            Self::InvalidSource(message) => write!(f, "invalid source: {message}"),
            Self::InvalidSnapshot(message) => write!(f, "invalid snapshot: {message}"),
        }
    }
}

impl std::error::Error for StationCatalogError {}

impl From<io::Error> for StationCatalogError {
    fn from(value: io::Error) -> Self {
        Self::Io(value)
    }
}

impl From<serde_json::Error> for StationCatalogError {
    fn from(value: serde_json::Error) -> Self {
        Self::Json(value)
    }
}

impl From<reqwest::Error> for StationCatalogError {
    fn from(value: reqwest::Error) -> Self {
        Self::Http(value)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StationCatalogRecord {
    pub provider: String,
    pub station_code: String,
    pub station_name_ko: String,
    pub station_name_en: Option<String>,
    #[serde(default)]
    pub station_name_ja_katakana: Option<String>,
    pub line_code: i32,
    pub selected: bool,
    pub remark: Option<String>,
    pub order_index: i32,
    pub normalized_name: String,
    pub normalized_remark: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StationCatalogSnapshot {
    pub schema_version: i32,
    pub generated_at: DateTime<Utc>,
    pub stations: Vec<StationCatalogRecord>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StationCatalogMetadata {
    pub schema_version: i32,
    pub source_url: String,
    pub source_sha256: String,
    pub generated_at: DateTime<Utc>,
}

#[derive(Debug, Clone)]
pub struct GeneratedCatalog {
    pub snapshot: StationCatalogSnapshot,
    pub metadata: StationCatalogMetadata,
}

pub async fn fetch_station_source(
    http_client: &reqwest::Client,
) -> Result<String, StationCatalogError> {
    let cache_buster = Utc::now().timestamp_millis();
    let url = format!("{STATION_SOURCE_URL}?_={cache_buster}");

    let response = http_client
        .get(url)
        .header("referer", STATION_SOURCE_REFERER)
        .header("user-agent", STATION_SOURCE_USER_AGENT)
        .send()
        .await?;

    if !response.status().is_success() {
        return Err(StationCatalogError::InvalidSource(format!(
            "source returned {}",
            response.status()
        )));
    }

    let bytes = response.bytes().await?;
    let body = match String::from_utf8(bytes.to_vec()) {
        Ok(value) => value,
        Err(_) => {
            let (decoded, _, _) = EUC_KR.decode(&bytes);
            decoded.into_owned()
        }
    };
    Ok(body)
}

pub fn compute_sha256_hex(raw: &[u8]) -> String {
    let mut digest = Sha256::new();
    digest.update(raw);
    format!("{:x}", digest.finalize())
}

pub fn generate_from_source(source: &str) -> Result<GeneratedCatalog, StationCatalogError> {
    let stations = parse_station_source(source)?;
    if stations.is_empty() {
        return Err(StationCatalogError::InvalidSource(
            "station source produced no entries".to_string(),
        ));
    }

    let generated_at = Utc::now();
    let source_sha256 = compute_sha256_hex(source.as_bytes());
    let snapshot = StationCatalogSnapshot {
        schema_version: STATION_CATALOG_SCHEMA_VERSION,
        generated_at,
        stations,
    };
    validate_snapshot(&snapshot)?;

    let metadata = StationCatalogMetadata {
        schema_version: STATION_CATALOG_SCHEMA_VERSION,
        source_url: STATION_SOURCE_URL.to_string(),
        source_sha256,
        generated_at,
    };

    Ok(GeneratedCatalog { snapshot, metadata })
}

pub fn load_snapshot(snapshot_path: &Path) -> Result<StationCatalogSnapshot, StationCatalogError> {
    let raw = fs::read_to_string(snapshot_path)?;
    let snapshot: StationCatalogSnapshot = serde_json::from_str(&raw)?;
    validate_snapshot(&snapshot)?;
    Ok(snapshot)
}

pub fn load_snapshot_with_hash(
    snapshot_path: &Path,
) -> Result<(StationCatalogSnapshot, String), StationCatalogError> {
    let bytes = fs::read(snapshot_path)?;
    let snapshot: StationCatalogSnapshot = serde_json::from_slice(&bytes)?;
    validate_snapshot(&snapshot)?;
    let snapshot_sha256 = compute_sha256_hex(&bytes);
    Ok((snapshot, snapshot_sha256))
}

pub fn load_metadata(metadata_path: &Path) -> Result<StationCatalogMetadata, StationCatalogError> {
    let raw = fs::read_to_string(metadata_path)?;
    let metadata: StationCatalogMetadata = serde_json::from_str(&raw)?;
    if metadata.schema_version != STATION_CATALOG_SCHEMA_VERSION {
        return Err(StationCatalogError::InvalidSnapshot(format!(
            "unsupported metadata schema version {}",
            metadata.schema_version
        )));
    }
    if metadata.source_url.trim().is_empty() {
        return Err(StationCatalogError::InvalidSnapshot(
            "metadata source_url is required".to_string(),
        ));
    }
    if metadata.source_sha256.trim().is_empty() {
        return Err(StationCatalogError::InvalidSnapshot(
            "metadata source_sha256 is required".to_string(),
        ));
    }
    Ok(metadata)
}

pub fn write_generated_catalog(
    snapshot_path: &Path,
    metadata_path: &Path,
    generated: &GeneratedCatalog,
) -> Result<(), StationCatalogError> {
    if let Some(parent) = snapshot_path.parent() {
        fs::create_dir_all(parent)?;
    }
    if let Some(parent) = metadata_path.parent() {
        fs::create_dir_all(parent)?;
    }

    let snapshot_json = serde_json::to_string_pretty(&generated.snapshot)?;
    let metadata_json = serde_json::to_string_pretty(&generated.metadata)?;

    fs::write(snapshot_path, format!("{snapshot_json}\n"))?;
    fs::write(metadata_path, format!("{metadata_json}\n"))?;
    Ok(())
}

pub fn validate_snapshot(snapshot: &StationCatalogSnapshot) -> Result<(), StationCatalogError> {
    if snapshot.schema_version != STATION_CATALOG_SCHEMA_VERSION {
        return Err(StationCatalogError::InvalidSnapshot(format!(
            "unsupported snapshot schema version {}",
            snapshot.schema_version
        )));
    }

    if snapshot.stations.is_empty() {
        return Err(StationCatalogError::InvalidSnapshot(
            "snapshot has no stations".to_string(),
        ));
    }

    let mut dedupe = HashSet::new();
    for station in &snapshot.stations {
        if station.provider != "srt" && station.provider != "ktx" {
            return Err(StationCatalogError::InvalidSnapshot(format!(
                "invalid provider {} for station {}",
                station.provider, station.station_code
            )));
        }
        if station.station_code.trim().is_empty() {
            return Err(StationCatalogError::InvalidSnapshot(
                "station_code is required".to_string(),
            ));
        }
        if station.station_name_ko.trim().is_empty() {
            return Err(StationCatalogError::InvalidSnapshot(format!(
                "station_name_ko is required for {}:{}",
                station.provider, station.station_code
            )));
        }
        if let Some(value) = station.station_name_ja_katakana.as_deref()
            && value.trim().is_empty()
        {
            return Err(StationCatalogError::InvalidSnapshot(format!(
                "station_name_ja_katakana cannot be blank for {}:{}",
                station.provider, station.station_code
            )));
        }

        let dedupe_key = format!("{}:{}", station.provider, station.station_code);
        if !dedupe.insert(dedupe_key) {
            return Err(StationCatalogError::InvalidSnapshot(format!(
                "duplicate provider/station pair {}:{}",
                station.provider, station.station_code
            )));
        }

        let expected_name = normalize_search_text(&station.station_name_ko);
        if station.normalized_name != expected_name {
            return Err(StationCatalogError::InvalidSnapshot(format!(
                "normalized_name mismatch for {}:{}",
                station.provider, station.station_code
            )));
        }

        let expected_remark = station
            .remark
            .as_ref()
            .map(|value| normalize_search_text(value));
        if station.normalized_remark != expected_remark {
            return Err(StationCatalogError::InvalidSnapshot(format!(
                "normalized_remark mismatch for {}:{}",
                station.provider, station.station_code
            )));
        }
    }

    Ok(())
}

pub fn normalize_search_text(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    for ch in input.chars() {
        if ch.is_ascii_alphanumeric() {
            out.push(ch.to_ascii_lowercase());
            continue;
        }

        if ('\u{ac00}'..='\u{d7a3}').contains(&ch) || ('\u{3131}'..='\u{318e}').contains(&ch) {
            out.push(ch);
        }
    }
    out
}

pub fn derive_station_name_ja_katakana(
    station_name_ko: &str,
    station_name_en: Option<&str>,
) -> String {
    let source = station_name_en
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToOwned::to_owned)
        .unwrap_or_else(|| korean_to_romaji(station_name_ko));

    if source.trim().is_empty() {
        return station_name_ko.to_string();
    }

    match to_kana::kata(source.trim()) {
        Ok(value) => value,
        Err(_) => station_name_ko.to_string(),
    }
}

fn korean_to_romaji(input: &str) -> String {
    const L_TABLE: [&str; 19] = [
        "g", "kk", "n", "d", "tt", "r", "m", "b", "pp", "s", "ss", "", "j", "jj", "ch", "k", "t",
        "p", "h",
    ];
    const V_TABLE: [&str; 21] = [
        "a", "ae", "ya", "yae", "eo", "e", "yeo", "ye", "o", "wa", "wae", "oe", "yo", "u", "wo",
        "we", "wi", "yu", "eu", "ui", "i",
    ];
    const T_TABLE: [&str; 28] = [
        "", "k", "k", "k", "n", "n", "n", "t", "l", "k", "m", "l", "l", "l", "p", "l", "m", "p",
        "p", "t", "t", "ng", "t", "t", "k", "t", "p", "t",
    ];

    let mut out = String::with_capacity(input.len() * 2);
    for ch in input.chars() {
        if ('\u{ac00}'..='\u{d7a3}').contains(&ch) {
            let s_index = (ch as u32) - 0xac00;
            let l_index = (s_index / 588) as usize;
            let v_index = ((s_index % 588) / 28) as usize;
            let t_index = (s_index % 28) as usize;
            out.push_str(L_TABLE[l_index]);
            out.push_str(V_TABLE[v_index]);
            out.push_str(T_TABLE[t_index]);
            continue;
        }

        if ch.is_ascii_alphanumeric() {
            out.push(ch.to_ascii_lowercase());
            continue;
        }

        if ch == ' ' {
            out.push(' ');
        }
    }
    out
}

pub fn parse_station_source(
    source: &str,
) -> Result<Vec<StationCatalogRecord>, StationCatalogError> {
    let stripped = strip_js_comments(source);
    let station_list = extract_station_list_segment(&stripped)?;
    let blocks = extract_object_blocks(station_list);
    if blocks.is_empty() {
        return Err(StationCatalogError::InvalidSource(
            "station source has no station blocks".to_string(),
        ));
    }

    let mut rows = Vec::new();
    for block in blocks {
        if let Some(parsed) = parse_station_object(block)? {
            rows.extend(parsed);
        }
    }

    let mut dedupe = HashMap::new();
    for row in rows {
        dedupe.insert(format!("{}:{}", row.provider, row.station_code), row);
    }

    let mut values = dedupe.into_values().collect::<Vec<_>>();
    values.sort_by(|left, right| {
        left.provider
            .cmp(&right.provider)
            .then_with(|| left.order_index.cmp(&right.order_index))
            .then_with(|| left.station_name_ko.cmp(&right.station_name_ko))
    });

    Ok(values)
}

fn strip_js_comments(input: &str) -> String {
    let mut out = Vec::with_capacity(input.len());
    let bytes = input.as_bytes();
    let mut idx = 0;
    let mut in_string = false;
    let mut escaped = false;

    while idx < bytes.len() {
        let byte = bytes[idx];

        if in_string {
            out.push(byte);
            if escaped {
                escaped = false;
            } else if byte == b'\\' {
                escaped = true;
            } else if byte == b'"' {
                in_string = false;
            }
            idx += 1;
            continue;
        }

        if byte == b'"' {
            in_string = true;
            out.push(b'"');
            idx += 1;
            continue;
        }

        if byte == b'/' && idx + 1 < bytes.len() {
            let next = bytes[idx + 1];
            if next == b'/' {
                idx += 2;
                while idx < bytes.len() && bytes[idx] != b'\n' {
                    idx += 1;
                }
                continue;
            }
            if next == b'*' {
                idx += 2;
                while idx + 1 < bytes.len() {
                    if bytes[idx] == b'*' && bytes[idx + 1] == b'/' {
                        idx += 2;
                        break;
                    }
                    idx += 1;
                }
                continue;
            }
        }

        out.push(byte);
        idx += 1;
    }

    String::from_utf8_lossy(&out).into_owned()
}

fn extract_station_list_segment(source: &str) -> Result<&str, StationCatalogError> {
    let marker = source.find("stationList").ok_or_else(|| {
        StationCatalogError::InvalidSource("stationList marker missing".to_string())
    })?;
    let slice = &source[marker..];
    let start_relative = slice.find('[').ok_or_else(|| {
        StationCatalogError::InvalidSource("stationList array start missing".to_string())
    })?;

    let start = marker + start_relative;
    let bytes = source.as_bytes();
    let mut idx = start;
    let mut depth = 0i32;
    let mut in_string = false;
    let mut escaped = false;

    while idx < bytes.len() {
        let byte = bytes[idx];

        if in_string {
            if escaped {
                escaped = false;
            } else if byte == b'\\' {
                escaped = true;
            } else if byte == b'"' {
                in_string = false;
            }
            idx += 1;
            continue;
        }

        if byte == b'"' {
            in_string = true;
            idx += 1;
            continue;
        }

        if byte == b'[' {
            depth += 1;
        } else if byte == b']' {
            depth -= 1;
            if depth == 0 {
                return Ok(&source[start + 1..idx]);
            }
        }

        idx += 1;
    }

    Err(StationCatalogError::InvalidSource(
        "stationList array end missing".to_string(),
    ))
}

fn extract_object_blocks(list_segment: &str) -> Vec<&str> {
    let mut values = Vec::new();
    let bytes = list_segment.as_bytes();
    let mut idx = 0usize;

    while idx < bytes.len() {
        if bytes[idx] != b'{' {
            idx += 1;
            continue;
        }

        let start = idx;
        let mut depth = 0i32;
        let mut in_string = false;
        let mut escaped = false;
        while idx < bytes.len() {
            let byte = bytes[idx];
            if in_string {
                if escaped {
                    escaped = false;
                } else if byte == b'\\' {
                    escaped = true;
                } else if byte == b'"' {
                    in_string = false;
                }
                idx += 1;
                continue;
            }

            if byte == b'"' {
                in_string = true;
                idx += 1;
                continue;
            }

            if byte == b'{' {
                depth += 1;
            } else if byte == b'}' {
                depth -= 1;
                if depth == 0 {
                    values.push(&list_segment[start + 1..idx]);
                    idx += 1;
                    break;
                }
            }

            idx += 1;
        }
    }

    values
}

fn parse_station_object(
    object_body: &str,
) -> Result<Option<Vec<StationCatalogRecord>>, StationCatalogError> {
    let fields = parse_js_object_fields(object_body);

    let gubun = match fields.get("gubun") {
        Some(value) => value,
        None => return Ok(None),
    };

    let station_code = fields
        .get("stn_cd")
        .map(|value| value.trim().to_ascii_uppercase())
        .filter(|value| !value.is_empty())
        .ok_or_else(|| {
            StationCatalogError::InvalidSource("station source missing stn_cd".to_string())
        })?;

    let station_name_ko = fields
        .get("stn_nm")
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
        .ok_or_else(|| {
            StationCatalogError::InvalidSource("station source missing stn_nm".to_string())
        })?;

    let line_code = fields
        .get("ln_cd")
        .and_then(|value| value.parse::<i32>().ok())
        .unwrap_or(0);

    let selected = fields
        .get("sel_yn")
        .map(|value| {
            let normalized = value.trim().to_ascii_lowercase();
            matches!(normalized.as_str(), "1" | "y" | "yes" | "true")
        })
        .unwrap_or(false);

    let remark = fields
        .get("rmk")
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty());

    let order_index = fields
        .get("ordr")
        .and_then(|value| value.parse::<i32>().ok())
        .unwrap_or(0);

    let providers = providers_from_gubun(gubun);
    if providers.is_empty() {
        return Ok(None);
    }

    let mut rows = Vec::new();
    for provider in providers {
        let station_name_en = station_name_alias(&station_name_ko).map(ToOwned::to_owned);
        let station_name_ja_katakana = Some(derive_station_name_ja_katakana(
            &station_name_ko,
            station_name_en.as_deref(),
        ));
        rows.push(StationCatalogRecord {
            provider: provider.to_string(),
            station_code: station_code.clone(),
            station_name_ko: station_name_ko.clone(),
            station_name_en,
            station_name_ja_katakana,
            line_code,
            selected,
            remark: remark.clone(),
            order_index,
            normalized_name: normalize_search_text(&station_name_ko),
            normalized_remark: remark.as_ref().map(|value| normalize_search_text(value)),
        });
    }

    Ok(Some(rows))
}

fn parse_js_object_fields(input: &str) -> HashMap<String, String> {
    let bytes = input.as_bytes();
    let mut idx = 0usize;
    let mut out = HashMap::new();

    while idx < bytes.len() {
        while idx < bytes.len() && (bytes[idx].is_ascii_whitespace() || bytes[idx] == b',') {
            idx += 1;
        }
        if idx >= bytes.len() {
            break;
        }

        let key_start = idx;
        while idx < bytes.len() && (bytes[idx].is_ascii_alphanumeric() || bytes[idx] == b'_') {
            idx += 1;
        }
        if idx == key_start {
            idx += 1;
            continue;
        }
        let key = String::from_utf8_lossy(&bytes[key_start..idx]).to_string();

        while idx < bytes.len() && bytes[idx].is_ascii_whitespace() {
            idx += 1;
        }
        if idx >= bytes.len() || bytes[idx] != b':' {
            continue;
        }
        idx += 1;
        while idx < bytes.len() && bytes[idx].is_ascii_whitespace() {
            idx += 1;
        }
        if idx >= bytes.len() {
            break;
        }

        let value = if bytes[idx] == b'"' {
            idx += 1;
            let mut raw = Vec::new();
            let mut escaped = false;
            while idx < bytes.len() {
                let byte = bytes[idx];
                idx += 1;
                if escaped {
                    raw.push(byte);
                    escaped = false;
                    continue;
                }
                if byte == b'\\' {
                    escaped = true;
                    continue;
                }
                if byte == b'"' {
                    break;
                }
                raw.push(byte);
            }
            String::from_utf8_lossy(&raw).into_owned()
        } else {
            let start = idx;
            while idx < bytes.len() && bytes[idx] != b',' {
                idx += 1;
            }
            String::from_utf8_lossy(&bytes[start..idx])
                .trim()
                .to_string()
        };

        out.insert(key, value);
    }

    out
}

fn providers_from_gubun(gubun: &str) -> Vec<&'static str> {
    let normalized = gubun.trim().to_ascii_lowercase();
    let mut values = Vec::new();
    if normalized.contains("srt") {
        values.push("srt");
    }
    if normalized.contains("korail") {
        values.push("ktx");
    }
    values
}

fn station_name_alias(name_ko: &str) -> Option<&'static str> {
    match name_ko {
        "수서" => Some("suseo"),
        "동탄" => Some("dongtan"),
        "평택지제" => Some("pyeongtaekjije"),
        "서울" => Some("seoul"),
        "광명" => Some("gwangmyeong"),
        "천안아산" => Some("cheonanasan"),
        "오송" => Some("osong"),
        "대전" => Some("daejeon"),
        "동대구" => Some("dongdaegu"),
        "경주" => Some("gyeongju"),
        "울산(통도사)" => Some("ulsan"),
        "부산" => Some("busan"),
        "공주" => Some("gongju"),
        "익산" => Some("iksan"),
        "정읍" => Some("jeongeup"),
        "광주송정" => Some("gwangjusongjeong"),
        "나주" => Some("naju"),
        "목포" => Some("mokpo"),
        "전주" => Some("jeonju"),
        "남원" => Some("namwon"),
        "곡성" => Some("gokseong"),
        "구례구" => Some("guryegu"),
        "순천" => Some("suncheon"),
        "여천" => Some("yeocheon"),
        "여수EXPO" => Some("yeosuexpo"),
        "밀양" => Some("miryang"),
        "진영" => Some("jinyeong"),
        "창원중앙" => Some("changwonjungang"),
        "창원" => Some("changwon"),
        "마산" => Some("masan"),
        "진주" => Some("jinju"),
        "포항" => Some("pohang"),
        "대구" => Some("daegu"),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_station_source_extracts_srt_and_ktx_entries() {
        let raw = r#"
            var stationList = [
                { gubun:"SRT", ln_cd:0, stn_cd:"0551", stn_nm:"수서", sel_yn:"1", rmk:"ㅅ", ordr:-1 },
                { gubun:"korail", ln_cd:1, stn_cd:"0010", stn_nm:"대전", sel_yn:"0", rmk:"ㄷ", ordr:2 }
            ];
        "#;

        let parsed = parse_station_source(raw).unwrap_or_else(|error| {
            panic!("parse failed: {error:?}");
        });

        assert_eq!(parsed.len(), 2);
        assert!(
            parsed
                .iter()
                .any(|row| row.provider == "srt" && row.station_code == "0551")
        );
        assert!(
            parsed
                .iter()
                .any(|row| row.provider == "ktx" && row.station_code == "0010")
        );
    }

    #[test]
    fn normalize_search_text_keeps_hangul_and_ascii() {
        let normalized = normalize_search_text("  수서 Station-01 ");
        assert_eq!(normalized, "수서station01");
    }
}
