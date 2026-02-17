from __future__ import annotations

from datetime import datetime

import pytest

from app.modules.restaurant.providers import (
    RESTAURANT_CANONICAL_OPERATIONS,
    get_restaurant_provider_client,
)
from app.modules.restaurant.providers.base import (
    RestaurantProviderClient,
    RestaurantProviderOutcome,
    RestaurantSearchSlot,
)


def test_restaurant_canonical_operations_are_stable():
    assert RESTAURANT_CANONICAL_OPERATIONS == (
        "auth.start",
        "auth.complete",
        "auth.refresh",
        "profile.get",
        "search.availability",
        "reservation.create",
        "reservation.cancel",
    )


@pytest.mark.parametrize("provider", ["RESY", "resy", "OpenTable", "OPENTABLE"])
def test_get_restaurant_provider_client_supports_resy_and_opentable(provider: str):
    client = get_restaurant_provider_client(provider)
    assert isinstance(client.provider_name, str)
    assert client.provider_name in {"RESY", "OPENTABLE"}


def test_get_restaurant_provider_client_rejects_unknown_provider():
    with pytest.raises(ValueError):
        get_restaurant_provider_client("UNKNOWN")


@pytest.mark.asyncio
async def test_restaurant_adapter_scaffold_returns_not_implemented_for_execution_paths():
    client = get_restaurant_provider_client("RESY")
    assert isinstance(client, RestaurantProviderClient)

    auth_start = await client.authenticate_start(
        account_identifier="user@example.com",
        password="secret",
        delivery_channel="email",
    )
    assert isinstance(auth_start, RestaurantProviderOutcome)
    assert auth_start.ok is False
    assert auth_start.error_code == "not_implemented"

    auth_complete = await client.authenticate_complete(
        account_identifier="user@example.com",
        challenge_token="challenge-token",
        otp_code="123456",
    )
    assert auth_complete.ok is False
    assert auth_complete.error_code == "not_implemented"

    refresh = await client.refresh_auth(account_ref="user@example.com")
    assert refresh.ok is False
    assert refresh.error_code == "not_implemented"

    profile = await client.get_user_profile(account_ref="user@example.com")
    assert profile.ok is False
    assert profile.error_code == "not_implemented"

    search = await client.search_availability(
        account_ref="user@example.com",
        restaurant_id="349132",
        party_size=2,
        date_time_local=datetime(2026, 2, 27, 19, 30),
        metadata={"market": "us-nyc"},
    )
    assert search.ok is False
    assert search.error_code == "not_implemented"

    create = await client.create_reservation(
        account_ref="user@example.com",
        slot=RestaurantSearchSlot(
            provider_slot_id="slot-1",
            provider="RESY",
            restaurant_id="349132",
            party_size=2,
            date_time_local=datetime(2026, 2, 27, 19, 30),
        ),
        metadata={"source": "test"},
    )
    assert create.ok is False
    assert create.error_code == "not_implemented"

    cancel = await client.cancel_reservation(
        account_ref="user@example.com",
        restaurant_id="349132",
        confirmation_number="2110076913",
        security_token="token",
        metadata={"source": "test"},
    )
    assert cancel.ok is False
    assert cancel.error_code == "not_implemented"
