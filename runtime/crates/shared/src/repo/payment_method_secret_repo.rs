use chrono::{DateTime, Utc};
use serde_json::Value;
use sqlx::{Postgres, postgres::PgArguments, query::Query};

pub const PAYMENT_METHOD_SECRET_UPSERT_SQL: &str = "insert into payment_method_secrets (provider, owner_ref, payment_method_ref, method_kind, card_brand, card_last4, card_exp_month, card_exp_year, payment_payload_envelope_ciphertext, payment_payload_envelope_dek_ciphertext, payment_payload_envelope_kek_version, payment_payload_envelope_aad_scope, payment_payload_envelope_aad_subject, payment_payload_envelope_aad_hash, redacted_metadata, updated_at, revoked_at) values ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, cast($15 as jsonb), $16, $17) on conflict (provider, owner_ref, payment_method_ref) do update set method_kind = excluded.method_kind, card_brand = excluded.card_brand, card_last4 = excluded.card_last4, card_exp_month = excluded.card_exp_month, card_exp_year = excluded.card_exp_year, payment_payload_envelope_ciphertext = excluded.payment_payload_envelope_ciphertext, payment_payload_envelope_dek_ciphertext = excluded.payment_payload_envelope_dek_ciphertext, payment_payload_envelope_kek_version = excluded.payment_payload_envelope_kek_version, payment_payload_envelope_aad_scope = excluded.payment_payload_envelope_aad_scope, payment_payload_envelope_aad_subject = excluded.payment_payload_envelope_aad_subject, payment_payload_envelope_aad_hash = excluded.payment_payload_envelope_aad_hash, redacted_metadata = excluded.redacted_metadata, updated_at = excluded.updated_at, revoked_at = excluded.revoked_at returning id, provider, owner_ref, payment_method_ref, method_kind, card_brand, card_last4, card_exp_month, card_exp_year, payment_payload_envelope_ciphertext, payment_payload_envelope_dek_ciphertext, payment_payload_envelope_kek_version, payment_payload_envelope_aad_scope, payment_payload_envelope_aad_subject, payment_payload_envelope_aad_hash, redacted_metadata::text, created_at, updated_at, revoked_at";
pub const PAYMENT_METHOD_SECRET_SELECT_ACTIVE_SQL: &str = "select id, provider, owner_ref, payment_method_ref, method_kind, card_brand, card_last4, card_exp_month, card_exp_year, payment_payload_envelope_ciphertext, payment_payload_envelope_dek_ciphertext, payment_payload_envelope_kek_version, payment_payload_envelope_aad_scope, payment_payload_envelope_aad_subject, payment_payload_envelope_aad_hash, redacted_metadata::text, created_at, updated_at, revoked_at from payment_method_secrets where provider = $1 and owner_ref = $2 and payment_method_ref = $3 and revoked_at is null";
pub const PAYMENT_METHOD_SECRET_REVOKE_SQL: &str = "update payment_method_secrets set revoked_at = $4, updated_at = $4 where provider = $1 and owner_ref = $2 and payment_method_ref = $3 and revoked_at is null";

#[derive(Debug, Clone)]
pub struct UpsertPaymentMethodSecretParams<'a> {
    pub provider: &'a str,
    pub owner_ref: &'a str,
    pub payment_method_ref: &'a str,
    pub method_kind: &'a str,
    pub card_brand: Option<&'a str>,
    pub card_last4: Option<&'a str>,
    pub card_exp_month: Option<i16>,
    pub card_exp_year: Option<i32>,
    pub payment_payload_envelope_ciphertext: &'a [u8],
    pub payment_payload_envelope_dek_ciphertext: &'a [u8],
    pub payment_payload_envelope_kek_version: i32,
    pub payment_payload_envelope_aad_scope: &'a str,
    pub payment_payload_envelope_aad_subject: &'a str,
    pub payment_payload_envelope_aad_hash: &'a [u8],
    pub redacted_metadata: &'a Value,
    pub updated_at: DateTime<Utc>,
    pub revoked_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct PaymentMethodSecretRecord {
    pub id: i64,
    pub provider: String,
    pub owner_ref: String,
    pub payment_method_ref: String,
    pub method_kind: String,
    pub card_brand: Option<String>,
    pub card_last4: Option<String>,
    pub card_exp_month: Option<i16>,
    pub card_exp_year: Option<i32>,
    pub payment_payload_envelope_ciphertext: Vec<u8>,
    pub payment_payload_envelope_dek_ciphertext: Vec<u8>,
    pub payment_payload_envelope_kek_version: i32,
    pub payment_payload_envelope_aad_scope: String,
    pub payment_payload_envelope_aad_subject: String,
    pub payment_payload_envelope_aad_hash: Vec<u8>,
    pub redacted_metadata: Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub revoked_at: Option<DateTime<Utc>>,
}

pub trait PaymentMethodSecretRepoContract {
    fn upsert_payment_method_secret_query<'q>(
        params: &'q UpsertPaymentMethodSecretParams<'q>,
    ) -> Query<'q, Postgres, PgArguments>;

    fn select_active_payment_method_secret_query<'q>(
        provider: &'q str,
        owner_ref: &'q str,
        payment_method_ref: &'q str,
    ) -> Query<'q, Postgres, PgArguments>;

    fn revoke_payment_method_secret_query<'q>(
        provider: &'q str,
        owner_ref: &'q str,
        payment_method_ref: &'q str,
        revoked_at: DateTime<Utc>,
    ) -> Query<'q, Postgres, PgArguments>;
}

#[derive(Debug, Clone, Copy, Default)]
pub struct SqlPaymentMethodSecretRepoContract;

impl PaymentMethodSecretRepoContract for SqlPaymentMethodSecretRepoContract {
    fn upsert_payment_method_secret_query<'q>(
        params: &'q UpsertPaymentMethodSecretParams<'q>,
    ) -> Query<'q, Postgres, PgArguments> {
        sqlx::query(PAYMENT_METHOD_SECRET_UPSERT_SQL)
            .bind(params.provider)
            .bind(params.owner_ref)
            .bind(params.payment_method_ref)
            .bind(params.method_kind)
            .bind(params.card_brand)
            .bind(params.card_last4)
            .bind(params.card_exp_month)
            .bind(params.card_exp_year)
            .bind(params.payment_payload_envelope_ciphertext)
            .bind(params.payment_payload_envelope_dek_ciphertext)
            .bind(params.payment_payload_envelope_kek_version)
            .bind(params.payment_payload_envelope_aad_scope)
            .bind(params.payment_payload_envelope_aad_subject)
            .bind(params.payment_payload_envelope_aad_hash)
            .bind(params.redacted_metadata)
            .bind(params.updated_at)
            .bind(params.revoked_at)
    }

    fn select_active_payment_method_secret_query<'q>(
        provider: &'q str,
        owner_ref: &'q str,
        payment_method_ref: &'q str,
    ) -> Query<'q, Postgres, PgArguments> {
        sqlx::query(PAYMENT_METHOD_SECRET_SELECT_ACTIVE_SQL)
            .bind(provider)
            .bind(owner_ref)
            .bind(payment_method_ref)
    }

    fn revoke_payment_method_secret_query<'q>(
        provider: &'q str,
        owner_ref: &'q str,
        payment_method_ref: &'q str,
        revoked_at: DateTime<Utc>,
    ) -> Query<'q, Postgres, PgArguments> {
        sqlx::query(PAYMENT_METHOD_SECRET_REVOKE_SQL)
            .bind(provider)
            .bind(owner_ref)
            .bind(payment_method_ref)
            .bind(revoked_at)
    }
}

pub fn upsert_payment_method_secret_query<'q>(
    params: &'q UpsertPaymentMethodSecretParams<'q>,
) -> Query<'q, Postgres, PgArguments> {
    <SqlPaymentMethodSecretRepoContract as PaymentMethodSecretRepoContract>::upsert_payment_method_secret_query(params)
}

pub fn select_active_payment_method_secret_query<'q>(
    provider: &'q str,
    owner_ref: &'q str,
    payment_method_ref: &'q str,
) -> Query<'q, Postgres, PgArguments> {
    <SqlPaymentMethodSecretRepoContract as PaymentMethodSecretRepoContract>::select_active_payment_method_secret_query(provider, owner_ref, payment_method_ref)
}

pub fn revoke_payment_method_secret_query<'q>(
    provider: &'q str,
    owner_ref: &'q str,
    payment_method_ref: &'q str,
    revoked_at: DateTime<Utc>,
) -> Query<'q, Postgres, PgArguments> {
    <SqlPaymentMethodSecretRepoContract as PaymentMethodSecretRepoContract>::revoke_payment_method_secret_query(provider, owner_ref, payment_method_ref, revoked_at)
}
