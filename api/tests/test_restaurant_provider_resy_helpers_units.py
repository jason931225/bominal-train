from __future__ import annotations

import json
from dataclasses import dataclass
from datetime import datetime
from typing import Any

import pytest

from app.modules.restaurant.providers.base import RestaurantSearchSlot
from app.modules.restaurant.providers.resy_adapter import (
    ResyProviderClient,
    _as_bool,
    _as_int,
    _deep_get,
    _extract_book_token,
    _find_first,
    _normalize_status_error,
    _parse_datetime,
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
        provider="RESY",
        restaurant_id="1505",
        party_size=2,
        date_time_local=datetime(2026, 2, 23, 19, 30),
        availability_token="token-1",
        metadata_safe={},
    )


def test_resy_helper_functions_cover_edge_branches():
    assert _safe_json_loads("{bad") == {}
    assert _safe_json_loads("[]") == {}
    assert _find_first({"x": [{"y": "z"}]}, ("y",)) == "z"
    assert _find_first(["x", {"a": 1}], ("missing",)) is None
    assert _deep_get({"a": {"b": 1}}, "a.c") is None

    assert _normalize_status_error("x", 401) == ("x_auth_required", False)
    assert _normalize_status_error("x", 429) == ("x_rate_limited", True)
    assert _normalize_status_error("x", 503) == ("x_provider_unavailable", True)
    assert _normalize_status_error("x", 400) == ("x_failed", False)

    fallback = datetime(2026, 2, 23, 11, 0)
    assert _parse_datetime(None, fallback) == fallback
    assert _parse_datetime("2026-02-23", fallback).hour == 11
    assert _parse_datetime("not-date", fallback) == fallback

    assert _as_bool("YES") is True
    assert _as_bool("off") is False
    assert _as_bool(2) is True
    assert _as_bool("anything") is True
    assert _as_int(True) is None
    assert _as_int("10") == 10
    assert _as_int("bad") is None
    assert _as_int(["list"]) is None

    assert _extract_book_token({"book_token": "abc"}) == "abc"
    assert _extract_book_token({"bookToken": {"token": "xyz"}}) == "xyz"
    assert _extract_book_token({"bookTokenValue": "fallback"}) == "fallback"
    assert _extract_book_token({"bookToken": {"token": " "}}) is None


@pytest.mark.asyncio
async def test_resy_auth_refresh_logout_profile_search_create_and_cancel_error_paths():
    auth_headers_client = ResyProviderClient(transport=_QueueTransport([]), auth_api_key=None)
    api_key_missing = await auth_headers_client.refresh_auth(account_ref="acct")
    assert api_key_missing.ok is False
    assert api_key_missing.error_code == "auth_refresh_api_key_unconfigured"

    account_ref_missing_client = ResyProviderClient(transport=_QueueTransport([]), auth_api_key="key-1")
    account_ref_missing = await account_ref_missing_client.refresh_auth(account_ref="")
    assert account_ref_missing.ok is False
    assert account_ref_missing.error_code == "auth_refresh_account_ref_missing"

    refresh_fail_payload_client = ResyProviderClient(
        transport=_QueueTransport([_json_response(200, {"success": False, "errorCode": "E"})]),
        auth_api_key="key-1",
    )
    refresh_fail_payload = await refresh_fail_payload_client.refresh_auth(account_ref="acct")
    assert refresh_fail_payload.ok is False
    assert refresh_fail_payload.error_code == "auth_refresh_failed"

    logout_error_outcome_client = ResyProviderClient(transport=_QueueTransport([]), auth_api_key="key-1")
    logout_error_outcome = await logout_error_outcome_client.logout(account_ref="")
    assert logout_error_outcome.ok is False
    assert logout_error_outcome.error_code == "logout_account_ref_missing"

    headers, error = logout_error_outcome_client._auth_headers(  # noqa: SLF001
        account_ref="acct",
        operation_prefix="logout",
        extra={"X-Test": "1"},
    )
    assert error is None
    assert headers is not None
    assert headers["X-Test"] == "1"

    logout_status_fail_client = ResyProviderClient(
        transport=_QueueTransport([_json_response(503, {})]),
        auth_api_key="key-1",
    )
    logout_status_fail = await logout_status_fail_client.logout(account_ref="acct")
    assert logout_status_fail.ok is False
    assert logout_status_fail.error_code == "logout_provider_unavailable"

    logout_payload_fail_client = ResyProviderClient(
        transport=_QueueTransport([_json_response(200, {"success": False, "errorCode": "E"})]),
        auth_api_key="key-1",
    )
    logout_payload_fail = await logout_payload_fail_client.logout(account_ref="acct")
    assert logout_payload_fail.ok is False
    assert logout_payload_fail.error_code == "logout_failed"

    profile_payload_fail_client = ResyProviderClient(
        transport=_QueueTransport([_json_response(200, {"success": False, "errorCode": "E"})]),
        auth_api_key="key-1",
    )
    profile_payload_fail = await profile_payload_fail_client.get_user_profile(account_ref="acct")
    assert profile_payload_fail.ok is False
    assert profile_payload_fail.error_code == "profile_get_failed"

    profile_status_fail_client = ResyProviderClient(
        transport=_QueueTransport([_json_response(503, {})]),
        auth_api_key="key-1",
    )
    profile_status_fail = await profile_status_fail_client.get_user_profile(account_ref="acct")
    assert profile_status_fail.ok is False
    assert profile_status_fail.error_code == "profile_get_provider_unavailable"

    search_client = ResyProviderClient(
        transport=_QueueTransport(
            [
                _json_response(200, {"results": {"venues": ["bad", {"venue": {"id": {"resy": 1505}}, "slots": "bad"}]}}),
            ]
        ),
        auth_api_key="key-1",
    )
    search_ok = await search_client.search_availability(
        account_ref="acct",
        restaurant_id="1505",
        party_size=2,
        date_time_local=datetime(2026, 2, 23, 19, 30),
        metadata={},
    )
    assert search_ok.ok is True
    assert search_ok.data["slot_count"] == 0

    search_status_fail_client = ResyProviderClient(
        transport=_QueueTransport([_json_response(503, {})]),
        auth_api_key="key-1",
    )
    search_status_fail = await search_status_fail_client.search_availability(
        account_ref="acct",
        restaurant_id="1505",
        party_size=2,
        date_time_local=datetime(2026, 2, 23, 19, 30),
        metadata={},
    )
    assert search_status_fail.ok is False
    assert search_status_fail.error_code == "search_provider_unavailable"

    search_payload_fail_client = ResyProviderClient(
        transport=_QueueTransport([_json_response(200, {"success": False})]),
        auth_api_key="key-1",
    )
    search_payload_fail = await search_payload_fail_client.search_availability(
        account_ref="acct",
        restaurant_id="1505",
        party_size=2,
        date_time_local=datetime(2026, 2, 23, 19, 30),
        metadata={},
    )
    assert search_payload_fail.ok is False
    assert search_payload_fail.error_code == "search_failed"

    search_slot_skip_client = ResyProviderClient(
        transport=_QueueTransport(
            [
                _json_response(
                    200,
                    {
                        "results": {
                            "venues": [
                                {
                                    "venue": {"id": "fallback-id", "name": "Resy Test"},
                                    "slots": [
                                        "invalid",
                                        {"date": {"start": "2026-02-23 19:30"}, "config": {"token": " ", "type": "Standard"}},
                                    ],
                                }
                            ]
                        }
                    },
                ),
            ]
        ),
        auth_api_key="key-1",
    )
    search_slot_skip = await search_slot_skip_client.search_availability(
        account_ref="acct",
        restaurant_id="1505",
        party_size=2,
        date_time_local=datetime(2026, 2, 23, 19, 30),
        metadata={},
    )
    assert search_slot_skip.ok is True
    assert search_slot_skip.data["slot_count"] == 0

    create_missing_token_client = ResyProviderClient(transport=_QueueTransport([]), auth_api_key="key-1")
    missing_token_slot = RestaurantSearchSlot(
        provider_slot_id="",
        provider="RESY",
        restaurant_id="1505",
        party_size=2,
        date_time_local=datetime(2026, 2, 23, 19, 30),
        availability_token=None,
        metadata_safe={},
    )
    create_missing_token = await create_missing_token_client.create_reservation(
        account_ref="acct",
        slot=missing_token_slot,
        metadata={},
    )
    assert create_missing_token.ok is False
    assert create_missing_token.error_code == "reservation_create_slot_token_missing"

    create_details_fail_client = ResyProviderClient(
        transport=_QueueTransport([_json_response(503, {})]),
        auth_api_key="key-1",
    )
    create_details_fail = await create_details_fail_client.create_reservation(
        account_ref="acct",
        slot=_slot(),
        metadata={"notes": "hello"},
    )
    assert create_details_fail.ok is False
    assert create_details_fail.error_code == "reservation_create_provider_unavailable"

    create_details_payload_fail_client = ResyProviderClient(
        transport=_QueueTransport([_json_response(200, {"success": False})]),
        auth_api_key="key-1",
    )
    create_details_payload_fail = await create_details_payload_fail_client.create_reservation(
        account_ref="acct",
        slot=_slot(),
        metadata={},
    )
    assert create_details_payload_fail.ok is False
    assert create_details_payload_fail.error_code == "reservation_create_details_failed"

    create_book_status_fail_client = ResyProviderClient(
        transport=_QueueTransport(
            [
                _json_response(200, {"book_token": {"token": "book-1"}}),
                _json_response(503, {}),
            ]
        ),
        auth_api_key="key-1",
    )
    create_book_status_fail = await create_book_status_fail_client.create_reservation(
        account_ref="acct",
        slot=_slot(),
        metadata={"struct_payment_method": "pm-token", "venue_marketing_opt_in": True},
    )
    assert create_book_status_fail.ok is False
    assert create_book_status_fail.error_code == "reservation_create_provider_unavailable"

    create_book_payload_fail_client = ResyProviderClient(
        transport=_QueueTransport(
            [
                _json_response(200, {"book_token": {"token": "book-1"}}),
                _json_response(200, {"success": False}),
            ]
        ),
        auth_api_key="key-1",
    )
    create_book_payload_fail = await create_book_payload_fail_client.create_reservation(
        account_ref="acct",
        slot=_slot(),
        metadata={"payment_method_id": "pm-1"},
    )
    assert create_book_payload_fail.ok is False
    assert create_book_payload_fail.error_code == "reservation_create_failed"

    create_book_missing_reference_client = ResyProviderClient(
        transport=_QueueTransport(
            [
                _json_response(200, {"book_token": {"token": "book-1"}}),
                _json_response(200, {"success": True}),
            ]
        ),
        auth_api_key="key-1",
    )
    create_book_missing_reference = await create_book_missing_reference_client.create_reservation(
        account_ref="acct",
        slot=_slot(),
        metadata={"idempotency_key": "idem-1"},
    )
    assert create_book_missing_reference.ok is False
    assert create_book_missing_reference.error_code == "reservation_create_missing_reference"

    create_struct_payment_dict_client = ResyProviderClient(
        transport=_QueueTransport(
            [
                _json_response(200, {"book_token": {"token": "book-1"}}),
                _json_response(200, {"success": True, "reservation_id": "res-1"}),
            ]
        ),
        auth_api_key="key-1",
    )
    create_struct_payment_dict = await create_struct_payment_dict_client.create_reservation(
        account_ref="acct",
        slot=_slot(),
        metadata={"struct_payment_method": {"id": "pm-dict"}},
    )
    assert create_struct_payment_dict.ok is True
    assert create_struct_payment_dict.data["reservation_id"] == "res-1"

    cancel_payload_fail_client = ResyProviderClient(
        transport=_QueueTransport([_json_response(200, {"success": False})]),
        auth_api_key="key-1",
    )
    cancel_payload_fail = await cancel_payload_fail_client.cancel_reservation(
        account_ref="acct",
        restaurant_id="1505",
        confirmation_number="resy-1",
    )
    assert cancel_payload_fail.ok is False
    assert cancel_payload_fail.error_code == "reservation_cancel_failed"
