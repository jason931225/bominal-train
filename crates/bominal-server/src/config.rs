use anyhow::{Context, Result};

#[derive(Clone, Debug)]
pub struct AppConfig {
    pub port: u16,
    pub database_url: String,
    pub encryption_key: String,
    pub environment: Environment,
    pub resend_api_key: String,
    pub email_from: String,
    pub app_base_url: String,
    pub ev_team_id: String,
    pub ev_app_id: String,
    pub ev_api_key: String,
    pub ev_srt_domain: String,
    pub ev_ktx_domain: String,
}

#[derive(Clone, Debug, PartialEq)]
pub enum Environment {
    Development,
    Production,
}

impl AppConfig {
    pub fn from_env() -> Result<Self> {
        let environment = match std::env::var("BOMINAL_ENV")
            .unwrap_or_else(|_| "development".into())
            .as_str()
        {
            "production" => Environment::Production,
            _ => Environment::Development,
        };

        Ok(Self {
            port: std::env::var("PORT")
                .unwrap_or_else(|_| "3000".into())
                .parse()
                .context("Invalid PORT")?,
            database_url: std::env::var("DATABASE_URL").context("DATABASE_URL is required")?,
            encryption_key: std::env::var("ENCRYPTION_KEY")
                .context("ENCRYPTION_KEY is required")?,
            resend_api_key: std::env::var("RESEND_API_KEY")
                .context("RESEND_API_KEY is required")?,
            email_from: std::env::var("EMAIL_FROM")
                .unwrap_or_else(|_| "Bominal <noreply@bominal.com>".into()),
            app_base_url: std::env::var("APP_BASE_URL")
                .unwrap_or_else(|_| "http://localhost:3000".into()),
            ev_team_id: std::env::var("EV_TEAM_ID").context("EV_TEAM_ID is required")?,
            ev_app_id: std::env::var("EV_APP_ID").context("EV_APP_ID is required")?,
            ev_api_key: std::env::var("EV_API_KEY").context("EV_API_KEY is required")?,
            ev_srt_domain: std::env::var("EV_SRT_DOMAIN").context("EV_SRT_DOMAIN is required")?,
            ev_ktx_domain: std::env::var("EV_KTX_DOMAIN").context("EV_KTX_DOMAIN is required")?,
            environment,
        })
    }

    pub fn is_production(&self) -> bool {
        self.environment == Environment::Production
    }

    /// Validate configuration values that can't be checked during parsing.
    pub fn validate(&self) -> Result<()> {
        // ENCRYPTION_KEY must be exactly 64 hex characters (32 bytes)
        if self.encryption_key.len() != 64
            || !self.encryption_key.chars().all(|c| c.is_ascii_hexdigit())
        {
            anyhow::bail!("ENCRYPTION_KEY must be exactly 64 hex characters (32 bytes)");
        }

        // Production must use HTTPS
        if self.is_production() && !self.app_base_url.starts_with("https://") {
            anyhow::bail!("APP_BASE_URL must use HTTPS in production");
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_test_config() -> AppConfig {
        AppConfig {
            port: 3000,
            database_url: "postgres://localhost/test".to_string(),
            encryption_key: "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef"
                .to_string(),
            environment: Environment::Development,
            resend_api_key: "re_test".to_string(),
            email_from: "test@test.com".to_string(),
            app_base_url: "http://localhost:3000".to_string(),
            ev_team_id: "team_test".to_string(),
            ev_app_id: "app_test".to_string(),
            ev_api_key: "ev_key".to_string(),
            ev_srt_domain: "srt.test".to_string(),
            ev_ktx_domain: "ktx.test".to_string(),
        }
    }

    #[test]
    fn validate_valid_config() {
        assert!(make_test_config().validate().is_ok());
    }

    #[test]
    fn validate_short_encryption_key() {
        let config = AppConfig {
            encryption_key: "too_short".to_string(),
            ..make_test_config()
        };
        let err = config.validate().unwrap_err();
        assert!(err.to_string().contains("ENCRYPTION_KEY"));
    }

    #[test]
    fn validate_non_hex_encryption_key() {
        let config = AppConfig {
            encryption_key: "zzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzz"
                .to_string(),
            ..make_test_config()
        };
        let err = config.validate().unwrap_err();
        assert!(err.to_string().contains("ENCRYPTION_KEY"));
    }

    #[test]
    fn validate_production_requires_https() {
        let config = AppConfig {
            environment: Environment::Production,
            app_base_url: "http://example.com".to_string(),
            ..make_test_config()
        };
        let err = config.validate().unwrap_err();
        assert!(err.to_string().contains("HTTPS"));
    }

    #[test]
    fn validate_production_with_https() {
        let config = AppConfig {
            environment: Environment::Production,
            app_base_url: "https://example.com".to_string(),
            ..make_test_config()
        };
        assert!(config.validate().is_ok());
    }
}
