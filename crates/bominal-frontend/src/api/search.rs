//! Search server functions.

use leptos::prelude::*;

pub use bominal_service::search::{StationInfo, TrainInfo};

/// Get station list for a provider.
#[server(prefix = "/sfn")]
pub async fn list_stations(provider: String) -> Result<Vec<StationInfo>, ServerFnError> {
    bominal_service::search::list_stations(&provider)
        .map_err(|e| ServerFnError::new(e.to_string()))
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
