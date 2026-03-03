use chrono::{DateTime, Utc};
use serde_json::Value;
use sqlx::{Postgres, postgres::PgArguments, query::Query};

pub const PROVIDER_AUTH_SECRET_UPSERT_SQL: &str = "insert into provider_auth_secrets (provider, subject_ref, credential_kind, secret_envelope_ciphertext, secret_envelope_dek_ciphertext, secret_envelope_kek_version, secret_envelope_aad_scope, secret_envelope_aad_subject, secret_envelope_aad_hash, redacted_metadata, updated_at, rotated_at, revoked_at) values ($1, $2, $3, $4, $5, $6, $7, $8, $9, cast($10 as jsonb), $11, $12, $13) on conflict (provider, subject_ref, credential_kind) do update set secret_envelope_ciphertext = excluded.secret_envelope_ciphertext, secret_envelope_dek_ciphertext = excluded.secret_envelope_dek_ciphertext, secret_envelope_kek_version = excluded.secret_envelope_kek_version, secret_envelope_aad_scope = excluded.secret_envelope_aad_scope, secret_envelope_aad_subject = excluded.secret_envelope_aad_subject, secret_envelope_aad_hash = excluded.secret_envelope_aad_hash, redacted_metadata = excluded.redacted_metadata, updated_at = excluded.updated_at, rotated_at = excluded.rotated_at, revoked_at = excluded.revoked_at returning id, provider, subject_ref, credential_kind, secret_envelope_ciphertext, secret_envelope_dek_ciphertext, secret_envelope_kek_version, secret_envelope_aad_scope, secret_envelope_aad_subject, secret_envelope_aad_hash, redacted_metadata::text, created_at, updated_at, rotated_at, revoked_at";
pub const PROVIDER_AUTH_SECRET_SELECT_ACTIVE_SQL: &str = "select id, provider, subject_ref, credential_kind, secret_envelope_ciphertext, secret_envelope_dek_ciphertext, secret_envelope_kek_version, secret_envelope_aad_scope, secret_envelope_aad_subject, secret_envelope_aad_hash, redacted_metadata::text, created_at, updated_at, rotated_at, revoked_at from provider_auth_secrets where provider = $1 and subject_ref = $2 and credential_kind = $3 and revoked_at is null";
pub const PROVIDER_AUTH_SECRET_REVOKE_SQL: &str = "update provider_auth_secrets set revoked_at = $4, updated_at = $4 where provider = $1 and subject_ref = $2 and credential_kind = $3 and revoked_at is null";

#[derive(Debug, Clone)]
pub struct UpsertProviderAuthSecretParams<'a> {
    pub provider: &'a str,
    pub subject_ref: &'a str,
    pub credential_kind: &'a str,
    pub secret_envelope_ciphertext: &'a [u8],
    pub secret_envelope_dek_ciphertext: &'a [u8],
    pub secret_envelope_kek_version: i32,
    pub secret_envelope_aad_scope: &'a str,
    pub secret_envelope_aad_subject: &'a str,
    pub secret_envelope_aad_hash: &'a [u8],
    pub redacted_metadata: &'a Value,
    pub updated_at: DateTime<Utc>,
    pub rotated_at: Option<DateTime<Utc>>,
    pub revoked_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ProviderAuthSecretRecord {
    pub id: i64,
    pub provider: String,
    pub subject_ref: String,
    pub credential_kind: String,
    pub secret_envelope_ciphertext: Vec<u8>,
    pub secret_envelope_dek_ciphertext: Vec<u8>,
    pub secret_envelope_kek_version: i32,
    pub secret_envelope_aad_scope: String,
    pub secret_envelope_aad_subject: String,
    pub secret_envelope_aad_hash: Vec<u8>,
    pub redacted_metadata: Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub rotated_at: Option<DateTime<Utc>>,
    pub revoked_at: Option<DateTime<Utc>>,
}

pub trait ProviderAuthSecretRepoContract {
    fn upsert_provider_auth_secret_query<'q>(
        params: &'q UpsertProviderAuthSecretParams<'q>,
    ) -> Query<'q, Postgres, PgArguments>;

    fn select_active_provider_auth_secret_query<'q>(
        provider: &'q str,
        subject_ref: &'q str,
        credential_kind: &'q str,
    ) -> Query<'q, Postgres, PgArguments>;

    fn revoke_provider_auth_secret_query<'q>(
        provider: &'q str,
        subject_ref: &'q str,
        credential_kind: &'q str,
        revoked_at: DateTime<Utc>,
    ) -> Query<'q, Postgres, PgArguments>;
}

#[derive(Debug, Clone, Copy, Default)]
pub struct SqlProviderAuthSecretRepoContract;

impl ProviderAuthSecretRepoContract for SqlProviderAuthSecretRepoContract {
    fn upsert_provider_auth_secret_query<'q>(
        params: &'q UpsertProviderAuthSecretParams<'q>,
    ) -> Query<'q, Postgres, PgArguments> {
        sqlx::query(PROVIDER_AUTH_SECRET_UPSERT_SQL)
            .bind(params.provider)
            .bind(params.subject_ref)
            .bind(params.credential_kind)
            .bind(params.secret_envelope_ciphertext)
            .bind(params.secret_envelope_dek_ciphertext)
            .bind(params.secret_envelope_kek_version)
            .bind(params.secret_envelope_aad_scope)
            .bind(params.secret_envelope_aad_subject)
            .bind(params.secret_envelope_aad_hash)
            .bind(params.redacted_metadata)
            .bind(params.updated_at)
            .bind(params.rotated_at)
            .bind(params.revoked_at)
    }

    fn select_active_provider_auth_secret_query<'q>(
        provider: &'q str,
        subject_ref: &'q str,
        credential_kind: &'q str,
    ) -> Query<'q, Postgres, PgArguments> {
        sqlx::query(PROVIDER_AUTH_SECRET_SELECT_ACTIVE_SQL)
            .bind(provider)
            .bind(subject_ref)
            .bind(credential_kind)
    }

    fn revoke_provider_auth_secret_query<'q>(
        provider: &'q str,
        subject_ref: &'q str,
        credential_kind: &'q str,
        revoked_at: DateTime<Utc>,
    ) -> Query<'q, Postgres, PgArguments> {
        sqlx::query(PROVIDER_AUTH_SECRET_REVOKE_SQL)
            .bind(provider)
            .bind(subject_ref)
            .bind(credential_kind)
            .bind(revoked_at)
    }
}

pub fn upsert_provider_auth_secret_query<'q>(
    params: &'q UpsertProviderAuthSecretParams<'q>,
) -> Query<'q, Postgres, PgArguments> {
    <SqlProviderAuthSecretRepoContract as ProviderAuthSecretRepoContract>::upsert_provider_auth_secret_query(params)
}

pub fn select_active_provider_auth_secret_query<'q>(
    provider: &'q str,
    subject_ref: &'q str,
    credential_kind: &'q str,
) -> Query<'q, Postgres, PgArguments> {
    <SqlProviderAuthSecretRepoContract as ProviderAuthSecretRepoContract>::select_active_provider_auth_secret_query(provider, subject_ref, credential_kind)
}

pub fn revoke_provider_auth_secret_query<'q>(
    provider: &'q str,
    subject_ref: &'q str,
    credential_kind: &'q str,
    revoked_at: DateTime<Utc>,
) -> Query<'q, Postgres, PgArguments> {
    <SqlProviderAuthSecretRepoContract as ProviderAuthSecretRepoContract>::revoke_provider_auth_secret_query(provider, subject_ref, credential_kind, revoked_at)
}
