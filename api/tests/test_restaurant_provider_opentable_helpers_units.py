from __future__ import annotations

import json
from dataclasses import dataclass
from datetime import datetime
from typing import Any

import pytest

import app.modules.restaurant.providers.opentable_adapter as opentable_module
from app.modules.restaurant.providers.base import RestaurantSearchSlot
from app.modules.restaurant.providers.opentable_adapter import (
    OpenTableProviderClient,
    _as_bool,
    _as_int_or_original,
    _coerce_int,
    _decode_challenge_token,
    _deep_get,
    _find_first,
    _first_dict_value,
    _normalize_channel_type,
    _normalize_list_of_strings,
    _normalize_status_error,
    _parse_datetime,
    _resolve_reservation_datetime,
    _safe_json_loads,
)


@dataclass(slots=True)
class _Response:
    status_code: int
    text: str


class _QueueTransport:
    def __init__(self, responses: list[_Response]) -> None:
        self._responses = list(responses)
        self.calls: list[dict[str, Any]] = []

    async def request(self, **kwargs) -> _Response:  # noqa: ANN003
        self.calls.append(kwargs)
        if not self._responses:
            raise AssertionError("No queued response")
        return self._responses.pop(0)


def _json_response(status_code: int, payload: dict[str, Any]) -> _Response:
    return _Response(status_code=status_code, text=json.dumps(payload))


def _slot() -> RestaurantSearchSlot:
    return RestaurantSearchSlot(
        provider_slot_id="slot-1",
        provider="OpenTable",
        restaurant_id="42",
        party_size=2,
        date_time_local=datetime(2026, 2, 23, 19, 30),
        availability_token="availability-1",
        metadata_safe={"reservation_type": "STANDARD", "seating_option": "DEFAULT"},
    )


def test_opentable_helper_functions_cover_edge_branches():
    assert _deep_get({"a": {"b": 1}}, "a.c") is None
    assert _find_first({"a": {"b": [{"target": "x"}]}}, ("target",)) == "x"
    assert _safe_json_loads("{oops") == {}
    assert _safe_json_loads("[]") == {}

    fallback = datetime(2026, 2, 23, 12, 0)
    assert _parse_datetime(None, fallback) == fallback
    assert _parse_datetime("not-a-date", fallback) == fallback
    assert _parse_datetime("2026-02-23T10:30:00Z", fallback).hour == 10

    assert _coerce_int(True, default=7) == 7
    assert _coerce_int("abc", default=7) == 7
    assert _coerce_int("3", default=0) == 3

    assert _resolve_reservation_datetime(raw_slot={"dateTime": "2026-02-23T12:40:00"}, day_date=None, fallback=fallback).hour == 12
    assert _resolve_reservation_datetime(raw_slot={"timeOffsetMinutes": 10}, day_date="bad-date", fallback=fallback) == fallback

    assert _normalize_list_of_strings("not-list", fallback=["A"]) == ["A"]
    assert _normalize_list_of_strings(["", "x", " y "], fallback=["A"]) == ["x", " y "]

    assert _as_int_or_original("100") == 100
    assert _as_int_or_original("R-100") == "R-100"

    assert _as_bool("YES") is True
    assert _as_bool(1) is True
    assert _as_bool("off") is False
    assert _as_bool("unknown", default=True) is True

    assert _first_dict_value({"x": 1}, ("a", "x")) == 1
    assert _first_dict_value({"x": 1}, ("a", "b")) is None

    assert _normalize_status_error("op", 401) == ("op_auth_required", False)
    assert _normalize_status_error("op", 429) == ("op_rate_limited", True)
    assert _normalize_status_error("op", 503) == ("op_provider_unavailable", True)
    assert _normalize_status_error("op", 400) == ("op_failed", False)

    assert _normalize_channel_type("sms") == "SMS"
    assert _normalize_channel_type("email") == "EMAIL"

    assert _decode_challenge_token(None) == {}
    assert _decode_challenge_token("{bad") == {}
    assert _decode_challenge_token('["list"]') == {}
    assert _decode_challenge_token('{"challenge_ref":"abc"}')["challenge_ref"] == "abc"

    headers = OpenTableProviderClient(transport=_QueueTransport([]))._headers(csrf_token="csrf")  # noqa: SLF001
    assert headers["x-csrf-token"] == "csrf"


@pytest.mark.asyncio
async def test_opentable_auth_refresh_profile_search_and_cancel_error_paths():
    client = OpenTableProviderClient(transport=_QueueTransport([_json_response(401, {})]))
    auth_start_fail = await client.authenticate_start(account_identifier="user@example.com")
    assert auth_start_fail.ok is False
    assert auth_start_fail.error_code == "auth_start_auth_required"

    client = OpenTableProviderClient(transport=_QueueTransport([_json_response(503, {})]))
    auth_complete_fail = await client.authenticate_complete(
        account_identifier="user@example.com",
        challenge_token='{"challenge_ref":"abc"}',
        otp_code="123456",
    )
    assert auth_complete_fail.ok is False
    assert auth_complete_fail.error_code == "auth_complete_provider_unavailable"

    client = OpenTableProviderClient(
        transport=_QueueTransport(
            [
                _json_response(200, {}),
                _json_response(200, {}),
                _json_response(503, {}),
            ]
        )
    )
    refresh_session_fail = await client.refresh_auth(account_ref="acct")
    assert refresh_session_fail.ok is False
    assert refresh_session_fail.error_code == "auth_refresh_provider_unavailable"

    client = OpenTableProviderClient(
        transport=_QueueTransport(
            [
                _json_response(200, {}),
                _json_response(503, {}),
                _json_response(200, {}),
            ]
        )
    )
    refresh_human_fail = await client.refresh_auth(account_ref="acct")
    assert refresh_human_fail.ok is False
    assert refresh_human_fail.error_code == "auth_refresh_provider_unavailable"

    client = OpenTableProviderClient(transport=_QueueTransport([]))

    async def _profile_status_fail(**_kwargs):  # noqa: ANN003
        return _json_response(503, {})

    async def _profile_payload_errors(**_kwargs):  # noqa: ANN003
        return _json_response(200, {"errors": [{"message": "bad"}]})

    async def _profile_missing(**_kwargs):  # noqa: ANN003
        return _json_response(200, {"data": {"other": {}}})

    client._gql_request = _profile_status_fail  # type: ignore[method-assign]  # noqa: SLF001
    profile_status_fail = await client.get_user_profile(account_ref="acct")
    assert profile_status_fail.ok is False
    assert profile_status_fail.error_code == "profile_get_provider_unavailable"

    client._gql_request = _profile_payload_errors  # type: ignore[method-assign]  # noqa: SLF001
    profile_graphql_fail = await client.get_user_profile(account_ref="acct")
    assert profile_graphql_fail.ok is False
    assert profile_graphql_fail.error_code == "profile_get_failed"

    client._gql_request = _profile_missing  # type: ignore[method-assign]  # noqa: SLF001
    profile_missing = await client.get_user_profile(account_ref="acct")
    assert profile_missing.ok is False
    assert profile_missing.error_code == "profile_missing"

    search_client = OpenTableProviderClient(
        transport=_QueueTransport([]),
        search_operation_sha256="search-hash",
    )

    async def _search_status_fail(**_kwargs):  # noqa: ANN003
        return _json_response(503, {})

    async def _search_graphql_fail(**_kwargs):  # noqa: ANN003
        return _json_response(200, {"errors": [{"message": "bad"}]})

    async def _search_fallback_slots(**_kwargs):  # noqa: ANN003
        return _json_response(
            200,
            {
                "slots": [
                    {"no_slot_id": True},
                    {
                        "slotHash": "fallback-slot",
                        "diningAreasBySeating": [{"diningAreaId": 7}],
                        "attributes": ["patio"],
                    },
                ]
            },
        )

    search_client._gql_request = _search_status_fail  # type: ignore[method-assign]  # noqa: SLF001
    search_status_fail = await search_client.search_availability(
        account_ref="acct",
        restaurant_id="42",
        party_size=2,
        date_time_local=datetime(2026, 2, 23, 19, 0),
        metadata={},
    )
    assert search_status_fail.ok is False
    assert search_status_fail.error_code == "search_provider_unavailable"

    search_client._gql_request = _search_graphql_fail  # type: ignore[method-assign]  # noqa: SLF001
    search_graphql_fail = await search_client.search_availability(
        account_ref="acct",
        restaurant_id="42",
        party_size=2,
        date_time_local=datetime(2026, 2, 23, 19, 0),
        metadata={},
    )
    assert search_graphql_fail.ok is False
    assert search_graphql_fail.error_code == "search_graphql_error"

    search_client._gql_request = _search_fallback_slots  # type: ignore[method-assign]  # noqa: SLF001
    search_fallback = await search_client.search_availability(
        account_ref="acct",
        restaurant_id="42",
        party_size=2,
        date_time_local=datetime(2026, 2, 23, 19, 0),
        metadata={},
    )
    assert search_fallback.ok is True
    assert search_fallback.data["slot_count"] == 1

    async def _search_slot_shape_with_skips(**_kwargs):  # noqa: ANN003
        return _json_response(
            200,
            {
                "data": {
                    "availability": [
                        "invalid-entry",
                        {
                            "restaurantId": "42",
                            "availabilityDays": [
                                "invalid-day",
                                {"date": "2026-02-23", "slots": "invalid-slots"},
                                {
                                    "date": "2026-02-23",
                                    "slots": [
                                        "invalid-slot",
                                        {
                                            "slotHash": "slot-a",
                                            "diningAreasBySeating": [{}],
                                            "attributes": ["bar"],
                                        },
                                    ],
                                },
                            ],
                        },
                    ]
                }
            },
        )

    search_client._gql_request = _search_slot_shape_with_skips  # type: ignore[method-assign]  # noqa: SLF001
    search_slot_shape = await search_client.search_availability(
        account_ref="acct",
        restaurant_id="42",
        party_size=2,
        date_time_local=datetime(2026, 2, 23, 19, 0),
        metadata={},
    )
    assert search_slot_shape.ok is True
    assert search_slot_shape.data["slot_count"] == 1

    cancel_status_fail_client = OpenTableProviderClient(transport=_QueueTransport([]))
    cancel_status_fail_client._gql_request = _search_status_fail  # type: ignore[method-assign]  # noqa: SLF001
    cancel_status_fail = await cancel_status_fail_client.cancel_reservation(
        account_ref="acct",
        restaurant_id="42",
        confirmation_number="100",
        security_token="sec",
    )
    assert cancel_status_fail.ok is False
    assert cancel_status_fail.error_code == "reservation_cancel_provider_unavailable"

    async def _cancel_graphql_fail(**_kwargs):  # noqa: ANN003
        return _json_response(200, {"errors": [{"message": "bad"}]})

    cancel_status_fail_client._gql_request = _cancel_graphql_fail  # type: ignore[method-assign]  # noqa: SLF001
    cancel_graphql_fail = await cancel_status_fail_client.cancel_reservation(
        account_ref="acct",
        restaurant_id="42",
        confirmation_number="100",
        security_token="sec",
    )
    assert cancel_graphql_fail.ok is False
    assert cancel_graphql_fail.error_code == "reservation_cancel_graphql_error"

    async def _cancel_status_payload_fail(**_kwargs):  # noqa: ANN003
        return _json_response(200, {"data": {"cancelReservation": {"statusCode": 409}}})

    cancel_status_fail_client._gql_request = _cancel_status_payload_fail  # type: ignore[method-assign]  # noqa: SLF001
    cancel_status_payload_fail = await cancel_status_fail_client.cancel_reservation(
        account_ref="acct",
        restaurant_id="42",
        confirmation_number="100",
        security_token="sec",
    )
    assert cancel_status_payload_fail.ok is False
    assert cancel_status_payload_fail.error_code == "reservation_cancel_failed"


@pytest.mark.asyncio
async def test_opentable_search_dining_area_loop_break_branch(monkeypatch: pytest.MonkeyPatch):
    client = OpenTableProviderClient(
        transport=_QueueTransport([]),
        search_operation_sha256="search-hash",
    )

    original_find_first = opentable_module._find_first

    def _patched_find_first(data: Any, keys: tuple[str, ...]) -> Any:
        if keys == ("diningAreaId",) and isinstance(data, dict) and "diningAreasBySeating" in data:
            return None
        return original_find_first(data, keys)

    monkeypatch.setattr(opentable_module, "_find_first", _patched_find_first)

    async def _search_with_nested_dining_area(**_kwargs):  # noqa: ANN003
        return _json_response(
            200,
            {
                "data": {
                    "availability": [
                        {
                            "restaurantId": "42",
                            "availabilityDays": [
                                {
                                    "date": "2026-02-23",
                                    "slots": [
                                        {
                                            "slotHash": "slot-b",
                                            "diningAreasBySeating": [{"diningAreaId": 17}],
                                        }
                                    ],
                                }
                            ],
                        }
                    ]
                }
            },
        )

    client._gql_request = _search_with_nested_dining_area  # type: ignore[method-assign]  # noqa: SLF001
    result = await client.search_availability(
        account_ref="acct",
        restaurant_id="42",
        party_size=2,
        date_time_local=datetime(2026, 2, 23, 19, 0),
        metadata={},
    )
    assert result.ok is True
    assert result.data["slot_count"] == 1


@pytest.mark.asyncio
async def test_opentable_create_reservation_error_paths_and_confirmation_enrichment_branches():
    slot = _slot()
    base_metadata = {
        "email": "user@example.com",
        "first_name": "User",
        "last_name": "Test",
        "phone_number": "01012345678",
    }

    client = OpenTableProviderClient(transport=_QueueTransport([]), create_operation_sha256="create-hash")
    missing_token_slot = RestaurantSearchSlot(
        provider_slot_id="slot-no-token",
        provider="OpenTable",
        restaurant_id="42",
        party_size=2,
        date_time_local=datetime(2026, 2, 23, 19, 30),
        availability_token=None,
        metadata_safe={},
    )
    missing_token = await client.create_reservation(account_ref="acct", slot=missing_token_slot, metadata=base_metadata)
    assert missing_token.ok is False
    assert missing_token.error_code == "reservation_create_slot_token_missing"

    async def _lock_status_fail(**_kwargs):  # noqa: ANN003
        return _json_response(503, {})

    client._gql_request = _lock_status_fail  # type: ignore[method-assign]  # noqa: SLF001
    lock_status_fail = await client.create_reservation(account_ref="acct", slot=slot, metadata=base_metadata)
    assert lock_status_fail.ok is False
    assert lock_status_fail.error_code == "reservation_create_provider_unavailable"

    async def _lock_graphql_fail(**_kwargs):  # noqa: ANN003
        return _json_response(200, {"errors": [{"message": "bad"}]})

    client._gql_request = _lock_graphql_fail  # type: ignore[method-assign]  # noqa: SLF001
    lock_graphql_fail = await client.create_reservation(account_ref="acct", slot=slot, metadata=base_metadata)
    assert lock_graphql_fail.ok is False
    assert lock_graphql_fail.error_code == "reservation_create_slot_lock_graphql_error"

    async def _lock_unsuccessful(**_kwargs):  # noqa: ANN003
        return _json_response(200, {"data": {"lockSlot": {"success": False}}})

    client._gql_request = _lock_unsuccessful  # type: ignore[method-assign]  # noqa: SLF001
    lock_unsuccessful = await client.create_reservation(account_ref="acct", slot=slot, metadata=base_metadata)
    assert lock_unsuccessful.ok is False
    assert lock_unsuccessful.error_code == "reservation_create_slot_lock_failed"

    async def _lock_missing_slot_lock(**_kwargs):  # noqa: ANN003
        return _json_response(200, {"data": {"lockSlot": {"success": True}}})

    client._gql_request = _lock_missing_slot_lock  # type: ignore[method-assign]  # noqa: SLF001
    lock_missing_slot_lock = await client.create_reservation(account_ref="acct", slot=slot, metadata=base_metadata)
    assert lock_missing_slot_lock.ok is False
    assert lock_missing_slot_lock.error_code == "reservation_create_slot_lock_missing"

    create_status_fail_client = OpenTableProviderClient(
        transport=_QueueTransport([_json_response(503, {})]),
        create_operation_sha256="create-hash",
    )

    async def _lock_success(**_kwargs):  # noqa: ANN003
        return _json_response(200, {"data": {"lockSlot": {"success": True, "slotLockId": "LOCK-1"}}})

    create_status_fail_client._gql_request = _lock_success  # type: ignore[method-assign]  # noqa: SLF001
    create_status_fail = await create_status_fail_client.create_reservation(
        account_ref="acct",
        slot=slot,
        metadata=base_metadata,
    )
    assert create_status_fail.ok is False
    assert create_status_fail.error_code == "reservation_create_provider_unavailable"

    create_payload_fail_client = OpenTableProviderClient(
        transport=_QueueTransport([_json_response(200, {"success": False})]),
        create_operation_sha256="create-hash",
    )
    create_payload_fail_client._gql_request = _lock_success  # type: ignore[method-assign]  # noqa: SLF001
    create_payload_fail = await create_payload_fail_client.create_reservation(
        account_ref="acct",
        slot=slot,
        metadata=base_metadata,
    )
    assert create_payload_fail.ok is False
    assert create_payload_fail.error_code == "reservation_create_failed"

    confirm_client = OpenTableProviderClient(
        transport=_QueueTransport(
            [_json_response(200, {"success": True, "confirmationNumber": "100", "securityToken": "sec", "reservationId": "res-1"})]
        ),
        create_operation_sha256="create-hash",
        confirmation_operation_sha256="confirm-hash",
    )
    gql_calls = {"count": 0}

    async def _lock_then_confirm(**_kwargs):  # noqa: ANN003
        gql_calls["count"] += 1
        if gql_calls["count"] == 1:
            return _json_response(200, {"data": {"lockSlot": {"success": True, "slotLockId": "LOCK-1"}}})
        return _json_response(
            200,
            {
                "data": {
                    "bookingConfirmationPageInFlow": None,
                    "bookingConfirmation": {"reservation": "bad", "restaurant": "bad"},
                }
            },
        )

    confirm_client._gql_request = _lock_then_confirm  # type: ignore[method-assign]  # noqa: SLF001
    confirm_result = await confirm_client.create_reservation(account_ref="acct", slot=slot, metadata=base_metadata)
    assert confirm_result.ok is True
    assert confirm_result.data["confirmation_enrichment_attempted"] is True

    confirm_exception_client = OpenTableProviderClient(
        transport=_QueueTransport(
            [_json_response(200, {"success": True, "confirmationNumber": "100", "securityToken": "sec", "reservationId": "res-1"})]
        ),
        create_operation_sha256="create-hash",
        confirmation_operation_sha256="confirm-hash",
    )
    gql_calls_exception = {"count": 0}

    async def _lock_then_raise(**_kwargs):  # noqa: ANN003
        gql_calls_exception["count"] += 1
        if gql_calls_exception["count"] == 1:
            return _json_response(200, {"data": {"lockSlot": {"success": True, "slotLockId": "LOCK-1"}}})
        raise RuntimeError("confirm down")

    confirm_exception_client._gql_request = _lock_then_raise  # type: ignore[method-assign]  # noqa: SLF001
    confirm_exception = await confirm_exception_client.create_reservation(
        account_ref="acct",
        slot=slot,
        metadata=base_metadata,
    )
    assert confirm_exception.ok is True
    assert confirm_exception.data["confirmation_enrichment_error_code"] == "reservation_create_confirmation_failed"
