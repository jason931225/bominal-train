"""Application configuration management.

Loads settings from environment variables with sensible defaults for development.
Production deployments must override security-critical settings.

Environment Variables:
    See Settings class fields for all supported environment variables.
    
Security:
    - MASTER_KEY must be overridden in production (used for envelope encryption)
    - INTERNAL_API_KEY must be set in production (for internal service auth)
"""

import json
from functools import lru_cache
from typing import Annotated
from typing import List
from urllib.parse import urlparse

from pydantic import Field, field_validator, model_validator
from pydantic_settings import BaseSettings, NoDecode, SettingsConfigDict

DEFAULT_MASTER_KEY_B64 = "MDEyMzQ1Njc4OWFiY2RlZjAxMjM0NTY3ODlhYmNkZWY="
UPSTASH_REDIS_HOST_SUFFIXES = ("upstash.io", "upstash.dev", "upstash.com")


def is_upstash_redis_url(url: str | None) -> bool:
    if not url:
        return False
    try:
        host = (urlparse(url).hostname or "").lower().strip()
    except Exception:
        return False
    if not host:
        return False
    return any(host == suffix or host.endswith(f".{suffix}") for suffix in UPSTASH_REDIS_HOST_SUFFIXES)


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
    app_public_base_url: str = Field(default="https://www.bominal.com", alias="APP_PUBLIC_BASE_URL")

    database_url: str = Field(
        default="postgresql+asyncpg://bominal:bominal@postgres:5432/bominal",
        alias="DATABASE_URL",
    )
    sync_database_url: str = Field(
        default="postgresql+psycopg://bominal:bominal@postgres:5432/bominal",
        alias="SYNC_DATABASE_URL",
    )

    cors_origins: Annotated[List[str], NoDecode] = Field(
        default=[
            "http://localhost:3000",
            "http://127.0.0.1:3000",
            "http://0.0.0.0:3000",
        ],
        alias="CORS_ORIGINS",
    )

    session_cookie_name: str = Field(default="bominal_session", alias="SESSION_COOKIE_NAME")
    session_days_default: int = Field(default=7, alias="SESSION_DAYS_DEFAULT")
    session_days_remember: int = Field(default=90, alias="SESSION_DAYS_REMEMBER")
    session_activity_debounce_seconds: int = Field(default=60, alias="SESSION_ACTIVITY_DEBOUNCE_SECONDS", ge=0)
    access_approval_required: bool = Field(default=True, alias="ACCESS_APPROVAL_REQUIRED")
    # e2-micro-safe Argon2id defaults; override in env for larger instances.
    password_hash_time_cost: int = Field(default=2, alias="PASSWORD_HASH_TIME_COST")
    password_hash_memory_cost_kib: int = Field(default=16384, alias="PASSWORD_HASH_MEMORY_COST_KIB")
    password_hash_parallelism: int = Field(default=1, alias="PASSWORD_HASH_PARALLELISM")

    rate_limit_window_seconds: int = Field(default=60, alias="RATE_LIMIT_WINDOW_SECONDS")
    rate_limit_max_requests: int = Field(default=20, alias="RATE_LIMIT_MAX_REQUESTS")
    rate_limit_use_redis: bool = Field(default=False, alias="RATE_LIMIT_USE_REDIS")

    redis_url: str = Field(default="redis://redis:6379/0", alias="REDIS_URL")
    redis_url_non_cde: str | None = Field(default=None, alias="REDIS_URL_NON_CDE")
    redis_url_cde: str | None = Field(default=None, alias="REDIS_URL_CDE")
    internal_api_key: str | None = Field(default=None, alias="INTERNAL_API_KEY")
    internal_identity_secret: str | None = Field(default=None, alias="INTERNAL_IDENTITY_SECRET")
    internal_identity_issuer: str = Field(default="bominal-internal", alias="INTERNAL_IDENTITY_ISSUER")
    internal_identity_ttl_seconds: int = Field(default=120, alias="INTERNAL_IDENTITY_TTL_SECONDS")

    master_key: str = Field(
        default=DEFAULT_MASTER_KEY_B64,
        alias="MASTER_KEY",
    )
    master_keys_by_version: dict[int, str] | None = Field(default=None, alias="MASTER_KEYS_BY_VERSION")
    kek_version: int = Field(default=1, alias="KEK_VERSION")
    kek_retirement_window_days: int = Field(default=30, alias="KEK_RETIREMENT_WINDOW_DAYS")
    auth_mode: str = Field(default="legacy", alias="AUTH_MODE")
    supabase_url: str | None = Field(default=None, alias="SUPABASE_URL")
    supabase_jwks_url: str | None = Field(default=None, alias="SUPABASE_JWKS_URL")
    supabase_jwt_issuer: str | None = Field(default=None, alias="SUPABASE_JWT_ISSUER")
    supabase_jwt_audience: str = Field(default="authenticated", alias="SUPABASE_JWT_AUDIENCE")
    supabase_auth_enabled: bool = Field(default=False, alias="SUPABASE_AUTH_ENABLED")
    supabase_auth_api_key: str | None = Field(default=None, alias="SUPABASE_AUTH_API_KEY")
    supabase_auth_timeout_seconds: float = Field(default=12.0, alias="SUPABASE_AUTH_TIMEOUT_SECONDS")
    supabase_storage_bucket: str = Field(default="artifacts-safe", alias="SUPABASE_STORAGE_BUCKET")
    supabase_service_role_key: str | None = Field(default=None, alias="SUPABASE_SERVICE_ROLE_KEY")
    supabase_storage_enabled: bool = Field(default=False, alias="SUPABASE_STORAGE_ENABLED")
    passkey_enabled: bool = Field(default=True, alias="PASSKEY_ENABLED")
    passkey_rp_id: str | None = Field(default=None, alias="PASSKEY_RP_ID")
    passkey_origin: str | None = Field(default=None, alias="PASSKEY_ORIGIN")
    passkey_timeout_ms: int = Field(default=60000, alias="PASSKEY_TIMEOUT_MS")

    train_provider_mode: str = Field(default="mock", alias="TRAIN_PROVIDER_MODE")
    train_provider_transport: str = Field(default="auto", alias="TRAIN_PROVIDER_TRANSPORT")
    train_provider_timeout_connect_seconds: float = Field(
        default=3.0,
        alias="TRAIN_PROVIDER_TIMEOUT_CONNECT_SECONDS",
    )
    train_provider_timeout_read_seconds: float = Field(
        default=8.0,
        alias="TRAIN_PROVIDER_TIMEOUT_READ_SECONDS",
    )
    train_provider_timeout_total_seconds: float = Field(
        default=12.0,
        alias="TRAIN_PROVIDER_TIMEOUT_TOTAL_SECONDS",
    )
    train_provider_egress_proxy_url: str | None = Field(
        default=None,
        alias="TRAIN_PROVIDER_EGRESS_PROXY_URL",
    )
    train_provider_retry_attempts: int = Field(
        default=2,
        alias="TRAIN_PROVIDER_RETRY_ATTEMPTS",
    )
    train_provider_retry_backoff_seconds: float = Field(
        default=0.2,
        alias="TRAIN_PROVIDER_RETRY_BACKOFF_SECONDS",
    )
    train_poll_min_seconds: float = Field(default=2.0, alias="TRAIN_POLL_MIN_SECONDS")
    train_poll_max_seconds: float = Field(default=6.0, alias="TRAIN_POLL_MAX_SECONDS")
    train_poll_force_max_rate: bool = Field(default=False, alias="TRAIN_POLL_FORCE_MAX_RATE")
    train_credential_verify_timeout_seconds: float = Field(
        default=8.0,
        alias="TRAIN_CREDENTIAL_VERIFY_TIMEOUT_SECONDS",
    )
    train_credential_cache_seconds: int = Field(
        default=3600,
        alias="TRAIN_CREDENTIAL_CACHE_SECONDS",
    )
    train_persist_all_attempts: bool = Field(
        default=False,
        alias="TRAIN_PERSIST_ALL_ATTEMPTS",
    )
    train_attempt_retention_days: int = Field(
        default=30,
        alias="TRAIN_ATTEMPT_RETENTION_DAYS",
        ge=1,
    )
    train_compact_repetitive_attempts: bool = Field(
        default=True,
        alias="TRAIN_COMPACT_REPETITIVE_ATTEMPTS",
    )
    train_sync_keep_latest_only: bool = Field(
        default=True,
        alias="TRAIN_SYNC_KEEP_LATEST_ONLY",
    )
    restaurant_auth_refresh_retries: int = Field(default=2, alias="RESTAURANT_AUTH_REFRESH_RETRIES")
    restaurant_payment_lease_ttl_seconds: int = Field(default=120, alias="RESTAURANT_PAYMENT_LEASE_TTL_SECONDS")
    restaurant_bootstrap_timeout_seconds: float = Field(default=20.0, alias="RESTAURANT_BOOTSTRAP_TIMEOUT_SECONDS")
    restaurant_opentable_base_url: str = Field(default="https://www.opentable.com", alias="RESTAURANT_OPENTABLE_BASE_URL")
    restaurant_opentable_timeout_seconds: float = Field(default=20.0, alias="RESTAURANT_OPENTABLE_TIMEOUT_SECONDS")
    restaurant_opentable_auth_start_path: str = Field(
        default="/dapi/authentication/sendotpfromsignin",
        alias="RESTAURANT_OPENTABLE_AUTH_START_PATH",
    )
    restaurant_opentable_auth_complete_path: str = Field(
        default="/dapi/authentication/signinwithotp",
        alias="RESTAURANT_OPENTABLE_AUTH_COMPLETE_PATH",
    )
    restaurant_opentable_autocomplete_operation_name: str = Field(
        default="Autocomplete",
        alias="RESTAURANT_OPENTABLE_AUTOCOMPLETE_OPERATION_NAME",
    )
    restaurant_opentable_autocomplete_operation_sha256: str = Field(
        default="fe1d118abd4c227750693027c2414d43014c2493f64f49bcef5a65274ce9c3c3",
        alias="RESTAURANT_OPENTABLE_AUTOCOMPLETE_OPERATION_SHA256",
    )
    restaurant_opentable_search_operation_name: str = Field(
        default="RestaurantsAvailability",
        alias="RESTAURANT_OPENTABLE_SEARCH_OPERATION_NAME",
    )
    restaurant_opentable_search_operation_sha256: str = Field(
        default="b2d05a06151b3cb21d9dfce4f021303eeba288fac347068b29c1cb66badc46af",
        alias="RESTAURANT_OPENTABLE_SEARCH_OPERATION_SHA256",
    )
    restaurant_opentable_search_slot_path: str = Field(
        default="data.availability",
        alias="RESTAURANT_OPENTABLE_SEARCH_SLOT_PATH",
    )
    restaurant_opentable_create_operation_name: str = Field(
        default="BookDetailsStandardSlotLock",
        alias="RESTAURANT_OPENTABLE_CREATE_OPERATION_NAME",
    )
    restaurant_opentable_create_operation_sha256: str = Field(
        default="1100bf68905fd7cb1d4fd0f4504a4954aa28ec45fb22913fa977af8b06fd97fa",
        alias="RESTAURANT_OPENTABLE_CREATE_OPERATION_SHA256",
    )
    restaurant_opentable_create_path: str = Field(
        default="/dapi/booking/make-reservation",
        alias="RESTAURANT_OPENTABLE_CREATE_PATH",
    )
    restaurant_opentable_confirmation_operation_name: str = Field(
        default="BookingConfirmationPageInFlow",
        alias="RESTAURANT_OPENTABLE_CONFIRMATION_OPERATION_NAME",
    )
    restaurant_opentable_confirmation_operation_sha256: str = Field(
        default="6be25f0bbc8fe75483bdfe96ae78fb20075b978842e4b44964aed3591611aa99",
        alias="RESTAURANT_OPENTABLE_CONFIRMATION_OPERATION_SHA256",
    )
    restaurant_resy_base_url: str = Field(default="https://api.resy.com", alias="RESTAURANT_RESY_BASE_URL")
    restaurant_resy_timeout_seconds: float = Field(default=20.0, alias="RESTAURANT_RESY_TIMEOUT_SECONDS")
    restaurant_resy_auth_password_path: str = Field(default="/4/auth/password", alias="RESTAURANT_RESY_AUTH_PASSWORD_PATH")
    restaurant_resy_auth_api_key: str | None = Field(default=None, alias="RESTAURANT_RESY_AUTH_API_KEY")
    restaurant_resy_x_origin: str = Field(default="https://resy.com", alias="RESTAURANT_RESY_X_ORIGIN")
    restaurant_resy_profile_path: str = Field(default="/2/user", alias="RESTAURANT_RESY_PROFILE_PATH")
    restaurant_resy_search_path: str = Field(default="/4/find", alias="RESTAURANT_RESY_SEARCH_PATH")
    restaurant_resy_create_details_path: str = Field(default="/3/details", alias="RESTAURANT_RESY_CREATE_DETAILS_PATH")
    restaurant_resy_create_book_path: str = Field(default="/3/book", alias="RESTAURANT_RESY_CREATE_BOOK_PATH")
    restaurant_resy_cancel_path: str = Field(default="/3/cancel", alias="RESTAURANT_RESY_CANCEL_PATH")
    restaurant_resy_source_id: str = Field(default="resy.com-venue-details", alias="RESTAURANT_RESY_SOURCE_ID")
    restaurant_resy_refresh_path: str = Field(default="/3/auth/refresh", alias="RESTAURANT_RESY_REFRESH_PATH")
    restaurant_resy_logout_path: str = Field(default="/3/auth/logout", alias="RESTAURANT_RESY_LOGOUT_PATH")
    payment_cvv_ttl_seconds: int = Field(default=600, alias="PAYMENT_CVV_TTL_SECONDS")
    payment_cvv_ttl_min_seconds: int = Field(default=60, alias="PAYMENT_CVV_TTL_MIN_SECONDS")
    payment_cvv_ttl_max_seconds: int = Field(default=900, alias="PAYMENT_CVV_TTL_MAX_SECONDS")
    payment_provider_allowed_hosts: Annotated[List[str], NoDecode] = Field(
        default=["app.srail.or.kr", "letskorail.com"],
        alias="PAYMENT_PROVIDER_ALLOWED_HOSTS",
    )
    payment_transport_trust_env: bool = Field(default=False, alias="PAYMENT_TRANSPORT_TRUST_ENV")
    restaurant_provider_egress_proxy_url: str | None = Field(
        default=None,
        alias="RESTAURANT_PROVIDER_EGRESS_PROXY_URL",
    )
    payment_enabled: bool = Field(default=True, alias="PAYMENT_ENABLED")
    payment_require_cvv_kek_version: bool = Field(default=False, alias="PAYMENT_REQUIRE_CVV_KEK_VERSION")

    email_provider: str = Field(default="smtp", alias="EMAIL_PROVIDER")
    email_from_name: str = Field(default="bominal", alias="EMAIL_FROM_NAME")
    email_from_address: str = Field(default="no-reply@bominal.local", alias="EMAIL_FROM_ADDRESS")
    email_reply_to: str | None = Field(default=None, alias="EMAIL_REPLY_TO")
    resend_api_key: str | None = Field(default=None, alias="RESEND_API_KEY")
    resend_api_base_url: str = Field(default="https://api.resend.com", alias="RESEND_API_BASE_URL")
    resend_timeout_seconds: float = Field(default=12.0, alias="RESEND_TIMEOUT_SECONDS")
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

    @field_validator("payment_provider_allowed_hosts", mode="before")
    @classmethod
    def parse_payment_provider_allowed_hosts(cls, value: str | list[str]) -> list[str]:
        if isinstance(value, str):
            parsed = [host.strip().lower() for host in value.split(",") if host.strip()]
            return parsed
        return [str(host).strip().lower() for host in value if str(host).strip()]

    @field_validator("train_provider_egress_proxy_url", "restaurant_provider_egress_proxy_url", mode="before")
    @classmethod
    def parse_optional_proxy_url(cls, value: str | None) -> str | None:
        if value is None:
            return None
        normalized = str(value).strip()
        return normalized or None

    @field_validator("master_keys_by_version", mode="before")
    @classmethod
    def parse_master_keys_by_version(cls, value: object) -> dict[int, str] | None:
        if value is None:
            return None
        if isinstance(value, str):
            normalized = value.strip()
            if not normalized:
                return None
            parsed = json.loads(normalized)
        else:
            parsed = value
        if not isinstance(parsed, dict):
            raise ValueError("MASTER_KEYS_BY_VERSION must be a JSON object")
        keyring: dict[int, str] = {}
        for raw_version, raw_key in parsed.items():
            try:
                version = int(raw_version)
            except Exception as exc:
                raise ValueError("MASTER_KEYS_BY_VERSION keys must be integers") from exc
            key_value = str(raw_key).strip()
            if not key_value:
                raise ValueError("MASTER_KEYS_BY_VERSION values must be non-empty base64 strings")
            keyring[version] = key_value
        return keyring

    @field_validator("email_provider")
    @classmethod
    def validate_email_provider(cls, value: str) -> str:
        normalized = value.strip().lower()
        if normalized not in {"smtp", "resend", "log", "disabled"}:
            raise ValueError("EMAIL_PROVIDER must be one of: smtp, resend, log, disabled")
        return normalized

    @field_validator("auth_mode")
    @classmethod
    def validate_auth_mode(cls, value: str) -> str:
        normalized = value.strip().lower()
        if normalized not in {"legacy", "supabase", "dual"}:
            raise ValueError("AUTH_MODE must be one of: legacy, supabase, dual")
        return normalized

    @field_validator("train_provider_retry_attempts")
    @classmethod
    def validate_train_provider_retry_attempts(cls, value: int) -> int:
        if value < 1:
            raise ValueError("TRAIN_PROVIDER_RETRY_ATTEMPTS must be at least 1")
        return value

    @field_validator(
        "train_provider_timeout_connect_seconds",
        "train_provider_timeout_read_seconds",
        "train_provider_timeout_total_seconds",
        "train_provider_retry_backoff_seconds",
    )
    @classmethod
    def validate_non_negative_train_provider_float_settings(cls, value: float) -> float:
        if value < 0:
            raise ValueError("TRAIN_PROVIDER timeout/retry float settings must be non-negative")
        return value

    @field_validator(
        "password_hash_time_cost",
        "password_hash_memory_cost_kib",
        "password_hash_parallelism",
    )
    @classmethod
    def validate_positive_password_hash_settings(cls, value: int) -> int:
        if value < 1:
            raise ValueError("PASSWORD_HASH_* settings must be >= 1")
        return value

    @model_validator(mode="after")
    def validate_security_settings(self) -> "Settings":
        # Keep local developer convenience, but hard-stop weak defaults in production.
        if self.app_env.lower() == "production" and self.master_key == DEFAULT_MASTER_KEY_B64:
            raise ValueError("MASTER_KEY must be overridden in production")
        if self.app_env.lower() == "production" and not self.internal_api_key:
            raise ValueError("INTERNAL_API_KEY must be set in production")
        if self.internal_identity_ttl_seconds < 1:
            raise ValueError("INTERNAL_IDENTITY_TTL_SECONDS must be >= 1")
        if self.kek_retirement_window_days < 1:
            raise ValueError("KEK_RETIREMENT_WINDOW_DAYS must be >= 1")
        if self.master_keys_by_version is not None:
            if self.kek_version not in self.master_keys_by_version and not self.master_key:
                raise ValueError(
                    "MASTER_KEYS_BY_VERSION must include KEK_VERSION or MASTER_KEY must be set"
                )
        if self.payment_enabled and is_upstash_redis_url(self.resolved_redis_url_cde):
            raise ValueError(
                "REDIS_URL_CDE (or REDIS_URL fallback) cannot point to Upstash for CDE/CVV usage; "
                "use a non-durable Redis runtime for CDE data"
            )
        if self.auth_mode == "supabase":
            if not self.supabase_url:
                raise ValueError("SUPABASE_URL must be set when AUTH_MODE is supabase")
            if not self.supabase_jwt_issuer:
                raise ValueError("SUPABASE_JWT_ISSUER must be set when AUTH_MODE is supabase")
        if self.auth_mode == "dual":
            has_url = bool(self.supabase_url)
            has_issuer = bool(self.supabase_jwt_issuer)
            if has_url != has_issuer:
                raise ValueError(
                    "SUPABASE_URL and SUPABASE_JWT_ISSUER must be set together when AUTH_MODE=dual"
                )
        if self.passkey_timeout_ms < 1:
            raise ValueError("PASSKEY_TIMEOUT_MS must be >= 1")
        if self.supabase_auth_enabled:
            if not self.supabase_url:
                raise ValueError("SUPABASE_URL must be set when SUPABASE_AUTH_ENABLED=true")
            if not self.resolved_supabase_auth_api_key:
                raise ValueError(
                    "SUPABASE_AUTH_API_KEY (or SUPABASE_SERVICE_ROLE_KEY fallback) is required when SUPABASE_AUTH_ENABLED=true"
                )
            if self.supabase_auth_timeout_seconds <= 0:
                raise ValueError("SUPABASE_AUTH_TIMEOUT_SECONDS must be > 0 when SUPABASE_AUTH_ENABLED=true")
        if self.passkey_enabled:
            if not (self.passkey_rp_id or self.app_public_base_url):
                raise ValueError("PASSKEY_RP_ID or APP_PUBLIC_BASE_URL must be set when PASSKEY_ENABLED=true")
            if self.app_env.lower() == "production" and not (self.passkey_origin or self.app_public_base_url):
                raise ValueError("PASSKEY_ORIGIN or APP_PUBLIC_BASE_URL must be set when PASSKEY_ENABLED=true")
        if self.supabase_storage_enabled and not self.supabase_service_role_key:
            raise ValueError("SUPABASE_SERVICE_ROLE_KEY is required when SUPABASE_STORAGE_ENABLED=true")
        if self.payment_enabled:
            if self.payment_cvv_ttl_min_seconds < 1:
                raise ValueError("PAYMENT_CVV_TTL_MIN_SECONDS must be >= 1")
            if self.payment_cvv_ttl_max_seconds < self.payment_cvv_ttl_min_seconds:
                raise ValueError("PAYMENT_CVV_TTL_MAX_SECONDS must be >= PAYMENT_CVV_TTL_MIN_SECONDS")
            if not (self.payment_cvv_ttl_min_seconds <= self.payment_cvv_ttl_seconds <= self.payment_cvv_ttl_max_seconds):
                raise ValueError(
                    "PAYMENT_CVV_TTL_SECONDS must be within PAYMENT_CVV_TTL_MIN_SECONDS..PAYMENT_CVV_TTL_MAX_SECONDS"
                )
            if self.app_env.lower() == "production" and not self.payment_provider_allowed_hosts:
                raise ValueError("PAYMENT_PROVIDER_ALLOWED_HOSTS must be set in production")
        if self.smtp_use_ssl and self.smtp_starttls:
            raise ValueError("SMTP_USE_SSL and SMTP_STARTTLS cannot both be true")
        if self.email_provider == "resend" and not self.resend_api_key:
            raise ValueError("RESEND_API_KEY is required when EMAIL_PROVIDER=resend")
        return self

    @property
    def is_production(self) -> bool:
        return self.app_env.lower() == "production"

    @property
    def resolved_redis_url_non_cde(self) -> str:
        return (self.redis_url_non_cde or self.redis_url).strip()

    @property
    def resolved_redis_url_cde(self) -> str:
        return (self.redis_url_cde or self.redis_url).strip()

    @property
    def resolved_supabase_jwks_url(self) -> str | None:
        if self.supabase_jwks_url:
            return self.supabase_jwks_url
        if not self.supabase_url:
            return None
        return f"{self.supabase_url.rstrip('/')}/auth/v1/.well-known/jwks.json"

    @property
    def resolved_supabase_auth_api_key(self) -> str | None:
        if self.supabase_auth_api_key:
            return self.supabase_auth_api_key
        return self.supabase_service_role_key


@lru_cache
def get_settings() -> Settings:
    return Settings()
