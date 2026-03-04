use std::{collections::BTreeMap, env};

use bominal_shared::{
    crypto::{EnvelopeAad, EnvelopeCipher, PayloadKind, ServerEnvelopeCipher, StaticKeyring},
    repo::{UpsertProviderAuthSecretParams, upsert_provider_auth_secret_query},
};
use chrono::Utc;
use tracing::error;
use uuid::Uuid;

use super::super::AppState;

const TEST_MASTER_KEY_B64_FALLBACK: &str = "MDEyMzQ1Njc4OWFiY2RlZjAxMjM0NTY3ODlhYmNkZWY=";

#[derive(Debug, serde::Deserialize)]
pub(crate) struct PutProviderCredentialsRequest {
    #[serde(default)]
    pub(crate) subject_ref: Option<String>,
    pub(crate) identity_ciphertext: String,
    pub(crate) password_ciphertext: String,
}

#[derive(Debug, serde::Serialize)]
pub(crate) struct PutProviderCredentialsResult {
    pub(crate) accepted: bool,
    pub(crate) provider: String,
    pub(crate) credential_ref: String,
    pub(crate) contract: &'static str,
}

#[derive(Debug)]
pub(crate) enum PutProviderCredentialsError {
    ValidationFailed,
    PersistenceUnavailable,
    CryptoUnavailable,
    PersistenceFailure,
}

pub(crate) async fn put_provider_credentials(
    state: &AppState,
    provider: &str,
    payload: PutProviderCredentialsRequest,
) -> Result<PutProviderCredentialsResult, PutProviderCredentialsError> {
    validate_provider_credentials_payload(&payload)?;
    let provider =
        canonical_provider(provider).ok_or(PutProviderCredentialsError::ValidationFailed)?;

    let Some(pool) = state.db_pool.as_ref() else {
        return Err(PutProviderCredentialsError::PersistenceUnavailable);
    };

    let subject_ref = payload
        .subject_ref
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToOwned::to_owned)
        .unwrap_or_else(|| format!("{provider}-subject-{}", Uuid::new_v4()));

    let aad = EnvelopeAad {
        payload_kind: PayloadKind::ProviderAuth,
        provider: Some(provider.to_string()),
        subject_id: Some(subject_ref.clone()),
        scope: format!("{provider}:credentials:login"),
        metadata: BTreeMap::from([("credential_kind".to_string(), "login".to_string())]),
    };

    let plaintext = serde_json::to_vec(&serde_json::json!({
        "identity_ciphertext": payload.identity_ciphertext,
        "password_ciphertext": payload.password_ciphertext
    }))
    .map_err(|_| PutProviderCredentialsError::ValidationFailed)?;

    let cipher = build_envelope_cipher_from_env(state)?;
    let encrypted = cipher
        .encrypt(&plaintext, aad.clone())
        .map_err(|_| PutProviderCredentialsError::CryptoUnavailable)?;
    let aad_hash =
        serde_json::to_vec(&aad).map_err(|_| PutProviderCredentialsError::CryptoUnavailable)?;
    let key_version = i32::try_from(encrypted.key_version)
        .map_err(|_| PutProviderCredentialsError::CryptoUnavailable)?;
    let now = Utc::now();
    let redacted_metadata = serde_json::json!({
        "provider": provider,
        "credential_kind": "login",
        "contract": "ciphertext-only-v1"
    });

    let params = UpsertProviderAuthSecretParams {
        provider,
        subject_ref: subject_ref.as_str(),
        credential_kind: "login",
        secret_envelope_ciphertext: encrypted.ciphertext.as_slice(),
        secret_envelope_dek_ciphertext: &encrypted.nonce,
        secret_envelope_kek_version: key_version,
        secret_envelope_aad_scope: aad.scope.as_str(),
        secret_envelope_aad_subject: subject_ref.as_str(),
        secret_envelope_aad_hash: aad_hash.as_slice(),
        redacted_metadata: &redacted_metadata,
        updated_at: now,
        rotated_at: Some(now),
        revoked_at: None,
    };

    if let Err(err) = upsert_provider_auth_secret_query(&params)
        .execute(pool)
        .await
    {
        error!(error = %err, "failed to persist provider auth secret");
        return Err(PutProviderCredentialsError::PersistenceFailure);
    }

    Ok(PutProviderCredentialsResult {
        accepted: true,
        provider: provider.to_string(),
        credential_ref: format!("{provider}_cred_{}", Uuid::new_v4()),
        contract: "ciphertext-only-v1",
    })
}

fn validate_provider_credentials_payload(
    payload: &PutProviderCredentialsRequest,
) -> Result<(), PutProviderCredentialsError> {
    if payload.identity_ciphertext.trim().is_empty()
        || payload.password_ciphertext.trim().is_empty()
    {
        return Err(PutProviderCredentialsError::ValidationFailed);
    }

    Ok(())
}

fn build_envelope_cipher_from_env(
    state: &AppState,
) -> Result<ServerEnvelopeCipher, PutProviderCredentialsError> {
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
        .ok_or(PutProviderCredentialsError::CryptoUnavailable)?;

    let key_bytes = decode_base64(encoded_master_key.as_str())?;
    if key_bytes.len() != 32 {
        return Err(PutProviderCredentialsError::CryptoUnavailable);
    }

    let mut keys = BTreeMap::new();
    keys.insert(key_version, key_bytes);

    let keyring = StaticKeyring::new(key_version, keys)
        .map_err(|_| PutProviderCredentialsError::CryptoUnavailable)?;
    Ok(ServerEnvelopeCipher::new(keyring))
}

fn decode_base64(input: &str) -> Result<Vec<u8>, PutProviderCredentialsError> {
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
            return Err(PutProviderCredentialsError::CryptoUnavailable);
        }

        let sextet = match byte {
            b'A'..=b'Z' => (byte - b'A') as u32,
            b'a'..=b'z' => (byte - b'a' + 26) as u32,
            b'0'..=b'9' => (byte - b'0' + 52) as u32,
            b'+' | b'-' => 62,
            b'/' | b'_' => 63,
            _ => return Err(PutProviderCredentialsError::CryptoUnavailable),
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
        return Err(PutProviderCredentialsError::CryptoUnavailable);
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
        _ => None,
    }
}
