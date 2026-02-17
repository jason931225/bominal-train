from __future__ import annotations

import json
from datetime import datetime

import pytest

from app.modules.restaurant.providers.base import RestaurantSearchSlot
from app.modules.restaurant.providers.resy_adapter import ResyProviderClient
from app.modules.train.providers.transport import TransportResponse


class _QueueTransport:
    def __init__(self, responses: list[TransportResponse]) -> None:
        self._responses = list(responses)
        self.requests: list[dict] = []

    async def request(
        self,
        *,
        method: str,
        url: str,
        headers: dict[str, str] | None = None,
        json_body: dict | None = None,
        data: dict | None = None,
        params: dict | None = None,
        timeout: float = 20.0,
    ) -> TransportResponse:
        self.requests.append(
            {
                "method": method,
                "url": url,
                "headers": headers or {},
                "json_body": json_body,
                "data": data,
                "params": params,
                "timeout": timeout,
            }
        )
        if not self._responses:
            raise AssertionError("No queued response available")
        return self._responses.pop(0)


def _response(payload: dict, *, status_code: int = 200) -> TransportResponse:
    return TransportResponse(status_code=status_code, text=json.dumps(payload), headers={})


@pytest.mark.asyncio
async def test_resy_auth_start_requires_password():
    client = ResyProviderClient(transport=_QueueTransport([]), auth_api_key="key-1")

    start_result = await client.authenticate_start(account_identifier="user@example.com", password=None)

    assert start_result.ok is False
    assert start_result.error_code == "auth_start_password_required"


@pytest.mark.asyncio
async def test_resy_auth_start_posts_password_contract_and_completes_login():
    transport = _QueueTransport([_response({"user": {"id": "user-1"}})])
    client = ResyProviderClient(
        transport=transport,
        base_url="https://api.resy.com",
        auth_password_path="/4/auth/password",
        auth_api_key="key-1",
        x_origin="https://resy.com",
    )

    start_result = await client.authenticate_start(account_identifier="user@example.com", password="secret")

    assert start_result.ok is True
    assert start_result.data["requires_otp"] is False
    assert start_result.data["password_flow_complete"] is True
    assert start_result.data["provider_account_ref"] == "user-1"
    challenge_payload = json.loads(start_result.data["challenge_token"])
    assert challenge_payload == {
        "password_flow_complete": True,
        "provider_account_ref": "user-1",
    }
    assert len(transport.requests) == 1
    request = transport.requests[0]
    assert request["method"] == "POST"
    assert request["url"].endswith("/4/auth/password")
    assert request["headers"]["Authorization"] == 'ResyAPI api_key="key-1"'
    assert request["headers"]["X-Origin"] == "https://resy.com"
    assert request["headers"]["Content-Type"] == "application/x-www-form-urlencoded"
    assert request["data"] == {
        "email": "user@example.com",
        "password": "secret",
    }


@pytest.mark.asyncio
async def test_resy_auth_start_handles_http_error_codes():
    transport = _QueueTransport([_response({"message": "Unauthorized"}, status_code=401)])
    client = ResyProviderClient(transport=transport, auth_api_key="key-1")

    start_result = await client.authenticate_start(account_identifier="user@example.com", password="secret")

    assert start_result.ok is False
    assert start_result.error_code == "auth_start_auth_required"
    assert start_result.retryable is False


@pytest.mark.asyncio
async def test_resy_auth_start_rejects_body_level_failure_on_http_200():
    transport = _QueueTransport([_response({"success": False, "code": "INVALID_CREDENTIALS"})])
    client = ResyProviderClient(transport=transport, auth_api_key="key-1")

    start_result = await client.authenticate_start(account_identifier="user@example.com", password="secret")

    assert start_result.ok is False
    assert start_result.error_code == "auth_start_failed"
    assert start_result.data["provider_error_code"] == "INVALID_CREDENTIALS"


@pytest.mark.asyncio
async def test_resy_auth_complete_uses_start_challenge_payload_without_network_call():
    transport = _QueueTransport([])
    client = ResyProviderClient(transport=transport, auth_api_key="key-1")

    complete_result = await client.authenticate_complete(
        account_identifier="user@example.com",
        challenge_token=json.dumps({"password_flow_complete": True, "provider_account_ref": "user-1"}),
        otp_code=None,
    )

    assert complete_result.ok is True
    assert complete_result.data["authenticated"] is True
    assert complete_result.data["provider_account_ref"] == "user-1"
    assert transport.requests == []


@pytest.mark.asyncio
async def test_resy_auth_complete_requires_password_flow_challenge_token():
    client = ResyProviderClient(transport=_QueueTransport([]), auth_api_key="key-1")

    complete_result = await client.authenticate_complete(
        account_identifier="user@example.com",
        challenge_token=None,
        otp_code=None,
    )

    assert complete_result.ok is False
    assert complete_result.error_code == "auth_complete_challenge_missing"


@pytest.mark.asyncio
async def test_resy_get_user_profile_uses_user_endpoint_and_safe_mapping():
    transport = _QueueTransport(
        [
            _response(
                {
                    "id": 12345,
                    "email": "user@example.com",
                    "first_name": "Jason",
                    "last_name": "Lee",
                    "reservations": [{"reservation_id": "r1"}, {"reservation_id": "r2"}],
                    "payment_methods": [{"id": "pm1"}],
                }
            )
        ]
    )
    client = ResyProviderClient(transport=transport, auth_api_key="key-1", profile_path="/2/user")

    result = await client.get_user_profile(account_ref="auth-token-1")

    assert result.ok is True
    assert result.data["provider_account_ref"] == "12345"
    assert result.data["email"] == "user@example.com"
    assert result.data["first_name"] == "Jason"
    assert result.data["last_name"] == "Lee"
    assert result.data["reservation_count"] == 2
    assert result.data["payment_method_count"] == 1
    request = transport.requests[0]
    assert request["method"] == "GET"
    assert request["url"].endswith("/2/user")
    assert request["headers"]["Authorization"] == 'ResyAPI api_key="key-1"'
    assert request["headers"]["X-Resy-Auth-Token"] == "auth-token-1"


@pytest.mark.asyncio
async def test_resy_search_availability_uses_find_endpoint_and_parses_slots():
    transport = _QueueTransport(
        [
            _response(
                {
                    "results": {
                        "venues": [
                            {
                                "venue": {
                                    "id": {"resy": 1505},
                                    "name": "Don Angie",
                                    "location": {"name": "West Village"},
                                },
                                "slots": [
                                    {
                                        "date": {"start": "2026-03-05 19:00:00"},
                                        "config": {"type": "Dining Room", "token": "cfg-token-1"},
                                    },
                                    {
                                        "date": {"start": "2026-03-05 20:00:00"},
                                        "config": {"type": "Dining Room", "token": "cfg-token-2"},
                                    },
                                ],
                            }
                        ]
                    }
                }
            )
        ]
    )
    client = ResyProviderClient(transport=transport, auth_api_key="key-1", search_path="/4/find")

    result = await client.search_availability(
        account_ref="auth-token-1",
        restaurant_id="1505",
        party_size=2,
        date_time_local=datetime(2026, 3, 5, 19, 0),
        metadata={"lat": 40.73, "long": -73.99},
    )

    assert result.ok is True
    assert result.data["slot_count"] == 2
    slots = result.data["slots"]
    assert len(slots) == 2
    assert isinstance(slots[0], RestaurantSearchSlot)
    assert slots[0].provider_slot_id == "cfg-token-1"
    assert slots[0].restaurant_id == "1505"
    assert slots[0].availability_token == "cfg-token-1"
    assert slots[0].date_time_local == datetime(2026, 3, 5, 19, 0)
    assert slots[0].metadata_safe["slot_type"] == "Dining Room"
    request = transport.requests[0]
    assert request["method"] == "GET"
    assert request["url"].endswith("/4/find")
    assert request["params"] == {
        "venue_id": "1505",
        "day": "2026-03-05",
        "party_size": 2,
        "lat": 40.73,
        "long": -73.99,
    }


@pytest.mark.asyncio
async def test_resy_create_reservation_uses_details_then_book_contract():
    transport = _QueueTransport(
        [
            _response({"book_token": {"value": "book-token-1"}, "payment": {"required": True}}),
            _response(
                {
                    "reservation_id": "resy_abc123",
                    "resy_token": "token-1",
                    "display_date": "March 5, 2026",
                    "display_time": "7:00 PM",
                }
            ),
        ]
    )
    client = ResyProviderClient(
        transport=transport,
        auth_api_key="key-1",
        create_details_path="/3/details",
        create_book_path="/3/book",
        source_id="resy.com-venue-details",
    )
    slot = RestaurantSearchSlot(
        provider_slot_id="cfg-token-1",
        provider="RESY",
        restaurant_id="1505",
        party_size=2,
        date_time_local=datetime(2026, 3, 5, 19, 0),
        availability_token="cfg-token-1",
    )

    result = await client.create_reservation(
        account_ref="auth-token-1",
        slot=slot,
        metadata={
            "idempotency_key": "idem-1",
            "payment_method_id": "pm-123",
            "venue_marketing_opt_in": False,
        },
    )

    assert result.ok is True
    assert result.data["reservation_id"] == "resy_abc123"
    assert result.data["resy_token_present"] is True
    assert result.data["payment_required"] is True
    assert result.data["source_id"] == "resy.com-venue-details"
    assert result.data["display_date"] == "March 5, 2026"
    assert result.data["display_time"] == "7:00 PM"

    assert len(transport.requests) == 2
    details_request = transport.requests[0]
    assert details_request["method"] == "POST"
    assert details_request["url"].endswith("/3/details")
    assert details_request["data"] == {
        "commit": 1,
        "config_id": "cfg-token-1",
        "day": "2026-03-05",
        "party_size": 2,
    }

    book_request = transport.requests[1]
    assert book_request["method"] == "POST"
    assert book_request["url"].endswith("/3/book")
    assert book_request["headers"]["Idempotency-Key"] == "idem-1"
    assert book_request["data"]["book_token"] == "book-token-1"
    assert book_request["data"]["source_id"] == "resy.com-venue-details"
    assert json.loads(book_request["data"]["struct_payment_method"]) == {"id": "pm-123"}
    assert book_request["data"]["venue_marketing_opt_in"] == 0


@pytest.mark.asyncio
async def test_resy_create_reservation_fails_when_book_token_missing():
    transport = _QueueTransport([_response({"user": {"id": 123}})])
    client = ResyProviderClient(transport=transport, auth_api_key="key-1")
    slot = RestaurantSearchSlot(
        provider_slot_id="cfg-token-1",
        provider="RESY",
        restaurant_id="1505",
        party_size=2,
        date_time_local=datetime(2026, 3, 5, 19, 0),
        availability_token="cfg-token-1",
    )

    result = await client.create_reservation(account_ref="auth-token-1", slot=slot, metadata={})

    assert result.ok is False
    assert result.error_code == "reservation_create_book_token_missing"


@pytest.mark.asyncio
async def test_resy_cancel_reservation_retries_with_resy_token_on_failure():
    transport = _QueueTransport(
        [
            _response({"message": "Token required"}, status_code=419),
            _response({"status": "cancelled"}),
        ]
    )
    client = ResyProviderClient(transport=transport, auth_api_key="key-1", cancel_path="/3/cancel")

    result = await client.cancel_reservation(
        account_ref="auth-token-1",
        restaurant_id="1505",
        confirmation_number="resy_abc123",
        security_token="token-1",
    )

    assert result.ok is True
    assert result.data["status"] == "cancelled"
    assert result.data["used_resy_token_fallback"] is True

    assert len(transport.requests) == 2
    first_request = transport.requests[0]
    second_request = transport.requests[1]
    assert first_request["data"] == {"reservation_id": "resy_abc123"}
    assert second_request["data"] == {"reservation_id": "resy_abc123", "resy_token": "token-1"}


@pytest.mark.asyncio
async def test_resy_cancel_reservation_returns_http_error_without_fallback():
    transport = _QueueTransport([_response({"message": "Unauthorized"}, status_code=401)])
    client = ResyProviderClient(transport=transport, auth_api_key="key-1")

    result = await client.cancel_reservation(
        account_ref="auth-token-1",
        restaurant_id="1505",
        confirmation_number="resy_abc123",
        security_token=None,
    )

    assert result.ok is False
    assert result.error_code == "reservation_cancel_auth_required"
