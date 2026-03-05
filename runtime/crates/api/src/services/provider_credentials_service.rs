use std::{collections::BTreeMap, env, time::Duration};

use bominal_shared::{
    crypto::{EnvelopeAad, EnvelopeCipher, PayloadKind, ServerEnvelopeCipher, StaticKeyring},
    providers::ProviderAdapter,
    providers::ProviderError,
    providers::ktx::{KtxProviderAdapter, ReqwestKtxClient},
    providers::model::{LoginAccountType, LoginRequest, SecretString},
    providers::srt::{
        ReqwestSrtClient, SrtProviderAdapter,
    },
    repo::{
        UpsertProviderAuthSecretParams, update_provider_auth_secret_metadata_query,
        upsert_provider_auth_secret_query,
    },
};
use chrono::Utc;
use tracing::{error, warn};
use uuid::Uuid;

use super::super::AppState;

const TEST_MASTER_KEY_B64_FALLBACK: &str = "MDEyMzQ1Njc4OWFiY2RlZjAxMjM0NTY3ODlhYmNkZWY=";
const PROVIDER_AUTH_PROBE_ENABLED_ENV: &str = "PROVIDER_AUTH_PROBE_ENABLED";
const PROVIDER_AUTH_PROBE_TIMEOUT_SECONDS: u64 = 4;

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
    pub(crate) auth_probe_status: &'static str,
    pub(crate) auth_probe_message: String,
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

    let credential_kind = "login";
    let aad = EnvelopeAad {
        payload_kind: PayloadKind::ProviderAuth,
        provider: Some(provider.to_string()),
        subject_id: Some(subject_ref.clone()),
        scope: format!("{provider}:credentials:login"),
        metadata: BTreeMap::from([("credential_kind".to_string(), credential_kind.to_string())]),
    };
    let identity_ciphertext = payload.identity_ciphertext.as_str();
    let password_ciphertext = payload.password_ciphertext.as_str();

    let plaintext = serde_json::to_vec(&serde_json::json!({
        "identity_ciphertext": identity_ciphertext,
        "password_ciphertext": password_ciphertext
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
    let initial_probe = AuthProbeResult::skipped(
        provider,
        "Authentication probe pending. Credentials were saved.",
    );
    let redacted_metadata = build_probe_metadata(
        provider,
        credential_kind,
        initial_probe.status,
        initial_probe.message.as_str(),
        now,
    );

    let params = UpsertProviderAuthSecretParams {
        provider,
        subject_ref: subject_ref.as_str(),
        credential_kind,
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

    let probe_result = probe_provider_login_once(
        &state.config.app_env,
        provider_auth_probe_enabled(),
        provider,
        identity_ciphertext,
        password_ciphertext,
    )
    .await;
    let probe_checked_at = Utc::now();
    let probe_metadata = build_probe_metadata(
        provider,
        credential_kind,
        probe_result.status,
        probe_result.message.as_str(),
        probe_checked_at,
    );
    if let Err(err) = update_provider_auth_secret_metadata_query(
        provider,
        subject_ref.as_str(),
        credential_kind,
        &probe_metadata,
        probe_checked_at,
    )
    .execute(pool)
    .await
    {
        warn!(
            error = %err,
            provider,
            "failed to persist provider auth probe metadata"
        );
    }

    Ok(PutProviderCredentialsResult {
        accepted: true,
        provider: provider.to_string(),
        credential_ref: format!("{provider}_cred_{}", Uuid::new_v4()),
        contract: "ciphertext-only-v1",
        auth_probe_status: probe_result.status,
        auth_probe_message: probe_result.message,
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

#[derive(Debug, Clone)]
struct AuthProbeResult {
    status: &'static str,
    message: String,
}

impl AuthProbeResult {
    fn success(provider: &str) -> Self {
        Self {
            status: "success",
            message: format!(
                "{}: Successfully authenticated.",
                provider.to_ascii_uppercase()
            ),
        }
    }

    fn error(provider: &str, detail: &str) -> Self {
        let detail = detail.trim();
        let message = if detail.is_empty() {
            format!(
                "{}: Authentication failed. Verify account identifier and password.",
                provider.to_ascii_uppercase()
            )
        } else {
            format!("{}: {detail}", provider.to_ascii_uppercase())
        };
        Self {
            status: "error",
            message,
        }
    }

    fn skipped(provider: &str, detail: &str) -> Self {
        let detail = detail.trim();
        let message = if detail.is_empty() {
            format!(
                "{}: Authentication probe skipped; credentials were saved.",
                provider.to_ascii_uppercase()
            )
        } else {
            format!("{}: {detail}", provider.to_ascii_uppercase())
        };
        Self {
            status: "skipped",
            message,
        }
    }
}

fn provider_auth_probe_enabled() -> bool {
    parse_probe_enabled(env::var(PROVIDER_AUTH_PROBE_ENABLED_ENV).ok())
}

fn parse_probe_enabled(raw: Option<String>) -> bool {
    raw.as_deref()
        .map(str::trim)
        .map(str::to_ascii_lowercase)
        .and_then(|value| match value.as_str() {
            "1" | "true" | "yes" | "on" => Some(true),
            "0" | "false" | "no" | "off" => Some(false),
            _ => None,
        })
        .unwrap_or(true)
}

fn build_probe_metadata(
    provider: &str,
    credential_kind: &str,
    auth_probe_status: &str,
    auth_probe_message: &str,
    checked_at: chrono::DateTime<Utc>,
) -> serde_json::Value {
    serde_json::json!({
        "provider": provider,
        "credential_kind": credential_kind,
        "contract": "ciphertext-only-v1",
        "auth_probe_status": auth_probe_status,
        "auth_probe_message": auth_probe_message,
        "auth_probe_checked_at": checked_at,
    })
}

async fn probe_provider_login_once(
    app_env: &str,
    probe_enabled: bool,
    provider: &str,
    account_identifier: &str,
    password: &str,
) -> AuthProbeResult {
    if !probe_enabled {
        return AuthProbeResult::skipped(
            provider,
            "Authentication probe skipped by configuration.",
        );
    }

    let app_env = normalize_env(app_env);
    if matches!(app_env.as_str(), "test" | "testing" | "ci" | "integration") {
        return AuthProbeResult::skipped(
            provider,
            "Authentication probe skipped in test-like environment.",
        );
    }

    let timeout = Duration::from_secs(PROVIDER_AUTH_PROBE_TIMEOUT_SECONDS);
    let provider_owned = provider.to_string();
    let account_owned = account_identifier.to_string();
    let password_owned = password.to_string();
    let handle = tokio::task::spawn_blocking(move || {
        probe_provider_login_blocking(
            provider_owned.as_str(),
            account_owned.as_str(),
            password_owned.as_str(),
            timeout,
        )
    });
    match handle.await {
        Ok(result) => result,
        Err(_) => AuthProbeResult::error(
            provider,
            "Authentication probe failed. Credentials were saved.",
        ),
    }
}

fn probe_provider_login_blocking(
    provider: &str,
    account_identifier: &str,
    password: &str,
    timeout: Duration,
) -> AuthProbeResult {
    let request = LoginRequest {
        account_type: LoginAccountType::MembershipNumber,
        account_identifier: account_identifier.to_string(),
        password: SecretString::new(password.to_string().into_boxed_str()),
    };

    let outcome = match provider {
        "srt" => {
            let mut adapter = SrtProviderAdapter::new(ReqwestSrtClient::live_with_timeout(
                "https://app.srail.or.kr",
                timeout,
            ));
            adapter.login(request)
        }
        "ktx" => {
            let mut adapter = KtxProviderAdapter::new(ReqwestKtxClient::live_with_timeout(
                "https://smart.letskorail.com",
                timeout,
            ));
            adapter.login(request)
        }
        _ => return AuthProbeResult::error(provider, "Authentication probe unsupported"),
    };

    match outcome {
        Ok(_) => AuthProbeResult::success(provider),
        Err(error) => AuthProbeResult::error(provider, probe_error_message(&error).as_str()),
    }
}

fn probe_error_message(error: &ProviderError) -> String {
    match error {
        ProviderError::Unauthorized | ProviderError::SessionExpired => {
            "Authentication failed. Verify account identifier and password.".to_string()
        }
        ProviderError::NotLoggedIn | ProviderError::ReloginUnavailable => {
            "Authentication failed. Provider session could not be created.".to_string()
        }
        ProviderError::Transport { message } | ProviderError::OperationFailed { message } => {
            let trimmed = message.trim();
            let lower = trimmed.to_ascii_lowercase();
            if lower.contains("timed out") || lower.contains("timeout") {
                return "Authentication probe timed out after 4 seconds. Credentials were saved."
                    .to_string();
            }
            if trimmed.is_empty() {
                "Authentication probe failed".to_string()
            } else {
                trimmed.to_string()
            }
        }
        ProviderError::UnsupportedOperation { .. } => {
            "Authentication probe is not supported for this provider.".to_string()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn auth_probe_result_messages_include_provider_prefix() {
        let success = AuthProbeResult::success("srt");
        assert_eq!(success.status, "success");
        assert_eq!(success.message, "SRT: Successfully authenticated.");

        let error = AuthProbeResult::error("ktx", "Authentication failed");
        assert_eq!(error.status, "error");
        assert_eq!(error.message, "KTX: Authentication failed");

        let skipped = AuthProbeResult::skipped("srt", "Authentication probe skipped.");
        assert_eq!(skipped.status, "skipped");
        assert_eq!(skipped.message, "SRT: Authentication probe skipped.");
    }

    #[test]
    fn probe_error_message_maps_auth_failures_to_actionable_text() {
        assert_eq!(
            probe_error_message(&ProviderError::Unauthorized),
            "Authentication failed. Verify account identifier and password."
        );
        assert_eq!(
            probe_error_message(&ProviderError::OperationFailed {
                message: "rate_limited for login".to_string()
            }),
            "rate_limited for login"
        );
    }

    #[test]
    fn parse_probe_enabled_defaults_to_true_and_accepts_flags() {
        assert!(parse_probe_enabled(None));
        assert!(parse_probe_enabled(Some("true".to_string())));
        assert!(parse_probe_enabled(Some("1".to_string())));
        assert!(!parse_probe_enabled(Some("false".to_string())));
        assert!(!parse_probe_enabled(Some("0".to_string())));
        assert!(parse_probe_enabled(Some("invalid".to_string())));
    }

    #[tokio::test]
    async fn probe_is_skipped_when_disabled_or_test_env() {
        let disabled = probe_provider_login_once("production", false, "srt", "id", "pw").await;
        assert_eq!(disabled.status, "skipped");
        assert!(disabled.message.contains("skipped"));

        let test_env = probe_provider_login_once("test", true, "ktx", "id", "pw").await;
        assert_eq!(test_env.status, "skipped");
        assert!(test_env.message.contains("test-like environment"));
    }
}
