use std::env;
use std::time::Duration;

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use sqlx::postgres::PgPoolOptions;

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

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum StationCatalogSourceMode {
    RepoOnly,
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

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum DbPoolTarget {
    Api,
    Worker,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DbPoolConfig {
    pub max_connections: u32,
    pub acquire_timeout: Duration,
    pub idle_timeout: Duration,
    pub max_lifetime: Duration,
}

impl DbPoolConfig {
    pub fn from_env(target: DbPoolTarget) -> Result<Self> {
        Self::from_lookup(target, |key| match env::var(key) {
            Ok(value) => Ok(Some(value)),
            Err(env::VarError::NotPresent) => Ok(None),
            Err(err) => Err(anyhow::anyhow!("failed to read {key}: {err}")).context("env read"),
        })
    }

    fn from_lookup<F>(target: DbPoolTarget, lookup: F) -> Result<Self>
    where
        F: Fn(&str) -> Result<Option<String>>,
    {
        let max_connections = match target {
            DbPoolTarget::Api => {
                parse_positive_u32_lookup(&lookup, "API_DB_POOL_MAX_CONNECTIONS", 10)?
            }
            DbPoolTarget::Worker => {
                parse_positive_u32_lookup(&lookup, "WORKER_DB_POOL_MAX_CONNECTIONS", 5)?
            }
        };

        Ok(Self {
            max_connections,
            acquire_timeout: parse_positive_seconds_lookup(
                &lookup,
                "DB_POOL_ACQUIRE_TIMEOUT_SECONDS",
                5,
            )?,
            idle_timeout: parse_positive_seconds_lookup(
                &lookup,
                "DB_POOL_IDLE_TIMEOUT_SECONDS",
                300,
            )?,
            max_lifetime: parse_positive_seconds_lookup(
                &lookup,
                "DB_POOL_MAX_LIFETIME_SECONDS",
                1800,
            )?,
        })
    }
}

pub fn pg_pool_options_from_config(config: &DbPoolConfig) -> PgPoolOptions {
    PgPoolOptions::new()
        .max_connections(config.max_connections)
        .acquire_timeout(config.acquire_timeout)
        .idle_timeout(config.idle_timeout)
        .max_lifetime(config.max_lifetime)
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
    pub station_catalog_json_path: String,
    pub station_catalog_source_mode: StationCatalogSourceMode,
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
        let station_catalog_json_path = env_or(
            "STATION_CATALOG_JSON_PATH",
            "data/train/station_catalog.v1.json",
        )?;
        let station_catalog_source_mode = parse_station_catalog_source_mode(&env_or(
            "STATION_CATALOG_SOURCE_MODE",
            "repo_only",
        )?)?;
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
            station_catalog_json_path,
            station_catalog_source_mode,
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

fn parse_positive_u32_lookup<F>(lookup: &F, key: &str, default: u32) -> Result<u32>
where
    F: Fn(&str) -> Result<Option<String>>,
{
    let Some(raw) = lookup(key)? else {
        return Ok(default);
    };

    let value = raw
        .parse::<u32>()
        .with_context(|| format!("{key} must be a valid u32"))?;
    if value == 0 {
        return Err(anyhow::anyhow!("{key} must be greater than zero"));
    }
    Ok(value)
}

fn parse_positive_seconds_lookup<F>(lookup: &F, key: &str, default: u64) -> Result<Duration>
where
    F: Fn(&str) -> Result<Option<String>>,
{
    let Some(raw) = lookup(key)? else {
        return Ok(Duration::from_secs(default));
    };

    let value = raw
        .parse::<u64>()
        .with_context(|| format!("{key} must be a positive integer in seconds"))?;
    if value == 0 {
        return Err(anyhow::anyhow!(
            "{key} must be a positive integer in seconds"
        ));
    }
    Ok(Duration::from_secs(value))
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

fn parse_station_catalog_source_mode(raw: &str) -> Result<StationCatalogSourceMode> {
    match raw.trim().to_ascii_lowercase().as_str() {
        "repo_only" => Ok(StationCatalogSourceMode::RepoOnly),
        _ => Err(anyhow::anyhow!(
            "STATION_CATALOG_SOURCE_MODE must be: repo_only"
        )),
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use std::time::Duration;

    use sqlx::postgres::PgPoolOptions;

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

    #[test]
    fn db_pool_config_defaults_match_local_runtime_expectations() {
        let env = HashMap::<String, String>::new();
        let api = match super::DbPoolConfig::from_lookup(super::DbPoolTarget::Api, |key| {
            Ok(env.get(key).cloned())
        }) {
            Ok(value) => value,
            Err(err) => panic!("api db pool config should parse defaults: {err}"),
        };
        let worker = match super::DbPoolConfig::from_lookup(super::DbPoolTarget::Worker, |key| {
            Ok(env.get(key).cloned())
        }) {
            Ok(value) => value,
            Err(err) => panic!("worker db pool config should parse defaults: {err}"),
        };

        assert_eq!(api.max_connections, 10);
        assert_eq!(worker.max_connections, 5);
        assert_eq!(api.acquire_timeout, Duration::from_secs(5));
        assert_eq!(worker.acquire_timeout, Duration::from_secs(5));
        assert_eq!(api.idle_timeout, Duration::from_secs(300));
        assert_eq!(worker.idle_timeout, Duration::from_secs(300));
        assert_eq!(api.max_lifetime, Duration::from_secs(1800));
        assert_eq!(worker.max_lifetime, Duration::from_secs(1800));
    }

    #[test]
    fn db_pool_config_reads_role_specific_and_shared_overrides() {
        let env = HashMap::from([
            ("API_DB_POOL_MAX_CONNECTIONS".to_string(), "4".to_string()),
            (
                "WORKER_DB_POOL_MAX_CONNECTIONS".to_string(),
                "3".to_string(),
            ),
            (
                "DB_POOL_ACQUIRE_TIMEOUT_SECONDS".to_string(),
                "2".to_string(),
            ),
            ("DB_POOL_IDLE_TIMEOUT_SECONDS".to_string(), "60".to_string()),
            (
                "DB_POOL_MAX_LIFETIME_SECONDS".to_string(),
                "300".to_string(),
            ),
        ]);

        let api = match super::DbPoolConfig::from_lookup(super::DbPoolTarget::Api, |key| {
            Ok(env.get(key).cloned())
        }) {
            Ok(value) => value,
            Err(err) => panic!("api db pool config should parse overrides: {err}"),
        };
        let worker = match super::DbPoolConfig::from_lookup(super::DbPoolTarget::Worker, |key| {
            Ok(env.get(key).cloned())
        }) {
            Ok(value) => value,
            Err(err) => panic!("worker db pool config should parse overrides: {err}"),
        };

        assert_eq!(api.max_connections, 4);
        assert_eq!(worker.max_connections, 3);
        assert_eq!(api.acquire_timeout, Duration::from_secs(2));
        assert_eq!(worker.acquire_timeout, Duration::from_secs(2));
        assert_eq!(api.idle_timeout, Duration::from_secs(60));
        assert_eq!(worker.idle_timeout, Duration::from_secs(60));
        assert_eq!(api.max_lifetime, Duration::from_secs(300));
        assert_eq!(worker.max_lifetime, Duration::from_secs(300));
    }

    #[test]
    fn pg_pool_options_from_config_applies_all_limits() {
        let config = super::DbPoolConfig {
            max_connections: 4,
            acquire_timeout: Duration::from_secs(2),
            idle_timeout: Duration::from_secs(60),
            max_lifetime: Duration::from_secs(300),
        };

        let options: PgPoolOptions = super::pg_pool_options_from_config(&config);

        assert_eq!(options.get_max_connections(), 4);
        assert_eq!(options.get_acquire_timeout(), Duration::from_secs(2));
        assert_eq!(options.get_idle_timeout(), Some(Duration::from_secs(60)));
        assert_eq!(options.get_max_lifetime(), Some(Duration::from_secs(300)));
    }
}
