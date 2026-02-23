import base64

import pytest

from app.core.config import Settings, is_upstash_redis_url


_VALID_MASTER_KEY = base64.b64encode(b"x" * 32).decode("utf-8")


def test_rejects_invalid_cvv_ttl_bounds(monkeypatch) -> None:
    monkeypatch.setenv("APP_ENV", "development")
    monkeypatch.setenv("PAYMENT_CVV_TTL_SECONDS", "1200")
    monkeypatch.setenv("PAYMENT_CVV_TTL_MIN_SECONDS", "60")
    monkeypatch.setenv("PAYMENT_CVV_TTL_MAX_SECONDS", "900")
    with pytest.raises(ValueError):
        Settings(_env_file=None)


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


def test_parses_optional_egress_proxy_urls(monkeypatch) -> None:
    monkeypatch.setenv("APP_ENV", "development")
    monkeypatch.setenv("TRAIN_PROVIDER_EGRESS_PROXY_URL", " http://egress-train:8080 ")
    monkeypatch.setenv("RESTAURANT_PROVIDER_EGRESS_PROXY_URL", "")

    settings = Settings(_env_file=None)

    assert settings.train_provider_egress_proxy_url == "http://egress-train:8080"
    assert settings.restaurant_provider_egress_proxy_url is None


def test_dual_mode_allows_legacy_only_configuration(monkeypatch) -> None:
    monkeypatch.setenv("APP_ENV", "development")
    monkeypatch.setenv("AUTH_MODE", "dual")
    monkeypatch.delenv("SUPABASE_URL", raising=False)
    monkeypatch.delenv("SUPABASE_JWT_ISSUER", raising=False)

    settings = Settings(_env_file=None)
    assert settings.auth_mode == "dual"
    assert settings.supabase_url is None
    assert settings.supabase_jwt_issuer is None


def test_dual_mode_requires_url_and_issuer_together(monkeypatch) -> None:
    monkeypatch.setenv("APP_ENV", "development")
    monkeypatch.setenv("AUTH_MODE", "dual")
    monkeypatch.setenv("SUPABASE_URL", "https://project.supabase.co")
    monkeypatch.delenv("SUPABASE_JWT_ISSUER", raising=False)

    with pytest.raises(ValueError):
        Settings(_env_file=None)


def test_supabase_mode_requires_jwt_issuer(monkeypatch) -> None:
    monkeypatch.setenv("APP_ENV", "development")
    monkeypatch.setenv("AUTH_MODE", "supabase")
    monkeypatch.setenv("SUPABASE_URL", "https://project.supabase.co")
    monkeypatch.delenv("SUPABASE_JWT_ISSUER", raising=False)

    with pytest.raises(ValueError):
        Settings(_env_file=None)


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
