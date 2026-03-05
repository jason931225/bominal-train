use chrono::{DateTime, Utc};
use serde::Serialize;

mod audit;
mod capabilities;
mod config;
mod incidents;
mod maintenance;
mod observability;
mod runtime;
mod security;
mod users;

pub(crate) use audit::{AppendAdminAuditInput, append_admin_audit, list_admin_audit};
pub(crate) use capabilities::{role_allows_admin_mutation, role_allows_admin_read};
pub(crate) use config::redacted_config_snapshot;
pub(crate) use incidents::{create_incident, list_incidents, update_incident_status};
pub(crate) use maintenance::{list_kill_switches, upsert_kill_switch};
pub(crate) use observability::{list_observability_events, list_observability_timeseries};
pub(crate) use runtime::{
    cancel_runtime_job, ensure_runtime_job_exists, get_runtime_job, list_runtime_job_events_page,
    list_runtime_jobs, requeue_runtime_job, retry_runtime_job,
};
pub(crate) use security::validate_sensitive_confirmation;
pub(crate) use users::{list_sessions, list_users, set_user_access, set_user_role};

#[derive(Debug)]
pub(crate) enum AdminServiceError {
    InvalidRequest(&'static str),
    NotFound(&'static str),
    ServiceUnavailable(&'static str),
    Internal,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct AdminUserRecord {
    pub(crate) user_id: String,
    pub(crate) email: String,
    pub(crate) status: String,
    pub(crate) role: String,
    pub(crate) access_enabled: bool,
    pub(crate) updated_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct SessionRecord {
    pub(crate) session_id_hash: String,
    pub(crate) user_id: Option<String>,
    pub(crate) email: String,
    pub(crate) role: String,
    pub(crate) issued_at: DateTime<Utc>,
    pub(crate) last_seen_at: DateTime<Utc>,
    pub(crate) step_up_verified_at: Option<DateTime<Utc>>,
    pub(crate) revoked_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct KillSwitchRecord {
    pub(crate) flag: String,
    pub(crate) enabled: bool,
    pub(crate) reason: String,
    pub(crate) updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct ObservabilityEventRecord {
    pub(crate) occurred_at: DateTime<Utc>,
    pub(crate) source: String,
    pub(crate) event_type: String,
    pub(crate) target_id: Option<String>,
    pub(crate) detail: serde_json::Value,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct AuditRecord {
    pub(crate) id: String,
    pub(crate) actor_user_id: Option<String>,
    pub(crate) actor_email: String,
    pub(crate) action: String,
    pub(crate) target_type: String,
    pub(crate) target_id: String,
    pub(crate) reason: String,
    pub(crate) request_id: String,
    pub(crate) metadata: serde_json::Value,
    pub(crate) created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct IncidentRecord {
    pub(crate) id: String,
    pub(crate) title: String,
    pub(crate) severity: String,
    pub(crate) status: String,
    pub(crate) summary: Option<String>,
    pub(crate) context: serde_json::Value,
    pub(crate) opened_at: DateTime<Utc>,
    pub(crate) resolved_at: Option<DateTime<Utc>>,
    pub(crate) created_by: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct ObservabilityTimeseriesPoint {
    pub(crate) bucket: DateTime<Utc>,
    pub(crate) total_events: i64,
    pub(crate) error_events: i64,
    pub(crate) runtime_events: i64,
    pub(crate) provider_events: i64,
    pub(crate) auth_events: i64,
    pub(crate) admin_events: i64,
}
