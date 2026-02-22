import base64

import pytest

from app.core.config import Settings


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
