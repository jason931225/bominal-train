import base64

import pytest

from app.core.config import Settings, is_upstash_redis_url


_VALID_MASTER_KEY = base64.b64encode(b"x" * 32).decode("utf-8")


@pytest.fixture(autouse=True)
def _enable_payment_guards_by_default(monkeypatch):
    monkeypatch.setenv("PAYMENT_ENABLED", "true")


def test_rejects_invalid_cvv_ttl_bounds(monkeypatch) -> None:
    monkeypatch.setenv("APP_ENV", "development")
    monkeypatch.setenv("PAYMENT_CVV_TTL_SECONDS", "1200")
    monkeypatch.setenv("PAYMENT_CVV_TTL_MIN_SECONDS", "60")
    monkeypatch.setenv("PAYMENT_CVV_TTL_MAX_SECONDS", "900")
    with pytest.raises(ValueError):
        Settings(_env_file=None)


def test_payment_disabled_skips_cde_upstash_and_ttl_guards(monkeypatch) -> None:
    monkeypatch.setenv("APP_ENV", "development")
    monkeypatch.setenv("PAYMENT_ENABLED", "false")
    monkeypatch.setenv("REDIS_URL", "rediss://example.upstash.io:6379/0")
    monkeypatch.setenv("PAYMENT_CVV_TTL_SECONDS", "1200")
    monkeypatch.setenv("PAYMENT_CVV_TTL_MIN_SECONDS", "60")
    monkeypatch.setenv("PAYMENT_CVV_TTL_MAX_SECONDS", "900")

    settings = Settings(_env_file=None)
    assert settings.payment_enabled is False
    assert settings.resolved_redis_url_cde == "rediss://example.upstash.io:6379/0"


def test_requires_provider_allowlist_in_production(monkeypatch) -> None:
    monkeypatch.setenv("APP_ENV", "production")
    monkeypatch.setenv("MASTER_KEY", _VALID_MASTER_KEY)
    monkeypatch.setenv("INTERNAL_API_KEY", "internal-key")
    monkeypatch.setenv("PAYMENT_PROVIDER_ALLOWED_HOSTS", "")
    with pytest.raises(ValueError):
        Settings(_env_file=None)


def test_parses_provider_allowlist_from_csv(monkeypatch) -> None:
    monkeypatch.setenv("APP_ENV", "production")
    monkeypatch.setenv("MASTER_KEY", _VALID_MASTER_KEY)
    monkeypatch.setenv("INTERNAL_API_KEY", "internal-key")
    monkeypatch.setenv("PAYMENT_PROVIDER_ALLOWED_HOSTS", "app.srail.or.kr, smart.letskorail.com")
    settings = Settings(
        _env_file=None,
    )

    assert settings.payment_provider_allowed_hosts == ["app.srail.or.kr", "smart.letskorail.com"]

def test_payment_provider_rejects_unknown_mode(monkeypatch) -> None:
    monkeypatch.setenv("APP_ENV", "development")
    monkeypatch.setenv("PAYMENT_PROVIDER", "custom")
    with pytest.raises(ValueError, match="PAYMENT_PROVIDER must be one of: legacy, evervault"):
        Settings(_env_file=None)

def test_production_allows_gsm_master_key_without_env_master_key(monkeypatch) -> None:
    monkeypatch.setenv("APP_ENV", "production")
    monkeypatch.setenv("MASTER_KEY", "")
    monkeypatch.setenv("INTERNAL_API_KEY", "internal-key")
    monkeypatch.setenv("GSM_MASTER_KEY_ENABLED", "true")
    monkeypatch.setenv("GCP_PROJECT_ID", "bominal")
    monkeypatch.setenv("GSM_MASTER_KEY_SECRET_ID", "bominal-master-key")
    monkeypatch.setenv("GSM_MASTER_KEY_VERSION", "5")
    monkeypatch.setenv("GSM_MASTER_KEY_ALLOW_ENV_FALLBACK", "false")

    settings = Settings(_env_file=None)
    assert settings.gsm_master_key_enabled is True
    assert settings.resolved_gsm_master_key_project_id == "bominal"


def test_production_rejects_gsm_latest_version(monkeypatch) -> None:
    monkeypatch.setenv("APP_ENV", "production")
    monkeypatch.setenv("MASTER_KEY", "")
    monkeypatch.setenv("INTERNAL_API_KEY", "internal-key")
    monkeypatch.setenv("GSM_MASTER_KEY_ENABLED", "true")
    monkeypatch.setenv("GCP_PROJECT_ID", "bominal")
    monkeypatch.setenv("GSM_MASTER_KEY_SECRET_ID", "bominal-master-key")
    monkeypatch.setenv("GSM_MASTER_KEY_VERSION", "latest")
    monkeypatch.setenv("GSM_MASTER_KEY_ALLOW_ENV_FALLBACK", "false")

    with pytest.raises(ValueError, match="must be pinned"):
        Settings(_env_file=None)


def test_production_rejects_gsm_env_fallback(monkeypatch) -> None:
    monkeypatch.setenv("APP_ENV", "production")
    monkeypatch.setenv("MASTER_KEY", _VALID_MASTER_KEY)
    monkeypatch.setenv("INTERNAL_API_KEY", "internal-key")
    monkeypatch.setenv("GSM_MASTER_KEY_ENABLED", "true")
    monkeypatch.setenv("GCP_PROJECT_ID", "bominal")
    monkeypatch.setenv("GSM_MASTER_KEY_SECRET_ID", "bominal-master-key")
    monkeypatch.setenv("GSM_MASTER_KEY_VERSION", "2")
    monkeypatch.setenv("GSM_MASTER_KEY_ALLOW_ENV_FALLBACK", "true")

    with pytest.raises(ValueError, match="ALLOW_ENV_FALLBACK must be false"):
        Settings(_env_file=None)


def test_evervault_enforce_rejects_server_fallback(monkeypatch) -> None:
    monkeypatch.setenv("APP_ENV", "development")
    monkeypatch.setenv("PAYMENT_PROVIDER", "evervault")
    monkeypatch.setenv("PAYMENT_EVERVAULT_ENFORCE", "true")
    monkeypatch.setenv("AUTOPAY_REQUIRE_USER_WALLET", "true")
    monkeypatch.setenv("AUTOPAY_ALLOW_SERVER_FALLBACK", "true")
    with pytest.raises(ValueError, match="AUTOPAY_ALLOW_SERVER_FALLBACK must be false"):
        Settings(_env_file=None)


def test_evervault_enforce_requires_user_wallet(monkeypatch) -> None:
    monkeypatch.setenv("APP_ENV", "development")
    monkeypatch.setenv("PAYMENT_PROVIDER", "evervault")
    monkeypatch.setenv("PAYMENT_EVERVAULT_ENFORCE", "true")
    monkeypatch.setenv("AUTOPAY_REQUIRE_USER_WALLET", "false")
    monkeypatch.setenv("AUTOPAY_ALLOW_SERVER_FALLBACK", "false")
    with pytest.raises(ValueError, match="AUTOPAY_REQUIRE_USER_WALLET must be true"):
        Settings(_env_file=None)


def test_evervault_enforce_requires_runtime_credentials(monkeypatch) -> None:
    monkeypatch.setenv("APP_ENV", "development")
    monkeypatch.setenv("PAYMENT_PROVIDER", "evervault")
    monkeypatch.setenv("PAYMENT_EVERVAULT_ENFORCE", "true")
    monkeypatch.setenv("AUTOPAY_REQUIRE_USER_WALLET", "true")
    monkeypatch.setenv("AUTOPAY_ALLOW_SERVER_FALLBACK", "false")
    monkeypatch.delenv("EVERVAULT_APP_ID", raising=False)
    monkeypatch.delenv("EVERVAULT_API_KEY", raising=False)
    with pytest.raises(ValueError, match="EVERVAULT_APP_ID and EVERVAULT_API_KEY"):
        Settings(_env_file=None)
def test_parses_optional_egress_proxy_urls(monkeypatch) -> None:
    monkeypatch.setenv("APP_ENV", "development")
    monkeypatch.setenv("TRAIN_PROVIDER_EGRESS_PROXY_URL", " http://egress-train:8080 ")
    monkeypatch.setenv("RESTAURANT_PROVIDER_EGRESS_PROXY_URL", "")

    settings = Settings(_env_file=None)

    assert settings.train_provider_egress_proxy_url == "http://egress-train:8080"
    assert settings.restaurant_provider_egress_proxy_url is None


def test_resolved_database_url_async_translates_sslmode_for_asyncpg(monkeypatch) -> None:
    monkeypatch.setenv("APP_ENV", "test")
    monkeypatch.setenv(
        "DATABASE_URL",
        "postgresql+asyncpg://user:pass@db.example:6543/postgres?sslmode=require&application_name=bominal",
    )

    settings = Settings(_env_file=None)

    assert "sslmode=" not in settings.resolved_database_url_async
    assert "ssl=require" in settings.resolved_database_url_async
    assert "application_name=bominal" in settings.resolved_database_url_async


def test_resolved_database_url_async_preserves_explicit_ssl_for_asyncpg(monkeypatch) -> None:
    monkeypatch.setenv("APP_ENV", "test")
    monkeypatch.setenv(
        "DATABASE_URL",
        "postgresql+asyncpg://user:pass@db.example:6543/postgres?ssl=require&application_name=bominal",
    )

    settings = Settings(_env_file=None)

    assert settings.resolved_database_url_async == settings.database_url


def test_legacy_mode_allows_missing_supabase_configuration(monkeypatch) -> None:
    monkeypatch.setenv("APP_ENV", "development")
    monkeypatch.setenv("AUTH_MODE", "legacy")
    monkeypatch.delenv("SUPABASE_URL", raising=False)
    monkeypatch.delenv("SUPABASE_JWT_ISSUER", raising=False)

    settings = Settings(_env_file=None)
    assert settings.auth_mode == "legacy"
    assert settings.supabase_url is None
    assert settings.supabase_jwt_issuer is None


def test_auth_mode_rejects_dual(monkeypatch) -> None:
    monkeypatch.setenv("APP_ENV", "development")
    monkeypatch.setenv("AUTH_MODE", "dual")

    with pytest.raises(ValueError, match="AUTH_MODE must be one of: legacy, supabase"):
        Settings(_env_file=None)


def test_supabase_mode_requires_jwt_issuer(monkeypatch) -> None:
    monkeypatch.setenv("APP_ENV", "development")
    monkeypatch.setenv("AUTH_MODE", "supabase")
    monkeypatch.setenv("SUPABASE_URL", "https://project.supabase.co")
    monkeypatch.delenv("SUPABASE_JWT_ISSUER", raising=False)

    with pytest.raises(ValueError):
        Settings(_env_file=None)


def test_development_rejects_non_local_database_url(monkeypatch) -> None:
    monkeypatch.setenv("APP_ENV", "development")
    monkeypatch.setenv("AUTH_MODE", "legacy")
    monkeypatch.setenv("DATABASE_URL", "postgresql+asyncpg://user:pass@db.example:5432/bominal")

    with pytest.raises(ValueError, match="DATABASE_URL must point to local Postgres hostnames"):
        Settings(_env_file=None)


def test_development_rejects_non_local_sync_database_url(monkeypatch) -> None:
    monkeypatch.setenv("APP_ENV", "development")
    monkeypatch.setenv("AUTH_MODE", "legacy")
    monkeypatch.setenv("SYNC_DATABASE_URL", "postgresql+psycopg://user:pass@db.example:5432/bominal")

    with pytest.raises(ValueError, match="SYNC_DATABASE_URL must point to local Postgres hostnames"):
        Settings(_env_file=None)


def test_supabase_auth_enabled_requires_auth_api_key(monkeypatch) -> None:
    monkeypatch.setenv("APP_ENV", "development")
    monkeypatch.setenv("SUPABASE_AUTH_ENABLED", "true")
    monkeypatch.setenv("SUPABASE_URL", "https://project.supabase.co")
    monkeypatch.setenv("SUPABASE_JWT_ISSUER", "https://project.supabase.co/auth/v1")
    monkeypatch.setenv("SUPABASE_AUTH_API_KEY", "")
    monkeypatch.setenv("SUPABASE_SERVICE_ROLE_KEY", "")
    with pytest.raises(ValueError, match="SUPABASE_AUTH_API_KEY"):
        Settings(_env_file=None)


def test_supabase_auth_enabled_requires_supabase_url(monkeypatch) -> None:
    monkeypatch.setenv("APP_ENV", "development")
    monkeypatch.setenv("SUPABASE_AUTH_ENABLED", "true")
    monkeypatch.setenv("SUPABASE_URL", "")
    monkeypatch.setenv("SUPABASE_JWT_ISSUER", "https://project.supabase.co/auth/v1")
    monkeypatch.setenv("SUPABASE_AUTH_API_KEY", "anon-key")
    with pytest.raises(ValueError, match="SUPABASE_URL must be set"):
        Settings(_env_file=None)


def test_supabase_auth_enabled_requires_positive_timeout(monkeypatch) -> None:
    monkeypatch.setenv("APP_ENV", "development")
    monkeypatch.setenv("SUPABASE_AUTH_ENABLED", "true")
    monkeypatch.setenv("SUPABASE_URL", "https://project.supabase.co")
    monkeypatch.setenv("SUPABASE_JWT_ISSUER", "https://project.supabase.co/auth/v1")
    monkeypatch.setenv("SUPABASE_AUTH_API_KEY", "anon-key")
    monkeypatch.setenv("SUPABASE_AUTH_TIMEOUT_SECONDS", "0")
    with pytest.raises(ValueError, match="SUPABASE_AUTH_TIMEOUT_SECONDS must be > 0"):
        Settings(_env_file=None)


def test_supabase_auth_enabled_allows_service_role_key_fallback(monkeypatch) -> None:
    monkeypatch.setenv("APP_ENV", "development")
    monkeypatch.setenv("SUPABASE_AUTH_ENABLED", "true")
    monkeypatch.setenv("SUPABASE_URL", "https://project.supabase.co")
    monkeypatch.setenv("SUPABASE_JWT_ISSUER", "https://project.supabase.co/auth/v1")
    monkeypatch.setenv("SUPABASE_AUTH_API_KEY", "")
    monkeypatch.setenv("SUPABASE_SERVICE_ROLE_KEY", "service-role-key")

    settings = Settings(_env_file=None)
    assert settings.resolved_supabase_auth_api_key == "service-role-key"


def test_supabase_auth_key_property_prefers_explicit_api_key(monkeypatch) -> None:
    monkeypatch.setenv("APP_ENV", "development")
    monkeypatch.setenv("SUPABASE_AUTH_API_KEY", "explicit-key")
    monkeypatch.setenv("SUPABASE_SERVICE_ROLE_KEY", "service-role-key")

    settings = Settings(_env_file=None)
    assert settings.resolved_supabase_auth_api_key == "explicit-key"


def test_rejects_upstash_when_cde_redis_uses_fallback(monkeypatch) -> None:
    monkeypatch.setenv("APP_ENV", "development")
    monkeypatch.setenv("REDIS_URL", "rediss://example.upstash.io:6379/0")
    monkeypatch.delenv("REDIS_URL_CDE", raising=False)
    monkeypatch.delenv("REDIS_URL_NON_CDE", raising=False)

    with pytest.raises(ValueError):
        Settings(_env_file=None)


def test_allows_upstash_for_non_cde_when_cde_redis_is_non_upstash(monkeypatch) -> None:
    monkeypatch.setenv("APP_ENV", "development")
    monkeypatch.setenv("REDIS_URL", "redis://redis:6379/0")
    monkeypatch.setenv("REDIS_URL_NON_CDE", "rediss://example.upstash.io:6379/0")
    monkeypatch.setenv("REDIS_URL_CDE", "redis://redis:6379/1")

    settings = Settings(_env_file=None)

    assert settings.resolved_redis_url_non_cde == "rediss://example.upstash.io:6379/0"
    assert settings.resolved_redis_url_cde == "redis://redis:6379/1"
    assert is_upstash_redis_url(settings.resolved_redis_url_non_cde) is True
    assert is_upstash_redis_url(settings.resolved_redis_url_cde) is False


def test_parses_master_keys_by_version_json(monkeypatch) -> None:
    monkeypatch.setenv("APP_ENV", "development")
    monkeypatch.setenv("MASTER_KEY", _VALID_MASTER_KEY)
    monkeypatch.setenv("MASTER_KEYS_BY_VERSION", '{"1":"%s","2":"%s"}' % (_VALID_MASTER_KEY, _VALID_MASTER_KEY))
    monkeypatch.setenv("KEK_VERSION", "2")

    settings = Settings(_env_file=None)
    assert settings.master_keys_by_version == {1: _VALID_MASTER_KEY, 2: _VALID_MASTER_KEY}


def test_master_keys_by_version_parser_branches() -> None:
    assert Settings.parse_master_keys_by_version(None) is None
    assert Settings.parse_master_keys_by_version("   ") is None
    assert Settings.parse_master_keys_by_version('{"1":"abc"}') == {1: "abc"}
    with pytest.raises(ValueError, match="keys must be integers"):
        Settings.parse_master_keys_by_version({"x": "abc"})
    with pytest.raises(ValueError, match="values must be non-empty"):
        Settings.parse_master_keys_by_version({"1": " "})


def test_rejects_invalid_master_keys_by_version_shape(monkeypatch) -> None:
    monkeypatch.setenv("APP_ENV", "development")
    monkeypatch.setenv("MASTER_KEYS_BY_VERSION", '["bad"]')
    with pytest.raises(ValueError):
        Settings(_env_file=None)


def test_rejects_master_keyring_missing_active_version_without_master_key(monkeypatch) -> None:
    monkeypatch.setenv("APP_ENV", "development")
    monkeypatch.setenv("MASTER_KEY", "")
    monkeypatch.setenv("KEK_VERSION", "2")
    monkeypatch.setenv("MASTER_KEYS_BY_VERSION", '{"1":"%s"}' % _VALID_MASTER_KEY)
    with pytest.raises(ValueError, match="MASTER_KEYS_BY_VERSION must include KEK_VERSION"):
        Settings(_env_file=None)


def test_rejects_non_positive_internal_identity_ttl(monkeypatch) -> None:
    monkeypatch.setenv("APP_ENV", "development")
    monkeypatch.setenv("INTERNAL_IDENTITY_TTL_SECONDS", "0")
    with pytest.raises(ValueError):
        Settings(_env_file=None)


def test_rejects_non_positive_kek_retirement_window(monkeypatch) -> None:
    monkeypatch.setenv("APP_ENV", "development")
    monkeypatch.setenv("KEK_RETIREMENT_WINDOW_DAYS", "0")
    with pytest.raises(ValueError):
        Settings(_env_file=None)


def test_rejects_non_positive_passkey_timeout(monkeypatch) -> None:
    monkeypatch.setenv("APP_ENV", "development")
    monkeypatch.setenv("PASSKEY_TIMEOUT_MS", "0")
    with pytest.raises(ValueError, match="PASSKEY_TIMEOUT_MS must be >= 1"):
        Settings(_env_file=None)


def test_requires_rp_or_public_base_when_passkeys_enabled(monkeypatch) -> None:
    monkeypatch.setenv("APP_ENV", "development")
    monkeypatch.setenv("PASSKEY_ENABLED", "true")
    monkeypatch.setenv("PASSKEY_RP_ID", "")
    monkeypatch.setenv("APP_PUBLIC_BASE_URL", "")
    with pytest.raises(ValueError, match="PASSKEY_RP_ID or APP_PUBLIC_BASE_URL"):
        Settings(_env_file=None)


def test_requires_origin_or_public_base_for_passkeys_in_production(monkeypatch) -> None:
    monkeypatch.setenv("APP_ENV", "production")
    monkeypatch.setenv("MASTER_KEY", _VALID_MASTER_KEY)
    monkeypatch.setenv("INTERNAL_API_KEY", "internal-key")
    monkeypatch.setenv("PAYMENT_PROVIDER_ALLOWED_HOSTS", "app.srail.or.kr")
    monkeypatch.setenv("PASSKEY_ENABLED", "true")
    monkeypatch.setenv("PASSKEY_RP_ID", "www.bominal.com")
    monkeypatch.setenv("APP_PUBLIC_BASE_URL", "")
    monkeypatch.setenv("PASSKEY_ORIGIN", "")
    with pytest.raises(ValueError, match="PASSKEY_ORIGIN or APP_PUBLIC_BASE_URL"):
        Settings(_env_file=None)
