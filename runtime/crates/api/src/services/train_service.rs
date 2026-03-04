use std::{
    collections::{HashMap, HashSet},
    path::Path,
    sync::OnceLock,
};

use bominal_shared::station_catalog;
use chrono::{DateTime, NaiveDate, Utc};
use redis::AsyncCommands;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use sqlx::{PgPool, Row};
use tokio::sync::Mutex;
use tracing::{error, warn};
use uuid::Uuid;

use super::super::AppState;
use super::{
    payment_method_service, provider_credentials_service, provider_jobs_service, station_search,
};
const STATION_CACHE_TTL_SECONDS: u64 = 300;

#[derive(Debug)]
pub(crate) enum TrainServiceError {
    InvalidRequest(String),
    Unauthorized(String),
    NotFound(String),
    ServiceUnavailable(String),
    Internal,
}

#[derive(Debug, Serialize)]
pub(crate) struct TrainPreflightResponse {
    pub(crate) providers: Vec<TrainProviderPreflight>,
    pub(crate) station_catalog: StationCatalogStatus,
}

#[derive(Debug, Serialize)]
pub(crate) struct TrainProviderPreflight {
    pub(crate) provider: String,
    pub(crate) credentials_ready: bool,
    pub(crate) payment_ready: bool,
    pub(crate) payment_method_ref: Option<String>,
}

#[derive(Debug, Serialize)]
pub(crate) struct StationCatalogStatus {
    pub(crate) loaded: bool,
    pub(crate) source_url: &'static str,
    pub(crate) counts: HashMap<String, i64>,
    pub(crate) last_refreshed_at: Option<DateTime<Utc>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) error: Option<String>,
}

#[derive(Debug, Serialize)]
pub(crate) struct StationSuggestResponse {
    pub(crate) query: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) corrected_query: Option<String>,
    pub(crate) autocorrect_applied: bool,
    pub(crate) suggestions: Vec<StationSuggestion>,
}

#[derive(Debug, Serialize)]
pub(crate) struct StationRegionsResponse {
    pub(crate) quick: Vec<StationRegionStation>,
    pub(crate) regions: Vec<StationRegionGroup>,
}

#[derive(Debug, Serialize)]
pub(crate) struct StationRegionGroup {
    pub(crate) key: String,
    pub(crate) label: String,
    pub(crate) stations: Vec<StationRegionStation>,
}

#[derive(Debug, Serialize, Clone)]
pub(crate) struct StationRegionStation {
    pub(crate) station_code: String,
    pub(crate) station_name_ko: String,
    pub(crate) station_name_en: Option<String>,
    pub(crate) station_name_ja_katakana: String,
    pub(crate) supported_providers: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub(crate) enum TrainPassengerKind {
    Adult,
    Child,
    Senior,
    #[serde(rename = "disability_1_to_3", alias = "disability1_to3")]
    Disability1To3,
    #[serde(rename = "disability_4_to_6", alias = "disability4_to6")]
    Disability4To6,
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq)]
pub(crate) struct TrainPassengerCount {
    pub(crate) kind: TrainPassengerKind,
    pub(crate) count: u8,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub(crate) struct StationSuggestion {
    pub(crate) provider: String,
    pub(crate) station_code: String,
    pub(crate) station_name_ko: String,
    pub(crate) station_name_en: Option<String>,
    pub(crate) station_name_ja_katakana: String,
    pub(crate) line_code: i32,
    pub(crate) selected: bool,
    pub(crate) order_index: i32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) match_source: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) confidence: Option<f32>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct CreateTrainSearchRequest {
    pub(crate) dep_station_code: String,
    pub(crate) arr_station_code: String,
    #[serde(default)]
    pub(crate) dep_date: Option<String>,
    #[serde(default)]
    pub(crate) dep_time: Option<String>,
    #[serde(default)]
    pub(crate) passengers: Option<Vec<TrainPassengerCount>>,
    #[serde(default)]
    pub(crate) passenger_count: Option<u8>,
    #[serde(default)]
    pub(crate) available_only: Option<bool>,
}

#[derive(Debug, Serialize)]
pub(crate) struct CreateTrainSearchResponse {
    pub(crate) accepted: bool,
    pub(crate) search_id: String,
    pub(crate) status: String,
    pub(crate) providers: Vec<String>,
    pub(crate) jobs: Vec<TrainProviderJobStatus>,
}

#[derive(Debug, Serialize)]
pub(crate) struct TrainSearchStatusResponse {
    pub(crate) search_id: String,
    pub(crate) status: String,
    pub(crate) request: TrainSearchRequestEcho,
    pub(crate) providers: Vec<TrainProviderJobStatus>,
    pub(crate) results: Vec<TrainSearchResult>,
    pub(crate) errors: Vec<TrainProviderError>,
    pub(crate) created_at: DateTime<Utc>,
    pub(crate) updated_at: DateTime<Utc>,
    pub(crate) completed_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Serialize)]
pub(crate) struct TrainSearchHistoryResponse {
    pub(crate) searches: Vec<TrainSearchHistoryItem>,
}

#[derive(Debug, Serialize)]
pub(crate) struct TrainSearchHistoryItem {
    pub(crate) search_id: String,
    pub(crate) status: String,
    pub(crate) dep_station_code: String,
    pub(crate) arr_station_code: String,
    pub(crate) dep_date: String,
    pub(crate) dep_time: String,
    pub(crate) providers: Vec<String>,
    pub(crate) created_at: DateTime<Utc>,
    pub(crate) updated_at: DateTime<Utc>,
    pub(crate) completed_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Serialize)]
pub(crate) struct TrainProviderJobStatus {
    pub(crate) provider: String,
    pub(crate) runtime_job_id: String,
    pub(crate) status: String,
    pub(crate) error_message: Option<String>,
}

#[derive(Debug, Serialize)]
pub(crate) struct TrainSearchResult {
    pub(crate) provider: String,
    pub(crate) runtime_job_id: String,
    pub(crate) train_code: String,
    pub(crate) train_number: String,
    pub(crate) dep_station_code: String,
    pub(crate) arr_station_code: String,
    pub(crate) dep_date: String,
    pub(crate) dep_time: String,
    pub(crate) arr_date: String,
    pub(crate) arr_time: String,
    pub(crate) general_seat_available: bool,
    pub(crate) special_seat_available: bool,
    pub(crate) standby_available: bool,
}

#[derive(Debug, Serialize)]
pub(crate) struct TrainProviderError {
    pub(crate) provider: String,
    pub(crate) runtime_job_id: String,
    pub(crate) status: String,
    pub(crate) message: String,
}

#[derive(Debug, Serialize)]
pub(crate) struct TrainSearchRequestEcho {
    pub(crate) dep_station_code: String,
    pub(crate) arr_station_code: String,
    pub(crate) dep_date: String,
    pub(crate) dep_time: String,
    pub(crate) passengers: Vec<TrainPassengerCount>,
    pub(crate) passenger_count: i32,
    pub(crate) available_only: bool,
}

#[derive(Debug, Deserialize)]
pub(crate) struct PutTrainProviderCredentialsRequest {
    pub(crate) account_identifier: String,
    pub(crate) password: String,
}

#[derive(Debug, Deserialize)]
pub(crate) struct PutTrainPaymentMethodRequest {
    pub(crate) pan_ev: String,
    pub(crate) expiry_month_ev: String,
    pub(crate) expiry_year_ev: String,
    pub(crate) birth_or_business_ev: String,
    pub(crate) card_password_two_digits_ev: String,
    #[serde(default)]
    pub(crate) card_last4: String,
    #[serde(default)]
    pub(crate) card_brand: Option<String>,
    #[serde(default)]
    pub(crate) payment_method_ref: Option<String>,
}

#[derive(Debug, Serialize)]
pub(crate) struct TrainPaymentMethodListResponse {
    pub(crate) cards: Vec<TrainPaymentCardSummary>,
}

#[derive(Debug, Serialize)]
pub(crate) struct TrainPaymentCardSummary {
    pub(crate) payment_method_ref: String,
    pub(crate) card_last4: String,
    pub(crate) card_brand: Option<String>,
    pub(crate) updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize)]
pub(crate) struct TrainPaymentMethodDeleteResponse {
    pub(crate) deleted: bool,
    pub(crate) payment_method_ref: String,
}

#[derive(Debug, Serialize)]
pub(crate) struct TrainProviderCredentialsDeleteResponse {
    pub(crate) deleted: bool,
    pub(crate) provider: String,
}

#[derive(Debug, Deserialize)]
pub(crate) struct StationSuggestQuery {
    pub(crate) q: String,
    #[serde(default)]
    pub(crate) provider: Option<String>,
    #[serde(default)]
    pub(crate) limit: Option<usize>,
    #[serde(default)]
    pub(crate) layout_hint: Option<String>,
    #[serde(default)]
    pub(crate) lang_hint: Option<String>,
    #[serde(default)]
    pub(crate) apply_mode: Option<String>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct SearchHistoryQuery {
    #[serde(default)]
    pub(crate) limit: Option<usize>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct StationRegionsQuery {
    #[serde(default)]
    pub(crate) provider: Option<String>,
}

#[derive(Debug)]
struct SearchSessionRow {
    search_id: String,
    dep_station_code: String,
    arr_station_code: String,
    dep_date: String,
    dep_time: String,
    passengers_json: Value,
    passenger_count: i32,
    available_only: bool,
    created_at: DateTime<Utc>,
    completed_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct StationCatalogEntry {
    provider: String,
    station_code: String,
    station_name_ko: String,
    station_name_en: Option<String>,
    station_name_ja_katakana: String,
    line_code: i32,
    selected: bool,
    remark: Option<String>,
    order_index: i32,
    normalized_name: String,
    normalized_remark: Option<String>,
}

static STATION_REFRESH_LOCK: OnceLock<Mutex<()>> = OnceLock::new();

fn station_refresh_lock() -> &'static Mutex<()> {
    STATION_REFRESH_LOCK.get_or_init(|| Mutex::new(()))
}

pub(crate) async fn load_preflight(
    state: &AppState,
    user_id: &str,
) -> Result<TrainPreflightResponse, TrainServiceError> {
    ensure_valid_user_id(user_id)?;

    let pool = require_pool(state)?;
    let providers = vec![
        provider_preflight(pool, "srt", user_id).await?,
        provider_preflight(pool, "ktx", user_id).await?,
    ];

    let station_catalog = load_station_catalog_status(state, pool).await;
    Ok(TrainPreflightResponse {
        providers,
        station_catalog,
    })
}

async fn load_station_catalog_status(state: &AppState, pool: &PgPool) -> StationCatalogStatus {
    if let Err(err) = ensure_station_catalog_loaded(state).await {
        let message = train_service_error_message(&err);
        warn!(error = ?err, "station catalog preflight degraded");
        return StationCatalogStatus {
            loaded: false,
            source_url: station_catalog::STATION_SOURCE_URL,
            counts: HashMap::new(),
            last_refreshed_at: None,
            error: Some(message),
        };
    }

    let counts_rows = match sqlx::query(
        "select provider, count(*)::bigint as count, max(updated_at) as refreshed_at from train_station_catalog group by provider",
    )
    .fetch_all(pool)
    .await
    {
        Ok(rows) => rows,
        Err(err) => {
            warn!(error = %err, "station catalog status query failed");
            return StationCatalogStatus {
                loaded: false,
                source_url: station_catalog::STATION_SOURCE_URL,
                counts: HashMap::new(),
                last_refreshed_at: None,
                error: Some("station catalog status query failed".to_string()),
            };
        }
    };

    let mut counts = HashMap::new();
    let mut last_refreshed: Option<DateTime<Utc>> = None;
    for row in counts_rows {
        let provider: String = match row.try_get("provider") {
            Ok(value) => value,
            Err(_) => {
                return StationCatalogStatus {
                    loaded: false,
                    source_url: station_catalog::STATION_SOURCE_URL,
                    counts: HashMap::new(),
                    last_refreshed_at: None,
                    error: Some("station catalog provider parse failed".to_string()),
                };
            }
        };
        let count: i64 = match row.try_get("count") {
            Ok(value) => value,
            Err(_) => {
                return StationCatalogStatus {
                    loaded: false,
                    source_url: station_catalog::STATION_SOURCE_URL,
                    counts: HashMap::new(),
                    last_refreshed_at: None,
                    error: Some("station catalog count parse failed".to_string()),
                };
            }
        };
        let refreshed_at: Option<DateTime<Utc>> = match row.try_get("refreshed_at") {
            Ok(value) => value,
            Err(_) => {
                return StationCatalogStatus {
                    loaded: false,
                    source_url: station_catalog::STATION_SOURCE_URL,
                    counts: HashMap::new(),
                    last_refreshed_at: None,
                    error: Some("station catalog refreshed_at parse failed".to_string()),
                };
            }
        };
        counts.insert(provider, count);
        if let Some(value) = refreshed_at
            && last_refreshed.is_none_or(|existing| value > existing)
        {
            last_refreshed = Some(value);
        }
    }

    StationCatalogStatus {
        loaded: counts.values().sum::<i64>() > 0,
        source_url: station_catalog::STATION_SOURCE_URL,
        counts,
        last_refreshed_at: last_refreshed,
        error: None,
    }
}

fn train_service_error_message(error: &TrainServiceError) -> String {
    match error {
        TrainServiceError::InvalidRequest(message)
        | TrainServiceError::Unauthorized(message)
        | TrainServiceError::NotFound(message)
        | TrainServiceError::ServiceUnavailable(message) => message.clone(),
        TrainServiceError::Internal => "train service internal failure".to_string(),
    }
}

pub(crate) async fn suggest_stations(
    state: &AppState,
    query: StationSuggestQuery,
) -> Result<StationSuggestResponse, TrainServiceError> {
    ensure_station_catalog_loaded(state).await?;

    let query_raw = query.q.trim().to_string();
    if query_raw.is_empty() {
        return Err(TrainServiceError::InvalidRequest(
            "station query is required".to_string(),
        ));
    }

    let limit = query.limit.unwrap_or(12).clamp(1, 30);
    let provider_scope = parse_provider_scope(query.provider.as_deref())?;
    let query_norm = station_catalog::normalize_search_text(&query_raw);
    let search_options = station_search::SearchOptions {
        mode: station_search::SearchMode::from_query(query.apply_mode.as_deref()),
        layout_hint: station_search::LayoutHint::from_query(query.layout_hint.as_deref()),
        lang_hint: station_search::LangHint::from_query(query.lang_hint.as_deref()),
        ..station_search::SearchOptions::default()
    };

    let mut scored: Vec<(
        usize,
        Option<f32>,
        Option<station_search::MatchSource>,
        StationCatalogEntry,
    )> = Vec::new();
    let mut corrected_candidates: Vec<(usize, f32, String)> = Vec::new();
    for provider in provider_scope {
        let stations = load_station_catalog_for_provider(state, provider).await?;
        let documents: Vec<station_search::StationSearchDocument<'_>> = stations
            .iter()
            .map(|station| station_search::StationSearchDocument {
                station_code: station.station_code.as_str(),
                station_name_ko: station.station_name_ko.as_str(),
                station_name_en: station.station_name_en.as_deref(),
                station_name_ja_katakana: station.station_name_ja_katakana.as_str(),
                normalized_name: station.normalized_name.as_str(),
            })
            .collect();

        let ranked = station_search::rank_station_documents(
            &documents,
            query_raw.as_str(),
            search_options,
            (limit * 3).clamp(12, 90),
        );
        if ranked.autocorrect_applied
            && let (Some(top), Some(corrected_query)) =
                (ranked.matches.first(), ranked.corrected_query.as_ref())
        {
            corrected_candidates.push((top.score, top.confidence, corrected_query.clone()));
        }

        if ranked.matches.is_empty() {
            // Compatibility fallback while the new ranker rolls out.
            for station in stations {
                if let Some(score) = station_match_score(&station, &query_raw, &query_norm) {
                    scored.push((score, None, None, station));
                }
            }
            continue;
        }

        for matched in ranked.matches {
            if let Some(station) = stations.get(matched.station_index) {
                scored.push((
                    matched.score,
                    Some(matched.confidence),
                    Some(matched.source),
                    station.clone(),
                ));
            }
        }
    }

    scored.sort_by(|left, right| {
        left.0
            .cmp(&right.0)
            .then_with(|| right.1.unwrap_or(0.0).total_cmp(&left.1.unwrap_or(0.0)))
            .then_with(|| right.3.selected.cmp(&left.3.selected))
            .then_with(|| left.3.order_index.cmp(&right.3.order_index))
            .then_with(|| left.3.station_name_ko.cmp(&right.3.station_name_ko))
    });

    let mut seen = HashSet::new();
    let mut suggestions = Vec::with_capacity(limit);
    for (_, confidence, source, station) in scored {
        let key = format!("{}:{}", station.provider, station.station_code);
        if !seen.insert(key) {
            continue;
        }

        suggestions.push(StationSuggestion {
            provider: station.provider,
            station_code: station.station_code,
            station_name_ko: station.station_name_ko,
            station_name_en: station.station_name_en,
            station_name_ja_katakana: station.station_name_ja_katakana,
            line_code: station.line_code,
            selected: station.selected,
            order_index: station.order_index,
            match_source: source.map(|value| value.as_api_str().to_string()),
            confidence,
        });

        if suggestions.len() >= limit {
            break;
        }
    }

    corrected_candidates.sort_by(|left, right| {
        left.0
            .cmp(&right.0)
            .then_with(|| right.1.total_cmp(&left.1))
            .then_with(|| left.2.cmp(&right.2))
    });
    let corrected_query = corrected_candidates.first().map(|item| item.2.clone());

    Ok(StationSuggestResponse {
        query: query_raw,
        corrected_query: corrected_query.clone(),
        autocorrect_applied: corrected_query.is_some(),
        suggestions,
    })
}

pub(crate) async fn load_station_regions(
    state: &AppState,
    query: StationRegionsQuery,
) -> Result<StationRegionsResponse, TrainServiceError> {
    ensure_station_catalog_loaded(state).await?;
    let provider_scope = parse_provider_scope(query.provider.as_deref())?;

    #[derive(Debug, Clone)]
    struct AggregatedStation {
        station_code: String,
        station_name_ko: String,
        station_name_en: Option<String>,
        station_name_ja_katakana: String,
        selected: bool,
        order_index: i32,
        supported_providers: Vec<String>,
    }

    let mut merged: HashMap<String, AggregatedStation> = HashMap::new();
    for provider in provider_scope {
        let stations = load_station_catalog_for_provider(state, provider).await?;
        for station in stations {
            let provider_name = provider.to_ascii_uppercase();
            let key = station.station_code.clone();
            match merged.get_mut(&key) {
                Some(existing) => {
                    if !existing
                        .supported_providers
                        .iter()
                        .any(|value| value == &provider_name)
                    {
                        existing.supported_providers.push(provider_name.clone());
                    }
                    existing.selected = existing.selected || station.selected;
                    if station.order_index < existing.order_index {
                        existing.order_index = station.order_index;
                    }
                    if existing.station_name_en.is_none() && station.station_name_en.is_some() {
                        existing.station_name_en = station.station_name_en.clone();
                    }
                }
                None => {
                    merged.insert(
                        key,
                        AggregatedStation {
                            station_code: station.station_code,
                            station_name_ko: station.station_name_ko,
                            station_name_en: station.station_name_en,
                            station_name_ja_katakana: station.station_name_ja_katakana,
                            selected: station.selected,
                            order_index: station.order_index,
                            supported_providers: vec![provider_name],
                        },
                    );
                }
            }
        }
    }

    let mut values = merged.into_values().collect::<Vec<_>>();
    values.sort_by(|left, right| {
        right
            .selected
            .cmp(&left.selected)
            .then_with(|| left.order_index.cmp(&right.order_index))
            .then_with(|| left.station_name_ko.cmp(&right.station_name_ko))
    });

    let mut regions_map: HashMap<&'static str, Vec<StationRegionStation>> = HashMap::new();
    let mut quick = Vec::new();
    for station in values {
        let region = curated_region_for_station(station.station_name_ko.as_str());
        let station_view = StationRegionStation {
            station_code: station.station_code,
            station_name_ko: station.station_name_ko,
            station_name_en: station.station_name_en,
            station_name_ja_katakana: station.station_name_ja_katakana,
            supported_providers: ordered_provider_labels(&station.supported_providers),
        };
        regions_map
            .entry(region)
            .or_default()
            .push(station_view.clone());
        if station.selected && quick.len() < 10 {
            quick.push(station_view);
        }
    }

    let mut all = Vec::new();
    for value in regions_map.values() {
        all.extend(value.iter().cloned());
    }
    all.sort_by(|left, right| left.station_name_ko.cmp(&right.station_name_ko));
    all.dedup_by(|left, right| left.station_code == right.station_code);
    regions_map.insert("all", all);

    regions_map.insert("major", quick.clone());

    let regions = curated_region_order()
        .iter()
        .map(|(key, label)| StationRegionGroup {
            key: (*key).to_string(),
            label: (*label).to_string(),
            stations: regions_map.remove(*key).unwrap_or_default(),
        })
        .collect::<Vec<_>>();

    Ok(StationRegionsResponse { quick, regions })
}

pub(crate) async fn create_search(
    state: &AppState,
    user_id: &str,
    payload: CreateTrainSearchRequest,
) -> Result<CreateTrainSearchResponse, TrainServiceError> {
    ensure_valid_user_id(user_id)?;
    ensure_station_catalog_loaded(state).await?;

    let dep_station_code = normalize_station_code(&payload.dep_station_code)?;
    let arr_station_code = normalize_station_code(&payload.arr_station_code)?;
    if dep_station_code == arr_station_code {
        return Err(TrainServiceError::InvalidRequest(
            "departure and arrival stations must differ".to_string(),
        ));
    }

    let dep_date = normalize_dep_date(payload.dep_date.as_deref())?;
    let dep_time = normalize_dep_time(payload.dep_time.as_deref())?;
    let passengers = normalize_passengers(payload.passengers, payload.passenger_count)?;
    let passenger_count = passengers
        .iter()
        .map(|item| i32::from(item.count))
        .sum::<i32>();
    let available_only = payload.available_only.unwrap_or(true);

    let pool = require_pool(state)?;
    let providers_ready = resolve_ready_providers(pool, user_id).await?;
    if providers_ready.is_empty() {
        return Err(TrainServiceError::InvalidRequest(
            "store provider credentials before searching".to_string(),
        ));
    }

    let mut selected_providers = Vec::new();
    for provider in providers_ready {
        if station_pair_supported_for_provider(pool, provider, &dep_station_code, &arr_station_code)
            .await?
        {
            selected_providers.push(provider.to_string());
        }
    }
    if selected_providers.is_empty() {
        return Err(TrainServiceError::InvalidRequest(
            "station pair is not supported by credential-ready providers".to_string(),
        ));
    }

    let search_id = Uuid::new_v4().to_string();
    let now = Utc::now();
    let passengers_json =
        serde_json::to_value(&passengers).map_err(|_| TrainServiceError::Internal)?;

    sqlx::query(
        "insert into train_search_sessions (search_id, user_id, dep_station_code, arr_station_code, dep_date, dep_time, available_only, passenger_count, passengers_json, providers, status, created_at, updated_at) values ($1, $2, $3, $4, $5, $6, $7, $8, cast($9 as jsonb), $10, 'queued', $11, $11)",
    )
    .bind(&search_id)
    .bind(user_id)
    .bind(&dep_station_code)
    .bind(&arr_station_code)
    .bind(&dep_date)
    .bind(&dep_time)
    .bind(available_only)
    .bind(passenger_count)
    .bind(&passengers_json)
    .bind(&selected_providers)
    .bind(now)
    .execute(pool)
    .await
    .map_err(|err| {
        error!(error = %err, search_id = %search_id, "failed to insert train search session");
        TrainServiceError::Internal
    })?;

    let mut provider_jobs = Vec::with_capacity(selected_providers.len());
    for provider in &selected_providers {
        let request_payload = json!({
            "user_id": user_id,
            "subject_ref": user_id,
            "refs": { "subject_ref": user_id },
            "request": {
                "dep_station_code": dep_station_code,
                "arr_station_code": arr_station_code,
                "dep_date": dep_date,
                "dep_time": dep_time,
                "available_only": available_only,
                "passengers": passengers.iter().map(|item| json!({
                    "kind": provider_passenger_kind(item.kind),
                    "count": item.count,
                })).collect::<Vec<Value>>(),
            }
        });

        let result = provider_jobs_service::create_provider_job(
            state,
            provider_jobs_service::CreateProviderJobRequest {
                provider: provider.clone(),
                operation: "search_train".to_string(),
                idempotency_key: Some(format!("{search_id}:{provider}:search_train")),
                payload: request_payload,
            },
        )
        .await
        .map_err(map_provider_jobs_error)?;

        sqlx::query(
            "insert into train_search_session_jobs (search_id, provider, runtime_job_id, status, created_at, updated_at) values ($1, $2, $3, $4, $5, $5)",
        )
        .bind(&search_id)
        .bind(provider)
        .bind(&result.job_id)
        .bind(&result.status)
        .bind(now)
        .execute(pool)
        .await
        .map_err(|err| {
            error!(
                error = %err,
                search_id = %search_id,
                provider = %provider,
                "failed to insert train search session job"
            );
            TrainServiceError::Internal
        })?;

        provider_jobs.push(TrainProviderJobStatus {
            provider: provider.clone(),
            runtime_job_id: result.job_id,
            status: result.status,
            error_message: None,
        });
    }

    let status = aggregate_status(provider_jobs.iter().map(|item| item.status.as_str()));
    sqlx::query(
        "update train_search_sessions set status = $2, updated_at = now() where search_id = $1",
    )
    .bind(&search_id)
    .bind(status)
    .execute(pool)
    .await
    .map_err(|err| {
        error!(error = %err, search_id = %search_id, "failed to update train search status");
        TrainServiceError::Internal
    })?;

    Ok(CreateTrainSearchResponse {
        accepted: true,
        search_id,
        status: status.to_string(),
        providers: selected_providers,
        jobs: provider_jobs,
    })
}

pub(crate) async fn get_search(
    state: &AppState,
    user_id: &str,
    search_id: &str,
) -> Result<TrainSearchStatusResponse, TrainServiceError> {
    ensure_valid_user_id(user_id)?;
    let search_id = search_id.trim();
    if search_id.is_empty() {
        return Err(TrainServiceError::InvalidRequest(
            "search id is required".to_string(),
        ));
    }

    let pool = require_pool(state)?;
    let session_row = load_search_session(pool, user_id, search_id)
        .await?
        .ok_or_else(|| TrainServiceError::NotFound("search session not found".to_string()))?;
    let request_passengers =
        parse_passengers_from_json(&session_row.passengers_json, session_row.passenger_count);

    let mut providers = load_search_jobs(pool, &session_row.search_id).await?;
    let mut results = Vec::new();
    let mut errors = Vec::new();

    for provider_job in &mut providers {
        let events = sqlx::query(
            "select event_type, event_payload from runtime_job_events where job_id = $1 order by id desc limit 24",
        )
        .bind(&provider_job.runtime_job_id)
        .fetch_all(pool)
        .await
        .map_err(|_| TrainServiceError::Internal)?;

        let mut extracted = extract_train_results_from_events(
            &provider_job.provider,
            &provider_job.runtime_job_id,
            &events,
        );
        results.append(&mut extracted);

        if is_failure_status(&provider_job.status) {
            let message = extract_error_message_from_events(&events)
                .or_else(|| provider_job.error_message.clone())
                .unwrap_or_else(|| "provider execution failed".to_string());
            provider_job.error_message = Some(message.clone());
            errors.push(TrainProviderError {
                provider: provider_job.provider.clone(),
                runtime_job_id: provider_job.runtime_job_id.clone(),
                status: provider_job.status.clone(),
                message,
            });
        }
    }

    results.sort_by(|left, right| {
        left.dep_date
            .cmp(&right.dep_date)
            .then_with(|| left.dep_time.cmp(&right.dep_time))
            .then_with(|| left.provider.cmp(&right.provider))
            .then_with(|| left.train_number.cmp(&right.train_number))
    });

    let status = aggregate_status(providers.iter().map(|item| item.status.as_str())).to_string();
    let completed_at = if is_terminal_status(&status) {
        Some(Utc::now())
    } else {
        None
    };

    for provider in &providers {
        sqlx::query("update train_search_session_jobs set status = $3, error_message = $4, updated_at = now() where search_id = $1 and provider = $2")
            .bind(&session_row.search_id)
            .bind(&provider.provider)
            .bind(&provider.status)
            .bind(&provider.error_message)
            .execute(pool)
            .await
            .map_err(|_| TrainServiceError::Internal)?;
    }

    let session_error_message = if errors.is_empty() {
        None
    } else {
        Some(
            errors
                .iter()
                .map(|item| format!("{}: {}", item.provider, item.message))
                .collect::<Vec<_>>()
                .join(" | "),
        )
    };

    sqlx::query("update train_search_sessions set status = $2, error_message = $3, completed_at = coalesce($4, completed_at), updated_at = now() where search_id = $1 and user_id = $5")
        .bind(&session_row.search_id)
        .bind(&status)
        .bind(&session_error_message)
        .bind(completed_at)
        .bind(user_id)
        .execute(pool)
        .await
        .map_err(|_| TrainServiceError::Internal)?;

    Ok(TrainSearchStatusResponse {
        search_id: session_row.search_id,
        status,
        request: TrainSearchRequestEcho {
            dep_station_code: session_row.dep_station_code,
            arr_station_code: session_row.arr_station_code,
            dep_date: session_row.dep_date,
            dep_time: session_row.dep_time,
            passengers: request_passengers,
            passenger_count: session_row.passenger_count,
            available_only: session_row.available_only,
        },
        providers,
        results,
        errors,
        created_at: session_row.created_at,
        updated_at: Utc::now(),
        completed_at: session_row.completed_at.or(completed_at),
    })
}

pub(crate) async fn list_search_history(
    state: &AppState,
    user_id: &str,
    limit: usize,
) -> Result<TrainSearchHistoryResponse, TrainServiceError> {
    ensure_valid_user_id(user_id)?;
    let pool = require_pool(state)?;
    let limit = i64::try_from(limit.clamp(1, 100)).unwrap_or(20);

    let rows = sqlx::query(
        "select search_id, status, dep_station_code, arr_station_code, dep_date, dep_time, providers, created_at, updated_at, completed_at from train_search_sessions where user_id = $1 order by created_at desc limit $2",
    )
    .bind(user_id)
    .bind(limit)
    .fetch_all(pool)
    .await
    .map_err(|_| TrainServiceError::Internal)?;

    let mut searches = Vec::with_capacity(rows.len());
    for row in rows {
        searches.push(TrainSearchHistoryItem {
            search_id: row
                .try_get("search_id")
                .map_err(|_| TrainServiceError::Internal)?,
            status: row
                .try_get("status")
                .map_err(|_| TrainServiceError::Internal)?,
            dep_station_code: row
                .try_get("dep_station_code")
                .map_err(|_| TrainServiceError::Internal)?,
            arr_station_code: row
                .try_get("arr_station_code")
                .map_err(|_| TrainServiceError::Internal)?,
            dep_date: row
                .try_get("dep_date")
                .map_err(|_| TrainServiceError::Internal)?,
            dep_time: row
                .try_get("dep_time")
                .map_err(|_| TrainServiceError::Internal)?,
            providers: row
                .try_get("providers")
                .map_err(|_| TrainServiceError::Internal)?,
            created_at: row
                .try_get("created_at")
                .map_err(|_| TrainServiceError::Internal)?,
            updated_at: row
                .try_get("updated_at")
                .map_err(|_| TrainServiceError::Internal)?,
            completed_at: row
                .try_get("completed_at")
                .map_err(|_| TrainServiceError::Internal)?,
        });
    }

    Ok(TrainSearchHistoryResponse { searches })
}

pub(crate) async fn put_provider_credentials_for_user(
    state: &AppState,
    user_id: &str,
    provider: &str,
    payload: PutTrainProviderCredentialsRequest,
) -> Result<provider_credentials_service::PutProviderCredentialsResult, TrainServiceError> {
    ensure_valid_user_id(user_id)?;
    let account_identifier = payload.account_identifier.trim();
    if account_identifier.is_empty() || payload.password.trim().is_empty() {
        return Err(TrainServiceError::InvalidRequest(
            "account_identifier and password are required".to_string(),
        ));
    }

    provider_credentials_service::put_provider_credentials(
        state,
        provider,
        provider_credentials_service::PutProviderCredentialsRequest {
            subject_ref: Some(user_id.to_string()),
            identity_ciphertext: account_identifier.to_string(),
            password_ciphertext: payload.password,
        },
    )
    .await
    .map_err(map_provider_credentials_error)
}

pub(crate) async fn delete_provider_credentials_for_user(
    state: &AppState,
    user_id: &str,
    provider: &str,
) -> Result<TrainProviderCredentialsDeleteResponse, TrainServiceError> {
    ensure_valid_user_id(user_id)?;
    let provider = normalize_train_provider(provider)?;
    let pool = require_pool(state)?;
    let result = sqlx::query(
        "update provider_auth_secrets
         set revoked_at = now(), updated_at = now()
         where provider = $1 and subject_ref = $2 and credential_kind = 'login' and revoked_at is null",
    )
    .bind(provider)
    .bind(user_id)
    .execute(pool)
    .await
    .map_err(|_| TrainServiceError::Internal)?;

    if result.rows_affected() == 0 {
        return Err(TrainServiceError::InvalidRequest(
            "provider credentials not found".to_string(),
        ));
    }

    Ok(TrainProviderCredentialsDeleteResponse {
        deleted: true,
        provider: provider.to_string(),
    })
}

pub(crate) async fn put_payment_method_for_user(
    state: &AppState,
    user_id: &str,
    _provider: &str,
    payload: PutTrainPaymentMethodRequest,
) -> Result<payment_method_service::PutProviderPaymentMethodResult, TrainServiceError> {
    ensure_valid_user_id(user_id)?;
    if payload.pan_ev.trim().is_empty()
        || payload.expiry_month_ev.trim().is_empty()
        || payload.expiry_year_ev.trim().is_empty()
        || payload.birth_or_business_ev.trim().is_empty()
        || payload.card_password_two_digits_ev.trim().is_empty()
        || payload.card_last4.trim().is_empty()
    {
        return Err(TrainServiceError::InvalidRequest(
            "all payment fields are required".to_string(),
        ));
    }
    let card_last4 = payload.card_last4.trim();
    if card_last4.len() != 4 || !card_last4.chars().all(|value| value.is_ascii_digit()) {
        return Err(TrainServiceError::InvalidRequest(
            "card_last4 must be exactly 4 digits".to_string(),
        ));
    }

    let pool = require_pool(state)?;
    enforce_active_card_limit(pool, user_id, payload.payment_method_ref.as_deref()).await?;

    payment_method_service::put_provider_payment_method(
        state,
        payment_method_service::UNIVERSAL_PAYMENT_PROVIDER,
        payment_method_service::PutProviderPaymentMethodRequest {
            owner_ref: Some(user_id.to_string()),
            payment_method_ref: payload.payment_method_ref,
            card_brand: payload.card_brand,
            card_last4: payload.card_last4,
            pan_ciphertext: payload.pan_ev,
            expiry_month_ciphertext: payload.expiry_month_ev,
            expiry_year_ciphertext: payload.expiry_year_ev,
            birth_or_business_number_ciphertext: payload.birth_or_business_ev,
            card_password_two_digits_ciphertext: payload.card_password_two_digits_ev,
        },
    )
    .await
    .map_err(map_payment_method_error)
}

pub(crate) async fn list_payment_methods_for_user(
    state: &AppState,
    user_id: &str,
) -> Result<TrainPaymentMethodListResponse, TrainServiceError> {
    ensure_valid_user_id(user_id)?;
    let pool = require_pool(state)?;
    let rows = sqlx::query(
        "select payment_method_ref, card_last4, card_brand, updated_at
         from (
            select distinct on (payment_method_ref)
                payment_method_ref, card_last4, card_brand, updated_at
            from payment_method_secrets
            where owner_ref = $1 and revoked_at is null
            order by payment_method_ref, updated_at desc
         ) latest
         order by updated_at desc
         limit 3",
    )
    .bind(user_id)
    .fetch_all(pool)
    .await
    .map_err(|_| TrainServiceError::Internal)?;

    let mut cards = Vec::with_capacity(rows.len());
    for row in rows {
        cards.push(TrainPaymentCardSummary {
            payment_method_ref: row
                .try_get("payment_method_ref")
                .map_err(|_| TrainServiceError::Internal)?,
            card_last4: row
                .try_get::<Option<String>, _>("card_last4")
                .map_err(|_| TrainServiceError::Internal)?
                .unwrap_or_else(|| "0000".to_string()),
            card_brand: row
                .try_get("card_brand")
                .map_err(|_| TrainServiceError::Internal)?,
            updated_at: row
                .try_get("updated_at")
                .map_err(|_| TrainServiceError::Internal)?,
        });
    }

    Ok(TrainPaymentMethodListResponse { cards })
}

pub(crate) async fn delete_payment_method_for_user(
    state: &AppState,
    user_id: &str,
    payment_method_ref: &str,
) -> Result<TrainPaymentMethodDeleteResponse, TrainServiceError> {
    ensure_valid_user_id(user_id)?;
    let payment_method_ref = payment_method_ref.trim();
    if payment_method_ref.is_empty() {
        return Err(TrainServiceError::InvalidRequest(
            "payment_method_ref is required".to_string(),
        ));
    }

    let pool = require_pool(state)?;
    let result = sqlx::query(
        "update payment_method_secrets
         set revoked_at = now(), updated_at = now()
         where owner_ref = $1 and payment_method_ref = $2 and revoked_at is null",
    )
    .bind(user_id)
    .bind(payment_method_ref)
    .execute(pool)
    .await
    .map_err(|_| TrainServiceError::Internal)?;

    if result.rows_affected() == 0 {
        return Err(TrainServiceError::InvalidRequest(
            "payment method not found".to_string(),
        ));
    }

    Ok(TrainPaymentMethodDeleteResponse {
        deleted: true,
        payment_method_ref: payment_method_ref.to_string(),
    })
}

async fn ensure_station_catalog_loaded(state: &AppState) -> Result<(), TrainServiceError> {
    let pool = require_pool(state)?;
    let snapshot_path =
        resolve_station_catalog_snapshot_path(&state.config.station_catalog_json_path);
    let (snapshot, snapshot_sha256) = station_catalog::load_snapshot_with_hash(&snapshot_path)
        .map_err(|err| {
            TrainServiceError::ServiceUnavailable(format!(
                "station catalog snapshot load failed ({}): {}",
                snapshot_path.display(),
                err
            ))
        })?;
    if snapshot.stations.is_empty() {
        return Err(TrainServiceError::ServiceUnavailable(
            "station catalog snapshot has no entries".to_string(),
        ));
    }

    if station_catalog_snapshot_applied(pool, &snapshot_sha256).await? {
        return Ok(());
    }

    let _guard = station_refresh_lock().lock().await;
    if station_catalog_snapshot_applied(pool, &snapshot_sha256).await? {
        return Ok(());
    }

    let mut tx = pool
        .begin()
        .await
        .map_err(|_| TrainServiceError::Internal)?;
    sqlx::query("delete from train_station_catalog")
        .execute(&mut *tx)
        .await
        .map_err(|_| TrainServiceError::Internal)?;

    for station in snapshot.stations {
        sqlx::query(
            "insert into train_station_catalog (provider, station_code, station_name_ko, station_name_en, line_code, order_index, selected, remark, normalized_name, normalized_remark, created_at, updated_at) values ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, now(), now())",
        )
        .bind(station.provider)
        .bind(station.station_code)
        .bind(station.station_name_ko)
        .bind(station.station_name_en)
        .bind(station.line_code)
        .bind(station.order_index)
        .bind(station.selected)
        .bind(station.remark)
        .bind(station.normalized_name)
        .bind(station.normalized_remark)
        .execute(&mut *tx)
        .await
        .map_err(|err| {
            error!(error = %err, "failed to insert station catalog row");
            TrainServiceError::Internal
        })?;
    }

    sqlx::query(
        "insert into train_station_catalog_state (id, snapshot_sha256, snapshot_version, applied_at, updated_at) values (1, $1, $2, now(), now())
        on conflict (id) do update set snapshot_sha256 = excluded.snapshot_sha256, snapshot_version = excluded.snapshot_version, applied_at = excluded.applied_at, updated_at = excluded.updated_at",
    )
    .bind(snapshot_sha256)
    .bind(station_catalog::STATION_CATALOG_SCHEMA_VERSION)
    .execute(&mut *tx)
    .await
    .map_err(|_| TrainServiceError::Internal)?;

    tx.commit().await.map_err(|_| TrainServiceError::Internal)?;
    invalidate_station_cache(state).await;
    Ok(())
}

async fn station_catalog_snapshot_applied(
    pool: &PgPool,
    snapshot_sha256: &str,
) -> Result<bool, TrainServiceError> {
    let row = sqlx::query(
        "select
            (select count(*)::bigint from train_station_catalog) as station_count,
            (select snapshot_sha256 from train_station_catalog_state where id = 1) as snapshot_sha256",
    )
    .fetch_one(pool)
    .await
    .map_err(|_| TrainServiceError::Internal)?;

    let station_count: i64 = row
        .try_get("station_count")
        .map_err(|_| TrainServiceError::Internal)?;
    if station_count == 0 {
        return Ok(false);
    }

    let applied_hash: Option<String> = row
        .try_get("snapshot_sha256")
        .map_err(|_| TrainServiceError::Internal)?;
    Ok(matches!(applied_hash, Some(value) if value == snapshot_sha256))
}

fn resolve_station_catalog_snapshot_path(configured_path: &str) -> std::path::PathBuf {
    let direct = Path::new(configured_path);
    if direct.exists() {
        return direct.to_path_buf();
    }

    if direct.is_absolute() {
        return direct.to_path_buf();
    }

    let cwd = std::env::current_dir().unwrap_or_else(|_| Path::new(".").to_path_buf());
    let candidates = [
        cwd.join(direct),
        cwd.join("..").join(direct),
        cwd.join("runtime").join(direct),
    ];
    for candidate in candidates {
        if candidate.exists() {
            return candidate;
        }
    }

    direct.to_path_buf()
}

fn station_match_score(
    station: &StationCatalogEntry,
    query_raw: &str,
    query_norm: &str,
) -> Option<usize> {
    if query_norm.is_empty() {
        return None;
    }

    let code = station.station_code.to_ascii_lowercase();
    let query_code = query_raw.trim().to_ascii_lowercase();
    let name_norm = &station.normalized_name;
    let remark_norm = station.normalized_remark.as_deref().unwrap_or("");
    let en_norm = station
        .station_name_en
        .as_deref()
        .map(station_catalog::normalize_search_text)
        .unwrap_or_default();

    let mut score: Option<usize> = None;

    if code == query_code {
        score = Some(0);
    }
    if code.starts_with(&query_code) {
        score = Some(score.map_or(5, |current| current.min(5)));
    }
    if station.station_name_ko.starts_with(query_raw) {
        score = Some(score.map_or(10, |current| current.min(10)));
    }
    if name_norm.starts_with(query_norm) {
        score = Some(score.map_or(15, |current| current.min(15)));
    }
    if !remark_norm.is_empty() && remark_norm.starts_with(query_norm) {
        score = Some(score.map_or(16, |current| current.min(16)));
    }
    if !en_norm.is_empty() && en_norm.starts_with(query_norm) {
        score = Some(score.map_or(18, |current| current.min(18)));
    }
    if name_norm.contains(query_norm) {
        score = Some(score.map_or(30, |current| current.min(30)));
    }
    if !en_norm.is_empty() && en_norm.contains(query_norm) {
        score = Some(score.map_or(34, |current| current.min(34)));
    }

    let distance = levenshtein(name_norm, query_norm);
    if distance <= 1 {
        score = Some(score.map_or(22, |current| current.min(22)));
    } else if distance == 2 {
        score = Some(score.map_or(40, |current| current.min(40)));
    }

    if !en_norm.is_empty() {
        let en_distance = levenshtein(&en_norm, query_norm);
        if en_distance <= 1 {
            score = Some(score.map_or(26, |current| current.min(26)));
        }
    }

    score
}

fn levenshtein(left: &str, right: &str) -> usize {
    if left == right {
        return 0;
    }
    if left.is_empty() {
        return right.chars().count();
    }
    if right.is_empty() {
        return left.chars().count();
    }

    let left_chars: Vec<char> = left.chars().collect();
    let right_chars: Vec<char> = right.chars().collect();

    let mut previous: Vec<usize> = (0..=right_chars.len()).collect();
    let mut current = vec![0usize; right_chars.len() + 1];

    for (left_idx, left_ch) in left_chars.iter().enumerate() {
        current[0] = left_idx + 1;

        for (right_idx, right_ch) in right_chars.iter().enumerate() {
            let insertion = current[right_idx] + 1;
            let deletion = previous[right_idx + 1] + 1;
            let substitution = previous[right_idx] + usize::from(left_ch != right_ch);
            current[right_idx + 1] = insertion.min(deletion).min(substitution);
        }

        previous.copy_from_slice(&current);
    }

    previous[right_chars.len()]
}

async fn load_station_catalog_for_provider(
    state: &AppState,
    provider: &str,
) -> Result<Vec<StationCatalogEntry>, TrainServiceError> {
    let cache_key = station_cache_key(provider);
    if let Some(redis_client) = state.redis_client.as_ref()
        && let Ok(mut conn) = redis_client.get_multiplexed_async_connection().await
    {
        let cached: redis::RedisResult<Option<String>> = conn.get(cache_key.as_str()).await;
        if let Ok(Some(payload)) = cached
            && let Ok(decoded) = serde_json::from_str::<Vec<StationCatalogEntry>>(&payload)
        {
            return Ok(decoded);
        }
    }

    let pool = require_pool(state)?;
    let rows = sqlx::query(
        "select provider, station_code, station_name_ko, station_name_en, coalesce(nullif(line_code::text, ''), '0')::int as line_code, selected, remark, order_index, normalized_name, normalized_remark from train_station_catalog where provider = $1 order by selected desc, order_index asc, station_name_ko asc",
    )
    .bind(provider)
    .fetch_all(pool)
    .await
    .map_err(|_| TrainServiceError::Internal)?;

    let mut stations = Vec::with_capacity(rows.len());
    for row in rows {
        let station_name_ko: String = row
            .try_get("station_name_ko")
            .map_err(|_| TrainServiceError::Internal)?;
        let station_name_en: Option<String> = row
            .try_get("station_name_en")
            .map_err(|_| TrainServiceError::Internal)?;
        let station_name_ja_katakana = station_catalog::derive_station_name_ja_katakana(
            &station_name_ko,
            station_name_en.as_deref(),
        );

        stations.push(StationCatalogEntry {
            provider: row
                .try_get("provider")
                .map_err(|_| TrainServiceError::Internal)?,
            station_code: row
                .try_get("station_code")
                .map_err(|_| TrainServiceError::Internal)?,
            station_name_ko,
            station_name_en,
            station_name_ja_katakana,
            line_code: row
                .try_get("line_code")
                .map_err(|_| TrainServiceError::Internal)?,
            selected: row
                .try_get("selected")
                .map_err(|_| TrainServiceError::Internal)?,
            remark: row
                .try_get("remark")
                .map_err(|_| TrainServiceError::Internal)?,
            order_index: row
                .try_get("order_index")
                .map_err(|_| TrainServiceError::Internal)?,
            normalized_name: row
                .try_get("normalized_name")
                .map_err(|_| TrainServiceError::Internal)?,
            normalized_remark: row
                .try_get("normalized_remark")
                .map_err(|_| TrainServiceError::Internal)?,
        });
    }

    if let Some(redis_client) = state.redis_client.as_ref()
        && let Ok(mut conn) = redis_client.get_multiplexed_async_connection().await
        && let Ok(encoded) = serde_json::to_string(&stations)
    {
        let set_result: redis::RedisResult<()> = conn
            .set_ex(cache_key.as_str(), encoded, STATION_CACHE_TTL_SECONDS)
            .await;
        if let Err(err) = set_result {
            warn!(error = %err, "failed to write station cache");
        }
    }

    Ok(stations)
}

fn station_cache_key(provider: &str) -> String {
    format!("train:station-catalog:{provider}:v1")
}

async fn invalidate_station_cache(state: &AppState) {
    if let Some(redis_client) = state.redis_client.as_ref()
        && let Ok(mut conn) = redis_client.get_multiplexed_async_connection().await
    {
        let keys = [station_cache_key("srt"), station_cache_key("ktx")];
        let delete_result: redis::RedisResult<usize> = conn.del(&keys).await;
        if let Err(err) = delete_result {
            warn!(error = %err, "failed to invalidate station cache keys");
        }
    }
}

const TRAIN_PASSENGER_LIMIT: u8 = 9;

fn passenger_kind_order(kind: TrainPassengerKind) -> usize {
    match kind {
        TrainPassengerKind::Adult => 0,
        TrainPassengerKind::Child => 1,
        TrainPassengerKind::Senior => 2,
        TrainPassengerKind::Disability1To3 => 3,
        TrainPassengerKind::Disability4To6 => 4,
    }
}

fn provider_passenger_kind(kind: TrainPassengerKind) -> &'static str {
    match kind {
        TrainPassengerKind::Adult => "adult",
        TrainPassengerKind::Child => "child",
        TrainPassengerKind::Senior => "senior",
        TrainPassengerKind::Disability1To3 => "disability1_to3",
        TrainPassengerKind::Disability4To6 => "disability4_to6",
    }
}

fn normalize_passengers(
    passengers: Option<Vec<TrainPassengerCount>>,
    passenger_count: Option<u8>,
) -> Result<Vec<TrainPassengerCount>, TrainServiceError> {
    let mut merged: HashMap<TrainPassengerKind, u16> = HashMap::new();
    let explicit = passengers.is_some();
    if let Some(items) = passengers {
        for item in items {
            if item.count == 0 {
                continue;
            }
            let next = merged.entry(item.kind).or_insert(0);
            *next = next.saturating_add(u16::from(item.count));
        }
    }

    if merged.is_empty() {
        if explicit {
            return Err(TrainServiceError::InvalidRequest(
                "at least one passenger is required".to_string(),
            ));
        }
        merged.insert(
            TrainPassengerKind::Adult,
            u16::from(passenger_count.unwrap_or(1).clamp(1, TRAIN_PASSENGER_LIMIT)),
        );
    }

    let total = merged.values().copied().sum::<u16>();
    if total == 0 {
        return Err(TrainServiceError::InvalidRequest(
            "at least one passenger is required".to_string(),
        ));
    }
    if total > u16::from(TRAIN_PASSENGER_LIMIT) {
        return Err(TrainServiceError::InvalidRequest(format!(
            "passenger total must be <= {TRAIN_PASSENGER_LIMIT}"
        )));
    }

    let mut normalized = merged
        .into_iter()
        .map(|(kind, count)| TrainPassengerCount {
            kind,
            count: u8::try_from(count).unwrap_or(TRAIN_PASSENGER_LIMIT),
        })
        .collect::<Vec<_>>();
    normalized.sort_by_key(|item| passenger_kind_order(item.kind));
    Ok(normalized)
}

fn parse_passengers_from_json(raw: &Value, fallback_count: i32) -> Vec<TrainPassengerCount> {
    if let Ok(mut parsed) = serde_json::from_value::<Vec<TrainPassengerCount>>(raw.clone()) {
        parsed.retain(|item| item.count > 0);
        parsed.sort_by_key(|item| passenger_kind_order(item.kind));
        if !parsed.is_empty() {
            return parsed;
        }
    }

    vec![TrainPassengerCount {
        kind: TrainPassengerKind::Adult,
        count: u8::try_from(fallback_count.clamp(1, i32::from(TRAIN_PASSENGER_LIMIT))).unwrap_or(1),
    }]
}

fn ordered_provider_labels(raw: &[String]) -> Vec<String> {
    let mut labels = raw
        .iter()
        .map(|value| value.trim().to_ascii_uppercase())
        .filter(|value| !value.is_empty())
        .collect::<Vec<_>>();
    labels.sort_by(|left, right| {
        let rank = |value: &str| match value {
            "KTX" => 0,
            "SRT" => 1,
            _ => 2,
        };
        rank(left).cmp(&rank(right)).then_with(|| left.cmp(right))
    });
    labels.dedup();
    labels
}

fn curated_region_order() -> &'static [(&'static str, &'static str)] {
    &[
        ("major", "주요역"),
        ("seoul", "서울"),
        ("gyeonggi", "경기"),
        ("gangwon", "강원"),
        ("chungbuk", "충북"),
        ("chungnam", "충남"),
        ("jeonbuk", "전북"),
        ("jeonnam", "전남"),
        ("gyeongbuk", "경북"),
        ("gyeongnam", "경남"),
        ("metropolitan", "광역시"),
        ("all", "전체"),
    ]
}

const CURATED_REGION_STATIONS_SEOUL: &[&str] = &[
    "서울",
    "상봉",
    "수서",
    "영등포",
    "옥수",
    "왕십리",
    "용산",
    "청량리",
];

const CURATED_REGION_STATIONS_GYEONGGI: &[&str] = &[
    "가평",
    "광명",
    "대곡",
    "덕소",
    "도라산",
    "동탄",
    "마석",
    "매곡",
    "문산",
    "사릉",
    "삼산",
    "서정리",
    "서화성",
    "석불",
    "송추",
    "수원",
    "안양",
    "안중",
    "양동",
    "양평",
    "오산",
    "용문",
    "운천",
    "원릉",
    "의정부",
    "일신",
    "일영",
    "임진강",
    "장흥",
    "지평",
    "청평",
    "퇴계원",
    "판교(경기)",
    "평내호평",
    "평택",
    "평택지제",
    "평해",
    "행신",
    "화성시청",
];

const CURATED_REGION_STATIONS_GANGWON: &[&str] = &[
    "강릉",
    "강촌",
    "고한",
    "근덕",
    "나전",
    "남창",
    "도계",
    "동백산",
    "동해",
    "둔내",
    "만종",
    "묵호",
    "민둥산",
    "백양리",
    "별어곡",
    "사북",
    "삼척",
    "삼척해변",
    "서원주",
    "선평",
    "신기",
    "쌍룡",
    "아우라지",
    "영월",
    "예미",
    "옥원",
    "원주",
    "임원",
    "정동진",
    "정선",
    "진부(오대산)",
    "철암",
    "추암",
    "춘천",
    "태백",
    "평창",
    "횡성",
];

const CURATED_REGION_STATIONS_CHUNGBUK: &[&str] = &[
    "각계",
    "단양",
    "봉양",
    "부강",
    "삼탄",
    "심천",
    "앙성온천",
    "영동",
    "오근장",
    "오송",
    "옥천",
    "음성",
    "이원",
    "제천",
    "주덕",
    "증평",
    "지탄",
    "청주",
    "청주공항",
    "추풍령",
    "충주",
    "황간",
];

const CURATED_REGION_STATIONS_CHUNGNAM: &[&str] = &[
    "강경",
    "계룡",
    "공주",
    "광천",
    "논산",
    "대천",
    "도고온천",
    "삽교",
    "서천",
    "성환",
    "신례원",
    "신창",
    "아산",
    "연산",
    "연풍",
    "예산",
    "온양온천",
    "웅천",
    "인주",
    "장항",
    "전의",
    "조치원",
    "천안",
    "천안아산",
    "청소",
    "판교(충남)",
    "합덕",
    "홍성",
];

const CURATED_REGION_STATIONS_JEONBUK: &[&str] = &[
    "군산",
    "김제",
    "남원",
    "대야",
    "삼례",
    "신태인",
    "오수",
    "익산",
    "임실",
    "전주",
    "정읍",
    "함열",
];

const CURATED_REGION_STATIONS_JEONNAM: &[&str] = &[
    "강진",
    "곡성",
    "광양",
    "구례구",
    "나주",
    "능주",
    "다시",
    "득량",
    "명봉",
    "목포",
    "몽탄",
    "무안",
    "백양사",
    "벌교",
    "보성",
    "신보성",
    "순천",
    "여수EXPO",
    "여천",
    "영암",
    "예당",
    "이양",
    "일로",
    "임성리",
    "장성",
    "장동",
    "전남장흥",
    "조성",
    "진상",
    "함평",
    "해남",
    "화순",
];

const CURATED_REGION_STATIONS_GYEONGBUK: &[&str] = &[
    "강구",
    "개포",
    "경산",
    "경주",
    "고래불",
    "구미",
    "군위",
    "기성",
    "김천",
    "김천구미",
    "남성현",
    "매화",
    "봉화",
    "북영천",
    "분천",
    "비동",
    "상주",
    "서경주",
    "석포",
    "승부",
    "신동",
    "아화",
    "안강",
    "안동",
    "약목",
    "양원",
    "영덕",
    "영주",
    "영천",
    "영해",
    "예천",
    "옥산",
    "왜관",
    "용궁",
    "울진",
    "월포",
    "의성",
    "임기",
    "장사",
    "점촌",
    "죽변",
    "청도",
    "청리",
    "춘양",
    "포항",
    "풍기",
    "하양",
    "함창",
    "현동",
    "후포",
    "흥부",
];

const CURATED_REGION_STATIONS_GYEONGNAM: &[&str] = &[
    "군북",
    "마산",
    "물금",
    "밀양",
    "반성",
    "북천",
    "삼랑진",
    "상동",
    "완사",
    "원동",
    "중리",
    "진례",
    "진영",
    "진주",
    "창원",
    "창원중앙",
    "하동",
    "한림정",
    "함안",
    "횡천",
];

const CURATED_REGION_STATIONS_METROPOLITAN: &[&str] = &[
    "광주",
    "광주송정",
    "구포",
    "극락강",
    "기장",
    "대구",
    "대전",
    "동대구",
    "부산",
    "부전",
    "사상",
    "서광주",
    "서대구",
    "서대전",
    "센텀",
    "신탄진",
    "신해운대",
    "울산(통도사)",
    "인천공항T1",
    "인천공항T2",
    "태화강",
    "화명",
    "효천",
];

const CURATED_REGION_STATION_TABLE: &[(&str, &[&str])] = &[
    ("seoul", CURATED_REGION_STATIONS_SEOUL),
    ("gyeonggi", CURATED_REGION_STATIONS_GYEONGGI),
    ("gangwon", CURATED_REGION_STATIONS_GANGWON),
    ("chungbuk", CURATED_REGION_STATIONS_CHUNGBUK),
    ("chungnam", CURATED_REGION_STATIONS_CHUNGNAM),
    ("jeonbuk", CURATED_REGION_STATIONS_JEONBUK),
    ("jeonnam", CURATED_REGION_STATIONS_JEONNAM),
    ("gyeongbuk", CURATED_REGION_STATIONS_GYEONGBUK),
    ("gyeongnam", CURATED_REGION_STATIONS_GYEONGNAM),
    ("metropolitan", CURATED_REGION_STATIONS_METROPOLITAN),
];

static CURATED_REGION_STATION_LOOKUP: OnceLock<HashMap<&'static str, &'static str>> =
    OnceLock::new();

fn curated_region_station_lookup() -> &'static HashMap<&'static str, &'static str> {
    CURATED_REGION_STATION_LOOKUP.get_or_init(|| {
        let mut lookup: HashMap<&'static str, &'static str> = HashMap::new();
        for &(region_key, stations) in CURATED_REGION_STATION_TABLE {
            for &station_name in stations {
                if let Some(existing_region) = lookup.insert(station_name, region_key) {
                    warn!(
                        station_name = station_name,
                        existing_region = existing_region,
                        next_region = region_key,
                        "duplicate station in curated region table"
                    );
                }
            }
        }
        lookup
    })
}

fn curated_region_for_station(station_name_ko: &str) -> &'static str {
    let normalized = station_name_ko
        .trim()
        .strip_suffix('역')
        .unwrap_or(station_name_ko.trim())
        .trim();
    curated_region_station_lookup()
        .get(normalized)
        .copied()
        .unwrap_or("all")
}

fn parse_provider_scope(raw: Option<&str>) -> Result<Vec<&'static str>, TrainServiceError> {
    match raw.map(|value| value.trim().to_ascii_lowercase()) {
        None => Ok(vec!["srt", "ktx"]),
        Some(value) if value.is_empty() || value == "all" => Ok(vec!["srt", "ktx"]),
        Some(value) if value == "srt" => Ok(vec!["srt"]),
        Some(value) if value == "ktx" => Ok(vec!["ktx"]),
        Some(_) => Err(TrainServiceError::InvalidRequest(
            "provider must be one of: srt, ktx, all".to_string(),
        )),
    }
}

fn normalize_train_provider(raw: &str) -> Result<&'static str, TrainServiceError> {
    match raw.trim().to_ascii_lowercase().as_str() {
        "srt" => Ok("srt"),
        "ktx" => Ok("ktx"),
        _ => Err(TrainServiceError::InvalidRequest(
            "provider must be one of: srt, ktx".to_string(),
        )),
    }
}

fn normalize_station_code(raw: &str) -> Result<String, TrainServiceError> {
    let code = raw.trim().to_ascii_uppercase();
    if code.is_empty() {
        return Err(TrainServiceError::InvalidRequest(
            "station code is required".to_string(),
        ));
    }

    if !code.chars().all(|ch| ch.is_ascii_alphanumeric()) {
        return Err(TrainServiceError::InvalidRequest(
            "station code must be alphanumeric".to_string(),
        ));
    }

    Ok(code)
}

fn normalize_dep_date(raw: Option<&str>) -> Result<String, TrainServiceError> {
    let value = raw
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToOwned::to_owned)
        .unwrap_or_else(|| Utc::now().format("%Y%m%d").to_string());

    if value.len() != 8 || !value.chars().all(|ch| ch.is_ascii_digit()) {
        return Err(TrainServiceError::InvalidRequest(
            "dep_date must be YYYYMMDD".to_string(),
        ));
    }

    if NaiveDate::parse_from_str(&value, "%Y%m%d").is_err() {
        return Err(TrainServiceError::InvalidRequest(
            "dep_date must be a valid date".to_string(),
        ));
    }

    Ok(value)
}

fn normalize_dep_time(raw: Option<&str>) -> Result<String, TrainServiceError> {
    let value = raw
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToOwned::to_owned)
        .unwrap_or_else(|| "000000".to_string());

    if value.len() != 6 || !value.chars().all(|ch| ch.is_ascii_digit()) {
        return Err(TrainServiceError::InvalidRequest(
            "dep_time must be HHMMSS".to_string(),
        ));
    }

    Ok(value)
}

fn ensure_valid_user_id(user_id: &str) -> Result<(), TrainServiceError> {
    if user_id.trim().is_empty() {
        return Err(TrainServiceError::Unauthorized(
            "session user is required".to_string(),
        ));
    }

    Ok(())
}

fn require_pool(state: &AppState) -> Result<&PgPool, TrainServiceError> {
    state
        .db_pool
        .as_ref()
        .ok_or_else(|| TrainServiceError::ServiceUnavailable("database unavailable".to_string()))
}

async fn provider_preflight(
    pool: &PgPool,
    provider: &str,
    user_id: &str,
) -> Result<TrainProviderPreflight, TrainServiceError> {
    let credentials_ready = credentials_ready(pool, provider, user_id).await?;
    let payment_method_ref = latest_payment_method_ref(pool, provider, user_id).await?;
    let payment_ready = payment_method_ref.is_some();

    Ok(TrainProviderPreflight {
        provider: provider.to_string(),
        credentials_ready,
        payment_ready,
        payment_method_ref,
    })
}

async fn credentials_ready(
    pool: &PgPool,
    provider: &str,
    user_id: &str,
) -> Result<bool, TrainServiceError> {
    let exists = sqlx::query_scalar::<_, i32>(
        "select 1 from provider_auth_secrets where provider = $1 and subject_ref = $2 and credential_kind = 'login' and revoked_at is null limit 1",
    )
    .bind(provider)
    .bind(user_id)
    .fetch_optional(pool)
    .await
    .map_err(|err| {
        error!(
            error = %err,
            provider = %provider,
            "train preflight credentials_ready query failed"
        );
        TrainServiceError::Internal
    })?
    .is_some();

    Ok(exists)
}

async fn latest_payment_method_ref(
    pool: &PgPool,
    _provider: &str,
    user_id: &str,
) -> Result<Option<String>, TrainServiceError> {
    let value = sqlx::query_scalar::<_, String>(
        "select payment_method_ref from payment_method_secrets where owner_ref = $1 and revoked_at is null order by updated_at desc limit 1",
    )
    .bind(user_id)
    .fetch_optional(pool)
    .await
    .map_err(|err| {
        error!(
            error = %err,
            "train preflight payment_method_ref query failed"
        );
        TrainServiceError::Internal
    })?;

    Ok(value)
}

async fn enforce_active_card_limit(
    pool: &PgPool,
    user_id: &str,
    payment_method_ref: Option<&str>,
) -> Result<(), TrainServiceError> {
    let provided_ref = payment_method_ref
        .map(str::trim)
        .filter(|value| !value.is_empty());
    if let Some(value) = provided_ref {
        let exists = sqlx::query_scalar::<_, i32>(
            "select 1 from payment_method_secrets where owner_ref = $1 and payment_method_ref = $2 and revoked_at is null limit 1",
        )
        .bind(user_id)
        .bind(value)
        .fetch_optional(pool)
        .await
        .map_err(|_| TrainServiceError::Internal)?
        .is_some();
        if exists {
            return Ok(());
        }
    }

    let active_count = sqlx::query_scalar::<_, i64>(
        "select count(distinct payment_method_ref)::bigint
         from payment_method_secrets
         where owner_ref = $1 and revoked_at is null",
    )
    .bind(user_id)
    .fetch_one(pool)
    .await
    .map_err(|_| TrainServiceError::Internal)?;

    if active_count >= 3 {
        return Err(TrainServiceError::InvalidRequest(
            "a maximum of 3 saved cards is allowed".to_string(),
        ));
    }

    Ok(())
}

async fn resolve_ready_providers(
    pool: &PgPool,
    user_id: &str,
) -> Result<Vec<&'static str>, TrainServiceError> {
    let mut providers = Vec::new();
    if credentials_ready(pool, "srt", user_id).await? {
        providers.push("srt");
    }
    if credentials_ready(pool, "ktx", user_id).await? {
        providers.push("ktx");
    }
    Ok(providers)
}

async fn station_pair_supported_for_provider(
    pool: &PgPool,
    provider: &str,
    dep_station_code: &str,
    arr_station_code: &str,
) -> Result<bool, TrainServiceError> {
    let count = sqlx::query_scalar::<_, i64>(
        "select count(*)::bigint from train_station_catalog where provider = $1 and station_code in ($2, $3)",
    )
    .bind(provider)
    .bind(dep_station_code)
    .bind(arr_station_code)
    .fetch_one(pool)
    .await
    .map_err(|_| TrainServiceError::Internal)?;

    Ok(count == 2)
}

async fn load_search_session(
    pool: &PgPool,
    user_id: &str,
    search_id: &str,
) -> Result<Option<SearchSessionRow>, TrainServiceError> {
    let row = sqlx::query(
        "select search_id, dep_station_code, arr_station_code, dep_date, dep_time, passengers_json, passenger_count, available_only, status, created_at, updated_at, completed_at from train_search_sessions where search_id = $1 and user_id = $2 limit 1",
    )
    .bind(search_id)
    .bind(user_id)
    .fetch_optional(pool)
    .await
    .map_err(|_| TrainServiceError::Internal)?;

    let Some(row) = row else {
        return Ok(None);
    };

    Ok(Some(SearchSessionRow {
        search_id: row
            .try_get("search_id")
            .map_err(|_| TrainServiceError::Internal)?,
        dep_station_code: row
            .try_get("dep_station_code")
            .map_err(|_| TrainServiceError::Internal)?,
        arr_station_code: row
            .try_get("arr_station_code")
            .map_err(|_| TrainServiceError::Internal)?,
        dep_date: row
            .try_get("dep_date")
            .map_err(|_| TrainServiceError::Internal)?,
        dep_time: row
            .try_get("dep_time")
            .map_err(|_| TrainServiceError::Internal)?,
        passengers_json: row
            .try_get("passengers_json")
            .map_err(|_| TrainServiceError::Internal)?,
        passenger_count: row
            .try_get("passenger_count")
            .map_err(|_| TrainServiceError::Internal)?,
        available_only: row
            .try_get("available_only")
            .map_err(|_| TrainServiceError::Internal)?,
        created_at: row
            .try_get("created_at")
            .map_err(|_| TrainServiceError::Internal)?,
        completed_at: row
            .try_get("completed_at")
            .map_err(|_| TrainServiceError::Internal)?,
    }))
}

async fn load_search_jobs(
    pool: &PgPool,
    search_id: &str,
) -> Result<Vec<TrainProviderJobStatus>, TrainServiceError> {
    let rows = sqlx::query(
        "select j.provider, j.runtime_job_id, coalesce(r.status, j.status) as status, coalesce(j.error_message, r.last_error) as last_error from train_search_session_jobs j left join runtime_jobs r on r.job_id = j.runtime_job_id where j.search_id = $1 order by j.provider asc",
    )
    .bind(search_id)
    .fetch_all(pool)
    .await
    .map_err(|_| TrainServiceError::Internal)?;

    let mut jobs = Vec::with_capacity(rows.len());
    for row in rows {
        jobs.push(TrainProviderJobStatus {
            provider: row
                .try_get("provider")
                .map_err(|_| TrainServiceError::Internal)?,
            runtime_job_id: row
                .try_get("runtime_job_id")
                .map_err(|_| TrainServiceError::Internal)?,
            status: row
                .try_get("status")
                .map_err(|_| TrainServiceError::Internal)?,
            error_message: row
                .try_get("last_error")
                .map_err(|_| TrainServiceError::Internal)?,
        });
    }

    Ok(jobs)
}

fn extract_train_results_from_events(
    provider: &str,
    runtime_job_id: &str,
    events: &[sqlx::postgres::PgRow],
) -> Vec<TrainSearchResult> {
    let mut collected = Vec::new();
    for row in events {
        let Ok(event_type) = row.try_get::<String, _>("event_type") else {
            continue;
        };
        if event_type != "completed" {
            continue;
        }

        let Ok(payload) = row.try_get::<Value, _>("event_payload") else {
            continue;
        };
        let Some(response) = payload.pointer("/result/response") else {
            continue;
        };

        let operation = response
            .get("operation")
            .and_then(Value::as_str)
            .unwrap_or_default();
        if operation != "search_train" {
            continue;
        }

        let Some(trains) = response
            .pointer("/payload/trains")
            .and_then(Value::as_array)
        else {
            continue;
        };

        for train in trains {
            let Some(train_number) = train.get("train_number").and_then(Value::as_str) else {
                continue;
            };
            collected.push(TrainSearchResult {
                provider: provider.to_string(),
                runtime_job_id: runtime_job_id.to_string(),
                train_code: value_as_string(train, "train_code"),
                train_number: train_number.to_string(),
                dep_station_code: value_as_string(train, "dep_station_code"),
                arr_station_code: value_as_string(train, "arr_station_code"),
                dep_date: value_as_string(train, "dep_date"),
                dep_time: value_as_string(train, "dep_time"),
                arr_date: value_as_string(train, "arr_date"),
                arr_time: value_as_string(train, "arr_time"),
                general_seat_available: train
                    .get("general_seat_available")
                    .and_then(Value::as_bool)
                    .unwrap_or(false),
                special_seat_available: train
                    .get("special_seat_available")
                    .and_then(Value::as_bool)
                    .unwrap_or(false),
                standby_available: train
                    .get("standby_available")
                    .and_then(Value::as_bool)
                    .unwrap_or(false),
            });
        }

        break;
    }

    collected
}

fn extract_error_message_from_events(events: &[sqlx::postgres::PgRow]) -> Option<String> {
    for row in events {
        let Ok(event_type) = row.try_get::<String, _>("event_type") else {
            continue;
        };
        if event_type != "failed" && event_type != "dead_lettered" {
            continue;
        }

        let Ok(payload) = row.try_get::<Value, _>("event_payload") else {
            continue;
        };

        let message = payload
            .get("message")
            .and_then(Value::as_str)
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(ToOwned::to_owned);
        if message.is_some() {
            return message;
        }

        if let Some(reason) = extract_error_reason(&payload)
            && let Some(message) = map_error_reason_to_message(reason)
        {
            return Some(message.to_string());
        }

        let fallback = payload
            .get("state")
            .and_then(Value::as_str)
            .map(|value| format!("provider job {value}"));
        if fallback.is_some() {
            return fallback;
        }
    }

    None
}

fn value_as_string(value: &Value, key: &str) -> String {
    value
        .get(key)
        .and_then(Value::as_str)
        .unwrap_or_default()
        .to_string()
}

fn extract_error_reason(payload: &Value) -> Option<&str> {
    payload
        .get("error_reason")
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .or_else(|| {
            payload
                .pointer("/context/class")
                .and_then(Value::as_str)
                .map(str::trim)
                .filter(|value| !value.is_empty())
        })
}

fn map_error_reason_to_message(reason: &str) -> Option<&'static str> {
    match reason {
        "missing_subject_ref"
        | "missing_session"
        | "auth_secret_missing"
        | "auth_secret_decode"
        | "auth_payload_decode"
        | "auth_aad_hash"
        | "auth_decrypt" => Some("provider credentials are missing or invalid"),
        "operation_failed" => Some("provider rejected request"),
        "unsupported_operation" => Some("provider operation is not supported"),
        "rate_limited" => Some("provider rate limited"),
        _ => None,
    }
}

fn is_failure_status(status: &str) -> bool {
    matches!(status, "failed" | "dead_lettered")
}

fn is_terminal_status(status: &str) -> bool {
    matches!(status, "completed" | "partial" | "failed")
}

fn aggregate_status<'a>(statuses: impl Iterator<Item = &'a str>) -> &'static str {
    let collected: Vec<&str> = statuses.collect();
    if collected.is_empty() {
        return "failed";
    }

    let all_completed = collected.iter().all(|status| *status == "completed");
    if all_completed {
        return "completed";
    }

    let any_completed = collected.iter().any(|status| *status == "completed");
    let any_running = collected
        .iter()
        .any(|status| matches!(*status, "running" | "queued"));
    let all_failed = collected
        .iter()
        .all(|status| matches!(*status, "failed" | "dead_lettered"));

    if all_failed {
        return "failed";
    }
    if any_running {
        return "running";
    }
    if any_completed {
        return "partial";
    }

    "queued"
}

fn map_provider_credentials_error(
    error: provider_credentials_service::PutProviderCredentialsError,
) -> TrainServiceError {
    match error {
        provider_credentials_service::PutProviderCredentialsError::ValidationFailed => {
            TrainServiceError::InvalidRequest("invalid provider credentials payload".to_string())
        }
        provider_credentials_service::PutProviderCredentialsError::PersistenceUnavailable
        | provider_credentials_service::PutProviderCredentialsError::CryptoUnavailable => {
            TrainServiceError::ServiceUnavailable(
                "provider credentials service unavailable".to_string(),
            )
        }
        provider_credentials_service::PutProviderCredentialsError::PersistenceFailure => {
            TrainServiceError::Internal
        }
    }
}

fn map_payment_method_error(
    error: payment_method_service::PutProviderPaymentMethodError,
) -> TrainServiceError {
    match error {
        payment_method_service::PutProviderPaymentMethodError::ValidationFailed => {
            TrainServiceError::InvalidRequest("invalid payment payload".to_string())
        }
        payment_method_service::PutProviderPaymentMethodError::PersistenceUnavailable
        | payment_method_service::PutProviderPaymentMethodError::CryptoUnavailable => {
            TrainServiceError::ServiceUnavailable("payment method service unavailable".to_string())
        }
        payment_method_service::PutProviderPaymentMethodError::PersistenceFailure => {
            TrainServiceError::Internal
        }
    }
}

fn map_provider_jobs_error(error: provider_jobs_service::ProviderJobsError) -> TrainServiceError {
    match error {
        provider_jobs_service::ProviderJobsError::ValidationFailed => {
            TrainServiceError::InvalidRequest("invalid provider job payload".to_string())
        }
        provider_jobs_service::ProviderJobsError::PersistenceUnavailable => {
            TrainServiceError::ServiceUnavailable(
                "provider job persistence unavailable".to_string(),
            )
        }
        provider_jobs_service::ProviderJobsError::DuplicateConflict => {
            TrainServiceError::InvalidRequest("provider job idempotency conflict".to_string())
        }
        provider_jobs_service::ProviderJobsError::NotFound => {
            TrainServiceError::NotFound("provider job not found".to_string())
        }
        provider_jobs_service::ProviderJobsError::PersistenceFailure => TrainServiceError::Internal,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn station_suggest_query_supports_extended_hints() {
        let parsed: StationSuggestQuery = serde_json::from_value(serde_json::json!({
            "q": "tntj",
            "provider": "srt",
            "limit": 8,
            "layout_hint": "qwerty",
            "lang_hint": "ko",
            "apply_mode": "submit"
        }))
        .unwrap_or_else(|err| panic!("failed to parse station suggest query: {err}"));

        assert_eq!(parsed.q, "tntj");
        assert_eq!(parsed.provider.as_deref(), Some("srt"));
        assert_eq!(parsed.limit, Some(8));
        assert_eq!(parsed.layout_hint.as_deref(), Some("qwerty"));
        assert_eq!(parsed.lang_hint.as_deref(), Some("ko"));
        assert_eq!(parsed.apply_mode.as_deref(), Some("submit"));
    }

    #[test]
    fn station_suggest_response_exposes_autocorrect_metadata() {
        let response = StationSuggestResponse {
            query: "tjdnf".to_string(),
            corrected_query: Some("서울".to_string()),
            autocorrect_applied: true,
            suggestions: vec![StationSuggestion {
                provider: "srt".to_string(),
                station_code: "0551".to_string(),
                station_name_ko: "수서".to_string(),
                station_name_en: Some("suseo".to_string()),
                station_name_ja_katakana: "スソ".to_string(),
                line_code: 0,
                selected: true,
                order_index: 1,
                match_source: Some("keyboard_layout".to_string()),
                confidence: Some(0.95),
            }],
        };

        assert_eq!(response.corrected_query.as_deref(), Some("서울"));
        assert!(response.autocorrect_applied);
        assert_eq!(
            response.suggestions[0].match_source.as_deref(),
            Some("keyboard_layout")
        );
        assert_eq!(response.suggestions[0].confidence, Some(0.95));
    }

    #[test]
    fn curated_region_table_maps_expected_stations() {
        assert_eq!(curated_region_for_station("서울"), "seoul");
        assert_eq!(curated_region_for_station("평택지제"), "gyeonggi");
        assert_eq!(curated_region_for_station("동탄역"), "gyeonggi");
        assert_eq!(curated_region_for_station("평택지제역"), "gyeonggi");
        assert_eq!(curated_region_for_station("진부(오대산)"), "gangwon");
        assert_eq!(curated_region_for_station("청주공항"), "chungbuk");
        assert_eq!(curated_region_for_station("천안아산"), "chungnam");
        assert_eq!(curated_region_for_station("함열"), "jeonbuk");
        assert_eq!(curated_region_for_station("전남장흥"), "jeonnam");
        assert_eq!(curated_region_for_station("김천구미"), "gyeongbuk");
        assert_eq!(curated_region_for_station("창원중앙"), "gyeongnam");
        assert_eq!(curated_region_for_station("인천공항T2"), "metropolitan");
        assert_eq!(curated_region_for_station("없는역"), "all");
    }

    #[test]
    fn curated_region_table_has_no_duplicate_station_names() {
        let mut seen = std::collections::HashSet::new();
        for &(_, stations) in CURATED_REGION_STATION_TABLE {
            for &station_name in stations {
                assert!(
                    seen.insert(station_name),
                    "station appears in multiple regions: {station_name}"
                );
            }
        }
    }

    #[test]
    fn station_match_score_prefers_exact_code_match() {
        let station = StationCatalogEntry {
            provider: "srt".to_string(),
            station_code: "0551".to_string(),
            station_name_ko: "수서".to_string(),
            station_name_en: Some("suseo".to_string()),
            station_name_ja_katakana: "スソ".to_string(),
            line_code: 0,
            selected: true,
            remark: Some("ㅅ".to_string()),
            order_index: 0,
            normalized_name: "수서".to_string(),
            normalized_remark: Some("ㅅ".to_string()),
        };

        let score = station_match_score(&station, "0551", "0551");
        assert_eq!(score, Some(0));
    }

    #[test]
    fn levenshtein_handles_small_edit_distance() {
        assert_eq!(levenshtein("seoul", "seol"), 1);
        assert_eq!(levenshtein("busan", "busan"), 0);
    }

    #[test]
    fn aggregate_status_prefers_running_when_in_progress() {
        let status = aggregate_status(vec!["queued", "completed"].into_iter());
        assert_eq!(status, "running");
    }

    #[test]
    fn normalize_passengers_uses_explicit_categories() {
        let passengers = normalize_passengers(
            Some(vec![
                TrainPassengerCount {
                    kind: TrainPassengerKind::Adult,
                    count: 1,
                },
                TrainPassengerCount {
                    kind: TrainPassengerKind::Child,
                    count: 2,
                },
            ]),
            Some(9),
        )
        .unwrap_or_else(|err| panic!("unexpected err: {err:?}"));

        assert_eq!(passengers.len(), 2);
        assert_eq!(passengers[0].kind, TrainPassengerKind::Adult);
        assert_eq!(passengers[1].kind, TrainPassengerKind::Child);
    }

    #[test]
    fn normalize_passengers_rejects_zero_total() {
        let result = normalize_passengers(
            Some(vec![TrainPassengerCount {
                kind: TrainPassengerKind::Adult,
                count: 0,
            }]),
            None,
        );
        assert!(matches!(result, Err(TrainServiceError::InvalidRequest(_))));
    }

    #[test]
    fn extract_error_reason_prefers_event_error_reason_field() {
        let payload = serde_json::json!({
            "error_reason": "missing_session",
            "context": { "class": "operation_failed" }
        });

        assert_eq!(extract_error_reason(&payload), Some("missing_session"));
    }

    #[test]
    fn map_error_reason_to_message_maps_known_dead_letter_classes() {
        assert_eq!(
            map_error_reason_to_message("missing_session"),
            Some("provider credentials are missing or invalid")
        );
        assert_eq!(
            map_error_reason_to_message("operation_failed"),
            Some("provider rejected request")
        );
        assert_eq!(
            map_error_reason_to_message("unsupported_operation"),
            Some("provider operation is not supported")
        );
        assert_eq!(
            map_error_reason_to_message("rate_limited"),
            Some("provider rate limited")
        );
        assert_eq!(map_error_reason_to_message("fatal"), None);
    }
}
