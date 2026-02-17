from __future__ import annotations

from app.core.config import Settings


def test_restaurant_policy_config_defaults():
    settings = Settings()

    assert settings.restaurant_auth_refresh_retries == 2
    assert settings.restaurant_payment_lease_ttl_seconds == 120
    assert settings.restaurant_bootstrap_timeout_seconds == 20.0
    assert settings.restaurant_opentable_base_url == "https://www.opentable.com"
    assert settings.restaurant_opentable_timeout_seconds == 20.0
    assert settings.restaurant_opentable_auth_start_path == "/dapi/authentication/sendotpfromsignin"
    assert settings.restaurant_opentable_auth_complete_path == "/dapi/authentication/signinwithotp"
    assert settings.restaurant_opentable_search_operation_name == "SearchRestaurantAvailability"
    assert settings.restaurant_opentable_search_operation_sha256 == ""
    assert settings.restaurant_opentable_search_slot_path == "data.search.availableSlots"
    assert settings.restaurant_opentable_create_operation_name == "CreateReservation"
    assert settings.restaurant_opentable_create_operation_sha256 == ""


def test_restaurant_policy_config_env_override(monkeypatch):
    monkeypatch.setenv("RESTAURANT_AUTH_REFRESH_RETRIES", "5")
    monkeypatch.setenv("RESTAURANT_PAYMENT_LEASE_TTL_SECONDS", "240")
    monkeypatch.setenv("RESTAURANT_BOOTSTRAP_TIMEOUT_SECONDS", "45")
    monkeypatch.setenv("RESTAURANT_OPENTABLE_BASE_URL", "https://ot.example.com")
    monkeypatch.setenv("RESTAURANT_OPENTABLE_TIMEOUT_SECONDS", "15")
    monkeypatch.setenv("RESTAURANT_OPENTABLE_AUTH_START_PATH", "/dapi/auth/start")
    monkeypatch.setenv("RESTAURANT_OPENTABLE_AUTH_COMPLETE_PATH", "/dapi/auth/verify")
    monkeypatch.setenv("RESTAURANT_OPENTABLE_SEARCH_OPERATION_NAME", "SearchLiveAvailability")
    monkeypatch.setenv("RESTAURANT_OPENTABLE_SEARCH_OPERATION_SHA256", "hash-search-override")
    monkeypatch.setenv("RESTAURANT_OPENTABLE_SEARCH_SLOT_PATH", "data.availability.slots")
    monkeypatch.setenv("RESTAURANT_OPENTABLE_CREATE_OPERATION_NAME", "CreateReservationLive")
    monkeypatch.setenv("RESTAURANT_OPENTABLE_CREATE_OPERATION_SHA256", "hash-create-override")

    settings = Settings()

    assert settings.restaurant_auth_refresh_retries == 5
    assert settings.restaurant_payment_lease_ttl_seconds == 240
    assert settings.restaurant_bootstrap_timeout_seconds == 45.0
    assert settings.restaurant_opentable_base_url == "https://ot.example.com"
    assert settings.restaurant_opentable_timeout_seconds == 15.0
    assert settings.restaurant_opentable_auth_start_path == "/dapi/auth/start"
    assert settings.restaurant_opentable_auth_complete_path == "/dapi/auth/verify"
    assert settings.restaurant_opentable_search_operation_name == "SearchLiveAvailability"
    assert settings.restaurant_opentable_search_operation_sha256 == "hash-search-override"
    assert settings.restaurant_opentable_search_slot_path == "data.availability.slots"
    assert settings.restaurant_opentable_create_operation_name == "CreateReservationLive"
    assert settings.restaurant_opentable_create_operation_sha256 == "hash-create-override"
