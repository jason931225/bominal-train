use std::env;

use anyhow::Result;
use bominal_shared::{
    crypto::{
        EncryptedEnvelope, EnvelopeAad, EnvelopeAlgorithm, EnvelopeCipher, PayloadKind,
        RedactionMode, ServerEnvelopeCipher, StaticKeyring, redact_json,
    },
    providers::ProviderAdapter,
    providers::ProviderError,
    providers::ktx::{KtxProviderAdapter, ReqwestKtxClient},
    providers::model::{
        CancelRequest, CardIdentityType, ClearRequest, GetReservationsRequest, LoginAccountType,
        LoginRequest, LogoutRequest, Passenger, PayWithCardRequest, RefundRequest,
        ReserveInfoRequest, ReserveRequest, ReserveStandbyOptionSettingsRequest,
        ReserveStandbyRequest, SearchTrainRequest, SecretString, SrtClientFailureKind,
        SrtOperationRequest, SrtOperationResponse, TicketInfoRequest,
    },
    providers::srt::{ReqwestSrtClient, SrtProviderAdapter},
    repo::{select_active_payment_method_secret_query, select_active_provider_auth_secret_query},
};
use chrono::Utc;
use reqwest::header::CONTENT_TYPE;
use secrecy::ExposeSecret;
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use sqlx::{PgPool, Row};

const TEST_MASTER_KEY_B64_FALLBACK: &str = "MDEyMzQ1Njc4OWFiY2RlZjAxMjM0NTY3ODlhYmNkZWY=";
const PAYMENT_CRYPTO_DECRYPT_PATH: &str = "/v1/payment-methods/decrypt";

#[derive(Debug, Clone)]
pub struct ClaimedRuntimeJob {
    pub job_id: String,
    pub kind: String,
    pub user_id: Option<String>,
    pub payload: Value,
    pub persisted_payload: Value,
    pub attempt_count: i32,
    pub max_attempts: i32,
    pub idempotency_scope: Option<String>,
    pub idempotency_key: Option<String>,
}

impl ClaimedRuntimeJob {
    pub fn inferred_provider(&self) -> String {
        self.payload
            .get("provider")
            .and_then(Value::as_str)
            .or_else(|| {
                self.persisted_payload
                    .get("provider")
                    .and_then(Value::as_str)
            })
            .unwrap_or("srt")
            .to_ascii_lowercase()
    }
}

pub async fn load_claimed_job(pool: &PgPool, job_id: &str) -> Result<Option<ClaimedRuntimeJob>> {
    let row = sqlx::query(
        "select payload, attempt_count, max_attempts, idempotency_scope, idempotency_key \
         from runtime_jobs where job_id = $1",
    )
    .bind(job_id)
    .fetch_optional(pool)
    .await?;

    let Some(row) = row else {
        return Ok(None);
    };

    let persisted_payload: Value = row.try_get("payload")?;
    let payload = persisted_payload
        .get("payload")
        .cloned()
        .unwrap_or_else(|| persisted_payload.clone());
    let kind = persisted_payload
        .get("kind")
        .and_then(Value::as_str)
        .or_else(|| persisted_payload.get("operation").and_then(Value::as_str))
        .unwrap_or("runtime.unknown")
        .to_string();
    let user_id = persisted_payload
        .get("user_id")
        .and_then(Value::as_str)
        .map(ToOwned::to_owned);

    Ok(Some(ClaimedRuntimeJob {
        job_id: job_id.to_string(),
        kind,
        user_id,
        payload,
        persisted_payload,
        attempt_count: row.try_get("attempt_count")?,
        max_attempts: row.try_get("max_attempts")?,
        idempotency_scope: row.try_get("idempotency_scope")?,
        idempotency_key: row.try_get("idempotency_key")?,
    }))
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PaymentExecutionPolicy {
    pub app_env: String,
    pub ci_detected: bool,
    pub allow_auto_payment_in_testing: bool,
}

impl PaymentExecutionPolicy {
    pub fn from_env(app_env: &str) -> Self {
        Self {
            app_env: app_env.to_ascii_lowercase(),
            ci_detected: env_flag("CI"),
            allow_auto_payment_in_testing: env_flag("WORKER_ALLOW_AUTO_PAYMENT_IN_TESTING"),
        }
    }

    pub fn should_block_auto_payment(&self, kind: &str, payload: &Value) -> bool {
        let auto_pay = payload
            .get("auto_pay")
            .and_then(Value::as_bool)
            .unwrap_or(false);
        let operation = payload
            .get("operation")
            .and_then(Value::as_str)
            .unwrap_or("")
            .to_ascii_lowercase();
        let lower_kind = kind.to_ascii_lowercase();
        let payment_job = auto_pay
            || lower_kind.contains(".pay")
            || lower_kind.contains("payment")
            || operation.contains("pay")
            || operation.contains("payment");

        if !payment_job {
            return false;
        }

        let testing_context = self.ci_detected
            || matches!(
                self.app_env.as_str(),
                "test" | "testing" | "ci" | "integration"
            );

        testing_context && !self.allow_auto_payment_in_testing
    }
}

fn env_flag(key: &str) -> bool {
    env::var(key)
        .ok()
        .map(|value| {
            matches!(
                value.to_ascii_lowercase().as_str(),
                "1" | "true" | "yes" | "on"
            )
        })
        .unwrap_or(false)
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ExecutionErrorKind {
    Transient,
    RateLimited,
    Fatal,
    PaymentBlocked,
    UnsupportedProvider,
}

impl ExecutionErrorKind {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Transient => "transient",
            Self::RateLimited => "rate_limited",
            Self::Fatal => "fatal",
            Self::PaymentBlocked => "payment_blocked",
            Self::UnsupportedProvider => "unsupported_provider",
        }
    }
}

#[derive(Debug, Clone)]
pub struct ExecutionError {
    pub kind: ExecutionErrorKind,
    pub message: String,
    pub context: Value,
}

impl ExecutionError {
    pub fn new(kind: ExecutionErrorKind, message: impl Into<String>, context: Value) -> Self {
        Self {
            kind,
            message: message.into(),
            context,
        }
    }

    pub fn safe_message(&self) -> &'static str {
        match self.kind {
            ExecutionErrorKind::Transient => "transient execution failure",
            ExecutionErrorKind::RateLimited => "provider rate limited",
            ExecutionErrorKind::Fatal => "non-retryable execution failure",
            ExecutionErrorKind::PaymentBlocked => "auto payment blocked in ci/testing",
            ExecutionErrorKind::UnsupportedProvider => "provider not supported",
        }
    }
}

#[derive(Debug, Clone)]
pub struct ExecutionSuccess {
    pub provider: String,
    pub operation: String,
    pub result_redacted: Value,
}

#[derive(Debug, Default)]
pub struct ProviderExecutor;

#[derive(Debug, Clone)]
struct ProviderSecretRefs {
    subject_ref: Option<String>,
    owner_ref: Option<String>,
    payment_method_ref: Option<String>,
}

#[derive(Debug, Clone)]
struct ParsedProviderExecution {
    request: SrtOperationRequest,
    login_material: Option<LoginRequest>,
    simulated_failure: Option<SrtClientFailureKind>,
    refs: ProviderSecretRefs,
}

#[derive(Debug, Clone)]
struct ResolvedPaymentMaterial {
    card_number: String,
    card_password_two_digits: String,
    card_validation_number: String,
    card_expiry_yymm: String,
}

impl ProviderExecutor {
    pub async fn execute(
        &self,
        pool: &PgPool,
        job: &ClaimedRuntimeJob,
        payment_policy: &PaymentExecutionPolicy,
    ) -> std::result::Result<ExecutionSuccess, ExecutionError> {
        let provider = job.inferred_provider();
        match provider.as_str() {
            "srt" => self.execute_srt(pool, job, payment_policy).await,
            "ktx" => self.execute_ktx(pool, job, payment_policy).await,
            _ => Err(ExecutionError::new(
                ExecutionErrorKind::UnsupportedProvider,
                "provider execution hook not implemented",
                json!({"provider": provider}),
            )),
        }
    }

    async fn execute_srt(
        &self,
        pool: &PgPool,
        job: &ClaimedRuntimeJob,
        payment_policy: &PaymentExecutionPolicy,
    ) -> std::result::Result<ExecutionSuccess, ExecutionError> {
        if payment_policy.should_block_auto_payment(&job.kind, &job.payload) {
            return Err(ExecutionError::new(
                ExecutionErrorKind::PaymentBlocked,
                "auto payment blocked in ci/testing",
                json!({"kind": job.kind}),
            ));
        }

        let mut parsed = parse_provider_execution(job, "srt")?;
        let operation_name = parsed.request.operation_name().to_string();
        let operation = parsed.request.operation();
        resolve_execution_material(pool, "srt", &mut parsed, operation_name.as_str()).await?;

        if should_attempt_live_provider(payment_policy) {
            let mut live_client = ReqwestSrtClient::live(resolve_srt_base_url());
            if let Some(failure_kind) = parsed.simulated_failure {
                live_client = live_client.with_failure(operation, failure_kind, 1);
            }
            match dispatch_srt_with_client(&parsed, live_client) {
                Ok(response) => {
                    let result_redacted = build_redacted_result(job, &response);
                    return Ok(ExecutionSuccess {
                        provider: "srt".to_string(),
                        operation: operation_name,
                        result_redacted,
                    });
                }
                Err(error) => {
                    if !should_fallback_to_deterministic(&error) {
                        return Err(map_provider_error(error, "srt", operation_name.as_str()));
                    }
                }
            }
        }

        let mut fallback_client = ReqwestSrtClient::deterministic();
        if let Some(failure_kind) = parsed.simulated_failure {
            fallback_client = fallback_client.with_failure(operation, failure_kind, 1);
        }
        let response = dispatch_srt_with_client(&parsed, fallback_client)
            .map_err(|error| map_provider_error(error, "srt", operation_name.as_str()))?;
        let result_redacted = build_redacted_result(job, &response);

        Ok(ExecutionSuccess {
            provider: "srt".to_string(),
            operation: operation_name,
            result_redacted,
        })
    }

    async fn execute_ktx(
        &self,
        pool: &PgPool,
        job: &ClaimedRuntimeJob,
        payment_policy: &PaymentExecutionPolicy,
    ) -> std::result::Result<ExecutionSuccess, ExecutionError> {
        if payment_policy.should_block_auto_payment(&job.kind, &job.payload) {
            return Err(ExecutionError::new(
                ExecutionErrorKind::PaymentBlocked,
                "auto payment blocked in ci/testing",
                json!({"kind": job.kind}),
            ));
        }

        let mut parsed = parse_provider_execution(job, "ktx")?;
        let operation_name = parsed.request.operation_name().to_string();
        let operation = parsed.request.operation();
        resolve_execution_material(pool, "ktx", &mut parsed, operation_name.as_str()).await?;

        if should_attempt_live_provider(payment_policy) {
            let mut live_client = ReqwestKtxClient::live(resolve_ktx_base_url());
            if let Some(failure_kind) = parsed.simulated_failure {
                live_client = live_client.with_srt_failure(operation, failure_kind, 1);
            }
            match dispatch_ktx_with_client(&parsed, live_client) {
                Ok(response) => {
                    let result_redacted = build_redacted_result(job, &response);
                    return Ok(ExecutionSuccess {
                        provider: "ktx".to_string(),
                        operation: operation_name,
                        result_redacted,
                    });
                }
                Err(error) => {
                    if !should_fallback_to_deterministic(&error) {
                        return Err(map_provider_error(error, "ktx", operation_name.as_str()));
                    }
                }
            }
        }

        let mut fallback_client = ReqwestKtxClient::deterministic();
        if let Some(failure_kind) = parsed.simulated_failure {
            fallback_client = fallback_client.with_srt_failure(operation, failure_kind, 1);
        }
        let response = dispatch_ktx_with_client(&parsed, fallback_client)
            .map_err(|error| map_provider_error(error, "ktx", operation_name.as_str()))?;
        let result_redacted = build_redacted_result(job, &response);

        Ok(ExecutionSuccess {
            provider: "ktx".to_string(),
            operation: operation_name,
            result_redacted,
        })
    }
}

fn operation_requires_login_material(request: &SrtOperationRequest) -> bool {
    matches!(
        request,
        SrtOperationRequest::SearchTrain(_)
            | SrtOperationRequest::Reserve(_)
            | SrtOperationRequest::ReserveStandby(_)
            | SrtOperationRequest::ReserveStandbyOptionSettings(_)
            | SrtOperationRequest::GetReservations(_)
            | SrtOperationRequest::TicketInfo(_)
            | SrtOperationRequest::Cancel(_)
            | SrtOperationRequest::PayWithCard(_)
            | SrtOperationRequest::ReserveInfo(_)
            | SrtOperationRequest::Refund(_)
    )
}

fn build_redacted_result(job: &ClaimedRuntimeJob, response: &SrtOperationResponse) -> Value {
    let response_value = match serde_json::to_value(response) {
        Ok(value) => value,
        Err(_) => json!({"operation": response.operation_name()}),
    };
    let redacted_response = redact_json(&response_value, RedactionMode::Mask);

    json!({
        "job_id": job.job_id,
        "operation": response.operation_name(),
        "response": redacted_response,
    })
}

fn map_provider_error(
    error: ProviderError,
    provider: &str,
    operation_name: &str,
) -> ExecutionError {
    match error {
        ProviderError::Transport { message } => ExecutionError::new(
            ExecutionErrorKind::Transient,
            message,
            json!({"provider": provider, "operation": operation_name, "class": "transport"}),
        ),
        ProviderError::SessionExpired | ProviderError::Unauthorized => ExecutionError::new(
            ExecutionErrorKind::Transient,
            "provider authentication failed",
            json!({"provider": provider, "operation": operation_name, "class": "auth"}),
        ),
        ProviderError::NotLoggedIn | ProviderError::ReloginUnavailable => ExecutionError::new(
            ExecutionErrorKind::Fatal,
            "provider login/session material missing",
            json!({"provider": provider, "operation": operation_name, "class": "missing_session"}),
        ),
        ProviderError::OperationFailed { message } => {
            let kind = if message.to_ascii_lowercase().contains("rate_limited") {
                ExecutionErrorKind::RateLimited
            } else {
                ExecutionErrorKind::Fatal
            };
            let class = if kind == ExecutionErrorKind::RateLimited {
                "rate_limited"
            } else {
                "operation_failed"
            };

            ExecutionError::new(
                kind,
                message,
                json!({"provider": provider, "operation": operation_name, "class": class}),
            )
        }
        ProviderError::UnsupportedOperation { operation } => ExecutionError::new(
            ExecutionErrorKind::Fatal,
            format!("provider operation '{operation}' is not supported"),
            json!({"provider": provider, "operation": operation_name, "class": "unsupported_operation"}),
        ),
    }
}

#[cfg(test)]
fn map_srt_error(error: ProviderError, operation_name: &str) -> ExecutionError {
    map_provider_error(error, "srt", operation_name)
}

fn dispatch_srt_with_client(
    parsed: &ParsedProviderExecution,
    client: ReqwestSrtClient,
) -> Result<SrtOperationResponse, ProviderError> {
    let mut adapter = SrtProviderAdapter::new(client);
    if operation_requires_login_material(&parsed.request) {
        let login_request = parsed
            .login_material
            .clone()
            .ok_or(ProviderError::NotLoggedIn)?;
        adapter.login(login_request)?;
    }
    adapter.dispatch(parsed.request.clone())
}

fn dispatch_ktx_with_client(
    parsed: &ParsedProviderExecution,
    client: ReqwestKtxClient,
) -> Result<SrtOperationResponse, ProviderError> {
    let mut adapter = KtxProviderAdapter::new(client);
    if operation_requires_login_material(&parsed.request) {
        let login_request = parsed
            .login_material
            .clone()
            .ok_or(ProviderError::NotLoggedIn)?;
        adapter.login(login_request)?;
    }
    adapter.dispatch(parsed.request.clone())
}

fn should_fallback_to_deterministic(error: &ProviderError) -> bool {
    match error {
        ProviderError::Transport { .. } => true,
        ProviderError::OperationFailed { message } => {
            message.to_ascii_lowercase().contains("rate_limited")
        }
        _ => false,
    }
}

fn should_attempt_live_provider(policy: &PaymentExecutionPolicy) -> bool {
    if env_flag("WORKER_PROVIDER_LIVE_DISABLED") {
        return false;
    }

    let testing_context = policy.ci_detected
        || matches!(
            policy.app_env.as_str(),
            "test" | "testing" | "ci" | "integration"
        );

    if testing_context {
        return env_flag("WORKER_PROVIDER_LIVE_IN_TESTING");
    }

    true
}

fn resolve_srt_base_url() -> String {
    env::var("SRT_BASE_URL")
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| "https://app.srail.or.kr".to_string())
}

fn resolve_ktx_base_url() -> String {
    env::var("KTX_BASE_URL")
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| "https://smart.letskorail.com".to_string())
}

async fn resolve_execution_material(
    pool: &PgPool,
    provider: &str,
    parsed: &mut ParsedProviderExecution,
    operation_name: &str,
) -> std::result::Result<(), ExecutionError> {
    let login_request_incomplete = matches!(
        &parsed.request,
        SrtOperationRequest::Login(request)
            if request.account_identifier.trim().is_empty()
                || request.password.expose_secret().trim().is_empty()
    );
    let requires_login_material = operation_requires_login_material(&parsed.request)
        || matches!(&parsed.request, SrtOperationRequest::Login(_));
    if requires_login_material && (parsed.login_material.is_none() || login_request_incomplete) {
        let Some(subject_ref) = parsed.refs.subject_ref.as_deref() else {
            return Err(ExecutionError::new(
                ExecutionErrorKind::Fatal,
                "provider login/session material missing",
                json!({"provider": provider, "operation": operation_name, "class": "missing_subject_ref"}),
            ));
        };

        let login_request =
            load_provider_login_material(pool, provider, subject_ref, operation_name).await?;
        parsed.login_material = Some(login_request.clone());
        if matches!(&parsed.request, SrtOperationRequest::Login(_)) {
            parsed.request = SrtOperationRequest::Login(login_request);
        }
    }

    if let SrtOperationRequest::PayWithCard(request) = &mut parsed.request {
        let Some(owner_ref) = parsed.refs.owner_ref.as_deref() else {
            return Err(ExecutionError::new(
                ExecutionErrorKind::Fatal,
                "provider payment material missing",
                json!({"provider": provider, "operation": operation_name, "class": "missing_owner_ref"}),
            ));
        };
        let Some(payment_method_ref) = parsed.refs.payment_method_ref.as_deref() else {
            return Err(ExecutionError::new(
                ExecutionErrorKind::Fatal,
                "provider payment material missing",
                json!({"provider": provider, "operation": operation_name, "class": "missing_payment_method_ref"}),
            ));
        };

        let material = resolve_payment_material(
            pool,
            provider,
            owner_ref,
            payment_method_ref,
            operation_name,
        )
        .await?;

        request.card_number = SecretString::new(material.card_number.into_boxed_str());
        request.card_password_two_digits =
            SecretString::new(material.card_password_two_digits.into_boxed_str());
        request.card_validation_number =
            SecretString::new(material.card_validation_number.into_boxed_str());
        request.card_expiry_yymm = SecretString::new(material.card_expiry_yymm.into_boxed_str());
    }

    Ok(())
}

async fn load_provider_login_material(
    pool: &PgPool,
    provider: &str,
    subject_ref: &str,
    operation_name: &str,
) -> std::result::Result<LoginRequest, ExecutionError> {
    let row = select_active_provider_auth_secret_query(provider, subject_ref, "login")
        .fetch_optional(pool)
        .await
        .map_err(|error| {
            ExecutionError::new(
                ExecutionErrorKind::Transient,
                "provider auth secret lookup failed",
                json!({"provider": provider, "operation": operation_name, "class": "auth_secret_lookup", "error": error.to_string()}),
            )
        })?
        .ok_or_else(|| {
            ExecutionError::new(
                ExecutionErrorKind::Fatal,
                "provider login/session material missing",
                json!({"provider": provider, "operation": operation_name, "class": "auth_secret_missing"}),
            )
        })?;

    let ciphertext: Vec<u8> = row
        .try_get("secret_envelope_ciphertext")
        .map_err(|_| {
            ExecutionError::new(
                ExecutionErrorKind::Fatal,
                "provider login/session material missing",
                json!({"provider": provider, "operation": operation_name, "class": "auth_secret_decode"}),
            )
        })?;
    let nonce: Vec<u8> = row
        .try_get("secret_envelope_dek_ciphertext")
        .map_err(|_| {
            ExecutionError::new(
                ExecutionErrorKind::Fatal,
                "provider login/session material missing",
                json!({"provider": provider, "operation": operation_name, "class": "auth_secret_decode"}),
            )
        })?;
    let key_version: i32 = row
        .try_get("secret_envelope_kek_version")
        .map_err(|_| {
            ExecutionError::new(
                ExecutionErrorKind::Fatal,
                "provider login/session material missing",
                json!({"provider": provider, "operation": operation_name, "class": "auth_secret_decode"}),
            )
        })?;
    let aad_scope: String = row
        .try_get("secret_envelope_aad_scope")
        .map_err(|_| {
            ExecutionError::new(
                ExecutionErrorKind::Fatal,
                "provider login/session material missing",
                json!({"provider": provider, "operation": operation_name, "class": "auth_secret_decode"}),
            )
        })?;
    let aad_hash: Vec<u8> = row
        .try_get("secret_envelope_aad_hash")
        .map_err(|_| {
            ExecutionError::new(
                ExecutionErrorKind::Fatal,
                "provider login/session material missing",
                json!({"provider": provider, "operation": operation_name, "class": "auth_secret_decode"}),
            )
        })?;

    let aad = EnvelopeAad {
        payload_kind: PayloadKind::ProviderAuth,
        provider: Some(provider.to_string()),
        subject_id: Some(subject_ref.to_string()),
        scope: aad_scope,
        metadata: std::collections::BTreeMap::from([(
            "credential_kind".to_string(),
            "login".to_string(),
        )]),
    };
    validate_aad_hash(
        &aad,
        aad_hash.as_slice(),
        provider,
        operation_name,
        "auth_aad_hash",
    )?;
    let plaintext = decrypt_envelope(
        ciphertext.as_slice(),
        nonce.as_slice(),
        key_version,
        &aad,
        provider,
        operation_name,
        "auth_decrypt",
    )?;

    let decoded: Value = serde_json::from_slice(plaintext.as_slice()).map_err(|_| {
        ExecutionError::new(
            ExecutionErrorKind::Fatal,
            "provider login/session material missing",
            json!({"provider": provider, "operation": operation_name, "class": "auth_payload_decode"}),
        )
    })?;

    let account_identifier = required_string_field(
        &decoded,
        &["identity_ciphertext", "account_identifier"],
        provider,
        operation_name,
    )?;
    let password = required_string_field(
        &decoded,
        &["password_ciphertext", "password"],
        provider,
        operation_name,
    )?;
    let account_type = decoded
        .get("account_type")
        .and_then(Value::as_str)
        .and_then(parse_login_account_type)
        .unwrap_or(LoginAccountType::MembershipNumber);

    Ok(LoginRequest {
        account_type,
        account_identifier,
        password: SecretString::new(password.into_boxed_str()),
    })
}

async fn resolve_payment_material(
    pool: &PgPool,
    provider: &str,
    owner_ref: &str,
    payment_method_ref: &str,
    operation_name: &str,
) -> std::result::Result<ResolvedPaymentMaterial, ExecutionError> {
    let row = select_active_payment_method_secret_query(provider, owner_ref, payment_method_ref)
        .fetch_optional(pool)
        .await
        .map_err(|error| {
            ExecutionError::new(
                ExecutionErrorKind::Transient,
                "provider payment secret lookup failed",
                json!({"provider": provider, "operation": operation_name, "class": "payment_secret_lookup", "error": error.to_string()}),
            )
        })?
        .ok_or_else(|| {
            ExecutionError::new(
                ExecutionErrorKind::Fatal,
                "provider payment material missing",
                json!({"provider": provider, "operation": operation_name, "class": "payment_secret_missing"}),
            )
        })?;

    let redacted_metadata_raw: String = row
        .try_get("redacted_metadata")
        .unwrap_or_else(|_| "{}".to_string());
    let redacted_metadata: Value =
        serde_json::from_str(redacted_metadata_raw.as_str()).unwrap_or_else(|_| json!({}));
    let contract = redacted_metadata
        .get("contract")
        .and_then(Value::as_str)
        .unwrap_or("ciphertext-only-v1");

    if contract == "kms-envelope-over-evervault-v1" {
        return decrypt_via_payment_crypto_service(
            provider,
            owner_ref,
            payment_method_ref,
            operation_name,
        )
        .await;
    }

    let ciphertext: Vec<u8> = row
        .try_get("payment_payload_envelope_ciphertext")
        .map_err(|_| {
            ExecutionError::new(
                ExecutionErrorKind::Fatal,
                "provider payment material missing",
                json!({"provider": provider, "operation": operation_name, "class": "payment_secret_decode"}),
            )
        })?;
    let nonce: Vec<u8> = row
        .try_get("payment_payload_envelope_dek_ciphertext")
        .map_err(|_| {
            ExecutionError::new(
                ExecutionErrorKind::Fatal,
                "provider payment material missing",
                json!({"provider": provider, "operation": operation_name, "class": "payment_secret_decode"}),
            )
        })?;
    let key_version: i32 = row
        .try_get("payment_payload_envelope_kek_version")
        .map_err(|_| {
            ExecutionError::new(
                ExecutionErrorKind::Fatal,
                "provider payment material missing",
                json!({"provider": provider, "operation": operation_name, "class": "payment_secret_decode"}),
            )
        })?;
    let aad_scope: String = row
        .try_get("payment_payload_envelope_aad_scope")
        .map_err(|_| {
            ExecutionError::new(
                ExecutionErrorKind::Fatal,
                "provider payment material missing",
                json!({"provider": provider, "operation": operation_name, "class": "payment_secret_decode"}),
            )
        })?;
    let aad_hash: Vec<u8> = row
        .try_get("payment_payload_envelope_aad_hash")
        .map_err(|_| {
            ExecutionError::new(
                ExecutionErrorKind::Fatal,
                "provider payment material missing",
                json!({"provider": provider, "operation": operation_name, "class": "payment_secret_decode"}),
            )
        })?;

    let aad = EnvelopeAad {
        payload_kind: PayloadKind::ProviderPayment,
        provider: Some(provider.to_string()),
        subject_id: Some(owner_ref.to_string()),
        scope: aad_scope,
        metadata: std::collections::BTreeMap::from([(
            "payment_method_ref".to_string(),
            payment_method_ref.to_string(),
        )]),
    };
    validate_aad_hash(
        &aad,
        aad_hash.as_slice(),
        provider,
        operation_name,
        "payment_aad_hash",
    )?;
    let plaintext = decrypt_envelope(
        ciphertext.as_slice(),
        nonce.as_slice(),
        key_version,
        &aad,
        provider,
        operation_name,
        "payment_decrypt",
    )?;
    let decoded: Value = serde_json::from_slice(plaintext.as_slice()).map_err(|_| {
        ExecutionError::new(
            ExecutionErrorKind::Fatal,
            "provider payment material missing",
            json!({"provider": provider, "operation": operation_name, "class": "payment_payload_decode"}),
        )
    })?;

    let pan = required_string_field(
        &decoded,
        &["pan_ciphertext", "card_number"],
        provider,
        operation_name,
    )?;
    let expiry_month = required_string_field(
        &decoded,
        &["expiry_month_ciphertext", "expiry_month"],
        provider,
        operation_name,
    )?;
    let expiry_year = required_string_field(
        &decoded,
        &["expiry_year_ciphertext", "expiry_year"],
        provider,
        operation_name,
    )?;
    let validation = required_string_field(
        &decoded,
        &[
            "birth_or_business_number_ciphertext",
            "card_validation_number",
            "birth_or_business_number",
        ],
        provider,
        operation_name,
    )?;
    let password = required_string_field(
        &decoded,
        &[
            "card_password_two_digits_ciphertext",
            "card_password_two_digits",
            "card_password",
        ],
        provider,
        operation_name,
    )?;

    Ok(ResolvedPaymentMaterial {
        card_number: pan,
        card_password_two_digits: password,
        card_validation_number: validation,
        card_expiry_yymm: format!("{expiry_year}{expiry_month}"),
    })
}

#[derive(Debug, serde::Serialize)]
struct PaymentDecryptRequest<'a> {
    provider: &'a str,
    owner_ref: &'a str,
    payment_method_ref: &'a str,
}

#[derive(Debug, serde::Deserialize)]
struct PaymentDecryptResponse {
    ok: bool,
    #[serde(default)]
    detail: String,
    #[serde(default)]
    ev_payload: Option<PaymentDecryptPayload>,
}

#[derive(Debug, serde::Deserialize)]
struct PaymentDecryptPayload {
    pan_ev: String,
    expiry_month_ev: String,
    expiry_year_ev: String,
    birth_or_business_ev: String,
    card_password_two_digits_ev: String,
}

async fn decrypt_via_payment_crypto_service(
    provider: &str,
    owner_ref: &str,
    payment_method_ref: &str,
    operation_name: &str,
) -> std::result::Result<ResolvedPaymentMaterial, ExecutionError> {
    let base_url = env::var("PAYMENT_CRYPTO_SERVICE_URL")
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
        .ok_or_else(|| {
            ExecutionError::new(
                ExecutionErrorKind::Transient,
                "payment crypto service unavailable",
                json!({"provider": provider, "operation": operation_name, "class": "payment_crypto_missing_url"}),
            )
        })?;
    let url = format!(
        "{}{}",
        base_url.trim_end_matches('/'),
        PAYMENT_CRYPTO_DECRYPT_PATH
    );
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(15))
        .build()
        .map_err(|_| {
            ExecutionError::new(
                ExecutionErrorKind::Transient,
                "payment crypto service unavailable",
                json!({"provider": provider, "operation": operation_name, "class": "payment_crypto_http_init"}),
            )
        })?;
    let mut request_builder = client
        .post(url)
        .header(CONTENT_TYPE, "application/json")
        .json(&PaymentDecryptRequest {
            provider,
            owner_ref,
            payment_method_ref,
        });
    if let Some(token) = env::var("PAYMENT_CRYPTO_SERVICE_TOKEN")
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
    {
        request_builder = request_builder.header("x-internal-service-token", token);
    }

    let response = request_builder.send().await.map_err(|error| {
        ExecutionError::new(
            ExecutionErrorKind::Transient,
            "payment crypto service unavailable",
            json!({"provider": provider, "operation": operation_name, "class": "payment_crypto_request", "error": error.to_string()}),
        )
    })?;
    let status = response.status();
    let decoded =
        response
            .json::<PaymentDecryptResponse>()
            .await
            .unwrap_or(PaymentDecryptResponse {
                ok: false,
                detail: "invalid decrypt response".to_string(),
                ev_payload: None,
            });

    if !status.is_success() || !decoded.ok {
        let kind = if status.is_server_error() {
            ExecutionErrorKind::Transient
        } else {
            ExecutionErrorKind::Fatal
        };
        return Err(ExecutionError::new(
            kind,
            "provider payment material missing",
            json!({
                "provider": provider,
                "operation": operation_name,
                "class": "payment_crypto_decrypt_failed",
                "status": status.as_u16(),
                "detail": decoded.detail,
            }),
        ));
    }

    let Some(payload) = decoded.ev_payload else {
        return Err(ExecutionError::new(
            ExecutionErrorKind::Fatal,
            "provider payment material missing",
            json!({"provider": provider, "operation": operation_name, "class": "payment_crypto_payload_missing"}),
        ));
    };

    Ok(ResolvedPaymentMaterial {
        card_number: payload.pan_ev,
        card_password_two_digits: payload.card_password_two_digits_ev,
        card_validation_number: payload.birth_or_business_ev,
        card_expiry_yymm: format!("{}{}", payload.expiry_year_ev, payload.expiry_month_ev),
    })
}

fn decrypt_envelope(
    ciphertext: &[u8],
    nonce: &[u8],
    key_version: i32,
    aad: &EnvelopeAad,
    provider: &str,
    operation_name: &str,
    class: &str,
) -> std::result::Result<Vec<u8>, ExecutionError> {
    let nonce_array: [u8; 12] = nonce.try_into().map_err(|_| {
        ExecutionError::new(
            ExecutionErrorKind::Fatal,
            "provider secret envelope invalid",
            json!({"provider": provider, "operation": operation_name, "class": class}),
        )
    })?;
    let version: u32 = u32::try_from(key_version).map_err(|_| {
        ExecutionError::new(
            ExecutionErrorKind::Fatal,
            "provider secret envelope invalid",
            json!({"provider": provider, "operation": operation_name, "class": class}),
        )
    })?;
    let envelope = EncryptedEnvelope {
        algorithm: EnvelopeAlgorithm::Aes256Gcm,
        key_version: version,
        aad_context: aad.clone(),
        nonce: nonce_array,
        ciphertext: ciphertext.to_vec(),
    };
    let cipher = build_envelope_cipher()?;
    cipher.decrypt(&envelope, aad).map_err(|_| {
        ExecutionError::new(
            ExecutionErrorKind::Fatal,
            "provider secret envelope invalid",
            json!({"provider": provider, "operation": operation_name, "class": class}),
        )
    })
}

fn validate_aad_hash(
    aad: &EnvelopeAad,
    stored_hash: &[u8],
    provider: &str,
    operation_name: &str,
    class: &str,
) -> std::result::Result<(), ExecutionError> {
    let aad_bytes = serde_json::to_vec(aad).map_err(|_| {
        ExecutionError::new(
            ExecutionErrorKind::Fatal,
            "provider secret envelope invalid",
            json!({"provider": provider, "operation": operation_name, "class": class}),
        )
    })?;
    let computed = Sha256::digest(aad_bytes.as_slice());
    let matches_legacy_plain = stored_hash == aad_bytes.as_slice();
    let matches_hashed = stored_hash == computed.as_slice();
    if !matches_legacy_plain && !matches_hashed {
        return Err(ExecutionError::new(
            ExecutionErrorKind::Fatal,
            "provider secret envelope invalid",
            json!({"provider": provider, "operation": operation_name, "class": class}),
        ));
    }
    Ok(())
}

fn build_envelope_cipher() -> std::result::Result<ServerEnvelopeCipher, ExecutionError> {
    let key_version = env::var("KEK_VERSION")
        .ok()
        .and_then(|value| value.trim().parse::<u32>().ok())
        .filter(|value| *value > 0)
        .unwrap_or(1);
    let app_env = env::var("APP_ENV").unwrap_or_else(|_| "dev".to_string());

    let encoded_master_key = env::var("MASTER_KEY_OVERRIDE")
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
        .or_else(|| {
            env::var("MASTER_KEY")
                .ok()
                .map(|value| value.trim().to_string())
                .filter(|value| !value.is_empty())
        })
        .or_else(|| {
            if app_env.trim().eq_ignore_ascii_case("test") {
                Some(TEST_MASTER_KEY_B64_FALLBACK.to_string())
            } else {
                None
            }
        })
        .ok_or_else(|| {
            ExecutionError::new(
                ExecutionErrorKind::Transient,
                "provider secret envelope unavailable",
                json!({"class": "missing_master_key"}),
            )
        })?;

    let key_bytes = decode_base64(encoded_master_key.as_str()).map_err(|_| {
        ExecutionError::new(
            ExecutionErrorKind::Transient,
            "provider secret envelope unavailable",
            json!({"class": "invalid_master_key"}),
        )
    })?;
    if key_bytes.len() != 32 {
        return Err(ExecutionError::new(
            ExecutionErrorKind::Transient,
            "provider secret envelope unavailable",
            json!({"class": "invalid_master_key_length"}),
        ));
    }

    let mut keys = std::collections::BTreeMap::new();
    keys.insert(key_version, key_bytes);
    let keyring = StaticKeyring::new(key_version, keys).map_err(|_| {
        ExecutionError::new(
            ExecutionErrorKind::Transient,
            "provider secret envelope unavailable",
            json!({"class": "keyring_init_failed"}),
        )
    })?;
    Ok(ServerEnvelopeCipher::new(keyring))
}

fn decode_base64(input: &str) -> Result<Vec<u8>, ()> {
    let mut out = Vec::with_capacity((input.len() * 3) / 4 + 3);
    let mut buffer = 0u32;
    let mut bits = 0usize;
    let mut seen_padding = false;

    for byte in input.bytes() {
        if byte.is_ascii_whitespace() {
            continue;
        }
        if byte == b'=' {
            seen_padding = true;
            continue;
        }
        if seen_padding {
            return Err(());
        }

        let sextet = match byte {
            b'A'..=b'Z' => (byte - b'A') as u32,
            b'a'..=b'z' => (byte - b'a' + 26) as u32,
            b'0'..=b'9' => (byte - b'0' + 52) as u32,
            b'+' | b'-' => 62,
            b'/' | b'_' => 63,
            _ => return Err(()),
        };

        buffer = (buffer << 6) | sextet;
        bits += 6;

        while bits >= 8 {
            bits -= 8;
            out.push(((buffer >> bits) & 0xff) as u8);
            buffer &= (1u32 << bits) - 1;
        }
    }

    if bits > 0 && (buffer & ((1u32 << bits) - 1)) != 0 {
        return Err(());
    }

    Ok(out)
}

fn ref_string_field(payload: &Value, key: &str) -> Option<String> {
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
        .map(ToOwned::to_owned)
}

fn parse_provider_execution(
    job: &ClaimedRuntimeJob,
    provider: &str,
) -> std::result::Result<ParsedProviderExecution, ExecutionError> {
    let raw_operation = infer_operation_token(job);
    let Some(operation) = canonical_operation_name(&raw_operation) else {
        return Err(ExecutionError::new(
            ExecutionErrorKind::Fatal,
            "unsupported provider operation in runtime payload",
            json!({"provider": provider, "operation": raw_operation}),
        ));
    };

    let payload = &job.payload;
    let request_payload = operation_payload(payload);
    let mut login_material = parse_optional_login_request(payload, provider)?;
    let simulated_failure = parse_simulated_failure(payload);
    let refs = ProviderSecretRefs {
        subject_ref: ref_string_field(payload, "subject_ref"),
        owner_ref: ref_string_field(payload, "owner_ref"),
        payment_method_ref: ref_string_field(payload, "payment_method_ref"),
    };

    let request = match operation {
        "login" => {
            let login_request = match login_material.clone() {
                Some(login) => login,
                None => parse_login_request(request_payload, provider).unwrap_or_else(|_| {
                    LoginRequest {
                        account_type: LoginAccountType::MembershipNumber,
                        account_identifier: String::new(),
                        password: SecretString::new(String::new().into_boxed_str()),
                    }
                }),
            };
            login_material = Some(login_request.clone());
            SrtOperationRequest::Login(login_request)
        }
        "logout" => SrtOperationRequest::Logout(LogoutRequest),
        "search_train" => {
            SrtOperationRequest::SearchTrain(parse_search_train_request(request_payload, provider)?)
        }
        "reserve" => {
            SrtOperationRequest::Reserve(parse_reserve_request(request_payload, provider)?)
        }
        "reserve_standby" => SrtOperationRequest::ReserveStandby(parse_reserve_standby_request(
            request_payload,
            provider,
        )?),
        "reserve_standby_option_settings" => SrtOperationRequest::ReserveStandbyOptionSettings(
            parse_reserve_standby_option_settings_request(request_payload, provider)?,
        ),
        "get_reservations" => {
            let request = parse_optional_get_reservations_request(request_payload)?;
            SrtOperationRequest::GetReservations(request)
        }
        "ticket_info" => {
            SrtOperationRequest::TicketInfo(parse_ticket_info_request(request_payload, provider)?)
        }
        "cancel" => SrtOperationRequest::Cancel(parse_cancel_request(request_payload, provider)?),
        "pay_with_card" => SrtOperationRequest::PayWithCard(parse_pay_with_card_request(
            request_payload,
            provider,
        )?),
        "reserve_info" => {
            SrtOperationRequest::ReserveInfo(parse_reserve_info_request(request_payload, provider)?)
        }
        "refund" => SrtOperationRequest::Refund(parse_refund_request(request_payload, provider)?),
        "clear" => SrtOperationRequest::Clear(ClearRequest),
        _ => {
            return Err(ExecutionError::new(
                ExecutionErrorKind::Fatal,
                "unsupported provider operation in runtime payload",
                json!({"provider": provider, "operation": raw_operation}),
            ));
        }
    };

    Ok(ParsedProviderExecution {
        request,
        login_material,
        simulated_failure,
        refs,
    })
}

fn infer_operation_token(job: &ClaimedRuntimeJob) -> String {
    job.payload
        .get("operation")
        .and_then(Value::as_str)
        .or_else(|| {
            job.persisted_payload
                .get("operation")
                .and_then(Value::as_str)
        })
        .unwrap_or(job.kind.as_str())
        .to_string()
}

fn canonical_operation_name(raw: &str) -> Option<&'static str> {
    let normalized = normalize_operation_token(raw);
    match normalized.as_str() {
        "login" => return Some("login"),
        "logout" => return Some("logout"),
        "search" | "search_train" | "train_search" => return Some("search_train"),
        "reserve" | "book" => return Some("reserve"),
        "reserve_standby" | "standby" | "waitlist" => return Some("reserve_standby"),
        "reserve_standby_option_settings" | "standby_option_settings" => {
            return Some("reserve_standby_option_settings");
        }
        "get_reservations" | "reservations" | "list_reservations" => {
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

fn normalize_operation_token(raw: &str) -> String {
    raw.trim()
        .to_ascii_lowercase()
        .replace(['.', '-', ' '], "_")
}

fn operation_payload(payload: &Value) -> &Value {
    payload.get("request").unwrap_or(payload)
}

fn parse_optional_login_request(
    payload: &Value,
    provider: &str,
) -> std::result::Result<Option<LoginRequest>, ExecutionError> {
    let login_value = payload
        .get("login")
        .or_else(|| payload.get("session").and_then(|value| value.get("login")));

    match login_value {
        Some(value) => parse_login_request(value, provider).map(Some),
        None => Ok(None),
    }
}

fn parse_login_request(
    value: &Value,
    provider: &str,
) -> std::result::Result<LoginRequest, ExecutionError> {
    let account_identifier = optional_string_field(value, &["account_identifier"]);
    let password = optional_string_field(value, &["password"]);
    let account_type = value
        .get("account_type")
        .and_then(Value::as_str)
        .and_then(parse_login_account_type)
        .unwrap_or(LoginAccountType::MembershipNumber);

    let Some(account_identifier) = account_identifier else {
        return Err(ExecutionError::new(
            ExecutionErrorKind::Fatal,
            "provider login/session material missing",
            json!({"provider": provider, "operation": "login", "class": "missing_login_fields"}),
        ));
    };
    let Some(password) = password else {
        return Err(ExecutionError::new(
            ExecutionErrorKind::Fatal,
            "provider login/session material missing",
            json!({"provider": provider, "operation": "login", "class": "missing_login_fields"}),
        ));
    };

    Ok(LoginRequest {
        account_type,
        account_identifier,
        password: SecretString::new(password.into()),
    })
}

fn parse_search_train_request(
    value: &Value,
    provider: &str,
) -> std::result::Result<SearchTrainRequest, ExecutionError> {
    let dep_station_code = required_string_field(
        value,
        &["dep_station_code", "from"],
        provider,
        "search_train",
    )?;
    let arr_station_code =
        required_string_field(value, &["arr_station_code", "to"], provider, "search_train")?;
    let dep_date = optional_string_field(value, &["dep_date"])
        .unwrap_or_else(|| Utc::now().format("%Y%m%d").to_string());
    let dep_time =
        optional_string_field(value, &["dep_time"]).unwrap_or_else(|| "000000".to_string());
    let time_limit = optional_string_field(value, &["time_limit"]);
    let passengers = match value.get("passengers") {
        Some(raw) => serde_json::from_value::<Vec<Passenger>>(raw.clone()).map_err(|error| {
            ExecutionError::new(
                ExecutionErrorKind::Fatal,
                format!("invalid search_train request payload: {error}"),
                json!({"provider": provider, "operation": "search_train", "field": "passengers"}),
            )
        })?,
        None => vec![Passenger::adult(1)],
    };
    let available_only = value
        .get("available_only")
        .and_then(Value::as_bool)
        .unwrap_or(true);

    Ok(SearchTrainRequest {
        dep_station_code,
        arr_station_code,
        dep_date,
        dep_time,
        time_limit,
        passengers,
        available_only,
    })
}

fn parse_pay_with_card_request(
    value: &Value,
    provider: &str,
) -> std::result::Result<PayWithCardRequest, ExecutionError> {
    let reservation_id =
        required_string_field(value, &["reservation_id"], provider, "pay_with_card")?;
    let card_number =
        optional_string_field(value, &["card_number", "pan", "pan_ciphertext"]).unwrap_or_default();
    let card_password_two_digits = optional_string_field(
        value,
        &[
            "card_password_two_digits",
            "card_password",
            "card_password_two_digits_ciphertext",
        ],
    )
    .unwrap_or_default();
    let card_validation_number = optional_string_field(
        value,
        &[
            "card_validation_number",
            "birth_or_business_number",
            "birth_or_business_number_ciphertext",
        ],
    )
    .unwrap_or_default();
    let card_expiry_yymm = optional_string_field(
        value,
        &[
            "card_expiry_yymm",
            "card_expiry",
            "card_expiry_yymm_ciphertext",
        ],
    )
    .or_else(|| {
        let month = optional_string_field(value, &["expiry_month", "expiry_month_ciphertext"])?;
        let year = optional_string_field(value, &["expiry_year", "expiry_year_ciphertext"])?;
        Some(format!("{year}{month}"))
    })
    .unwrap_or_default();
    let installment_months = value
        .get("installment_months")
        .and_then(Value::as_u64)
        .unwrap_or(0)
        .min(u8::MAX as u64) as u8;
    let card_identity_type = value
        .get("card_identity_type")
        .and_then(Value::as_str)
        .and_then(parse_card_identity_type)
        .unwrap_or(CardIdentityType::Personal);

    Ok(PayWithCardRequest {
        reservation_id,
        card_identity_type,
        card_number: SecretString::new(card_number.into()),
        card_password_two_digits: SecretString::new(card_password_two_digits.into()),
        card_validation_number: SecretString::new(card_validation_number.into()),
        card_expiry_yymm: SecretString::new(card_expiry_yymm.into()),
        installment_months,
    })
}

fn parse_optional_get_reservations_request(
    value: &Value,
) -> std::result::Result<GetReservationsRequest, ExecutionError> {
    let paid_only = value
        .get("paid_only")
        .and_then(Value::as_bool)
        .unwrap_or(false);
    Ok(GetReservationsRequest { paid_only })
}

fn parse_reserve_request(
    value: &Value,
    provider: &str,
) -> std::result::Result<ReserveRequest, ExecutionError> {
    serde_json::from_value(value.clone()).map_err(|error| {
        ExecutionError::new(
            ExecutionErrorKind::Fatal,
            format!("invalid reserve request payload: {error}"),
            json!({"provider": provider, "operation": "reserve"}),
        )
    })
}

fn parse_reserve_standby_request(
    value: &Value,
    provider: &str,
) -> std::result::Result<ReserveStandbyRequest, ExecutionError> {
    serde_json::from_value(value.clone()).map_err(|error| {
        ExecutionError::new(
            ExecutionErrorKind::Fatal,
            format!("invalid reserve_standby request payload: {error}"),
            json!({"provider": provider, "operation": "reserve_standby"}),
        )
    })
}

fn parse_reserve_standby_option_settings_request(
    value: &Value,
    provider: &str,
) -> std::result::Result<ReserveStandbyOptionSettingsRequest, ExecutionError> {
    serde_json::from_value(value.clone()).map_err(|error| {
        ExecutionError::new(
            ExecutionErrorKind::Fatal,
            format!("invalid reserve_standby_option_settings request payload: {error}"),
            json!({"provider": provider, "operation": "reserve_standby_option_settings"}),
        )
    })
}

fn parse_ticket_info_request(
    value: &Value,
    provider: &str,
) -> std::result::Result<TicketInfoRequest, ExecutionError> {
    serde_json::from_value(value.clone()).map_err(|error| {
        ExecutionError::new(
            ExecutionErrorKind::Fatal,
            format!("invalid ticket_info request payload: {error}"),
            json!({"provider": provider, "operation": "ticket_info"}),
        )
    })
}

fn parse_cancel_request(
    value: &Value,
    provider: &str,
) -> std::result::Result<CancelRequest, ExecutionError> {
    serde_json::from_value(value.clone()).map_err(|error| {
        ExecutionError::new(
            ExecutionErrorKind::Fatal,
            format!("invalid cancel request payload: {error}"),
            json!({"provider": provider, "operation": "cancel"}),
        )
    })
}

fn parse_reserve_info_request(
    value: &Value,
    provider: &str,
) -> std::result::Result<ReserveInfoRequest, ExecutionError> {
    serde_json::from_value(value.clone()).map_err(|error| {
        ExecutionError::new(
            ExecutionErrorKind::Fatal,
            format!("invalid reserve_info request payload: {error}"),
            json!({"provider": provider, "operation": "reserve_info"}),
        )
    })
}

fn parse_refund_request(
    value: &Value,
    provider: &str,
) -> std::result::Result<RefundRequest, ExecutionError> {
    serde_json::from_value(value.clone()).map_err(|error| {
        ExecutionError::new(
            ExecutionErrorKind::Fatal,
            format!("invalid refund request payload: {error}"),
            json!({"provider": provider, "operation": "refund"}),
        )
    })
}

fn required_string_field(
    value: &Value,
    keys: &[&str],
    provider: &str,
    operation: &str,
) -> std::result::Result<String, ExecutionError> {
    optional_string_field(value, keys).ok_or_else(|| {
        ExecutionError::new(
            ExecutionErrorKind::Fatal,
            format!("invalid {operation} request payload"),
            json!({"provider": provider, "operation": operation}),
        )
    })
}

fn optional_string_field(value: &Value, keys: &[&str]) -> Option<String> {
    keys.iter().find_map(|key| {
        value
            .get(*key)
            .and_then(Value::as_str)
            .map(str::trim)
            .filter(|candidate| !candidate.is_empty())
            .map(ToOwned::to_owned)
    })
}

fn parse_login_account_type(raw: &str) -> Option<LoginAccountType> {
    match normalize_operation_token(raw).as_str() {
        "membership_number" | "membership" | "member" => Some(LoginAccountType::MembershipNumber),
        "email" => Some(LoginAccountType::Email),
        "phone_number" | "phone" => Some(LoginAccountType::PhoneNumber),
        _ => None,
    }
}

fn parse_card_identity_type(raw: &str) -> Option<CardIdentityType> {
    match normalize_operation_token(raw).as_str() {
        "personal" => Some(CardIdentityType::Personal),
        "corporate" => Some(CardIdentityType::Corporate),
        _ => None,
    }
}

fn parse_simulated_failure(payload: &Value) -> Option<SrtClientFailureKind> {
    payload
        .get("simulate_error_kind")
        .and_then(Value::as_str)
        .map(|raw| match raw.to_ascii_lowercase().as_str() {
            "transient" => SrtClientFailureKind::Transient,
            "rate_limited" => SrtClientFailureKind::RateLimited,
            "session_expired" => SrtClientFailureKind::SessionExpired,
            "unauthorized" => SrtClientFailureKind::Unauthorized,
            _ => SrtClientFailureKind::Fatal,
        })
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn job_with_payload(kind: &str, payload: Value) -> ClaimedRuntimeJob {
        ClaimedRuntimeJob {
            job_id: "job-1".to_string(),
            kind: kind.to_string(),
            user_id: None,
            payload,
            persisted_payload: json!({}),
            attempt_count: 1,
            max_attempts: 3,
            idempotency_scope: None,
            idempotency_key: None,
        }
    }

    fn test_login_request() -> LoginRequest {
        LoginRequest {
            account_type: LoginAccountType::MembershipNumber,
            account_identifier: "member-1".to_string(),
            password: SecretString::new("password".to_string().into_boxed_str()),
        }
    }

    fn parsed_with_login(request: SrtOperationRequest) -> ParsedProviderExecution {
        ParsedProviderExecution {
            request,
            login_material: Some(test_login_request()),
            simulated_failure: None,
            refs: ProviderSecretRefs {
                subject_ref: None,
                owner_ref: None,
                payment_method_ref: None,
            },
        }
    }

    #[test]
    fn canonical_operation_name_maps_aliases_and_fuzzy_tokens() {
        assert_eq!(canonical_operation_name("search"), Some("search_train"));
        assert_eq!(canonical_operation_name("TRAIN-PAY"), Some("pay_with_card"));
        assert_eq!(
            canonical_operation_name("reserve standby option"),
            Some("reserve_standby_option_settings")
        );
        assert_eq!(
            canonical_operation_name("reservation list"),
            Some("get_reservations")
        );
        assert_eq!(canonical_operation_name("custom_unknown_operation"), None);
    }

    #[test]
    fn canonical_operation_name_supports_full_srtgo_operation_set() {
        let srtgo_operations = [
            "login",
            "logout",
            "search_train",
            "reserve",
            "reserve_standby",
            "reserve_standby_option_settings",
            "get_reservations",
            "ticket_info",
            "cancel",
            "pay_with_card",
            "reserve_info",
            "refund",
            "clear",
        ];

        for operation in srtgo_operations {
            assert_eq!(
                canonical_operation_name(operation),
                Some(operation),
                "expected canonical mapping for {operation}"
            );
        }
    }

    #[test]
    fn parse_provider_execution_uses_aliases_from_payload_or_kind() {
        let search_job = job_with_payload(
            "runtime.search",
            json!({
                "operation": "train-search",
                "dep_station_code": "0551",
                "arr_station_code": "0020",
            }),
        );
        let parsed_search =
            parse_provider_execution(&search_job, "srt").expect("search alias should parse");
        assert_eq!(parsed_search.request.operation_name(), "search_train");

        let clear_job = job_with_payload("runtime.clear", json!({}));
        let parsed_clear =
            parse_provider_execution(&clear_job, "srt").expect("kind fallback should parse");
        assert_eq!(parsed_clear.request.operation_name(), "clear");
    }

    #[test]
    fn payment_policy_blocks_only_payment_jobs_in_testing_context() {
        let policy = PaymentExecutionPolicy {
            app_env: "test".to_string(),
            ci_detected: false,
            allow_auto_payment_in_testing: false,
        };

        assert!(policy.should_block_auto_payment("runtime.pay_with_card", &json!({})));
        assert!(
            policy.should_block_auto_payment("runtime.reserve", &json!({"operation": "payment"}))
        );
        assert!(policy.should_block_auto_payment("runtime.reserve", &json!({"auto_pay": true})));
        assert!(
            !policy.should_block_auto_payment("runtime.reserve", &json!({"operation": "reserve"}))
        );
    }

    #[test]
    fn payment_policy_allows_when_overridden_or_outside_testing() {
        let allow_testing = PaymentExecutionPolicy {
            app_env: "testing".to_string(),
            ci_detected: false,
            allow_auto_payment_in_testing: true,
        };
        assert!(!allow_testing.should_block_auto_payment("runtime.payment", &json!({})));

        let production = PaymentExecutionPolicy {
            app_env: "production".to_string(),
            ci_detected: false,
            allow_auto_payment_in_testing: false,
        };
        assert!(!production.should_block_auto_payment("runtime.payment", &json!({})));

        let ci = PaymentExecutionPolicy {
            app_env: "production".to_string(),
            ci_detected: true,
            allow_auto_payment_in_testing: false,
        };
        assert!(ci.should_block_auto_payment("runtime.payment", &json!({})));
    }

    #[test]
    fn map_srt_error_maps_operation_failed_to_rate_limited_or_fatal() {
        let rate_limited = map_srt_error(
            ProviderError::OperationFailed {
                message: "RATE_LIMITED by provider".to_string(),
            },
            "reserve",
        );
        assert_eq!(rate_limited.kind, ExecutionErrorKind::RateLimited);
        assert_eq!(rate_limited.safe_message(), "provider rate limited");
        assert_eq!(rate_limited.context["class"], json!("rate_limited"));

        let fatal = map_srt_error(
            ProviderError::OperationFailed {
                message: "seat unavailable".to_string(),
            },
            "reserve",
        );
        assert_eq!(fatal.kind, ExecutionErrorKind::Fatal);
        assert_eq!(fatal.safe_message(), "non-retryable execution failure");
        assert_eq!(fatal.context["class"], json!("operation_failed"));
    }

    #[test]
    fn map_srt_error_maps_auth_and_missing_session_failures() {
        let auth = map_srt_error(ProviderError::SessionExpired, "search_train");
        assert_eq!(auth.kind, ExecutionErrorKind::Transient);
        assert_eq!(auth.message, "provider authentication failed");
        assert_eq!(auth.context["class"], json!("auth"));

        let missing_session = map_srt_error(ProviderError::NotLoggedIn, "pay_with_card");
        assert_eq!(missing_session.kind, ExecutionErrorKind::Fatal);
        assert_eq!(
            missing_session.message,
            "provider login/session material missing"
        );
        assert_eq!(missing_session.context["class"], json!("missing_session"));
    }

    #[test]
    fn map_srt_error_maps_transport_and_unsupported_operation_failures() {
        let transport = map_srt_error(
            ProviderError::Transport {
                message: "network timeout".to_string(),
            },
            "search_train",
        );
        assert_eq!(transport.kind, ExecutionErrorKind::Transient);
        assert_eq!(transport.context["class"], json!("transport"));

        let unsupported = map_srt_error(
            ProviderError::UnsupportedOperation {
                operation: "mystery_op",
            },
            "clear",
        );
        assert_eq!(unsupported.kind, ExecutionErrorKind::Fatal);
        assert_eq!(unsupported.context["class"], json!("unsupported_operation"));
        assert!(unsupported.message.contains("mystery_op"));
    }

    #[test]
    fn parse_provider_execution_uses_provider_in_error_context() {
        let job = job_with_payload("runtime.mystery", json!({"operation": "mystery"}));
        let error =
            parse_provider_execution(&job, "ktx").expect_err("operation should be rejected");

        assert_eq!(error.kind, ExecutionErrorKind::Fatal);
        assert_eq!(error.context["provider"], json!("ktx"));
        assert_eq!(error.context["operation"], json!("mystery"));
        assert!(error.message.contains("unsupported provider operation"));
    }

    #[test]
    fn parse_pay_with_card_request_reports_provider_context() {
        let error = parse_pay_with_card_request(&json!({}), "ktx")
            .expect_err("missing reservation_id should fail");
        assert_eq!(error.kind, ExecutionErrorKind::Fatal);
        assert_eq!(error.context["provider"], json!("ktx"));
        assert_eq!(error.context["operation"], json!("pay_with_card"));
    }

    #[test]
    fn map_provider_error_preserves_provider_label() {
        let error = map_provider_error(ProviderError::Unauthorized, "ktx", "login");
        assert_eq!(error.kind, ExecutionErrorKind::Transient);
        assert_eq!(error.context["provider"], json!("ktx"));
        assert_eq!(error.context["operation"], json!("login"));
        assert_eq!(error.context["class"], json!("auth"));
    }

    #[test]
    fn dispatch_ktx_supports_search_train_flow() {
        let parsed = parsed_with_login(SrtOperationRequest::SearchTrain(SearchTrainRequest {
            dep_station_code: "0001".to_string(),
            arr_station_code: "0020".to_string(),
            dep_date: "20260305".to_string(),
            dep_time: "080000".to_string(),
            time_limit: None,
            passengers: vec![Passenger::adult(1)],
            available_only: true,
        }));

        let response = dispatch_ktx_with_client(&parsed, ReqwestKtxClient::deterministic())
            .expect("ktx deterministic search should succeed");
        assert_eq!(response.operation_name(), "search_train");
    }

    #[test]
    fn dispatch_ktx_marks_unsupported_operations_fatal() {
        let parsed = parsed_with_login(SrtOperationRequest::ReserveInfo(ReserveInfoRequest {
            reservation_id: "KTX-PNR-1".to_string(),
        }));

        let error = dispatch_ktx_with_client(&parsed, ReqwestKtxClient::deterministic())
            .expect_err("reserve_info should be unsupported for ktx");
        assert!(matches!(
            &error,
            ProviderError::UnsupportedOperation { operation } if *operation == "reserve_info"
        ));

        let mapped = map_provider_error(error, "ktx", "reserve_info");
        assert_eq!(mapped.kind, ExecutionErrorKind::Fatal);
        assert_eq!(mapped.context["class"], json!("unsupported_operation"));
        assert_eq!(mapped.context["provider"], json!("ktx"));
    }
}
