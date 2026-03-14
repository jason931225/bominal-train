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

/// POST /api/search — search for trains.
pub async fn search_trains(
    user: AuthUser,
    State(state): State<SharedState>,
    Json(req): Json<SearchRequest>,
) -> Result<Json<Vec<bominal_service::search::TrainInfo>>, AppError> {
    // Verify user has valid credentials for this provider (auth concern stays in handler)
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

    let available_only = req.available_only.unwrap_or(false);

    let result = bominal_service::search::search_trains(
        &req.provider,
        &req.departure,
        &req.arrival,
        req.date.as_deref(),
        req.time.as_deref(),
        available_only,
    )
    .await?;

    Ok(Json(result))
}

/// GET /api/stations/:provider — list stations for a provider.
pub async fn list_stations(
    Path(provider): Path<String>,
) -> Result<Json<Vec<bominal_service::search::StationInfo>>, AppError> {
    let result = bominal_service::search::list_stations(&provider)?;
    Ok(Json(result))
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
