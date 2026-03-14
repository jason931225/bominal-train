//! Search service — search trains, list stations.

use serde::{Deserialize, Serialize};

use crate::error::ServiceError;

/// Unified train search result.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TrainInfo {
    pub provider: String,
    pub train_type: String,
    pub train_type_name: String,
    pub train_number: String,
    pub dep_station: String,
    pub dep_date: String,
    pub dep_time: String,
    pub arr_station: String,
    pub arr_time: String,
    pub general_available: bool,
    pub special_available: bool,
    pub standby_available: bool,
}

/// Station display entry.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct StationInfo {
    pub name_ko: String,
    pub name_en: String,
    pub name_ja: String,
}

/// Get station list for a provider.
pub fn list_stations(provider: &str) -> Result<Vec<StationInfo>, ServiceError> {
    let stations = match provider {
        "SRT" => bominal_domain::i18n::stations::SRT_STATIONS,
        "KTX" => bominal_domain::i18n::stations::KTX_STATIONS,
        _ => return Err(ServiceError::validation(format!("Invalid provider: {provider}"))),
    };

    Ok(stations
        .iter()
        .map(|s| StationInfo {
            name_ko: s.korean.to_string(),
            name_en: s.english.to_string(),
            name_ja: s.japanese.to_string(),
        })
        .collect())
}

/// Search for trains on a given route.
pub async fn search_trains(
    provider: &str,
    departure: &str,
    arrival: &str,
    date: Option<&str>,
    time: Option<&str>,
    available_only: bool,
) -> Result<Vec<TrainInfo>, ServiceError> {
    match provider {
        "SRT" => search_srt(departure, arrival, date, time, available_only).await,
        "KTX" => search_ktx(departure, arrival, date, time, available_only).await,
        _ => Err(ServiceError::validation(format!("Invalid provider: {provider}"))),
    }
}

// ── Provider-specific search ────────────────────────────────────────

async fn search_srt(
    departure: &str,
    arrival: &str,
    date: Option<&str>,
    time: Option<&str>,
    available_only: bool,
) -> Result<Vec<TrainInfo>, ServiceError> {
    let client = bominal_provider::srt::SrtClient::new();

    let trains = client
        .search_train(departure, arrival, date, time, available_only)
        .await?;

    Ok(trains
        .iter()
        .map(|t| TrainInfo {
            provider: "SRT".to_string(),
            train_type: t.train_code.clone(),
            train_type_name: t.display_name().to_string(),
            train_number: t.train_number.clone(),
            dep_station: t.dep_station_name.clone(),
            dep_date: t.dep_date.clone(),
            dep_time: t.dep_time.clone(),
            arr_station: t.arr_station_name.clone(),
            arr_time: t.arr_time.clone(),
            general_available: t.general_seat_available(),
            special_available: t.special_seat_available(),
            standby_available: t.reserve_standby_available(),
        })
        .collect())
}

async fn search_ktx(
    departure: &str,
    arrival: &str,
    date: Option<&str>,
    time: Option<&str>,
    available_only: bool,
) -> Result<Vec<TrainInfo>, ServiceError> {
    let client = bominal_provider::ktx::KtxClient::new();

    let trains = client
        .search_train(departure, arrival, date, time, available_only)
        .await?;

    Ok(trains
        .iter()
        .map(|t| TrainInfo {
            provider: "KTX".to_string(),
            train_type: t.train_type.clone(),
            train_type_name: t.display_name().to_string(),
            train_number: t.train_no.clone(),
            dep_station: t.dep_name.clone(),
            dep_date: t.dep_date.clone(),
            dep_time: t.dep_time.clone(),
            arr_station: t.arr_name.clone(),
            arr_time: t.arr_time.clone(),
            general_available: t.general_seat_available(),
            special_available: t.special_seat_available(),
            standby_available: t.waiting_available(),
        })
        .collect())
}
