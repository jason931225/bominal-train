use std::env;
use std::time::Duration;

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RedisConfig {
    pub url: String,
    pub queue_key: String,
    pub queue_dlq_key: String,
    pub lease_prefix: String,
    pub rate_limit_prefix: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvervaultConfig {
    pub relay_base_url: String,
    pub app_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResendConfig {
    pub api_base_url: String,
    pub api_key_present: bool,
    pub from_address: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum PasskeyProvider {
    ServerWebauthn,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum AdminRole {
    Admin,
    Operator,
    Viewer,
    User,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PasskeyConfig {
    pub provider: PasskeyProvider,
    pub webauthn_rp_id: String,
    pub webauthn_rp_origin: String,
    pub webauthn_rp_name: String,
    pub webauthn_challenge_ttl_seconds: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuntimeSchedule {
    pub poll_interval: Duration,
    pub reconcile_interval: Duration,
    pub watch_interval: Duration,
    pub key_rotation_interval: Duration,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub app_env: String,
    pub app_host: String,
    pub app_port: u16,
    pub log_json: bool,
    pub session_cookie_name: String,
    pub session_cookie_domain: Option<String>,
    pub session_ttl_seconds: u64,
    pub step_up_ttl_seconds: u64,
    pub session_secret: String,
    pub invite_base_url: String,
    pub user_app_host: String,
    pub admin_app_host: String,
    pub ui_theme_cookie_name: String,
    pub database_url: String,
    pub redis: RedisConfig,
    pub evervault: EvervaultConfig,
    pub resend: Option<ResendConfig>,
    pub passkey: PasskeyConfig,
    pub runtime: RuntimeSchedule,
}

impl AppConfig {
    pub fn from_env() -> Result<Self> {
        dotenvy::dotenv().ok();

        let database_url = normalize_database_url(&env_or("DATABASE_URL", "")?);
        let app_env = env_or("APP_ENV", "dev")?;
        let app_host = env_or("APP_HOST", "0.0.0.0")?;
        let app_port = parse_u16("APP_PORT", 8080)?;
        let session_cookie_name = env_or("SESSION_COOKIE_NAME", "bominal_session")?;
        let session_cookie_domain = env_opt("SESSION_COOKIE_DOMAIN");
        let session_ttl_seconds = parse_u64("SESSION_TTL_SECONDS", 86400)?;
        let step_up_ttl_seconds = parse_u64("STEP_UP_TTL_SECONDS", 600)?;
        let session_secret = env_or("SESSION_SECRET", "dev-session-secret-change-me")?;
        let invite_base_url = env_or("INVITE_BASE_URL", "http://127.0.0.1:8000")?;
        let user_app_host = env_or("USER_APP_HOST", "www.bominal.com")?;
        let admin_app_host = env_or("ADMIN_APP_HOST", "ops.bominal.com")?;
        let ui_theme_cookie_name = env_or("UI_THEME_COOKIE_NAME", "bominal_theme")?;
        let webauthn_rp_id = env_or("WEBAUTHN_RP_ID", "localhost")?;
        let webauthn_rp_origin_default = format!("http://localhost:{app_port}");
        let webauthn_rp_origin = env_or("WEBAUTHN_RP_ORIGIN", &webauthn_rp_origin_default)?;
        let webauthn_rp_name = env_or("WEBAUTHN_RP_NAME", "bominal")?;

        Ok(Self {
            app_env,
            app_host,
            app_port,
            log_json: parse_bool("LOG_JSON", true),
            session_cookie_name,
            session_cookie_domain,
            session_ttl_seconds,
            step_up_ttl_seconds,
            session_secret,
            invite_base_url,
            user_app_host,
            admin_app_host,
            ui_theme_cookie_name,
            database_url,
            redis: RedisConfig {
                url: env_or("REDIS_URL", "redis://127.0.0.1:6379")?,
                queue_key: env_or("RUNTIME_QUEUE_KEY", "train:queue")?,
                queue_dlq_key: env_or("RUNTIME_QUEUE_DLQ_KEY", "train:queue:dlq")?,
                lease_prefix: env_or("RUNTIME_LEASE_PREFIX", "train:lease")?,
                rate_limit_prefix: env_or("RUNTIME_RATE_LIMIT_PREFIX", "rate_limit")?,
            },
            evervault: EvervaultConfig {
                relay_base_url: env_or("EVERVAULT_RELAY_BASE_URL", "https://relay.evervault.com")?,
                app_id: env_opt("EVERVAULT_APP_ID"),
            },
            resend: maybe_resend()?,
            passkey: PasskeyConfig {
                provider: parse_passkey_provider(&env_or("PASSKEY_PROVIDER", "server_webauthn")?)?,
                webauthn_rp_id,
                webauthn_rp_origin,
                webauthn_rp_name,
                webauthn_challenge_ttl_seconds: parse_u64("WEBAUTHN_CHALLENGE_TTL_SECONDS", 300)?,
            },
            runtime: RuntimeSchedule {
                poll_interval: parse_seconds("WORKER_POLL_SECONDS", 3)?,
                reconcile_interval: parse_seconds("WORKER_RECONCILE_SECONDS", 30)?,
                watch_interval: parse_seconds("WORKER_WATCH_SECONDS", 5)?,
                key_rotation_interval: parse_seconds("KEY_ROTATION_SECONDS", 3600)?,
            },
        })
    }
}

fn normalize_database_url(raw: &str) -> String {
    let normalized = raw
        .replace("postgresql+asyncpg://", "postgresql://")
        .replace("postgresql+psycopg://", "postgresql://")
        .replace("postgresql+psycopg2://", "postgresql://");

    normalized
        .replace("?ssl=true", "?sslmode=require")
        .replace("&ssl=true", "&sslmode=require")
        .replace("?ssl=false", "?sslmode=disable")
        .replace("&ssl=false", "&sslmode=disable")
        .replace("?ssl=", "?sslmode=")
        .replace("&ssl=", "&sslmode=")
}

fn env_or(key: &str, default: &str) -> Result<String> {
    match env::var(key) {
        Ok(value) => Ok(value),
        Err(env::VarError::NotPresent) => Ok(default.to_string()),
        Err(err) => Err(anyhow::anyhow!("failed to read {key}: {err}")).context("env read"),
    }
}

fn env_opt(key: &str) -> Option<String> {
    match env::var(key) {
        Ok(value) if !value.is_empty() => Some(value),
        _ => None,
    }
}

fn parse_bool(key: &str, default: bool) -> bool {
    env::var(key)
        .ok()
        .and_then(|raw| match raw.to_ascii_lowercase().as_str() {
            "1" | "true" | "yes" | "on" => Some(true),
            "0" | "false" | "no" | "off" => Some(false),
            _ => None,
        })
        .unwrap_or(default)
}

fn parse_u16(key: &str, default: u16) -> Result<u16> {
    env::var(key)
        .ok()
        .map(|raw| {
            raw.parse::<u16>()
                .with_context(|| format!("{key} must be a valid u16"))
        })
        .transpose()?
        .map_or(Ok(default), Ok)
}

fn parse_u64(key: &str, default: u64) -> Result<u64> {
    env::var(key)
        .ok()
        .map(|raw| {
            raw.parse::<u64>()
                .with_context(|| format!("{key} must be a valid u64"))
        })
        .transpose()?
        .map_or(Ok(default), Ok)
}

fn parse_seconds(key: &str, default: u64) -> Result<Duration> {
    env::var(key)
        .ok()
        .map(|raw| {
            raw.parse::<u64>()
                .map(Duration::from_secs)
                .with_context(|| format!("{key} must be a positive integer in seconds"))
        })
        .transpose()?
        .map_or(Ok(Duration::from_secs(default)), Ok)
}

fn maybe_resend() -> Result<Option<ResendConfig>> {
    let api_key_present = env_opt("RESEND_API_KEY").is_some();
    let base_url = env_or("RESEND_BASE_URL", "https://api.resend.com")?;
    let from_address = env_opt("EMAIL_FROM_ADDRESS");

    if !api_key_present && from_address.is_none() {
        return Ok(None);
    }

    Ok(Some(ResendConfig {
        api_base_url: base_url,
        api_key_present,
        from_address,
    }))
}

fn parse_passkey_provider(raw: &str) -> Result<PasskeyProvider> {
    match raw.trim().to_ascii_lowercase().as_str() {
        "server_webauthn" => Ok(PasskeyProvider::ServerWebauthn),
        _ => Err(anyhow::anyhow!("PASSKEY_PROVIDER must be: server_webauthn")),
    }
}

#[cfg(test)]
mod tests {
    use super::normalize_database_url;

    #[test]
    fn normalize_database_url_rewrites_driver_and_ssl_true() {
        let input = "postgresql+asyncpg://user:pw@db:5432/bominal?ssl=true&application_name=api";
        let expected = "postgresql://user:pw@db:5432/bominal?sslmode=require&application_name=api";
        assert_eq!(normalize_database_url(input), expected);
    }

    #[test]
    fn normalize_database_url_rewrites_ssl_false() {
        let input = "postgresql://user:pw@db:5432/bominal?pool=5&ssl=false";
        let expected = "postgresql://user:pw@db:5432/bominal?pool=5&sslmode=disable";
        assert_eq!(normalize_database_url(input), expected);
    }

    #[test]
    fn normalize_database_url_rewrites_generic_ssl_param() {
        let input = "postgresql+psycopg2://user:pw@db:5432/bominal?ssl=verify-full&x=1";
        let expected = "postgresql://user:pw@db:5432/bominal?sslmode=verify-full&x=1";
        assert_eq!(normalize_database_url(input), expected);
    }
}
