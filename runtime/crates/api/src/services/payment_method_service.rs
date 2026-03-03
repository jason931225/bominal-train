use std::{collections::BTreeMap, env};

use bominal_shared::{
    crypto::{EnvelopeAad, EnvelopeCipher, PayloadKind, ServerEnvelopeCipher, StaticKeyring},
    repo::{UpsertPaymentMethodSecretParams, upsert_payment_method_secret_query},
};
use chrono::Utc;
use tracing::{error, warn};
use uuid::Uuid;

use super::super::AppState;

const TEST_MASTER_KEY_B64_FALLBACK: &str = "MDEyMzQ1Njc4OWFiY2RlZjAxMjM0NTY3ODlhYmNkZWY=";
const PAYMENT_CRYPTO_SERVICE_STORE_PATH: &str = "/v1/payment-methods/store";

#[derive(Debug, serde::Deserialize)]
pub(crate) struct PutSrtPaymentMethodRequest {
    #[serde(default)]
    pub(crate) owner_ref: Option<String>,
    #[serde(default)]
    pub(crate) payment_method_ref: Option<String>,
    pub(crate) pan_ciphertext: String,
    pub(crate) expiry_month_ciphertext: String,
    pub(crate) expiry_year_ciphertext: String,
    pub(crate) birth_or_business_number_ciphertext: String,
    pub(crate) card_password_two_digits_ciphertext: String,
}

#[derive(Debug, serde::Serialize)]
pub(crate) struct PutSrtPaymentMethodResult {
    pub(crate) accepted: bool,
    pub(crate) provider: &'static str,
    pub(crate) payment_method_ref: String,
    pub(crate) contract: &'static str,
}

#[derive(Debug)]
pub(crate) enum PutSrtPaymentMethodError {
    ValidationFailed,
    PersistenceUnavailable,
    CryptoUnavailable,
    PersistenceFailure,
}

#[derive(Debug, serde::Serialize)]
struct CloudRunStoreRequest<'a> {
    provider: &'a str,
    owner_ref: &'a str,
    payment_method_ref: &'a str,
    ev_payload: CloudRunEVPayload<'a>,
    metadata: CloudRunMetadata<'a>,
}

#[derive(Debug, serde::Serialize)]
struct CloudRunEVPayload<'a> {
    pan_ev: &'a str,
    expiry_month_ev: &'a str,
    expiry_year_ev: &'a str,
    birth_or_business_ev: &'a str,
    card_password_two_digits_ev: &'a str,
}

#[derive(Debug, serde::Serialize)]
struct CloudRunMetadata<'a> {
    #[serde(skip_serializing_if = "Option::is_none")]
    brand: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    last4: Option<&'a str>,
}

#[derive(Debug, serde::Deserialize, Default)]
struct CloudRunStoreResponse {
    #[serde(default)]
    ok: bool,
    #[serde(default)]
    payment_method_ref: String,
    #[serde(default)]
    storage_mode: String,
    #[serde(default)]
    detail: String,
}

pub(crate) async fn put_srt_payment_method(
    state: &AppState,
    payload: PutSrtPaymentMethodRequest,
) -> Result<PutSrtPaymentMethodResult, PutSrtPaymentMethodError> {
    validate_payment_payload(&payload)?;

    let owner_ref = payload
        .owner_ref
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToOwned::to_owned)
        .unwrap_or_else(|| format!("srt-owner-{}", Uuid::new_v4()));

    let payment_method_ref = payload
        .payment_method_ref
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToOwned::to_owned)
        .unwrap_or_else(|| format!("srt_pm_{}", Uuid::new_v4()));

    if let Some(base_url) = payment_crypto_service_base_url() {
        return store_via_payment_crypto_service(
            state,
            payload,
            owner_ref.as_str(),
            payment_method_ref.as_str(),
            base_url.as_str(),
        )
        .await;
    }

    let Some(pool) = state.db_pool.as_ref() else {
        return Err(PutSrtPaymentMethodError::PersistenceUnavailable);
    };

    let aad = EnvelopeAad {
        payload_kind: PayloadKind::ProviderPayment,
        provider: Some("srt".to_string()),
        subject_id: Some(owner_ref.clone()),
        scope: "srt:payment-method:card".to_string(),
        metadata: BTreeMap::from([("payment_method_ref".to_string(), payment_method_ref.clone())]),
    };
    let plaintext = serde_json::to_vec(&serde_json::json!({
        "pan_ciphertext": payload.pan_ciphertext,
        "expiry_month_ciphertext": payload.expiry_month_ciphertext,
        "expiry_year_ciphertext": payload.expiry_year_ciphertext,
        "birth_or_business_number_ciphertext": payload.birth_or_business_number_ciphertext,
        "card_password_two_digits_ciphertext": payload.card_password_two_digits_ciphertext
    }))
    .map_err(|_| PutSrtPaymentMethodError::ValidationFailed)?;

    let cipher = build_envelope_cipher_from_env(state)?;
    let encrypted = cipher
        .encrypt(&plaintext, aad.clone())
        .map_err(|_| PutSrtPaymentMethodError::CryptoUnavailable)?;
    let aad_hash =
        serde_json::to_vec(&aad).map_err(|_| PutSrtPaymentMethodError::CryptoUnavailable)?;
    let key_version = i32::try_from(encrypted.key_version)
        .map_err(|_| PutSrtPaymentMethodError::CryptoUnavailable)?;
    let now = Utc::now();
    let redacted_metadata = serde_json::json!({
        "provider": "srt",
        "method_kind": "card",
        "contract": "ciphertext-only-v1"
    });

    let params = UpsertPaymentMethodSecretParams {
        provider: "srt",
        owner_ref: owner_ref.as_str(),
        payment_method_ref: payment_method_ref.as_str(),
        method_kind: "card",
        card_brand: None,
        card_last4: None,
        card_exp_month: None,
        card_exp_year: None,
        payment_payload_envelope_ciphertext: encrypted.ciphertext.as_slice(),
        payment_payload_envelope_dek_ciphertext: &encrypted.nonce,
        payment_payload_envelope_kek_version: key_version,
        payment_payload_envelope_aad_scope: aad.scope.as_str(),
        payment_payload_envelope_aad_subject: owner_ref.as_str(),
        payment_payload_envelope_aad_hash: aad_hash.as_slice(),
        redacted_metadata: &redacted_metadata,
        updated_at: now,
        revoked_at: None,
    };

    if let Err(err) = upsert_payment_method_secret_query(&params)
        .execute(pool)
        .await
    {
        error!(error = %err, "failed to persist payment method secret");
        return Err(PutSrtPaymentMethodError::PersistenceFailure);
    }

    Ok(PutSrtPaymentMethodResult {
        accepted: true,
        provider: "srt",
        payment_method_ref,
        contract: "ciphertext-only-v1",
    })
}

async fn store_via_payment_crypto_service(
    state: &AppState,
    payload: PutSrtPaymentMethodRequest,
    owner_ref: &str,
    payment_method_ref: &str,
    base_url: &str,
) -> Result<PutSrtPaymentMethodResult, PutSrtPaymentMethodError> {
    let url = format!(
        "{}{}",
        trim_trailing_slash(base_url),
        PAYMENT_CRYPTO_SERVICE_STORE_PATH
    );
    let request_body = CloudRunStoreRequest {
        provider: "srt",
        owner_ref,
        payment_method_ref,
        ev_payload: CloudRunEVPayload {
            pan_ev: payload.pan_ciphertext.as_str(),
            expiry_month_ev: payload.expiry_month_ciphertext.as_str(),
            expiry_year_ev: payload.expiry_year_ciphertext.as_str(),
            birth_or_business_ev: payload.birth_or_business_number_ciphertext.as_str(),
            card_password_two_digits_ev: payload.card_password_two_digits_ciphertext.as_str(),
        },
        metadata: CloudRunMetadata {
            brand: None,
            last4: None,
        },
    };

    let mut request_builder = state.http_client.post(url).json(&request_body);
    if let Some(token) = payment_crypto_service_token() {
        request_builder = request_builder.header("x-internal-service-token", token);
    }

    let response = request_builder.send().await.map_err(|err| {
        error!(error = %err, "payment-crypto service request failed");
        PutSrtPaymentMethodError::CryptoUnavailable
    })?;
    let status = response.status();
    let decoded = response
        .json::<CloudRunStoreResponse>()
        .await
        .unwrap_or_else(|_| CloudRunStoreResponse::default());

    if !status.is_success() || !decoded.ok {
        let detail = redact_detail(decoded.detail.as_str());
        if status.is_client_error() {
            warn!(
                status = %status,
                detail = %detail,
                "payment-crypto service rejected request"
            );
            return Err(PutSrtPaymentMethodError::ValidationFailed);
        }

        error!(
            status = %status,
            detail = %detail,
            "payment-crypto service unavailable"
        );
        return Err(PutSrtPaymentMethodError::CryptoUnavailable);
    }

    let resolved_ref = if decoded.payment_method_ref.trim().is_empty() {
        payment_method_ref.to_string()
    } else {
        decoded.payment_method_ref
    };
    let _storage_mode = decoded.storage_mode;

    Ok(PutSrtPaymentMethodResult {
        accepted: true,
        provider: "srt",
        payment_method_ref: resolved_ref,
        contract: "kms-envelope-over-evervault-v1",
    })
}

fn validate_payment_payload(
    payload: &PutSrtPaymentMethodRequest,
) -> Result<(), PutSrtPaymentMethodError> {
    if payload.pan_ciphertext.trim().is_empty()
        || payload.expiry_month_ciphertext.trim().is_empty()
        || payload.expiry_year_ciphertext.trim().is_empty()
        || payload
            .birth_or_business_number_ciphertext
            .trim()
            .is_empty()
        || payload
            .card_password_two_digits_ciphertext
            .trim()
            .is_empty()
    {
        return Err(PutSrtPaymentMethodError::ValidationFailed);
    }

    Ok(())
}

fn payment_crypto_service_base_url() -> Option<String> {
    env::var("PAYMENT_CRYPTO_SERVICE_URL")
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
}

fn payment_crypto_service_token() -> Option<String> {
    env::var("PAYMENT_CRYPTO_SERVICE_TOKEN")
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
}

fn trim_trailing_slash(raw: &str) -> String {
    raw.trim_end_matches('/').to_string()
}

fn redact_detail(raw: &str) -> String {
    let cleaned = raw.trim();
    if cleaned.is_empty() {
        return "upstream_error".to_string();
    }
    cleaned.chars().take(120).collect()
}

fn build_envelope_cipher_from_env(
    state: &AppState,
) -> Result<ServerEnvelopeCipher, PutSrtPaymentMethodError> {
    let key_version = env::var("KEK_VERSION")
        .ok()
        .and_then(|value| value.trim().parse::<u32>().ok())
        .filter(|value| *value > 0)
        .unwrap_or(1);

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
            if normalize_env(&state.config.app_env) == "test" {
                Some(TEST_MASTER_KEY_B64_FALLBACK.to_string())
            } else {
                None
            }
        })
        .ok_or(PutSrtPaymentMethodError::CryptoUnavailable)?;

    let key_bytes = decode_base64(encoded_master_key.as_str())?;
    if key_bytes.len() != 32 {
        return Err(PutSrtPaymentMethodError::CryptoUnavailable);
    }

    let mut keys = BTreeMap::new();
    keys.insert(key_version, key_bytes);

    let keyring = StaticKeyring::new(key_version, keys)
        .map_err(|_| PutSrtPaymentMethodError::CryptoUnavailable)?;
    Ok(ServerEnvelopeCipher::new(keyring))
}

fn decode_base64(input: &str) -> Result<Vec<u8>, PutSrtPaymentMethodError> {
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
            return Err(PutSrtPaymentMethodError::CryptoUnavailable);
        }

        let sextet = match byte {
            b'A'..=b'Z' => (byte - b'A') as u32,
            b'a'..=b'z' => (byte - b'a' + 26) as u32,
            b'0'..=b'9' => (byte - b'0' + 52) as u32,
            b'+' | b'-' => 62,
            b'/' | b'_' => 63,
            _ => return Err(PutSrtPaymentMethodError::CryptoUnavailable),
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
        return Err(PutSrtPaymentMethodError::CryptoUnavailable);
    }

    Ok(out)
}

fn normalize_env(raw: &str) -> String {
    raw.trim().to_ascii_lowercase()
}
