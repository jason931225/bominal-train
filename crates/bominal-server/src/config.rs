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
}
