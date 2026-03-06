use bominal_shared::{
    crypto::{RedactionMode, redact_json},
    repo::{InsertRuntimeJobV2Params, insert_runtime_job_v2_query},
};
use chrono::Utc;
use tracing::error;
use uuid::Uuid;

use super::super::super::AppState;

use super::{
    CreateProviderJobRequest, CreateProviderJobResult, ProviderJobsError,
    mapping::{
        canonical_operation_name, canonical_provider, payload_user_id,
        validate_runtime_payload_contract,
    },
    state::{insert_provider_job_event, load_runtime_job_status},
};

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
