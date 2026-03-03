use chrono::{DateTime, Utc};
use serde_json::Value;
use sqlx::{Postgres, postgres::PgArguments, query::Query};

pub const PROVIDER_CONTRACT_LEDGER_INSERT_SQL: &str = "insert into provider_contract_ledger (job_id, provider, operation, request_idempotency_key, request_fingerprint, request_redacted, response_redacted, response_status_code, outcome, error_class, error_message_redacted, created_at) values ($1, $2, $3, $4, $5, cast($6 as jsonb), cast($7 as jsonb), $8, $9, $10, $11, $12) returning id, job_id, provider, operation, request_idempotency_key, request_fingerprint, request_redacted::text, response_redacted::text, response_status_code, outcome, error_class, error_message_redacted, created_at";
pub const PROVIDER_CONTRACT_LEDGER_SELECT_BY_JOB_SQL: &str = "select id, job_id, provider, operation, request_idempotency_key, request_fingerprint, request_redacted::text, response_redacted::text, response_status_code, outcome, error_class, error_message_redacted, created_at from provider_contract_ledger where job_id = $1 order by created_at asc";

#[derive(Debug, Clone)]
pub struct InsertProviderContractLedgerParams<'a> {
    pub job_id: Option<&'a str>,
    pub provider: &'a str,
    pub operation: &'a str,
    pub request_idempotency_key: Option<&'a str>,
    pub request_fingerprint: Option<&'a str>,
    pub request_redacted: &'a Value,
    pub response_redacted: &'a Value,
    pub response_status_code: Option<i32>,
    pub outcome: &'a str,
    pub error_class: Option<&'a str>,
    pub error_message_redacted: Option<&'a str>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ProviderContractLedgerRecord {
    pub id: i64,
    pub job_id: Option<String>,
    pub provider: String,
    pub operation: String,
    pub request_idempotency_key: Option<String>,
    pub request_fingerprint: Option<String>,
    pub request_redacted: Value,
    pub response_redacted: Value,
    pub response_status_code: Option<i32>,
    pub outcome: String,
    pub error_class: Option<String>,
    pub error_message_redacted: Option<String>,
    pub created_at: DateTime<Utc>,
}

pub trait ProviderContractLedgerRepoContract {
    fn insert_provider_contract_ledger_query<'q>(
        params: &'q InsertProviderContractLedgerParams<'q>,
    ) -> Query<'q, Postgres, PgArguments>;

    fn select_provider_contract_ledger_by_job_query<'q>(
        job_id: &'q str,
    ) -> Query<'q, Postgres, PgArguments>;
}

#[derive(Debug, Clone, Copy, Default)]
pub struct SqlProviderContractLedgerRepoContract;

impl ProviderContractLedgerRepoContract for SqlProviderContractLedgerRepoContract {
    fn insert_provider_contract_ledger_query<'q>(
        params: &'q InsertProviderContractLedgerParams<'q>,
    ) -> Query<'q, Postgres, PgArguments> {
        sqlx::query(PROVIDER_CONTRACT_LEDGER_INSERT_SQL)
            .bind(params.job_id)
            .bind(params.provider)
            .bind(params.operation)
            .bind(params.request_idempotency_key)
            .bind(params.request_fingerprint)
            .bind(params.request_redacted)
            .bind(params.response_redacted)
            .bind(params.response_status_code)
            .bind(params.outcome)
            .bind(params.error_class)
            .bind(params.error_message_redacted)
            .bind(params.created_at)
    }

    fn select_provider_contract_ledger_by_job_query<'q>(
        job_id: &'q str,
    ) -> Query<'q, Postgres, PgArguments> {
        sqlx::query(PROVIDER_CONTRACT_LEDGER_SELECT_BY_JOB_SQL).bind(job_id)
    }
}

pub fn insert_provider_contract_ledger_query<'q>(
    params: &'q InsertProviderContractLedgerParams<'q>,
) -> Query<'q, Postgres, PgArguments> {
    <SqlProviderContractLedgerRepoContract as ProviderContractLedgerRepoContract>::insert_provider_contract_ledger_query(params)
}

pub fn select_provider_contract_ledger_by_job_query<'q>(
    job_id: &'q str,
) -> Query<'q, Postgres, PgArguments> {
    <SqlProviderContractLedgerRepoContract as ProviderContractLedgerRepoContract>::select_provider_contract_ledger_by_job_query(job_id)
}
