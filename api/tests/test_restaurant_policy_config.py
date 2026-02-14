from __future__ import annotations

from app.core.config import Settings


def test_restaurant_policy_config_defaults():
    settings = Settings()

    assert settings.restaurant_auth_refresh_retries == 2
    assert settings.restaurant_payment_lease_ttl_seconds == 120
    assert settings.restaurant_bootstrap_timeout_seconds == 20.0


def test_restaurant_policy_config_env_override(monkeypatch):
    monkeypatch.setenv("RESTAURANT_AUTH_REFRESH_RETRIES", "5")
    monkeypatch.setenv("RESTAURANT_PAYMENT_LEASE_TTL_SECONDS", "240")
    monkeypatch.setenv("RESTAURANT_BOOTSTRAP_TIMEOUT_SECONDS", "45")

    settings = Settings()

    assert settings.restaurant_auth_refresh_retries == 5
    assert settings.restaurant_payment_lease_ttl_seconds == 240
    assert settings.restaurant_bootstrap_timeout_seconds == 45.0
