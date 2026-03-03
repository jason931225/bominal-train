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
pub(crate) struct PutSrtCredentialsRequest {
    #[serde(default)]
    pub(crate) subject_ref: Option<String>,
    pub(crate) identity_ciphertext: String,
    pub(crate) password_ciphertext: String,
}

#[derive(Debug, serde::Serialize)]
pub(crate) struct PutSrtCredentialsResult {
    pub(crate) accepted: bool,
    pub(crate) provider: &'static str,
    pub(crate) credential_ref: String,
}

#[derive(Debug)]
pub(crate) enum PutSrtCredentialsError {
    ValidationFailed,
    PersistenceUnavailable,
    CryptoUnavailable,
    PersistenceFailure,
}

pub(crate) async fn put_srt_credentials(
    state: &AppState,
    payload: PutSrtCredentialsRequest,
) -> Result<PutSrtCredentialsResult, PutSrtCredentialsError> {
    validate_srt_credentials_payload(&payload)?;

    let Some(pool) = state.db_pool.as_ref() else {
        return Err(PutSrtCredentialsError::PersistenceUnavailable);
    };

    let subject_ref = payload
        .subject_ref
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToOwned::to_owned)
        .unwrap_or_else(|| format!("srt-subject-{}", Uuid::new_v4()));

    let aad = EnvelopeAad {
        payload_kind: PayloadKind::ProviderAuth,
        provider: Some("srt".to_string()),
        subject_id: Some(subject_ref.clone()),
        scope: "srt:credentials:login".to_string(),
        metadata: BTreeMap::from([("credential_kind".to_string(), "login".to_string())]),
    };

    let plaintext = serde_json::to_vec(&serde_json::json!({
        "identity_ciphertext": payload.identity_ciphertext,
        "password_ciphertext": payload.password_ciphertext
    }))
    .map_err(|_| PutSrtCredentialsError::ValidationFailed)?;

    let cipher = build_envelope_cipher_from_env(state)?;
    let encrypted = cipher
        .encrypt(&plaintext, aad.clone())
        .map_err(|_| PutSrtCredentialsError::CryptoUnavailable)?;
    let aad_hash =
        serde_json::to_vec(&aad).map_err(|_| PutSrtCredentialsError::CryptoUnavailable)?;
    let key_version = i32::try_from(encrypted.key_version)
        .map_err(|_| PutSrtCredentialsError::CryptoUnavailable)?;
    let now = Utc::now();
    let redacted_metadata = serde_json::json!({
        "provider": "srt",
        "credential_kind": "login",
        "contract": "ciphertext-only-v1"
    });

    let params = UpsertProviderAuthSecretParams {
        provider: "srt",
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
        return Err(PutSrtCredentialsError::PersistenceFailure);
    }

    Ok(PutSrtCredentialsResult {
        accepted: true,
        provider: "srt",
        credential_ref: format!("srt_cred_{}", Uuid::new_v4()),
    })
}

fn validate_srt_credentials_payload(
    payload: &PutSrtCredentialsRequest,
) -> Result<(), PutSrtCredentialsError> {
    if payload.identity_ciphertext.trim().is_empty()
        || payload.password_ciphertext.trim().is_empty()
    {
        return Err(PutSrtCredentialsError::ValidationFailed);
    }

    Ok(())
}

fn build_envelope_cipher_from_env(
    state: &AppState,
) -> Result<ServerEnvelopeCipher, PutSrtCredentialsError> {
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
        .ok_or(PutSrtCredentialsError::CryptoUnavailable)?;

    let key_bytes = decode_base64(encoded_master_key.as_str())?;
    if key_bytes.len() != 32 {
        return Err(PutSrtCredentialsError::CryptoUnavailable);
    }

    let mut keys = BTreeMap::new();
    keys.insert(key_version, key_bytes);

    let keyring = StaticKeyring::new(key_version, keys)
        .map_err(|_| PutSrtCredentialsError::CryptoUnavailable)?;
    Ok(ServerEnvelopeCipher::new(keyring))
}

fn decode_base64(input: &str) -> Result<Vec<u8>, PutSrtCredentialsError> {
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
            return Err(PutSrtCredentialsError::CryptoUnavailable);
        }

        let sextet = match byte {
            b'A'..=b'Z' => (byte - b'A') as u32,
            b'a'..=b'z' => (byte - b'a' + 26) as u32,
            b'0'..=b'9' => (byte - b'0' + 52) as u32,
            b'+' | b'-' => 62,
            b'/' | b'_' => 63,
            _ => return Err(PutSrtCredentialsError::CryptoUnavailable),
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
        return Err(PutSrtCredentialsError::CryptoUnavailable);
    }

    Ok(out)
}

fn normalize_env(raw: &str) -> String {
    raw.trim().to_ascii_lowercase()
}
