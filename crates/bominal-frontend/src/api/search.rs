//! Search server functions.

use leptos::prelude::*;
use serde::{Deserialize, Serialize};

pub use bominal_domain::dto::{StationInfo, TrainInfo};

// ── Station suggest DTOs ────────────────────────────────────────────

/// A single ranked station match from the suggest engine.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SuggestMatch {
    pub name_ko: String,
    pub name_en: String,
    pub name_ja: String,
    pub score: usize,
    pub confidence: f32,
}

/// Response from the station suggest server function.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SuggestResult {
    pub matches: Vec<SuggestMatch>,
    pub corrected_query: Option<String>,
    pub autocorrect_applied: bool,
}

// ── Server functions ────────────────────────────────────────────────

/// Get station list for a provider.
#[server(prefix = "/sfn")]
pub async fn list_stations(provider: String) -> Result<Vec<StationInfo>, ServerFnError> {
    bominal_service::search::list_stations(&provider).map_err(|e| ServerFnError::new(e.to_string()))
}

/// Search for trains.
#[server(prefix = "/sfn")]
pub async fn search_trains(
    provider: String,
    departure: String,
    arrival: String,
    date: Option<String>,
    time: Option<String>,
) -> Result<Vec<TrainInfo>, ServerFnError> {
    bominal_service::search::search_trains(
        &provider,
        &departure,
        &arrival,
        date.as_deref(),
        time.as_deref(),
        false,
    )
    .await
    .map_err(|e| ServerFnError::new(e.to_string()))
}

/// Station suggest/autocorrect — calls the domain search engine directly.
#[server(prefix = "/sfn")]
pub async fn suggest_stations(
    provider: String,
    query: String,
    mode: Option<String>,
) -> Result<SuggestResult, ServerFnError> {
    use bominal_domain::i18n::stations::stations_for_provider;
    use bominal_domain::station_search::{self, SearchMode, SearchOptions, StationSearchDocument};

    let stations = stations_for_provider(&provider);

    let documents: Vec<StationSearchDocument<'_>> = stations
        .iter()
        .map(|s| StationSearchDocument {
            station_name_ko: s.korean,
            station_name_en: Some(s.english),
            station_name_ja_katakana: s.japanese,
            normalized_name: s.korean,
        })
        .collect();

    let options = SearchOptions {
        mode: SearchMode::from_query(mode.as_deref()),
        ..SearchOptions::default()
    };

    let result = station_search::rank_station_documents(&documents, &query, options, 10);

    let matches = result
        .matches
        .iter()
        .map(|m| {
            let station = &stations[m.station_index];
            SuggestMatch {
                name_ko: station.korean.to_string(),
                name_en: station.english.to_string(),
                name_ja: station.japanese.to_string(),
                score: m.score,
                confidence: m.confidence,
            }
        })
        .collect();

    Ok(SuggestResult {
        matches,
        corrected_query: result.corrected_query,
        autocorrect_applied: result.autocorrect_applied,
    })
}
