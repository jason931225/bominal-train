"""Application configuration management.

Loads settings from environment variables with sensible defaults for development.
Production deployments must override security-critical settings.

Environment Variables:
    See Settings class fields for all supported environment variables.
    
Security:
    - MASTER_KEY must be overridden in production (used for envelope encryption)
    - INTERNAL_API_KEY must be set in production (for internal service auth)
"""

from functools import lru_cache
from typing import Annotated
from typing import List

from pydantic import Field, field_validator, model_validator
from pydantic_settings import BaseSettings, NoDecode, SettingsConfigDict

DEFAULT_MASTER_KEY_B64 = "MDEyMzQ1Njc4OWFiY2RlZjAxMjM0NTY3ODlhYmNkZWY="


class Settings(BaseSettings):
    """Application settings loaded from environment variables.
    
    Grouped by functionality:
    - Core: app_name, app_env, database_url, redis_url
    - Auth: session_*, rate_limit_*
    - Security: master_key, kek_version, internal_api_key
    - Train: train_provider_*, payment_*
    - Email: email_*, smtp_*, resend_*
    """
    
    model_config = SettingsConfigDict(env_file=".env", extra="ignore")

    app_name: str = "bominal"
    app_env: str = "development"

    database_url: str = Field(
        default="postgresql+asyncpg://bominal:bominal@postgres:5432/bominal",
        alias="DATABASE_URL",
    )
    sync_database_url: str = Field(
        default="postgresql+psycopg://bominal:bominal@postgres:5432/bominal",
        alias="SYNC_DATABASE_URL",
    )

    cors_origins: Annotated[List[str], NoDecode] = Field(
        default=["http://localhost:3000"],
        alias="CORS_ORIGINS",
    )

    session_cookie_name: str = Field(default="bominal_session", alias="SESSION_COOKIE_NAME")
    session_days_default: int = Field(default=7, alias="SESSION_DAYS_DEFAULT")
    session_days_remember: int = Field(default=90, alias="SESSION_DAYS_REMEMBER")

    rate_limit_window_seconds: int = Field(default=60, alias="RATE_LIMIT_WINDOW_SECONDS")
    rate_limit_max_requests: int = Field(default=20, alias="RATE_LIMIT_MAX_REQUESTS")
    rate_limit_use_redis: bool = Field(default=False, alias="RATE_LIMIT_USE_REDIS")

    redis_url: str = Field(default="redis://redis:6379/0", alias="REDIS_URL")
    internal_api_key: str | None = Field(default=None, alias="INTERNAL_API_KEY")

    master_key: str = Field(
        default=DEFAULT_MASTER_KEY_B64,
        alias="MASTER_KEY",
    )
    kek_version: int = Field(default=1, alias="KEK_VERSION")

    train_provider_mode: str = Field(default="mock", alias="TRAIN_PROVIDER_MODE")
    train_provider_transport: str = Field(default="auto", alias="TRAIN_PROVIDER_TRANSPORT")
    train_poll_min_seconds: float = Field(default=2.0, alias="TRAIN_POLL_MIN_SECONDS")
    train_poll_max_seconds: float = Field(default=6.0, alias="TRAIN_POLL_MAX_SECONDS")
    train_credential_verify_timeout_seconds: float = Field(
        default=8.0,
        alias="TRAIN_CREDENTIAL_VERIFY_TIMEOUT_SECONDS",
    )
    train_credential_cache_seconds: int = Field(
        default=3600,
        alias="TRAIN_CREDENTIAL_CACHE_SECONDS",
    )
    payment_cvv_ttl_seconds: int = Field(default=3600, alias="PAYMENT_CVV_TTL_SECONDS")

    email_provider: str = Field(default="smtp", alias="EMAIL_PROVIDER")
    email_from_name: str = Field(default="bominal", alias="EMAIL_FROM_NAME")
    email_from_address: str = Field(default="no-reply@bominal.local", alias="EMAIL_FROM_ADDRESS")
    email_reply_to: str | None = Field(default=None, alias="EMAIL_REPLY_TO")
    resend_api_key: str | None = Field(default=None, alias="RESEND_API_KEY")
    resend_api_base_url: str = Field(default="https://api.resend.com", alias="RESEND_API_BASE_URL")
    smtp_host: str = Field(default="mailpit", alias="SMTP_HOST")
    smtp_port: int = Field(default=1025, alias="SMTP_PORT")
    smtp_username: str | None = Field(default=None, alias="SMTP_USERNAME")
    smtp_password: str | None = Field(default=None, alias="SMTP_PASSWORD")
    smtp_use_ssl: bool = Field(default=False, alias="SMTP_USE_SSL")
    smtp_starttls: bool = Field(default=False, alias="SMTP_STARTTLS")
    smtp_timeout_seconds: float = Field(default=10.0, alias="SMTP_TIMEOUT_SECONDS")

    @field_validator("cors_origins", mode="before")
    @classmethod
    def parse_cors_origins(cls, value: str | list[str]) -> list[str]:
        if isinstance(value, str):
            return [origin.strip() for origin in value.split(",") if origin.strip()]
        return value

    @field_validator("email_provider")
    @classmethod
    def validate_email_provider(cls, value: str) -> str:
        normalized = value.strip().lower()
        if normalized not in {"smtp", "resend", "log", "disabled"}:
            raise ValueError("EMAIL_PROVIDER must be one of: smtp, resend, log, disabled")
        return normalized

    @model_validator(mode="after")
    def validate_security_settings(self) -> "Settings":
        # Keep local developer convenience, but hard-stop weak defaults in production.
        if self.app_env.lower() == "production" and self.master_key == DEFAULT_MASTER_KEY_B64:
            raise ValueError("MASTER_KEY must be overridden in production")
        if self.app_env.lower() == "production" and not self.internal_api_key:
            raise ValueError("INTERNAL_API_KEY must be set in production")
        if self.smtp_use_ssl and self.smtp_starttls:
            raise ValueError("SMTP_USE_SSL and SMTP_STARTTLS cannot both be true")
        if self.email_provider == "resend" and not self.resend_api_key:
            raise ValueError("RESEND_API_KEY is required when EMAIL_PROVIDER=resend")
        return self

    @property
    def is_production(self) -> bool:
        return self.app_env.lower() == "production"


@lru_cache
def get_settings() -> Settings:
    return Settings()
