from __future__ import annotations

import asyncio
import json
import time
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
async def test_opentable_refresh_auth_runs_heartbeat_calls_concurrently():
    class _DelayByUrlTransport:
        def __init__(self) -> None:
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
            await asyncio.sleep(0.05)
            if url.endswith("/dapi/v1/session"):
                return _response({"facebook": "x", "sojern": "y"})
            return _response({"ok": True})

    transport = _DelayByUrlTransport()
    client = OpenTableProviderClient(transport=transport)

    started_at = time.perf_counter()
    result = await client.refresh_auth(account_ref="acct-1")
    elapsed = time.perf_counter() - started_at

    assert result.ok is True
    # Sequential calls are ~0.15s with this transport. Parallel should complete faster.
    assert elapsed < 0.12


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
async def test_opentable_auth_start_freezes_success_response_schema():
    transport = _QueueTransport(
        [
            _response(
                {
                    "otpMfaToken": "otp-mfa-token-1",
                    "phoneCountryCode": "1",
                    "countryCode": "US",
                    "suppressOtpMfaTokenValidationFailure": True,
                }
            )
        ]
    )
    client = OpenTableProviderClient(transport=transport)

    start_result = await client.authenticate_start(account_identifier="user@example.com", delivery_channel="sms")

    assert start_result.ok is True
    assert start_result.data["requires_otp"] is True
    assert start_result.data["challenge_ref_present"] is True
    assert start_result.data["phone_country_code"] == "1"
    assert start_result.data["country_code"] == "US"
    assert start_result.data["suppress_otp_mfa_token_validation_failure"] is True
    challenge_payload = json.loads(start_result.data["challenge_token"])
    assert challenge_payload == {
        "challenge_ref": "otp-mfa-token-1",
        "phone_country_code": "1",
        "country_code": "US",
        "suppress_otp_mfa_token_validation_failure": True,
    }


@pytest.mark.asyncio
async def test_opentable_auth_start_requires_challenge_reference_on_success_status():
    transport = _QueueTransport([_response({"success": True})])
    client = OpenTableProviderClient(transport=transport)

    start_result = await client.authenticate_start(account_identifier="user@example.com", delivery_channel="email")

    assert start_result.ok is False
    assert start_result.error_code == "auth_start_challenge_missing"
    assert start_result.retryable is False


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
async def test_opentable_auth_complete_rejects_body_level_failure_on_http_200():
    transport = _QueueTransport([_response({"success": False, "errorCode": "OTP_INVALID"})])
    client = OpenTableProviderClient(transport=transport)

    result = await client.authenticate_complete(
        account_identifier="user@example.com",
        challenge_token=json.dumps({"challenge_ref": "otp-mfa-token-1"}),
        otp_code="123456",
    )

    assert result.ok is False
    assert result.error_code == "auth_complete_failed"
    assert result.data["provider_error_code"] == "OTP_INVALID"


@pytest.mark.asyncio
async def test_opentable_auth_complete_extracts_nested_provider_account_ref():
    transport = _QueueTransport([_response({"data": {"userProfile": {"gpid": 100153968640}}})])
    client = OpenTableProviderClient(transport=transport)

    result = await client.authenticate_complete(
        account_identifier="user@example.com",
        challenge_token=json.dumps({"challenge_ref": "otp-mfa-token-1"}),
        otp_code="123456",
    )

    assert result.ok is True
    assert result.data["authenticated"] is True
    assert result.data["provider_account_ref"] == "100153968640"


@pytest.mark.asyncio
async def test_opentable_search_and_create_require_sha256_contract_configuration():
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
    assert search_result.error_code == "search_contract_sha256_unconfigured"
    assert create_result.ok is False
    assert create_result.error_code == "reservation_create_contract_sha256_unconfigured"


@pytest.mark.asyncio
async def test_opentable_search_and_create_use_frozen_contract_configuration():
    slot_time = datetime(2026, 2, 27, 19, 30)
    transport = _QueueTransport(
        [
            _response(
                {
                    "data": {
                        "availability": [
                            {
                                "restaurantId": 349132,
                                "availabilityDays": [
                                    {
                                        "date": "2026-02-27",
                                        "slots": [
                                            {
                                                "slotHash": "3881139754",
                                                "timeOffsetMinutes": 1170,
                                                "slotAvailabilityToken": "avt-1",
                                                "type": "Standard",
                                                "attributes": ["default"],
                                                "diningAreasBySeating": [{"diningAreaId": 1}],
                                            },
                                            {
                                                "slotHash": "3881139755",
                                                "timeOffsetMinutes": 1200,
                                                "slotAvailabilityToken": "avt-2",
                                                "type": "Standard",
                                                "attributes": ["default"],
                                                "diningAreasBySeating": [{"diningAreaId": 1}],
                                            },
                                        ],
                                    }
                                ],
                            }
                        ]
                    }
                }
            ),
            _response(
                {
                    "data": {
                        "lockSlot": {
                            "success": True,
                            "slotLock": {
                                "slotLockId": 1587118118,
                            },
                            "slotLockErrors": None,
                        }
                    }
                }
            ),
            _response(
                {
                    "success": True,
                    "confirmationNumber": 2110076913,
                    "reservationId": 2018060113,
                    "securityToken": "sec-1",
                }
            ),
        ]
    )
    client = OpenTableProviderClient(
        transport=transport,
        search_operation_name="RestaurantsAvailability",
        search_operation_sha256="hash-search-1",
        search_slot_path="data.availability",
        create_operation_name="BookDetailsStandardSlotLock",
        create_operation_sha256="hash-create-1",
        create_path="/dapi/booking/make-reservation",
    )

    search_result = await client.search_availability(
        account_ref="acct-1",
        restaurant_id="349132",
        party_size=2,
        date_time_local=slot_time,
        metadata={
            "restaurant_availability_token": "restaurant-token-1",
            "database_region": "EU",
            "correlation_id": "corr-1",
            "attribution_token": "attr-1",
        },
    )
    assert search_result.ok is True
    assert search_result.data["slot_count"] == 2
    slots = search_result.data["slots"]
    assert slots[0].provider_slot_id == "3881139754"
    assert slots[0].availability_token == "avt-1"
    assert slots[0].metadata_safe["dining_area_id"] == 1
    search_request = transport.requests[0]
    assert "optype=query&opname=RestaurantsAvailability" in search_request["url"]
    assert search_request["json_body"]["extensions"]["persistedQuery"]["sha256Hash"] == "hash-search-1"
    assert search_request["json_body"]["variables"] == {
        "onlyPop": False,
        "forwardDays": 0,
        "requireTimes": False,
        "requireTypes": ["Standard", "Experience"],
        "privilegedAccess": [
            "UberOneDiningProgram",
            "VisaDiningProgram",
            "VisaEventsProgram",
            "ChaseDiningProgram",
        ],
        "restaurantIds": [349132],
        "date": "2026-02-27",
        "time": "19:30",
        "partySize": 2,
        "databaseRegion": "EU",
        "restaurantAvailabilityTokens": ["restaurant-token-1"],
        "loyaltyRedemptionTiers": [],
        "attributionToken": "attr-1",
        "correlationId": "corr-1",
    }

    create_result = await client.create_reservation(
        account_ref="acct-1",
        slot=slots[0],
        metadata={
            "email": "user@example.com",
            "first_name": "Jason",
            "last_name": "Lee",
            "phone_number": "6460001111",
            "phone_number_country_id": "US",
            "country": "US",
            "points": 100,
            "points_type": "Standard",
            "confirm_points": True,
            "opt_in_email_restaurant": False,
            "reservation_attribute": "default",
            "database_region": "NA",
            "dining_area_id": 1,
            "seating_option": "DEFAULT",
            "reservation_type": "STANDARD",
            "reservation_type_display": "Standard",
            "correlation_id": "corr-1",
            "attribution_token": "attr-1",
        },
    )
    assert create_result.ok is True
    assert create_result.data["confirmation_number"] == "2110076913"
    assert create_result.data["reservation_id"] == "2018060113"
    assert create_result.data["security_token_present"] is True
    assert create_result.data["slot_lock_id"] == "1587118118"
    assert create_result.data["confirmation_enrichment_attempted"] is False
    assert create_result.data["confirmation_enrichment"] is None
    assert create_result.data["confirmation_enrichment_error_code"] is None
    assert create_result.data["policy_safe"] == {
        "confirm_points": True,
        "marketing_opt_in_restaurant": False,
        "restaurant_policy_acknowledged": False,
    }

    lock_request = transport.requests[1]
    assert "optype=mutation&opname=BookDetailsStandardSlotLock" in lock_request["url"]
    assert lock_request["json_body"]["extensions"]["persistedQuery"]["sha256Hash"] == "hash-create-1"
    assert lock_request["json_body"]["variables"] == {
        "input": {
            "restaurantId": 349132,
            "seatingOption": "DEFAULT",
            "reservationDateTime": "2026-02-27T19:30",
            "partySize": 2,
            "databaseRegion": "NA",
            "slotHash": "3881139754",
            "reservationType": "STANDARD",
            "diningAreaId": 1,
        }
    }

    make_request = transport.requests[2]
    assert make_request["url"].endswith("/dapi/booking/make-reservation")
    assert make_request["json_body"]["restaurantId"] == 349132
    assert make_request["json_body"]["slotHash"] == "3881139754"
    assert make_request["json_body"]["slotAvailabilityToken"] == "avt-1"
    assert make_request["json_body"]["slotLockId"] == 1587118118
    assert make_request["json_body"]["email"] == "user@example.com"
    assert make_request["json_body"]["firstName"] == "Jason"
    assert make_request["json_body"]["lastName"] == "Lee"
    assert make_request["json_body"]["phoneNumber"] == "6460001111"
    assert make_request["json_body"]["confirmPoints"] is True
    assert make_request["json_body"]["optInEmailRestaurant"] is False


@pytest.mark.asyncio
async def test_opentable_create_requires_contact_fields():
    transport = _QueueTransport([])
    client = OpenTableProviderClient(
        transport=transport,
        create_operation_name="BookDetailsStandardSlotLock",
        create_operation_sha256="hash-create-1",
    )

    create_result = await client.create_reservation(
        account_ref="acct-1",
        slot=RestaurantSearchSlot(
            provider_slot_id="3881139754",
            provider="OPENTABLE",
            restaurant_id="349132",
            party_size=2,
            date_time_local=datetime(2026, 2, 27, 19, 30),
            availability_token="avt-1",
        ),
        metadata={"email": "user@example.com"},
    )

    assert create_result.ok is False
    assert create_result.error_code == "reservation_create_contact_missing"
    assert create_result.data["missing_fields"] == ["first_name", "last_name", "phone_number"]
    assert transport.requests == []


@pytest.mark.asyncio
async def test_opentable_create_uses_optional_booking_confirmation_enrichment_when_configured():
    slot_time = datetime(2026, 2, 27, 19, 30)
    transport = _QueueTransport(
        [
            _response(
                {
                    "data": {
                        "lockSlot": {
                            "success": True,
                            "slotLock": {"slotLockId": 1587118118},
                            "slotLockErrors": None,
                        }
                    }
                }
            ),
            _response(
                {
                    "success": True,
                    "confirmationNumber": 2110076913,
                    "reservationId": 2018060113,
                    "securityToken": "sec-1",
                }
            ),
            _response(
                {
                    "data": {
                        "bookingConfirmationPageInFlow": {
                            "reservation": {
                                "confirmationNumber": 2110076913,
                                "reservationId": 2018060113,
                                "reservationState": "Confirmed",
                            },
                            "restaurant": {
                                "id": 349132,
                                "name": "Le Monde",
                            },
                        }
                    }
                }
            ),
        ]
    )
    client = OpenTableProviderClient(
        transport=transport,
        create_operation_name="BookDetailsStandardSlotLock",
        create_operation_sha256="hash-create-1",
        confirmation_operation_name="BookingConfirmationPageInFlow",
        confirmation_operation_sha256="hash-confirmation-1",
    )
    create_result = await client.create_reservation(
        account_ref="acct-1",
        slot=RestaurantSearchSlot(
            provider_slot_id="3881139754",
            provider="OPENTABLE",
            restaurant_id="349132",
            party_size=2,
            date_time_local=slot_time,
            availability_token="avt-1",
            metadata_safe={"dining_area_id": 1},
        ),
        metadata={
            "email": "user@example.com",
            "first_name": "Jason",
            "last_name": "Lee",
            "phone_number": "6460001111",
            "database_region": "NA",
            "confirm_points": False,
            "opt_in_email_restaurant": True,
        },
    )

    assert create_result.ok is True
    assert create_result.data["confirmation_enrichment_attempted"] is True
    assert create_result.data["confirmation_enrichment_error_code"] is None
    assert create_result.data["confirmation_enrichment"] == {
        "restaurant_id": "349132",
        "restaurant_name": "Le Monde",
        "confirmation_number": "2110076913",
        "reservation_id": "2018060113",
        "reservation_state": "Confirmed",
    }
    assert create_result.data["policy_safe"] == {
        "confirm_points": False,
        "marketing_opt_in_restaurant": True,
        "restaurant_policy_acknowledged": False,
    }
    confirmation_request = transport.requests[2]
    assert "optype=query&opname=BookingConfirmationPageInFlow" in confirmation_request["url"]
    assert confirmation_request["json_body"]["extensions"]["persistedQuery"]["sha256Hash"] == "hash-confirmation-1"
    assert confirmation_request["json_body"]["variables"] == {
        "rid": 349132,
        "confirmationNumber": 2110076913,
        "databaseRegion": "NA",
        "securityToken": "sec-1",
        "isLoggedIn": True,
    }


@pytest.mark.asyncio
async def test_opentable_create_keeps_success_when_confirmation_enrichment_fails():
    slot_time = datetime(2026, 2, 27, 19, 30)
    transport = _QueueTransport(
        [
            _response(
                {
                    "data": {
                        "lockSlot": {
                            "success": True,
                            "slotLock": {"slotLockId": 1587118118},
                            "slotLockErrors": None,
                        }
                    }
                }
            ),
            _response(
                {
                    "success": True,
                    "confirmationNumber": 2110076913,
                    "reservationId": 2018060113,
                    "securityToken": "sec-1",
                }
            ),
            _response({}, status_code=503),
        ]
    )
    client = OpenTableProviderClient(
        transport=transport,
        create_operation_name="BookDetailsStandardSlotLock",
        create_operation_sha256="hash-create-1",
        confirmation_operation_name="BookingConfirmationPageInFlow",
        confirmation_operation_sha256="hash-confirmation-1",
    )
    create_result = await client.create_reservation(
        account_ref="acct-1",
        slot=RestaurantSearchSlot(
            provider_slot_id="3881139754",
            provider="OPENTABLE",
            restaurant_id="349132",
            party_size=2,
            date_time_local=slot_time,
            availability_token="avt-1",
            metadata_safe={"dining_area_id": 1},
        ),
        metadata={
            "email": "user@example.com",
            "first_name": "Jason",
            "last_name": "Lee",
            "phone_number": "6460001111",
            "database_region": "NA",
        },
    )

    assert create_result.ok is True
    assert create_result.data["confirmation_enrichment_attempted"] is True
    assert create_result.data["confirmation_enrichment"] is None
    assert create_result.data["confirmation_enrichment_error_code"] == "reservation_create_confirmation_failed"


@pytest.mark.asyncio
async def test_opentable_create_normalizes_policy_flags_to_safe_booleans():
    slot_time = datetime(2026, 2, 27, 19, 30)
    transport = _QueueTransport(
        [
            _response(
                {
                    "data": {
                        "lockSlot": {
                            "success": True,
                            "slotLock": {"slotLockId": 1587118118},
                            "slotLockErrors": None,
                        }
                    }
                }
            ),
            _response(
                {
                    "success": True,
                    "confirmationNumber": 2110076913,
                    "reservationId": 2018060113,
                    "securityToken": "sec-1",
                }
            ),
        ]
    )
    client = OpenTableProviderClient(
        transport=transport,
        create_operation_name="BookDetailsStandardSlotLock",
        create_operation_sha256="hash-create-1",
    )
    create_result = await client.create_reservation(
        account_ref="acct-1",
        slot=RestaurantSearchSlot(
            provider_slot_id="3881139754",
            provider="OPENTABLE",
            restaurant_id="349132",
            party_size=2,
            date_time_local=slot_time,
            availability_token="avt-1",
            metadata_safe={"dining_area_id": 1},
        ),
        metadata={
            "email": "user@example.com",
            "first_name": "Jason",
            "last_name": "Lee",
            "phone_number": "6460001111",
            "confirm_points": "1",
            "opt_in_email_restaurant": "yes",
            "restaurant_policy_acknowledged": "true",
        },
    )

    assert create_result.ok is True
    assert create_result.data["policy_safe"] == {
        "confirm_points": True,
        "marketing_opt_in_restaurant": True,
        "restaurant_policy_acknowledged": True,
    }
