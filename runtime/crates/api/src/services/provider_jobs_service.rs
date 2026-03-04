use bominal_shared::{
    crypto::{RedactionMode, redact_json},
    repo::{InsertRuntimeJobV2Params, insert_runtime_job_v2_query},
};
use chrono::{DateTime, Utc};
use serde_json::Value;
use sqlx::PgPool;
use tracing::error;
use uuid::Uuid;

use super::super::AppState;

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
    pub(crate) occurred_at: chrono::DateTime<Utc>,
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

pub(crate) async fn create_provider_job(
    state: &AppState,
    payload: CreateProviderJobRequest,
) -> Result<CreateProviderJobResult, ProviderJobsError> {
    let provider =
        canonical_provider(payload.provider.trim()).ok_or(ProviderJobsError::ValidationFailed)?;
    let operation = canonical_operation_name(payload.operation.trim())
        .ok_or(ProviderJobsError::ValidationFailed)?;
    validate_runtime_payload_contract(operation, &payload.payload)?;

    let Some(pool) = state.db_pool.as_ref() else {
        return Err(ProviderJobsError::PersistenceUnavailable);
    };

    let idempotency_key = payload
        .idempotency_key
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToOwned::to_owned);
    let job_id = idempotency_key
        .clone()
        .unwrap_or_else(|| Uuid::new_v4().to_string());
    let idempotency_scope = idempotency_key
        .as_ref()
        .map(|_| format!("provider:{}:{}", provider, operation));
    let now = Utc::now();
    let mut runtime_payload = serde_json::Map::new();
    runtime_payload.insert(
        "provider".to_string(),
        serde_json::Value::String(provider.to_string()),
    );
    runtime_payload.insert(
        "operation".to_string(),
        serde_json::Value::String(operation.to_string()),
    );
    runtime_payload.insert("payload".to_string(), payload.payload.clone());
    if let Some(user_id) = payload_user_id(&payload.payload) {
        runtime_payload.insert(
            "user_id".to_string(),
            serde_json::Value::String(user_id.to_string()),
        );
    }

    let runtime_payload = serde_json::Value::Object(runtime_payload);

    let params = InsertRuntimeJobV2Params {
        job_id: job_id.as_str(),
        payload: &runtime_payload,
        next_run_at: Some(now),
        idempotency_scope: idempotency_scope.as_deref(),
        idempotency_key: idempotency_key.as_deref(),
        max_attempts: 5,
        created_at: now,
    };

    let inserted = match insert_runtime_job_v2_query(&params).execute(pool).await {
        Ok(_) => true,
        Err(sqlx::Error::Database(database_error)) if database_error.is_unique_violation() => false,
        Err(err) => {
            error!(error = %err, "failed to insert runtime v2 provider job");
            return Err(ProviderJobsError::PersistenceFailure);
        }
    };

    let status = if inserted {
        let queued_event_detail = redact_json(
            &serde_json::json!({
                "provider": provider,
                "operation": operation,
                "state": "queued"
            }),
            RedactionMode::Mask,
        );

        if let Err(err) =
            insert_provider_job_event(pool, job_id.as_str(), "queued", &queued_event_detail, now)
                .await
        {
            error!(error = ?err, "failed to persist provider job queued event");
            return Err(ProviderJobsError::PersistenceFailure);
        }

        "queued".to_string()
    } else {
        load_runtime_job_status(pool, job_id.as_str()).await?
    };

    Ok(CreateProviderJobResult {
        accepted: true,
        job_id,
        status,
    })
}

pub(crate) async fn get_provider_job(
    state: &AppState,
    job_id: &str,
) -> Result<ProviderJobResult, ProviderJobsError> {
    validate_job_id(job_id)?;

    let Some(pool) = state.db_pool.as_ref() else {
        return Err(ProviderJobsError::PersistenceUnavailable);
    };

    let row = sqlx::query_as::<_, (String, String)>(
        "select status, payload::text from runtime_jobs where job_id = $1",
    )
    .bind(job_id)
    .fetch_optional(pool)
    .await
    .map_err(|err| {
        error!(error = %err, "failed to load runtime job row");
        ProviderJobsError::PersistenceFailure
    })?
    .ok_or(ProviderJobsError::NotFound)?;

    let payload = serde_json::from_str::<serde_json::Value>(&row.1).map_err(|err| {
        error!(error = %err, "failed to decode runtime job payload json");
        ProviderJobsError::PersistenceFailure
    })?;
    let provider = payload
        .get("provider")
        .and_then(serde_json::Value::as_str)
        .unwrap_or("unknown")
        .to_string();
    let operation = payload
        .get("operation")
        .and_then(serde_json::Value::as_str)
        .unwrap_or("unknown")
        .to_string();

    Ok(ProviderJobResult {
        job_id: job_id.to_string(),
        provider,
        operation,
        status: row.0,
    })
}

pub(crate) async fn list_provider_job_events_page(
    state: &AppState,
    job_id: &str,
    after_id: i64,
    limit: usize,
) -> Result<ProviderJobEventsPage, ProviderJobsError> {
    validate_job_id(job_id)?;
    if after_id < 0 || limit == 0 {
        return Err(ProviderJobsError::ValidationFailed);
    }

    let Some(pool) = state.db_pool.as_ref() else {
        return Err(ProviderJobsError::PersistenceUnavailable);
    };

    let limit_plus_one =
        i64::try_from(limit.saturating_add(1)).map_err(|_| ProviderJobsError::ValidationFailed)?;
    let rows = sqlx::query_as::<_, (i64, String, DateTime<Utc>, String)>(
        "select id, event_type, created_at, event_payload::text
         from runtime_job_events
         where job_id = $1 and id > $2
         order by id asc
         limit $3",
    )
    .bind(job_id)
    .bind(after_id)
    .bind(limit_plus_one)
    .fetch_all(pool)
    .await
    .map_err(|err| {
        error!(error = %err, "failed to load runtime job events");
        ProviderJobsError::PersistenceFailure
    })?;

    if rows.is_empty() && !runtime_job_exists(pool, job_id).await? {
        return Err(ProviderJobsError::NotFound);
    }

    let has_more = rows.len() > limit;
    let mut items = Vec::with_capacity(rows.len().min(limit));
    for row in rows.into_iter().take(limit) {
        let detail_raw = serde_json::from_str::<serde_json::Value>(&row.3).map_err(|err| {
            error!(error = %err, "failed to decode runtime job event payload");
            ProviderJobsError::PersistenceFailure
        })?;
        let detail = redact_json(&detail_raw, RedactionMode::Mask);

        items.push(ProviderJobEvent {
            sequence: row.0,
            event_type: row.1,
            occurred_at: row.2,
            detail,
        });
    }
    let next_after_id = items.last().map(|item| item.sequence);

    Ok(ProviderJobEventsPage {
        items,
        has_more,
        next_after_id,
    })
}

fn canonical_provider(raw: &str) -> Option<&'static str> {
    match normalize_token(raw).as_str() {
        "srt" => Some("srt"),
        "ktx" => Some("ktx"),
        _ => None,
    }
}

fn canonical_operation_name(raw: &str) -> Option<&'static str> {
    let normalized = normalize_token(raw);
    match normalized.as_str() {
        "login" => return Some("login"),
        "logout" => return Some("logout"),
        "search" | "search_train" | "train_search" => return Some("search_train"),
        "reserve" | "book" => return Some("reserve"),
        "reserve_standby" | "standby" | "waitlist" => return Some("reserve_standby"),
        "reserve_standby_option_settings" | "standby_option_settings" => {
            return Some("reserve_standby_option_settings");
        }
        "get_reservations" | "reservations" | "list_reservations" | "reservation_list" => {
            return Some("get_reservations");
        }
        "ticket_info" | "ticket" | "tickets" => return Some("ticket_info"),
        "cancel" => return Some("cancel"),
        "pay" | "payment" | "pay_with_card" | "train_pay" => return Some("pay_with_card"),
        "reserve_info" => return Some("reserve_info"),
        "refund" => return Some("refund"),
        "clear" => return Some("clear"),
        _ => {}
    }

    if normalized.contains("standby") && normalized.contains("option") {
        return Some("reserve_standby_option_settings");
    }
    if normalized.contains("standby") {
        return Some("reserve_standby");
    }
    if normalized.contains("search") {
        return Some("search_train");
    }
    if normalized.contains("reservation") && normalized.contains("list") {
        return Some("get_reservations");
    }
    if normalized.contains("ticket") {
        return Some("ticket_info");
    }
    if normalized.contains("pay") {
        return Some("pay_with_card");
    }
    if normalized.contains("reserve") && normalized.contains("info") {
        return Some("reserve_info");
    }
    if normalized.contains("refund") {
        return Some("refund");
    }
    if normalized.contains("reserve") {
        return Some("reserve");
    }
    if normalized.contains("login") {
        return Some("login");
    }
    if normalized.contains("logout") {
        return Some("logout");
    }
    if normalized.contains("cancel") {
        return Some("cancel");
    }
    if normalized.contains("clear") {
        return Some("clear");
    }

    None
}

fn validate_runtime_payload_contract(
    operation: &str,
    payload: &Value,
) -> Result<(), ProviderJobsError> {
    let Value::Object(_) = payload else {
        return Err(ProviderJobsError::ValidationFailed);
    };

    if contains_inline_secret_material(payload) {
        return Err(ProviderJobsError::ValidationFailed);
    }

    if operation != "clear" && ref_field(payload, "subject_ref").is_none() {
        return Err(ProviderJobsError::ValidationFailed);
    }

    if operation == "pay_with_card"
        && (ref_field(payload, "owner_ref").is_none()
            || ref_field(payload, "payment_method_ref").is_none())
    {
        return Err(ProviderJobsError::ValidationFailed);
    }

    Ok(())
}

fn ref_field<'a>(payload: &'a Value, key: &str) -> Option<&'a str> {
    payload
        .get(key)
        .and_then(Value::as_str)
        .or_else(|| {
            payload
                .get("refs")
                .and_then(Value::as_object)
                .and_then(|refs| refs.get(key))
                .and_then(Value::as_str)
        })
        .map(str::trim)
        .filter(|value| !value.is_empty())
}

fn payload_user_id(payload: &Value) -> Option<&str> {
    payload
        .get("user_id")
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .or_else(|| ref_field(payload, "subject_ref"))
}

fn contains_inline_secret_material(value: &Value) -> bool {
    match value {
        Value::Object(map) => map.iter().any(|(key, nested)| {
            if is_inline_secret_key(key) && !nested.is_null() {
                return true;
            }
            contains_inline_secret_material(nested)
        }),
        Value::Array(items) => items.iter().any(contains_inline_secret_material),
        _ => false,
    }
}

fn is_inline_secret_key(key: &str) -> bool {
    const INLINE_SECRET_KEYS: &[&str] = &[
        "identityciphertext",
        "passwordciphertext",
        "password",
        "panciphertext",
        "expirymonthciphertext",
        "expiryyearciphertext",
        "birthorbusinessnumberciphertext",
        "cardpasswordtwodigitsciphertext",
        "cardnumber",
        "cardpasswordtwodigits",
        "cardvalidationnumber",
        "cardexpiryyymm",
        "txtpwd",
        "hidstlcrcrdno1",
        "hidvanpwd1",
        "hidathnval1",
        "hidcrdvlidtrm1",
    ];

    let normalized = normalize_token(key).replace('_', "");
    INLINE_SECRET_KEYS.contains(&normalized.as_str())
}

fn validate_job_id(job_id: &str) -> Result<(), ProviderJobsError> {
    if job_id.trim().is_empty() {
        return Err(ProviderJobsError::ValidationFailed);
    }

    Ok(())
}

async fn insert_provider_job_event(
    pool: &PgPool,
    job_id: &str,
    event_type: &str,
    event_payload: &serde_json::Value,
    occurred_at: DateTime<Utc>,
) -> Result<(), ProviderJobsError> {
    sqlx::query(
        "insert into runtime_job_events (job_id, event_type, event_payload, created_at) values ($1, $2, cast($3 as jsonb), $4)",
    )
    .bind(job_id)
    .bind(event_type)
    .bind(event_payload)
    .bind(occurred_at)
    .execute(pool)
    .await
    .map_err(|err| {
        error!(error = %err, "failed to insert runtime job event row");
        ProviderJobsError::PersistenceFailure
    })?;

    Ok(())
}

async fn load_runtime_job_status(pool: &PgPool, job_id: &str) -> Result<String, ProviderJobsError> {
    let status =
        sqlx::query_scalar::<_, String>("select status from runtime_jobs where job_id = $1")
            .bind(job_id)
            .fetch_optional(pool)
            .await
            .map_err(|err| {
                error!(error = %err, "failed to load runtime job status");
                ProviderJobsError::PersistenceFailure
            })?
            .ok_or(ProviderJobsError::DuplicateConflict)?;

    Ok(status)
}

async fn runtime_job_exists(pool: &PgPool, job_id: &str) -> Result<bool, ProviderJobsError> {
    let exists = sqlx::query_scalar::<_, i64>("select 1 from runtime_jobs where job_id = $1")
        .bind(job_id)
        .fetch_optional(pool)
        .await
        .map_err(|err| {
            error!(error = %err, "failed to check runtime job existence");
            ProviderJobsError::PersistenceFailure
        })?
        .is_some();

    Ok(exists)
}

fn normalize_token(raw: &str) -> String {
    raw.trim()
        .to_ascii_lowercase()
        .replace(['.', '-', ' '], "_")
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn canonical_provider_accepts_srt_and_ktx_only() {
        assert_eq!(canonical_provider("srt"), Some("srt"));
        assert_eq!(canonical_provider("KTX"), Some("ktx"));
        assert_eq!(canonical_provider(" srt "), Some("srt"));
        assert_eq!(canonical_provider("korail"), None);
    }

    #[test]
    fn canonical_operation_name_maps_aliases() {
        assert_eq!(canonical_operation_name("search"), Some("search_train"));
        assert_eq!(canonical_operation_name("train-pay"), Some("pay_with_card"));
        assert_eq!(
            canonical_operation_name("reservation list"),
            Some("get_reservations")
        );
        assert_eq!(canonical_operation_name("unknown-op"), None);
    }

    #[test]
    fn payload_contract_requires_subject_ref_for_non_clear_operations() {
        let payload = json!({
            "request": {"dep_station_code": "0551", "arr_station_code": "0020"}
        });
        assert!(matches!(
            validate_runtime_payload_contract("search_train", &payload),
            Err(ProviderJobsError::ValidationFailed)
        ));

        let payload_with_ref = json!({
            "subject_ref": "subject-1",
            "request": {"dep_station_code": "0551", "arr_station_code": "0020"}
        });
        assert!(validate_runtime_payload_contract("search_train", &payload_with_ref).is_ok());

        let clear_payload = json!({});
        assert!(validate_runtime_payload_contract("clear", &clear_payload).is_ok());
    }

    #[test]
    fn payload_contract_requires_payment_refs_for_pay_with_card() {
        let missing_refs = json!({
            "subject_ref": "subject-1",
            "request": {"reservation_id": "PNR-1"}
        });
        assert!(matches!(
            validate_runtime_payload_contract("pay_with_card", &missing_refs),
            Err(ProviderJobsError::ValidationFailed)
        ));

        let with_refs = json!({
            "subject_ref": "subject-1",
            "owner_ref": "owner-1",
            "payment_method_ref": "pm-1",
            "request": {"reservation_id": "PNR-1"}
        });
        assert!(validate_runtime_payload_contract("pay_with_card", &with_refs).is_ok());
    }

    #[test]
    fn payload_contract_rejects_inline_secret_material_recursively() {
        let payload = json!({
            "subject_ref": "subject-1",
            "request": {
                "card_number": "4111111111111111"
            }
        });
        assert!(matches!(
            validate_runtime_payload_contract("search_train", &payload),
            Err(ProviderJobsError::ValidationFailed)
        ));

        let nested_secret_payload = json!({
            "subject_ref": "subject-1",
            "request": {
                "nested": {
                    "pan_ciphertext": "ciphertext"
                }
            }
        });
        assert!(matches!(
            validate_runtime_payload_contract("search_train", &nested_secret_payload),
            Err(ProviderJobsError::ValidationFailed)
        ));
    }

    #[test]
    fn payload_user_id_prefers_explicit_user_id_then_subject_ref() {
        let with_user_id = json!({
            "user_id": "user-123",
            "subject_ref": "subject-456"
        });
        assert_eq!(payload_user_id(&with_user_id), Some("user-123"));

        let with_subject_ref_only = json!({
            "subject_ref": "subject-456"
        });
        assert_eq!(payload_user_id(&with_subject_ref_only), Some("subject-456"));
    }
}
