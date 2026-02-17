from __future__ import annotations

import json
from datetime import datetime

import pytest

from app.modules.restaurant.providers.base import RestaurantSearchSlot
from app.modules.restaurant.providers.opentable_adapter import OpenTableProviderClient
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
async def test_opentable_refresh_auth_calls_cpr_human_and_session_endpoints():
    transport = _QueueTransport(
        [
            _response({"ok": True}),
            _response({"ok": True}),
            _response({"facebook": "x", "sojern": "y"}),
        ]
    )
    client = OpenTableProviderClient(transport=transport)

    result = await client.refresh_auth(account_ref="acct-1")

    assert result.ok is True
    assert result.data["cpr_status_code"] == 200
    assert result.data["human_status_code"] == 200
    assert result.data["session_status_code"] == 200
    assert len(transport.requests) == 3
    assert transport.requests[0]["url"].endswith("/_sec/cpr/params")
    assert transport.requests[1]["url"].endswith("/dapi/fe/human")
    assert transport.requests[2]["method"] == "POST"
    assert transport.requests[2]["url"].endswith("/dapi/v1/session")


@pytest.mark.asyncio
async def test_opentable_get_user_profile_uses_gql_and_parses_safe_fields():
    transport = _QueueTransport(
        [
            _response(
                {
                    "data": {
                        "userProfile": {
                            "firstName": "Jason",
                            "lastName": "Lee",
                            "email": "user@example.com",
                            "gpid": 100153968640,
                            "countryId": "US",
                            "mobilePhoneNumber": {"number": "6460001111"},
                        }
                    }
                }
            )
        ]
    )
    client = OpenTableProviderClient(transport=transport)

    result = await client.get_user_profile(account_ref="acct-1")

    assert result.ok is True
    assert result.data["first_name"] == "Jason"
    assert result.data["last_name"] == "Lee"
    assert result.data["email"] == "user@example.com"
    assert result.data["gpid"] == 100153968640
    assert result.data["mobile_phone_number"] == "6460001111"

    request = transport.requests[0]
    assert "optype=query&opname=HeaderUserProfile" in request["url"]
    assert request["json_body"]["operationName"] == "HeaderUserProfile"


@pytest.mark.asyncio
async def test_opentable_cancel_reservation_uses_known_mutation_contract():
    transport = _QueueTransport(
        [
            _response(
                {
                    "data": {
                        "cancelReservation": {
                            "statusCode": 200,
                            "data": {
                                "reservationState": "CancelledWeb",
                                "reservationId": 2018060113,
                                "confirmationNumber": 2110076913,
                            },
                        }
                    }
                }
            )
        ]
    )
    client = OpenTableProviderClient(transport=transport)

    result = await client.cancel_reservation(
        account_ref="acct-1",
        restaurant_id="349132",
        confirmation_number="2110076913",
        security_token="token-1",
    )

    assert result.ok is True
    assert result.data["reservation_state"] == "CancelledWeb"
    assert result.data["reservation_id"] == 2018060113
    assert result.data["confirmation_number"] == 2110076913

    request = transport.requests[0]
    assert "optype=mutation&opname=CancelReservation" in request["url"]
    input_payload = request["json_body"]["variables"]["input"]
    assert input_payload["restaurantId"] == 349132
    assert input_payload["confirmationNumber"] == 2110076913
    assert input_payload["securityToken"] == "token-1"


@pytest.mark.asyncio
async def test_opentable_auth_start_uses_default_endpoint_and_payload():
    transport = _QueueTransport([_response({"challengeToken": "otp-challenge-1"})])
    client = OpenTableProviderClient(transport=transport)

    start_result = await client.authenticate_start(account_identifier="user@example.com", delivery_channel="email")

    assert start_result.ok is True
    assert start_result.data["requires_otp"] is True
    request = transport.requests[0]
    assert request["url"].endswith("/dapi/authentication/sendotpfromsignin")
    assert request["json_body"]["phoneNumberOrEmail"] == "user@example.com"
    assert request["json_body"]["channelType"] == "EMAIL"
    assert request["json_body"]["isReauthentication"] is False


@pytest.mark.asyncio
async def test_opentable_auth_start_password_input_is_rejected():
    client = OpenTableProviderClient(transport=_QueueTransport([]))

    start_result = await client.authenticate_start(
        account_identifier="user@example.com",
        password="secret",
        delivery_channel="email",
    )

    assert start_result.ok is False
    assert start_result.error_code == "auth_start_password_not_supported"


@pytest.mark.asyncio
async def test_opentable_auth_start_and_complete_call_configured_paths():
    transport = _QueueTransport(
        [
            _response({"challengeToken": "otp-challenge-1"}),
            _response({"userId": "10001"}),
        ]
    )
    client = OpenTableProviderClient(
        transport=transport,
        auth_start_path="/dapi/auth/start-otp",
        auth_complete_path="/dapi/auth/verify-otp",
    )

    start_result = await client.authenticate_start(account_identifier="user@example.com", delivery_channel="sms")
    challenge_token = start_result.data["challenge_token"]
    challenge_payload = json.loads(challenge_token)
    challenge_payload["phone_country_code"] = "1"
    challenge_payload["country_code"] = "US"
    challenge_payload["suppress_otp_mfa_token_validation_failure"] = True

    complete_result = await client.authenticate_complete(
        account_identifier="user@example.com",
        challenge_token=json.dumps(challenge_payload),
        otp_code="123456",
    )

    assert start_result.ok is True
    assert start_result.data["requires_otp"] is True
    assert complete_result.ok is True
    assert complete_result.data["provider_account_ref"] == "10001"

    assert transport.requests[0]["url"].endswith("/dapi/auth/start-otp")
    assert transport.requests[0]["json_body"]["channelType"] == "SMS"
    assert transport.requests[0]["json_body"]["phoneNumberOrEmail"] == "user@example.com"
    assert transport.requests[1]["url"].endswith("/dapi/auth/verify-otp")
    assert transport.requests[1]["json_body"]["phoneNumberOrEmail"] == "user@example.com"
    assert transport.requests[1]["json_body"]["phoneCountryCode"] == "1"
    assert transport.requests[1]["json_body"]["countryCode"] == "US"
    assert transport.requests[1]["json_body"]["otp"] == "123456"
    assert transport.requests[1]["json_body"]["isReauthentication"] is False
    assert transport.requests[1]["json_body"]["suppressOtpMfaTokenValidationFailure"] is True
    assert transport.requests[1]["json_body"]["challengeToken"] == "otp-challenge-1"


@pytest.mark.asyncio
async def test_opentable_auth_complete_requires_otp():
    client = OpenTableProviderClient(transport=_QueueTransport([]))

    result = await client.authenticate_complete(account_identifier="user@example.com", challenge_token=None, otp_code=None)

    assert result.ok is False
    assert result.error_code == "auth_complete_otp_missing"


@pytest.mark.asyncio
async def test_opentable_search_and_create_require_query_contract_metadata():
    client = OpenTableProviderClient(transport=_QueueTransport([]))
    slot_time = datetime(2026, 2, 27, 19, 30)

    search_result = await client.search_availability(
        account_ref="acct-1",
        restaurant_id="349132",
        party_size=2,
        date_time_local=slot_time,
    )
    create_result = await client.create_reservation(
        account_ref="acct-1",
        slot=RestaurantSearchSlot(
            provider_slot_id="slot-1",
            provider="OPENTABLE",
            restaurant_id="349132",
            party_size=2,
            date_time_local=slot_time,
            availability_token="avt-1",
        ),
    )

    assert search_result.ok is False
    assert search_result.error_code == "search_contract_unconfigured"
    assert create_result.ok is False
    assert create_result.error_code == "reservation_create_contract_unconfigured"


@pytest.mark.asyncio
async def test_opentable_search_and_create_work_with_contract_metadata():
    slot_time = datetime(2026, 2, 27, 19, 30)
    transport = _QueueTransport(
        [
            _response(
                {
                    "data": {
                        "search": {
                            "availableSlots": [
                                {
                                    "slotHash": "3881139754",
                                    "dateTime": "2026-02-27T19:30:00",
                                    "availabilityToken": "avt-1",
                                },
                                {
                                    "slotHash": "3881139755",
                                    "dateTime": "2026-02-27T20:00:00",
                                    "availabilityToken": "avt-2",
                                },
                            ]
                        }
                    }
                }
            ),
            _response(
                {
                    "data": {
                        "createReservation": {
                            "statusCode": 200,
                            "data": {
                                "confirmationNumber": 2110076913,
                                "reservationId": 2018060113,
                                "securityToken": "sec-1",
                            },
                        }
                    }
                }
            ),
        ]
    )
    client = OpenTableProviderClient(transport=transport)

    search_result = await client.search_availability(
        account_ref="acct-1",
        restaurant_id="349132",
        party_size=2,
        date_time_local=slot_time,
        metadata={
            "operation_name": "SearchRestaurantAvailability",
            "sha256_hash": "hash-search-1",
            "variables": {"rid": 349132, "partySize": 2},
            "slot_path": "data.search.availableSlots",
        },
    )
    assert search_result.ok is True
    assert search_result.data["slot_count"] == 2
    slots = search_result.data["slots"]
    assert slots[0].provider_slot_id == "3881139754"
    assert slots[0].availability_token == "avt-1"

    create_result = await client.create_reservation(
        account_ref="acct-1",
        slot=slots[0],
        metadata={
            "operation_name": "CreateReservation",
            "sha256_hash": "hash-create-1",
            "variables": {"input": {"rid": 349132, "slotHash": "3881139754"}},
        },
    )
    assert create_result.ok is True
    assert create_result.data["confirmation_number"] == "2110076913"
    assert create_result.data["reservation_id"] == "2018060113"
    assert create_result.data["security_token_present"] is True
