use std::env;
use std::time::Duration;

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SupabaseConfig {
    pub url: String,
    pub jwt_issuer: String,
    pub jwt_audience: Option<String>,
    pub jwks_url: String,
    pub jwks_cache_seconds: u64,
    pub auth_webhook_secret: Option<String>,
}

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
    pub database_url: String,
    pub supabase: SupabaseConfig,
    pub redis: RedisConfig,
    pub evervault: EvervaultConfig,
    pub resend: Option<ResendConfig>,
    pub runtime: RuntimeSchedule,
}

impl AppConfig {
    pub fn from_env() -> Result<Self> {
        dotenvy::dotenv().ok();

        let supabase_url = env_or("SUPABASE_URL", "")?;
        let jwt_issuer = env_or("SUPABASE_JWT_ISSUER", "")?;
        let database_url = normalize_database_url(&env_or("DATABASE_URL", "")?);

        Ok(Self {
            app_env: env_or("APP_ENV", "dev")?,
            app_host: env_or("APP_HOST", "0.0.0.0")?,
            app_port: parse_u16("APP_PORT", 8080)?,
            log_json: parse_bool("LOG_JSON", true),
            database_url,
            supabase: SupabaseConfig {
                jwks_url: env_or(
                    "SUPABASE_JWKS_URL",
                    &format!("{}/auth/v1/.well-known/jwks.json", supabase_url),
                )?,
                url: supabase_url,
                jwt_issuer,
                jwt_audience: env_opt("SUPABASE_JWT_AUDIENCE"),
                jwks_cache_seconds: parse_u64("SUPABASE_JWKS_CACHE_SECONDS", 300)?,
                auth_webhook_secret: env_opt("SUPABASE_AUTH_WEBHOOK_SECRET"),
            },
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
    raw.replace("postgresql+asyncpg://", "postgresql://")
        .replace("postgresql+psycopg://", "postgresql://")
        .replace("postgresql+psycopg2://", "postgresql://")
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
