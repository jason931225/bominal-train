use std::{collections::BTreeMap, env};

use bominal_shared::{
    crypto::{EnvelopeAad, EnvelopeCipher, PayloadKind, ServerEnvelopeCipher, StaticKeyring},
    repo::{UpsertPaymentMethodSecretParams, upsert_payment_method_secret_query},
};
use chrono::Utc;
use sha2::Digest;
use tracing::{error, warn};
use uuid::Uuid;

use super::super::AppState;

const TEST_MASTER_KEY_B64_FALLBACK: &str = "MDEyMzQ1Njc4OWFiY2RlZjAxMjM0NTY3ODlhYmNkZWY=";
const PAYMENT_CRYPTO_SERVICE_STORE_PATH: &str = "/v1/payment-methods/store";
pub(crate) const UNIVERSAL_PAYMENT_PROVIDER: &str = "universal";

#[derive(Debug, serde::Deserialize)]
pub(crate) struct PutProviderPaymentMethodRequest {
    #[serde(default)]
    pub(crate) owner_ref: Option<String>,
    #[serde(default)]
    pub(crate) payment_method_ref: Option<String>,
    #[serde(default)]
    pub(crate) card_brand: Option<String>,
    #[serde(default)]
    pub(crate) card_last4: String,
    pub(crate) pan_ciphertext: String,
    pub(crate) expiry_month_ciphertext: String,
    pub(crate) expiry_year_ciphertext: String,
    pub(crate) birth_or_business_number_ciphertext: String,
    pub(crate) card_password_two_digits_ciphertext: String,
}

#[derive(Debug, serde::Serialize)]
pub(crate) struct PutProviderPaymentMethodResult {
    pub(crate) accepted: bool,
    pub(crate) provider: String,
    pub(crate) payment_method_ref: String,
    pub(crate) contract: &'static str,
}

#[derive(Debug)]
pub(crate) enum PutProviderPaymentMethodError {
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

pub(crate) async fn put_provider_payment_method(
    state: &AppState,
    provider: &str,
    payload: PutProviderPaymentMethodRequest,
) -> Result<PutProviderPaymentMethodResult, PutProviderPaymentMethodError> {
    let input_provider =
        canonical_provider(provider).ok_or(PutProviderPaymentMethodError::ValidationFailed)?;
    validate_payment_payload(&payload)?;
    let card_last4 = normalize_card_last4(&payload.card_last4)?;
    let card_brand = payload
        .card_brand
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToOwned::to_owned);
    let storage_provider = UNIVERSAL_PAYMENT_PROVIDER;

    let owner_ref = payload
        .owner_ref
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToOwned::to_owned)
        .unwrap_or_else(|| format!("payment-owner-{}", Uuid::new_v4()));

    let payment_method_ref = payload
        .payment_method_ref
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToOwned::to_owned)
        .unwrap_or_else(|| format!("pm_{}", Uuid::new_v4()));

    if let Some(base_url) = payment_crypto_service_base_url() {
        return store_via_payment_crypto_service(
            state,
            StoreViaPaymentCryptoServiceInput {
                provider: input_provider,
                payload,
                card_brand: card_brand.as_deref(),
                card_last4: card_last4.as_str(),
                owner_ref: owner_ref.as_str(),
                payment_method_ref: payment_method_ref.as_str(),
                base_url: base_url.as_str(),
            },
        )
        .await;
    }

    let Some(pool) = state.db_pool.as_ref() else {
        return Err(PutProviderPaymentMethodError::PersistenceUnavailable);
    };

    let aad = EnvelopeAad {
        payload_kind: PayloadKind::ProviderPayment,
        provider: Some(storage_provider.to_string()),
        subject_id: Some(owner_ref.clone()),
        scope: format!("{storage_provider}:payment-method:card"),
        metadata: BTreeMap::from([("payment_method_ref".to_string(), payment_method_ref.clone())]),
    };
    let plaintext = serde_json::to_vec(&serde_json::json!({
        "pan_ciphertext": payload.pan_ciphertext,
        "expiry_month_ciphertext": payload.expiry_month_ciphertext,
        "expiry_year_ciphertext": payload.expiry_year_ciphertext,
        "birth_or_business_number_ciphertext": payload.birth_or_business_number_ciphertext,
        "card_password_two_digits_ciphertext": payload.card_password_two_digits_ciphertext
    }))
    .map_err(|_| PutProviderPaymentMethodError::ValidationFailed)?;

    let cipher = build_envelope_cipher_from_env(state)?;
    let encrypted = cipher
        .encrypt(&plaintext, aad.clone())
        .map_err(|_| PutProviderPaymentMethodError::CryptoUnavailable)?;
    let aad_bytes =
        serde_json::to_vec(&aad).map_err(|_| PutProviderPaymentMethodError::CryptoUnavailable)?;
    let aad_hash = sha2::Sha256::digest(aad_bytes.as_slice());
    let key_version = i32::try_from(encrypted.key_version)
        .map_err(|_| PutProviderPaymentMethodError::CryptoUnavailable)?;
    let now = Utc::now();
    let redacted_metadata = serde_json::json!({
        "provider": storage_provider,
        "method_kind": "card",
        "card_last4": card_last4,
        "card_brand": card_brand,
        "contract": "ciphertext-only-v1"
    });

    let params = UpsertPaymentMethodSecretParams {
        provider: storage_provider,
        owner_ref: owner_ref.as_str(),
        payment_method_ref: payment_method_ref.as_str(),
        method_kind: "card",
        card_brand: card_brand.as_deref(),
        card_last4: Some(card_last4.as_str()),
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
        return Err(PutProviderPaymentMethodError::PersistenceFailure);
    }

    Ok(PutProviderPaymentMethodResult {
        accepted: true,
        provider: storage_provider.to_string(),
        payment_method_ref,
        contract: "ciphertext-only-v1",
    })
}

struct StoreViaPaymentCryptoServiceInput<'a> {
    provider: &'a str,
    payload: PutProviderPaymentMethodRequest,
    card_brand: Option<&'a str>,
    card_last4: &'a str,
    owner_ref: &'a str,
    payment_method_ref: &'a str,
    base_url: &'a str,
}

async fn store_via_payment_crypto_service(
    state: &AppState,
    input: StoreViaPaymentCryptoServiceInput<'_>,
) -> Result<PutProviderPaymentMethodResult, PutProviderPaymentMethodError> {
    let url = format!(
        "{}{}",
        trim_trailing_slash(input.base_url),
        PAYMENT_CRYPTO_SERVICE_STORE_PATH
    );
    let request_body = CloudRunStoreRequest {
        provider: input.provider,
        owner_ref: input.owner_ref,
        payment_method_ref: input.payment_method_ref,
        ev_payload: CloudRunEVPayload {
            pan_ev: input.payload.pan_ciphertext.as_str(),
            expiry_month_ev: input.payload.expiry_month_ciphertext.as_str(),
            expiry_year_ev: input.payload.expiry_year_ciphertext.as_str(),
            birth_or_business_ev: input.payload.birth_or_business_number_ciphertext.as_str(),
            card_password_two_digits_ev: input.payload.card_password_two_digits_ciphertext.as_str(),
        },
        metadata: CloudRunMetadata {
            brand: input.card_brand,
            last4: Some(input.card_last4),
        },
    };

    let mut request_builder = state.http_client.post(url).json(&request_body);
    if let Some(token) = payment_crypto_service_token() {
        request_builder = request_builder.header("x-internal-service-token", token);
    }

    let response = request_builder.send().await.map_err(|err| {
        error!(error = %err, "payment-crypto service request failed");
        PutProviderPaymentMethodError::CryptoUnavailable
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
            return Err(PutProviderPaymentMethodError::ValidationFailed);
        }

        error!(
            status = %status,
            detail = %detail,
            "payment-crypto service unavailable"
        );
        return Err(PutProviderPaymentMethodError::CryptoUnavailable);
    }

    let resolved_ref = if decoded.payment_method_ref.trim().is_empty() {
        input.payment_method_ref.to_string()
    } else {
        decoded.payment_method_ref
    };
    let _storage_mode = decoded.storage_mode;

    Ok(PutProviderPaymentMethodResult {
        accepted: true,
        provider: UNIVERSAL_PAYMENT_PROVIDER.to_string(),
        payment_method_ref: resolved_ref,
        contract: "kms-envelope-over-evervault-v1",
    })
}

fn validate_payment_payload(
    payload: &PutProviderPaymentMethodRequest,
) -> Result<(), PutProviderPaymentMethodError> {
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
        || payload.card_last4.trim().is_empty()
    {
        return Err(PutProviderPaymentMethodError::ValidationFailed);
    }

    Ok(())
}

fn normalize_card_last4(raw: &str) -> Result<String, PutProviderPaymentMethodError> {
    let normalized = raw.trim();
    if normalized.len() != 4 || !normalized.chars().all(|value| value.is_ascii_digit()) {
        return Err(PutProviderPaymentMethodError::ValidationFailed);
    }
    Ok(normalized.to_string())
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
) -> Result<ServerEnvelopeCipher, PutProviderPaymentMethodError> {
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
        .ok_or(PutProviderPaymentMethodError::CryptoUnavailable)?;

    let key_bytes = decode_base64(encoded_master_key.as_str())?;
    if key_bytes.len() != 32 {
        return Err(PutProviderPaymentMethodError::CryptoUnavailable);
    }

    let mut keys = BTreeMap::new();
    keys.insert(key_version, key_bytes);

    let keyring = StaticKeyring::new(key_version, keys)
        .map_err(|_| PutProviderPaymentMethodError::CryptoUnavailable)?;
    Ok(ServerEnvelopeCipher::new(keyring))
}

fn decode_base64(input: &str) -> Result<Vec<u8>, PutProviderPaymentMethodError> {
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
            return Err(PutProviderPaymentMethodError::CryptoUnavailable);
        }

        let sextet = match byte {
            b'A'..=b'Z' => (byte - b'A') as u32,
            b'a'..=b'z' => (byte - b'a' + 26) as u32,
            b'0'..=b'9' => (byte - b'0' + 52) as u32,
            b'+' | b'-' => 62,
            b'/' | b'_' => 63,
            _ => return Err(PutProviderPaymentMethodError::CryptoUnavailable),
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
        return Err(PutProviderPaymentMethodError::CryptoUnavailable);
    }

    Ok(out)
}

fn normalize_env(raw: &str) -> String {
    raw.trim().to_ascii_lowercase()
}

fn canonical_provider(raw: &str) -> Option<&'static str> {
    match normalize_env(raw).as_str() {
        "srt" => Some("srt"),
        "ktx" => Some("ktx"),
        "universal" | "global" => Some(UNIVERSAL_PAYMENT_PROVIDER),
        _ => None,
    }
}
