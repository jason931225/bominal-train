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
    assert settings.restaurant_opentable_autocomplete_operation_name == "Autocomplete"
    assert settings.restaurant_opentable_autocomplete_operation_sha256 == (
        "fe1d118abd4c227750693027c2414d43014c2493f64f49bcef5a65274ce9c3c3"
    )
    assert settings.restaurant_opentable_search_operation_name == "RestaurantsAvailability"
    assert settings.restaurant_opentable_search_operation_sha256 == (
        "b2d05a06151b3cb21d9dfce4f021303eeba288fac347068b29c1cb66badc46af"
    )
    assert settings.restaurant_opentable_search_slot_path == "data.availability"
    assert settings.restaurant_opentable_create_operation_name == "BookDetailsStandardSlotLock"
    assert settings.restaurant_opentable_create_operation_sha256 == (
        "1100bf68905fd7cb1d4fd0f4504a4954aa28ec45fb22913fa977af8b06fd97fa"
    )
    assert settings.restaurant_opentable_create_path == "/dapi/booking/make-reservation"
    assert settings.restaurant_opentable_confirmation_operation_name == "BookingConfirmationPageInFlow"
    assert settings.restaurant_opentable_confirmation_operation_sha256 == (
        "6be25f0bbc8fe75483bdfe96ae78fb20075b978842e4b44964aed3591611aa99"
    )
    assert settings.restaurant_resy_base_url == "https://api.resy.com"
    assert settings.restaurant_resy_timeout_seconds == 20.0
    assert settings.restaurant_resy_auth_password_path == "/4/auth/password"
    assert settings.restaurant_resy_auth_api_key is None
    assert settings.restaurant_resy_x_origin == "https://resy.com"
    assert settings.restaurant_resy_profile_path == "/2/user"
    assert settings.restaurant_resy_search_path == "/4/find"
    assert settings.restaurant_resy_create_details_path == "/3/details"
    assert settings.restaurant_resy_create_book_path == "/3/book"
    assert settings.restaurant_resy_cancel_path == "/3/cancel"
    assert settings.restaurant_resy_source_id == "resy.com-venue-details"
    assert settings.restaurant_resy_refresh_path == "/3/auth/refresh"
    assert settings.restaurant_resy_logout_path == "/3/auth/logout"


def test_restaurant_policy_config_env_override(monkeypatch):
    monkeypatch.setenv("RESTAURANT_AUTH_REFRESH_RETRIES", "5")
    monkeypatch.setenv("RESTAURANT_PAYMENT_LEASE_TTL_SECONDS", "240")
    monkeypatch.setenv("RESTAURANT_BOOTSTRAP_TIMEOUT_SECONDS", "45")
    monkeypatch.setenv("RESTAURANT_OPENTABLE_BASE_URL", "https://ot.example.com")
    monkeypatch.setenv("RESTAURANT_OPENTABLE_TIMEOUT_SECONDS", "15")
    monkeypatch.setenv("RESTAURANT_OPENTABLE_AUTH_START_PATH", "/dapi/auth/start")
    monkeypatch.setenv("RESTAURANT_OPENTABLE_AUTH_COMPLETE_PATH", "/dapi/auth/verify")
    monkeypatch.setenv("RESTAURANT_OPENTABLE_AUTOCOMPLETE_OPERATION_NAME", "AutocompleteLive")
    monkeypatch.setenv("RESTAURANT_OPENTABLE_AUTOCOMPLETE_OPERATION_SHA256", "hash-autocomplete-override")
    monkeypatch.setenv("RESTAURANT_OPENTABLE_SEARCH_OPERATION_NAME", "SearchLiveAvailability")
    monkeypatch.setenv("RESTAURANT_OPENTABLE_SEARCH_OPERATION_SHA256", "hash-search-override")
    monkeypatch.setenv("RESTAURANT_OPENTABLE_SEARCH_SLOT_PATH", "data.availability.slots")
    monkeypatch.setenv("RESTAURANT_OPENTABLE_CREATE_OPERATION_NAME", "CreateReservationLive")
    monkeypatch.setenv("RESTAURANT_OPENTABLE_CREATE_OPERATION_SHA256", "hash-create-override")
    monkeypatch.setenv("RESTAURANT_OPENTABLE_CREATE_PATH", "/dapi/booking/create")
    monkeypatch.setenv("RESTAURANT_OPENTABLE_CONFIRMATION_OPERATION_NAME", "BookingConfirmationLive")
    monkeypatch.setenv("RESTAURANT_OPENTABLE_CONFIRMATION_OPERATION_SHA256", "hash-confirmation-override")
    monkeypatch.setenv("RESTAURANT_RESY_BASE_URL", "https://api.resy.example.com")
    monkeypatch.setenv("RESTAURANT_RESY_TIMEOUT_SECONDS", "15")
    monkeypatch.setenv("RESTAURANT_RESY_AUTH_PASSWORD_PATH", "/4/auth/password-live")
    monkeypatch.setenv("RESTAURANT_RESY_AUTH_API_KEY", "resy-api-key-override")
    monkeypatch.setenv("RESTAURANT_RESY_X_ORIGIN", "https://resy.example.com")
    monkeypatch.setenv("RESTAURANT_RESY_PROFILE_PATH", "/2/user-live")
    monkeypatch.setenv("RESTAURANT_RESY_SEARCH_PATH", "/4/find-live")
    monkeypatch.setenv("RESTAURANT_RESY_CREATE_DETAILS_PATH", "/3/details-live")
    monkeypatch.setenv("RESTAURANT_RESY_CREATE_BOOK_PATH", "/3/book-live")
    monkeypatch.setenv("RESTAURANT_RESY_CANCEL_PATH", "/3/cancel-live")
    monkeypatch.setenv("RESTAURANT_RESY_SOURCE_ID", "resy.com-live")
    monkeypatch.setenv("RESTAURANT_RESY_REFRESH_PATH", "/3/auth/refresh-live")
    monkeypatch.setenv("RESTAURANT_RESY_LOGOUT_PATH", "/3/auth/logout-live")

    settings = Settings()

    assert settings.restaurant_auth_refresh_retries == 5
    assert settings.restaurant_payment_lease_ttl_seconds == 240
    assert settings.restaurant_bootstrap_timeout_seconds == 45.0
    assert settings.restaurant_opentable_base_url == "https://ot.example.com"
    assert settings.restaurant_opentable_timeout_seconds == 15.0
    assert settings.restaurant_opentable_auth_start_path == "/dapi/auth/start"
    assert settings.restaurant_opentable_auth_complete_path == "/dapi/auth/verify"
    assert settings.restaurant_opentable_autocomplete_operation_name == "AutocompleteLive"
    assert settings.restaurant_opentable_autocomplete_operation_sha256 == "hash-autocomplete-override"
    assert settings.restaurant_opentable_search_operation_name == "SearchLiveAvailability"
    assert settings.restaurant_opentable_search_operation_sha256 == "hash-search-override"
    assert settings.restaurant_opentable_search_slot_path == "data.availability.slots"
    assert settings.restaurant_opentable_create_operation_name == "CreateReservationLive"
    assert settings.restaurant_opentable_create_operation_sha256 == "hash-create-override"
    assert settings.restaurant_opentable_create_path == "/dapi/booking/create"
    assert settings.restaurant_opentable_confirmation_operation_name == "BookingConfirmationLive"
    assert settings.restaurant_opentable_confirmation_operation_sha256 == "hash-confirmation-override"
    assert settings.restaurant_resy_base_url == "https://api.resy.example.com"
    assert settings.restaurant_resy_timeout_seconds == 15.0
    assert settings.restaurant_resy_auth_password_path == "/4/auth/password-live"
    assert settings.restaurant_resy_auth_api_key == "resy-api-key-override"
    assert settings.restaurant_resy_x_origin == "https://resy.example.com"
    assert settings.restaurant_resy_profile_path == "/2/user-live"
    assert settings.restaurant_resy_search_path == "/4/find-live"
    assert settings.restaurant_resy_create_details_path == "/3/details-live"
    assert settings.restaurant_resy_create_book_path == "/3/book-live"
    assert settings.restaurant_resy_cancel_path == "/3/cancel-live"
    assert settings.restaurant_resy_source_id == "resy.com-live"
    assert settings.restaurant_resy_refresh_path == "/3/auth/refresh-live"
    assert settings.restaurant_resy_logout_path == "/3/auth/logout-live"
