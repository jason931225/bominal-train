use std::env;

use anyhow::Result;
use bominal_shared::{
    crypto::{RedactionMode, redact_json},
    providers::ProviderAdapter,
    providers::srt::{
        CancelRequest, CardIdentityType, ClearRequest, GetReservationsRequest, LoginAccountType,
        LoginRequest, LogoutRequest, Passenger, PayWithCardRequest, RefundRequest,
        ReqwestSrtClient, ReserveInfoRequest, ReserveRequest, ReserveStandbyOptionSettingsRequest,
        ReserveStandbyRequest, SearchTrainRequest, SecretString, SrtClientFailureKind,
        SrtOperationRequest, SrtOperationResponse, SrtProviderAdapter, SrtProviderError,
        TicketInfoRequest,
    },
};
use chrono::Utc;
use serde_json::{Value, json};
use sqlx::{PgPool, Row};

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
struct ParsedSrtExecution {
    request: SrtOperationRequest,
    login_material: Option<LoginRequest>,
    simulated_failure: Option<SrtClientFailureKind>,
}

impl ProviderExecutor {
    pub async fn execute(
        &self,
        job: &ClaimedRuntimeJob,
        payment_policy: &PaymentExecutionPolicy,
    ) -> std::result::Result<ExecutionSuccess, ExecutionError> {
        let provider = job.inferred_provider();
        if provider != "srt" {
            return Err(ExecutionError::new(
                ExecutionErrorKind::UnsupportedProvider,
                "provider execution hook not implemented",
                json!({"provider": provider}),
            ));
        }

        self.execute_srt(job, payment_policy).await
    }

    async fn execute_srt(
        &self,
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

        let parsed = parse_srt_execution(job)?;
        let operation_name = parsed.request.operation_name();
        let operation = parsed.request.operation();
        let requires_login = operation_requires_login_material(&parsed.request);

        let mut client = ReqwestSrtClient::deterministic();
        if let Some(failure_kind) = parsed.simulated_failure {
            client = client.with_failure(operation, failure_kind, 1);
        }
        let mut adapter = SrtProviderAdapter::new(client);

        if requires_login {
            let Some(login_request) = parsed.login_material else {
                return Err(ExecutionError::new(
                    ExecutionErrorKind::Fatal,
                    "provider login/session material missing",
                    json!({"provider": "srt", "operation": operation_name}),
                ));
            };

            adapter
                .login(login_request)
                .map_err(|error| map_srt_error(error, operation_name))?;
        }

        let response = adapter
            .dispatch(parsed.request)
            .map_err(|error| map_srt_error(error, operation_name))?;
        let result_redacted = build_redacted_result(job, &response);

        Ok(ExecutionSuccess {
            provider: "srt".to_string(),
            operation: operation_name.to_string(),
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

fn map_srt_error(error: SrtProviderError, operation_name: &str) -> ExecutionError {
    match error {
        SrtProviderError::Transport { message } => ExecutionError::new(
            ExecutionErrorKind::Transient,
            message,
            json!({"provider": "srt", "operation": operation_name, "class": "transport"}),
        ),
        SrtProviderError::SessionExpired | SrtProviderError::Unauthorized => ExecutionError::new(
            ExecutionErrorKind::Transient,
            "provider authentication failed",
            json!({"provider": "srt", "operation": operation_name, "class": "auth"}),
        ),
        SrtProviderError::NotLoggedIn | SrtProviderError::ReloginUnavailable => {
            ExecutionError::new(
                ExecutionErrorKind::Fatal,
                "provider login/session material missing",
                json!({"provider": "srt", "operation": operation_name, "class": "missing_session"}),
            )
        }
        SrtProviderError::OperationFailed { message } => {
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
                json!({"provider": "srt", "operation": operation_name, "class": class}),
            )
        }
        SrtProviderError::UnsupportedOperation { operation } => ExecutionError::new(
            ExecutionErrorKind::Fatal,
            format!("provider operation '{operation}' is not supported"),
            json!({"provider": "srt", "operation": operation_name, "class": "unsupported_operation"}),
        ),
    }
}

fn parse_srt_execution(
    job: &ClaimedRuntimeJob,
) -> std::result::Result<ParsedSrtExecution, ExecutionError> {
    let raw_operation = infer_operation_token(job);
    let Some(operation) = canonical_operation_name(&raw_operation) else {
        return Err(ExecutionError::new(
            ExecutionErrorKind::Fatal,
            "unsupported srt operation in runtime payload",
            json!({"provider": "srt", "operation": raw_operation}),
        ));
    };

    let payload = &job.payload;
    let request_payload = operation_payload(payload);
    let login_material = parse_optional_login_request(payload)?;
    let simulated_failure = parse_simulated_failure(payload);

    let request = match operation {
        "login" => {
            let login_request = match login_material.clone() {
                Some(login) => login,
                None => parse_login_request(request_payload)?,
            };
            SrtOperationRequest::Login(login_request)
        }
        "logout" => SrtOperationRequest::Logout(LogoutRequest::default()),
        "search_train" => {
            SrtOperationRequest::SearchTrain(parse_search_train_request(request_payload)?)
        }
        "reserve" => SrtOperationRequest::Reserve(parse_reserve_request(request_payload)?),
        "reserve_standby" => {
            SrtOperationRequest::ReserveStandby(parse_reserve_standby_request(request_payload)?)
        }
        "reserve_standby_option_settings" => SrtOperationRequest::ReserveStandbyOptionSettings(
            parse_reserve_standby_option_settings_request(request_payload)?,
        ),
        "get_reservations" => {
            let request = parse_optional_get_reservations_request(request_payload)?;
            SrtOperationRequest::GetReservations(request)
        }
        "ticket_info" => {
            SrtOperationRequest::TicketInfo(parse_ticket_info_request(request_payload)?)
        }
        "cancel" => SrtOperationRequest::Cancel(parse_cancel_request(request_payload)?),
        "pay_with_card" => {
            SrtOperationRequest::PayWithCard(parse_pay_with_card_request(request_payload)?)
        }
        "reserve_info" => {
            SrtOperationRequest::ReserveInfo(parse_reserve_info_request(request_payload)?)
        }
        "refund" => SrtOperationRequest::Refund(parse_refund_request(request_payload)?),
        "clear" => SrtOperationRequest::Clear(ClearRequest),
        _ => {
            return Err(ExecutionError::new(
                ExecutionErrorKind::Fatal,
                "unsupported srt operation in runtime payload",
                json!({"provider": "srt", "operation": raw_operation}),
            ));
        }
    };

    Ok(ParsedSrtExecution {
        request,
        login_material,
        simulated_failure,
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
) -> std::result::Result<Option<LoginRequest>, ExecutionError> {
    let login_value = payload
        .get("login")
        .or_else(|| payload.get("session").and_then(|value| value.get("login")));

    match login_value {
        Some(value) => parse_login_request(value).map(Some),
        None => Ok(None),
    }
}

fn parse_login_request(value: &Value) -> std::result::Result<LoginRequest, ExecutionError> {
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
            json!({"provider": "srt", "operation": "login", "class": "missing_login_fields"}),
        ));
    };
    let Some(password) = password else {
        return Err(ExecutionError::new(
            ExecutionErrorKind::Fatal,
            "provider login/session material missing",
            json!({"provider": "srt", "operation": "login", "class": "missing_login_fields"}),
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
) -> std::result::Result<SearchTrainRequest, ExecutionError> {
    let dep_station_code =
        required_string_field(value, &["dep_station_code", "from"], "search_train")?;
    let arr_station_code =
        required_string_field(value, &["arr_station_code", "to"], "search_train")?;
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
                json!({"provider": "srt", "operation": "search_train", "field": "passengers"}),
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
) -> std::result::Result<PayWithCardRequest, ExecutionError> {
    let reservation_id = required_string_field(value, &["reservation_id"], "pay_with_card")?;
    let card_number = required_string_field(
        value,
        &["card_number", "pan", "pan_ciphertext"],
        "pay_with_card",
    )?;
    let card_password_two_digits = required_string_field(
        value,
        &[
            "card_password_two_digits",
            "card_password",
            "card_password_two_digits_ciphertext",
        ],
        "pay_with_card",
    )?;
    let card_validation_number = required_string_field(
        value,
        &[
            "card_validation_number",
            "birth_or_business_number",
            "birth_or_business_number_ciphertext",
        ],
        "pay_with_card",
    )?;
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
    .ok_or_else(|| {
        ExecutionError::new(
            ExecutionErrorKind::Fatal,
            "invalid pay_with_card request payload",
            json!({"provider": "srt", "operation": "pay_with_card"}),
        )
    })?;
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

fn parse_reserve_request(value: &Value) -> std::result::Result<ReserveRequest, ExecutionError> {
    serde_json::from_value(value.clone()).map_err(|error| {
        ExecutionError::new(
            ExecutionErrorKind::Fatal,
            format!("invalid reserve request payload: {error}"),
            json!({"provider": "srt", "operation": "reserve"}),
        )
    })
}

fn parse_reserve_standby_request(
    value: &Value,
) -> std::result::Result<ReserveStandbyRequest, ExecutionError> {
    serde_json::from_value(value.clone()).map_err(|error| {
        ExecutionError::new(
            ExecutionErrorKind::Fatal,
            format!("invalid reserve_standby request payload: {error}"),
            json!({"provider": "srt", "operation": "reserve_standby"}),
        )
    })
}

fn parse_reserve_standby_option_settings_request(
    value: &Value,
) -> std::result::Result<ReserveStandbyOptionSettingsRequest, ExecutionError> {
    serde_json::from_value(value.clone()).map_err(|error| {
        ExecutionError::new(
            ExecutionErrorKind::Fatal,
            format!("invalid reserve_standby_option_settings request payload: {error}"),
            json!({"provider": "srt", "operation": "reserve_standby_option_settings"}),
        )
    })
}

fn parse_ticket_info_request(
    value: &Value,
) -> std::result::Result<TicketInfoRequest, ExecutionError> {
    serde_json::from_value(value.clone()).map_err(|error| {
        ExecutionError::new(
            ExecutionErrorKind::Fatal,
            format!("invalid ticket_info request payload: {error}"),
            json!({"provider": "srt", "operation": "ticket_info"}),
        )
    })
}

fn parse_cancel_request(value: &Value) -> std::result::Result<CancelRequest, ExecutionError> {
    serde_json::from_value(value.clone()).map_err(|error| {
        ExecutionError::new(
            ExecutionErrorKind::Fatal,
            format!("invalid cancel request payload: {error}"),
            json!({"provider": "srt", "operation": "cancel"}),
        )
    })
}

fn parse_reserve_info_request(
    value: &Value,
) -> std::result::Result<ReserveInfoRequest, ExecutionError> {
    serde_json::from_value(value.clone()).map_err(|error| {
        ExecutionError::new(
            ExecutionErrorKind::Fatal,
            format!("invalid reserve_info request payload: {error}"),
            json!({"provider": "srt", "operation": "reserve_info"}),
        )
    })
}

fn parse_refund_request(value: &Value) -> std::result::Result<RefundRequest, ExecutionError> {
    serde_json::from_value(value.clone()).map_err(|error| {
        ExecutionError::new(
            ExecutionErrorKind::Fatal,
            format!("invalid refund request payload: {error}"),
            json!({"provider": "srt", "operation": "refund"}),
        )
    })
}

fn required_string_field(
    value: &Value,
    keys: &[&str],
    operation: &str,
) -> std::result::Result<String, ExecutionError> {
    optional_string_field(value, keys).ok_or_else(|| {
        ExecutionError::new(
            ExecutionErrorKind::Fatal,
            format!("invalid {operation} request payload"),
            json!({"provider": "srt", "operation": operation}),
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
    fn parse_srt_execution_uses_aliases_from_payload_or_kind() {
        let search_job = job_with_payload(
            "runtime.search",
            json!({
                "operation": "train-search",
                "dep_station_code": "0551",
                "arr_station_code": "0020",
            }),
        );
        let parsed_search = parse_srt_execution(&search_job).expect("search alias should parse");
        assert_eq!(parsed_search.request.operation_name(), "search_train");

        let clear_job = job_with_payload("runtime.clear", json!({}));
        let parsed_clear = parse_srt_execution(&clear_job).expect("kind fallback should parse");
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
            SrtProviderError::OperationFailed {
                message: "RATE_LIMITED by provider".to_string(),
            },
            "reserve",
        );
        assert_eq!(rate_limited.kind, ExecutionErrorKind::RateLimited);
        assert_eq!(rate_limited.safe_message(), "provider rate limited");
        assert_eq!(rate_limited.context["class"], json!("rate_limited"));

        let fatal = map_srt_error(
            SrtProviderError::OperationFailed {
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
        let auth = map_srt_error(SrtProviderError::SessionExpired, "search_train");
        assert_eq!(auth.kind, ExecutionErrorKind::Transient);
        assert_eq!(auth.message, "provider authentication failed");
        assert_eq!(auth.context["class"], json!("auth"));

        let missing_session = map_srt_error(SrtProviderError::NotLoggedIn, "pay_with_card");
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
            SrtProviderError::Transport {
                message: "network timeout".to_string(),
            },
            "search_train",
        );
        assert_eq!(transport.kind, ExecutionErrorKind::Transient);
        assert_eq!(transport.context["class"], json!("transport"));

        let unsupported = map_srt_error(
            SrtProviderError::UnsupportedOperation {
                operation: "mystery_op",
            },
            "clear",
        );
        assert_eq!(unsupported.kind, ExecutionErrorKind::Fatal);
        assert_eq!(unsupported.context["class"], json!("unsupported_operation"));
        assert!(unsupported.message.contains("mystery_op"));
    }
}
