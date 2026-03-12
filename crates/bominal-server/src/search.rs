//! Train search route handlers.
//!
//! - POST /api/search                      — search for trains
//! - GET  /api/stations/:provider          — get station list
//! - GET  /api/stations/:provider/suggest  — station autofill suggestions

use axum::Json;
use axum::extract::{Path, Query, State};
use serde::{Deserialize, Serialize};

use bominal_domain::station_search::{
    self, LangHint, LayoutHint, SearchMode, SearchOptions, StationSearchDocument,
};

use crate::error::AppError;
use crate::extractors::AuthUser;
use crate::state::SharedState;

/// Search request body.
#[derive(Debug, Deserialize)]
pub struct SearchRequest {
    pub provider: String,
    pub departure: String,
    pub arrival: String,
    pub date: Option<String>,
    pub time: Option<String>,
    pub available_only: Option<bool>,
}

/// Unified train result (provider-agnostic).
#[derive(Debug, Serialize)]
pub struct TrainResult {
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

/// Station entry for a provider.
#[derive(Debug, Serialize)]
pub struct StationEntry {
    pub name_ko: String,
    pub name_en: String,
    pub name_ja: String,
}

/// POST /api/search — search for trains.
pub async fn search_trains(
    user: AuthUser,
    State(state): State<SharedState>,
    Json(req): Json<SearchRequest>,
) -> Result<Json<Vec<TrainResult>>, AppError> {
    let available_only = req.available_only.unwrap_or(false);

    // Verify user has valid credentials for this provider
    let cred =
        bominal_db::provider::find_by_user_and_provider(&state.db, user.user_id, &req.provider)
            .await
            .map_err(|e| AppError::Internal(e.into()))?;

    match &cred {
        Some(c) if c.status == "valid" => {}
        Some(c) if c.status == "invalid" => {
            return Err(AppError::BadRequest(format!(
                "{} credentials are invalid. Please update in settings.",
                req.provider
            )));
        }
        _ => {
            return Err(AppError::BadRequest(format!(
                "{} credentials required. Please add in settings.",
                req.provider
            )));
        }
    }

    match req.provider.as_str() {
        "SRT" => search_srt(&req, available_only).await,
        "KTX" => search_ktx(&req, available_only).await,
        _ => Err(AppError::BadRequest(format!(
            "Invalid provider: {}",
            req.provider
        ))),
    }
}

/// GET /api/stations/:provider — list stations for a provider.
pub async fn list_stations(
    Path(provider): Path<String>,
) -> Result<Json<Vec<StationEntry>>, AppError> {
    match provider.as_str() {
        "SRT" => Ok(Json(srt_stations())),
        "KTX" => Ok(Json(ktx_stations())),
        _ => Err(AppError::BadRequest(format!(
            "Invalid provider: {provider}"
        ))),
    }
}

// ── Station suggest ─────────────────────────────────────────────────

/// Query parameters for the station suggest endpoint.
#[derive(Debug, Deserialize)]
pub struct SuggestQuery {
    pub q: String,
    #[serde(default)]
    pub mode: Option<String>,
    #[serde(default)]
    pub layout: Option<String>,
    #[serde(default)]
    pub lang: Option<String>,
    #[serde(default)]
    pub limit: Option<usize>,
}

/// A single station match from the suggest engine.
#[derive(Debug, Serialize)]
pub struct SuggestMatch {
    pub name_ko: String,
    pub name_en: String,
    pub name_ja: String,
    pub score: usize,
    pub confidence: f32,
    pub source: String,
}

/// Response from the station suggest endpoint.
#[derive(Debug, Serialize)]
pub struct SuggestResult {
    pub matches: Vec<SuggestMatch>,
    pub corrected_query: Option<String>,
    pub autocorrect_applied: bool,
}

/// GET /api/stations/:provider/suggest — station autofill suggestions.
pub async fn suggest_stations(
    Path(provider): Path<String>,
    Query(params): Query<SuggestQuery>,
) -> Result<Json<SuggestResult>, AppError> {
    let stations = match provider.as_str() {
        "SRT" => bominal_domain::i18n::stations::SRT_STATIONS,
        "KTX" => bominal_domain::i18n::stations::KTX_STATIONS,
        _ => {
            return Err(AppError::BadRequest(format!(
                "Invalid provider: {provider}"
            )));
        }
    };

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
        mode: SearchMode::from_query(params.mode.as_deref()),
        layout_hint: LayoutHint::from_query(params.layout.as_deref()),
        lang_hint: LangHint::from_query(params.lang.as_deref()),
        ..SearchOptions::default()
    };

    let limit = params.limit.unwrap_or(10).clamp(1, 30);
    let result = station_search::rank_station_documents(&documents, &params.q, options, limit);

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
                source: format!("{:?}", m.source),
            }
        })
        .collect();

    Ok(Json(SuggestResult {
        matches,
        corrected_query: result.corrected_query,
        autocorrect_applied: result.autocorrect_applied,
    }))
}

// ── Provider-specific search ─────────────────────────────────────────

async fn search_srt(
    req: &SearchRequest,
    available_only: bool,
) -> Result<Json<Vec<TrainResult>>, AppError> {
    let client = bominal_provider::srt::SrtClient::new();

    let trains = client
        .search_train(
            &req.departure,
            &req.arrival,
            req.date.as_deref(),
            req.time.as_deref(),
            available_only,
        )
        .await
        .map_err(map_provider_error)?;

    let results: Vec<TrainResult> = trains
        .iter()
        .map(|t| TrainResult {
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
        .collect();

    Ok(Json(results))
}

async fn search_ktx(
    req: &SearchRequest,
    available_only: bool,
) -> Result<Json<Vec<TrainResult>>, AppError> {
    let client = bominal_provider::ktx::KtxClient::new();

    let trains = client
        .search_train(
            &req.departure,
            &req.arrival,
            req.date.as_deref(),
            req.time.as_deref(),
            available_only,
        )
        .await
        .map_err(map_provider_error)?;

    let results: Vec<TrainResult> = trains
        .iter()
        .map(|t| TrainResult {
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
        .collect();

    Ok(Json(results))
}

// ── Station lists ────────────────────────────────────────────────────

fn srt_stations() -> Vec<StationEntry> {
    bominal_domain::i18n::stations::SRT_STATIONS
        .iter()
        .map(|s| StationEntry {
            name_ko: s.korean.to_string(),
            name_en: s.english.to_string(),
            name_ja: s.japanese.to_string(),
        })
        .collect()
}

fn ktx_stations() -> Vec<StationEntry> {
    bominal_domain::i18n::stations::KTX_STATIONS
        .iter()
        .map(|s| StationEntry {
            name_ko: s.korean.to_string(),
            name_en: s.english.to_string(),
            name_ja: s.japanese.to_string(),
        })
        .collect()
}

// ── Error mapping ────────────────────────────────────────────────────

pub(crate) fn map_provider_error(err: bominal_provider::types::ProviderError) -> AppError {
    use bominal_provider::types::ProviderError;
    match err {
        ProviderError::NoResults => AppError::BadRequest("No trains found".to_string()),
        ProviderError::SessionExpired => AppError::BadRequest(
            "Provider session expired. Please re-verify credentials.".to_string(),
        ),
        ProviderError::NetworkError(e) => {
            tracing::warn!(error = %e, "Provider network error");
            AppError::Internal(anyhow::anyhow!("Network error"))
        }
        ProviderError::NetFunnelBlocked => {
            AppError::BadRequest("Server busy. Please try again.".to_string())
        }
        other => AppError::BadRequest(other.to_string()),
    }
}
