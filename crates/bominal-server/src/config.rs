use anyhow::{Context, Result};

#[derive(Clone, Debug)]
pub struct AppConfig {
    pub port: u16,
    pub database_url: String,
    pub valkey_url: String,
    pub encryption_key: String,
    pub environment: Environment,
    pub resend_api_key: String,
    pub email_from: String,
    pub app_base_url: String,
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
            database_url: std::env::var("DATABASE_URL")
                .context("DATABASE_URL is required")?,
            valkey_url: std::env::var("VALKEY_URL")
                .unwrap_or_else(|_| "redis://127.0.0.1:6379".into()),
            encryption_key: std::env::var("ENCRYPTION_KEY")
                .context("ENCRYPTION_KEY is required")?,
            resend_api_key: std::env::var("RESEND_API_KEY")
                .context("RESEND_API_KEY is required")?,
            email_from: std::env::var("EMAIL_FROM")
                .unwrap_or_else(|_| "Bominal <noreply@bominal.com>".into()),
            app_base_url: std::env::var("APP_BASE_URL")
                .unwrap_or_else(|_| "http://localhost:3000".into()),
            environment,
        })
    }

    pub fn is_production(&self) -> bool {
        self.environment == Environment::Production
    }
}
