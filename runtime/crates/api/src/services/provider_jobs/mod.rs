use chrono::{DateTime, Utc};

mod commands;
mod mapping;
mod queries;
mod state;

#[derive(Debug, serde::Deserialize)]
pub(crate) struct CreateProviderJobRequest {
    pub(crate) provider: String,
    pub(crate) operation: String,
    pub(crate) idempotency_key: Option<String>,
    pub(crate) payload: serde_json::Value,
}

#[derive(Debug, serde::Serialize)]
pub(crate) struct CreateProviderJobResult {
    pub(crate) accepted: bool,
    pub(crate) job_id: String,
    pub(crate) status: String,
}

#[derive(Debug, serde::Serialize)]
pub(crate) struct ProviderJobResult {
    pub(crate) job_id: String,
    pub(crate) provider: String,
    pub(crate) operation: String,
    pub(crate) status: String,
}

#[derive(Debug, Clone, serde::Serialize)]
pub(crate) struct ProviderJobEvent {
    pub(crate) sequence: i64,
    pub(crate) event_type: String,
    pub(crate) occurred_at: DateTime<Utc>,
    pub(crate) detail: serde_json::Value,
}

#[derive(Debug, Clone, serde::Serialize)]
pub(crate) struct ProviderJobEventsPage {
    pub(crate) items: Vec<ProviderJobEvent>,
    pub(crate) has_more: bool,
    pub(crate) next_after_id: Option<i64>,
}

#[derive(Debug)]
pub(crate) enum ProviderJobsError {
    ValidationFailed,
    PersistenceUnavailable,
    DuplicateConflict,
    NotFound,
    PersistenceFailure,
}

pub(crate) use commands::create_provider_job;
pub(crate) use queries::{get_provider_job, list_provider_job_events_page};
