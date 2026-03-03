use axum::http::HeaderMap;
use redis::AsyncCommands;
use serde::de::DeserializeOwned;
use sqlx::types::Json;
use uuid::Uuid;
use webauthn_rs::prelude::{
    CreationChallengeResponse, DiscoverableAuthentication, DiscoverableKey, Passkey,
    PasskeyRegistration, PublicKeyCredential, RegisterPublicKeyCredential,
    RequestChallengeResponse,
};

use super::super::AppState;
use super::auth_service::{self, AuthServiceError};

const REGISTRATION_STATE_PREFIX: &str = "auth:webauthn:reg:";
const AUTHENTICATION_STATE_PREFIX: &str = "auth:webauthn:auth:";

#[derive(Debug)]
pub(crate) enum PasskeyFlowError {
    Disabled,
    Unauthorized(&'static str),
    InvalidRequest(&'static str),
    NotFound(&'static str),
    ServiceUnavailable(&'static str),
    Internal,
}

#[derive(Debug, serde::Deserialize)]
pub(crate) struct StartPasskeyRegistrationRequest {
    pub(crate) friendly_name: Option<String>,
}

#[derive(Debug, serde::Serialize)]
pub(crate) struct StartPasskeyRegistrationResponse {
    pub(crate) flow_id: String,
    pub(crate) options: CreationChallengeResponse,
}

#[derive(Debug, serde::Deserialize)]
pub(crate) struct FinishPasskeyRegistrationRequest {
    pub(crate) flow_id: String,
    pub(crate) credential: RegisterPublicKeyCredential,
}

#[derive(Debug, serde::Serialize)]
pub(crate) struct FinishPasskeyRegistrationResponse {
    pub(crate) registered: bool,
    pub(crate) credential_id: String,
}

#[derive(Debug, serde::Serialize)]
pub(crate) struct StartPasskeyAuthenticationResponse {
    pub(crate) flow_id: String,
    pub(crate) options: RequestChallengeResponse,
}

#[derive(Debug, serde::Deserialize)]
pub(crate) struct FinishPasskeyAuthenticationRequest {
    pub(crate) flow_id: String,
    pub(crate) credential: PublicKeyCredential,
}

#[derive(Debug, serde::Serialize)]
pub(crate) struct FinishPasskeyAuthenticationResponse {
    pub(crate) authenticated: bool,
    pub(crate) user_id: String,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
struct RegistrationFlowState {
    user_id: String,
    friendly_name: Option<String>,
    state: PasskeyRegistration,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
struct AuthenticationFlowState {
    state: DiscoverableAuthentication,
}

#[derive(Debug)]
struct StoredPasskey {
    user_id: String,
    passkey: Passkey,
}

pub(crate) async fn start_passkey_registration(
    state: &AppState,
    headers: &HeaderMap,
    request: StartPasskeyRegistrationRequest,
) -> Result<StartPasskeyRegistrationResponse, PasskeyFlowError> {
    let webauthn = ensure_webauthn(state)?;
    let session_user = auth_service::require_session_user(state, headers)
        .await
        .map_err(map_auth_error)?;

    let existing = load_passkeys_by_user_id(state, &session_user.user_id).await?;
    let exclude_credentials = (!existing.is_empty()).then(|| {
        existing
            .iter()
            .map(|row| row.passkey.cred_id().clone())
            .collect()
    });

    let webauthn_user_uuid = stable_user_uuid(&session_user.user_id);
    let (options, registration_state) = webauthn
        .start_passkey_registration(
            webauthn_user_uuid,
            session_user.email.as_str(),
            session_user.email.as_str(),
            exclude_credentials,
        )
        .map_err(|_| PasskeyFlowError::InvalidRequest("failed to start passkey registration"))?;

    let flow_id = Uuid::new_v4().to_string();
    let flow_state = RegistrationFlowState {
        user_id: session_user.user_id,
        friendly_name: request
            .friendly_name
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(ToOwned::to_owned),
        state: registration_state,
    };

    let key = format!("{REGISTRATION_STATE_PREFIX}{flow_id}");
    put_json_with_ttl(state, &key, &flow_state).await?;

    Ok(StartPasskeyRegistrationResponse { flow_id, options })
}

pub(crate) async fn finish_passkey_registration(
    state: &AppState,
    headers: &HeaderMap,
    request: FinishPasskeyRegistrationRequest,
) -> Result<FinishPasskeyRegistrationResponse, PasskeyFlowError> {
    let webauthn = ensure_webauthn(state)?;
    let session_user = auth_service::require_session_user(state, headers)
        .await
        .map_err(map_auth_error)?;

    let flow_id = request.flow_id.trim();
    if flow_id.is_empty() {
        return Err(PasskeyFlowError::InvalidRequest("flow_id is required"));
    }

    let key = format!("{REGISTRATION_STATE_PREFIX}{flow_id}");
    let flow_state: RegistrationFlowState = consume_json(state, &key).await?;
    if flow_state.user_id != session_user.user_id {
        return Err(PasskeyFlowError::Unauthorized(
            "registration flow does not belong to session",
        ));
    }

    let passkey = webauthn
        .finish_passkey_registration(&request.credential, &flow_state.state)
        .map_err(|_| {
            PasskeyFlowError::InvalidRequest("passkey registration verification failed")
        })?;

    let credential_id = credential_id_to_string(passkey.cred_id())?;
    upsert_passkey(
        state,
        &session_user.user_id,
        &stable_user_uuid(&session_user.user_id),
        &credential_id,
        &passkey,
        flow_state.friendly_name.as_deref(),
    )
    .await?;

    Ok(FinishPasskeyRegistrationResponse {
        registered: true,
        credential_id,
    })
}

pub(crate) async fn start_passkey_authentication(
    state: &AppState,
) -> Result<StartPasskeyAuthenticationResponse, PasskeyFlowError> {
    let webauthn = ensure_webauthn(state)?;

    let (options, authentication_state) = webauthn
        .start_discoverable_authentication()
        .map_err(|_| PasskeyFlowError::InvalidRequest("failed to start passkey authentication"))?;

    let flow_id = Uuid::new_v4().to_string();
    let flow_state = AuthenticationFlowState {
        state: authentication_state,
    };
    let key = format!("{AUTHENTICATION_STATE_PREFIX}{flow_id}");
    put_json_with_ttl(state, &key, &flow_state).await?;

    Ok(StartPasskeyAuthenticationResponse { flow_id, options })
}

pub(crate) async fn finish_passkey_authentication(
    state: &AppState,
    request: FinishPasskeyAuthenticationRequest,
) -> Result<FinishPasskeyAuthenticationResponse, PasskeyFlowError> {
    let webauthn = ensure_webauthn(state)?;

    let flow_id = request.flow_id.trim();
    if flow_id.is_empty() {
        return Err(PasskeyFlowError::InvalidRequest("flow_id is required"));
    }

    let key = format!("{AUTHENTICATION_STATE_PREFIX}{flow_id}");
    let flow_state: AuthenticationFlowState = consume_json(state, &key).await?;

    let (user_uuid, _credential_id) = webauthn
        .identify_discoverable_authentication(&request.credential)
        .map_err(|_| PasskeyFlowError::Unauthorized("invalid discoverable credential payload"))?;

    let mut passkeys = load_passkeys_by_user_uuid(state, &user_uuid).await?;
    if passkeys.is_empty() {
        return Err(PasskeyFlowError::NotFound("passkey not found"));
    }

    let discoverable: Vec<DiscoverableKey> = passkeys
        .iter()
        .map(|row| DiscoverableKey::from(&row.passkey))
        .collect();

    let auth_result = webauthn
        .finish_discoverable_authentication(&request.credential, flow_state.state, &discoverable)
        .map_err(|_| PasskeyFlowError::Unauthorized("passkey authentication failed"))?;

    let mut updated = false;
    for row in &mut passkeys {
        if row.passkey.update_credential(&auth_result).unwrap_or(false) {
            let cred_id = credential_id_to_string(row.passkey.cred_id())?;
            update_passkey_credential(state, &cred_id, &row.passkey).await?;
            updated = true;
        }
    }

    if !updated {
        let cred_id = credential_id_to_string(auth_result.cred_id())?;
        if let Some(row) = passkeys.iter().find(|entry| {
            credential_id_to_string(entry.passkey.cred_id())
                .map(|value| value == cred_id)
                .unwrap_or(false)
        }) {
            update_passkey_credential(state, &cred_id, &row.passkey).await?;
        }
    }

    let user_id = passkeys
        .first()
        .map(|row| row.user_id.clone())
        .ok_or(PasskeyFlowError::NotFound("user not found"))?;

    Ok(FinishPasskeyAuthenticationResponse {
        authenticated: true,
        user_id,
    })
}

fn ensure_webauthn(state: &AppState) -> Result<&webauthn_rs::prelude::Webauthn, PasskeyFlowError> {
    state.webauthn.as_ref().ok_or(PasskeyFlowError::Disabled)
}

fn map_auth_error(error: AuthServiceError) -> PasskeyFlowError {
    match error {
        AuthServiceError::InvalidRequest(message) => PasskeyFlowError::InvalidRequest(message),
        AuthServiceError::Unauthorized(message) => PasskeyFlowError::Unauthorized(message),
        AuthServiceError::NotFound(message) => PasskeyFlowError::NotFound(message),
        AuthServiceError::Conflict(message) => PasskeyFlowError::InvalidRequest(message),
        AuthServiceError::ServiceUnavailable(message) => {
            PasskeyFlowError::ServiceUnavailable(message)
        }
        AuthServiceError::Internal => PasskeyFlowError::Internal,
    }
}

fn stable_user_uuid(user_id: &str) -> Uuid {
    Uuid::parse_str(user_id).unwrap_or_else(|_| {
        Uuid::new_v5(
            &Uuid::NAMESPACE_URL,
            format!("bominal/passkey/{user_id}").as_bytes(),
        )
    })
}

async fn put_json_with_ttl<T: serde::Serialize>(
    state: &AppState,
    key: &str,
    value: &T,
) -> Result<(), PasskeyFlowError> {
    let redis_client = state
        .redis_client
        .as_ref()
        .ok_or(PasskeyFlowError::ServiceUnavailable(
            "passkey state store unavailable",
        ))?;

    let mut conn = redis_client
        .get_multiplexed_async_connection()
        .await
        .map_err(|_| PasskeyFlowError::ServiceUnavailable("passkey state store unavailable"))?;

    let payload = serde_json::to_string(value).map_err(|_| PasskeyFlowError::Internal)?;
    conn.set_ex::<_, _, ()>(
        key,
        payload,
        state.config.passkey.webauthn_challenge_ttl_seconds,
    )
    .await
    .map_err(|_| PasskeyFlowError::ServiceUnavailable("passkey state store unavailable"))?;

    Ok(())
}

async fn consume_json<T: DeserializeOwned>(
    state: &AppState,
    key: &str,
) -> Result<T, PasskeyFlowError> {
    let redis_client = state
        .redis_client
        .as_ref()
        .ok_or(PasskeyFlowError::ServiceUnavailable(
            "passkey state store unavailable",
        ))?;

    let mut conn = redis_client
        .get_multiplexed_async_connection()
        .await
        .map_err(|_| PasskeyFlowError::ServiceUnavailable("passkey state store unavailable"))?;

    let raw: Option<String> = redis::cmd("GETDEL")
        .arg(key)
        .query_async(&mut conn)
        .await
        .map_err(|_| PasskeyFlowError::ServiceUnavailable("passkey state store unavailable"))?;

    let Some(raw) = raw else {
        return Err(PasskeyFlowError::NotFound("passkey flow state not found"));
    };

    serde_json::from_str(&raw)
        .map_err(|_| PasskeyFlowError::InvalidRequest("invalid passkey flow state"))
}

async fn load_passkeys_by_user_id(
    state: &AppState,
    user_id: &str,
) -> Result<Vec<StoredPasskey>, PasskeyFlowError> {
    let pool = state
        .db_pool
        .as_ref()
        .ok_or(PasskeyFlowError::ServiceUnavailable("database unavailable"))?;

    let user_uuid = Uuid::parse_str(user_id)
        .map_err(|_| PasskeyFlowError::InvalidRequest("invalid user id"))?;

    let rows = sqlx::query_as::<_, (Uuid, Json<Passkey>)>(
        "select user_id, passkey from user_passkeys where user_id = $1 order by created_at asc",
    )
    .bind(user_uuid)
    .fetch_all(pool)
    .await
    .map_err(|_| PasskeyFlowError::Internal)?;

    Ok(rows
        .into_iter()
        .map(|(user_id, Json(passkey))| StoredPasskey {
            user_id: user_id.to_string(),
            passkey,
        })
        .collect())
}

async fn load_passkeys_by_user_uuid(
    state: &AppState,
    user_uuid: &Uuid,
) -> Result<Vec<StoredPasskey>, PasskeyFlowError> {
    let pool = state
        .db_pool
        .as_ref()
        .ok_or(PasskeyFlowError::ServiceUnavailable("database unavailable"))?;

    let rows = sqlx::query_as::<_, (Uuid, Json<Passkey>)>(
        "select user_id, passkey from user_passkeys where webauthn_user_uuid = $1 order by created_at asc",
    )
    .bind(user_uuid)
    .fetch_all(pool)
    .await
    .map_err(|_| PasskeyFlowError::Internal)?;

    Ok(rows
        .into_iter()
        .map(|(user_id, Json(passkey))| StoredPasskey {
            user_id: user_id.to_string(),
            passkey,
        })
        .collect())
}

async fn upsert_passkey(
    state: &AppState,
    user_id: &str,
    webauthn_user_uuid: &Uuid,
    credential_id: &str,
    passkey: &Passkey,
    friendly_name: Option<&str>,
) -> Result<(), PasskeyFlowError> {
    let pool = state
        .db_pool
        .as_ref()
        .ok_or(PasskeyFlowError::ServiceUnavailable("database unavailable"))?;

    let user_uuid = Uuid::parse_str(user_id)
        .map_err(|_| PasskeyFlowError::InvalidRequest("invalid user id"))?;

    sqlx::query(
        "insert into user_passkeys (id, user_id, webauthn_user_uuid, credential_id, passkey, friendly_name, created_at, updated_at) values ($1, $2, $3, $4, $5, $6, now(), now()) on conflict (credential_id) do update set user_id = excluded.user_id, webauthn_user_uuid = excluded.webauthn_user_uuid, passkey = excluded.passkey, friendly_name = coalesce(excluded.friendly_name, user_passkeys.friendly_name), updated_at = now()",
    )
    .bind(Uuid::new_v4())
    .bind(user_uuid)
    .bind(webauthn_user_uuid)
    .bind(credential_id)
    .bind(Json(passkey.clone()))
    .bind(friendly_name)
    .execute(pool)
    .await
    .map_err(|_| PasskeyFlowError::Internal)?;

    Ok(())
}

async fn update_passkey_credential(
    state: &AppState,
    credential_id: &str,
    passkey: &Passkey,
) -> Result<(), PasskeyFlowError> {
    let pool = state
        .db_pool
        .as_ref()
        .ok_or(PasskeyFlowError::ServiceUnavailable("database unavailable"))?;

    sqlx::query(
        "update user_passkeys set passkey = $2, updated_at = now(), last_used_at = now() where credential_id = $1",
    )
    .bind(credential_id)
    .bind(Json(passkey.clone()))
    .execute(pool)
    .await
    .map_err(|_| PasskeyFlowError::Internal)?;

    Ok(())
}

fn credential_id_to_string(
    credential_id: &webauthn_rs::prelude::CredentialID,
) -> Result<String, PasskeyFlowError> {
    let value = serde_json::to_value(credential_id).map_err(|_| PasskeyFlowError::Internal)?;
    value
        .as_str()
        .map(ToOwned::to_owned)
        .ok_or(PasskeyFlowError::Internal)
}
