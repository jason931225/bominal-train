from __future__ import annotations

import asyncio
import json
from datetime import datetime, timedelta
from typing import Any
from urllib.parse import urlencode

from app.modules.restaurant.providers.base import RestaurantProviderOutcome, RestaurantSearchSlot
from app.modules.restaurant.providers.constants import RESTAURANT_PROVIDER_OPENTABLE
from app.modules.train.providers.transport import AsyncTransport, HttpxTransport

_DEFAULT_OPENTABLE_BASE_URL = "https://www.opentable.com"
_DEFAULT_TIMEOUT_SECONDS = 20.0
_DEFAULT_AUTH_START_PATH = "/dapi/authentication/sendotpfromsignin"
_DEFAULT_AUTH_COMPLETE_PATH = "/dapi/authentication/signinwithotp"
_DEFAULT_SEARCH_OPERATION_NAME = "RestaurantsAvailability"
_DEFAULT_SEARCH_SLOT_PATH = "data.availability"
_DEFAULT_CREATE_OPERATION_NAME = "BookDetailsStandardSlotLock"
_DEFAULT_CREATE_PATH = "/dapi/booking/make-reservation"
_DEFAULT_CONFIRMATION_OPERATION_NAME = "BookingConfirmationPageInFlow"

_HEADER_USER_PROFILE_OPERATION = "HeaderUserProfile"
_HEADER_USER_PROFILE_SHA256 = "31a457d7e16bd701258d3ee9f998ad9b59b5d3521927363fdde7164f15ff2924"

_CANCEL_RESERVATION_OPERATION = "CancelReservation"
_CANCEL_RESERVATION_SHA256 = "4ee53a006030f602bdeb1d751fa90ddc4240d9e17d015fb7976f8efcb80a026e"


def _deep_get(data: Any, dotted_path: str) -> Any:
    node = data
    for part in dotted_path.split("."):
        if isinstance(node, dict) and part in node:
            node = node[part]
            continue
        return None
    return node


def _find_first(data: Any, keys: tuple[str, ...]) -> Any:
    if isinstance(data, dict):
        for key in keys:
            if key in data:
                return data[key]
        for value in data.values():
            found = _find_first(value, keys)
            if found is not None:
                return found
        return None
    if isinstance(data, list):
        for item in data:
            found = _find_first(item, keys)
            if found is not None:
                return found
        return None
    return None


def _safe_json_loads(raw: str) -> dict[str, Any]:
    try:
        loaded = json.loads(raw)
    except Exception:
        return {}
    return loaded if isinstance(loaded, dict) else {}


def _parse_datetime(value: str | None, fallback: datetime) -> datetime:
    if not value:
        return fallback
    normalized = value.replace("Z", "+00:00")
    try:
        return datetime.fromisoformat(normalized)
    except ValueError:
        return fallback


def _coerce_int(value: Any, default: int) -> int:
    if isinstance(value, bool):
        return default
    if isinstance(value, int):
        return value
    if isinstance(value, str):
        try:
            return int(value)
        except ValueError:
            return default
    return default


def _resolve_reservation_datetime(
    *,
    raw_slot: dict[str, Any],
    day_date: str | None,
    fallback: datetime,
) -> datetime:
    raw_dt = _find_first(raw_slot, ("dateTime", "dateTimeLocal", "time"))
    if raw_dt:
        return _parse_datetime(str(raw_dt), fallback)

    time_offset_minutes = raw_slot.get("timeOffsetMinutes")
    if day_date and isinstance(time_offset_minutes, int):
        try:
            day_start = datetime.fromisoformat(f"{day_date}T00:00:00")
        except ValueError:
            return fallback
        return day_start + timedelta(minutes=time_offset_minutes)

    return fallback


def _normalize_list_of_strings(raw: Any, fallback: list[str]) -> list[str]:
    if not isinstance(raw, list):
        return fallback
    normalized = [str(value) for value in raw if isinstance(value, str) and value.strip()]
    return normalized or fallback


def _as_int_or_original(raw: str) -> int | str:
    if raw.isdigit():
        return int(raw)
    return raw


def _as_bool(raw: Any, *, default: bool = False) -> bool:
    if isinstance(raw, bool):
        return raw
    if isinstance(raw, int):
        return raw != 0
    if isinstance(raw, str):
        normalized = raw.strip().lower()
        if normalized in {"1", "true", "t", "yes", "y", "on"}:
            return True
        if normalized in {"0", "false", "f", "no", "n", "off", ""}:
            return False
    return default


def _first_dict_value(data: dict[str, Any], keys: tuple[str, ...]) -> Any:
    for key in keys:
        if key in data:
            return data[key]
    return None


def _format_local_datetime(value: datetime) -> str:
    return value.strftime("%Y-%m-%dT%H:%M")


def _normalize_status_error(prefix: str, status_code: int) -> tuple[str, bool]:
    if status_code in {401, 403}:
        return f"{prefix}_auth_required", False
    if status_code == 429:
        return f"{prefix}_rate_limited", True
    if status_code >= 500:
        return f"{prefix}_provider_unavailable", True
    return f"{prefix}_failed", False


def _normalize_channel_type(delivery_channel: str) -> str:
    normalized = delivery_channel.strip().lower()
    if normalized == "sms":
        return "SMS"
    return "EMAIL"


def _decode_challenge_token(challenge_token: str | None) -> dict[str, Any]:
    if not challenge_token:
        return {}
    try:
        decoded = json.loads(challenge_token)
    except Exception:
        return {}
    return decoded if isinstance(decoded, dict) else {}


class OpenTableProviderClient:
    provider_name = RESTAURANT_PROVIDER_OPENTABLE

    def __init__(
        self,
        *,
        transport: AsyncTransport | None = None,
        base_url: str = _DEFAULT_OPENTABLE_BASE_URL,
        timeout_seconds: float = _DEFAULT_TIMEOUT_SECONDS,
        auth_start_path: str = _DEFAULT_AUTH_START_PATH,
        auth_complete_path: str = _DEFAULT_AUTH_COMPLETE_PATH,
        search_operation_name: str = _DEFAULT_SEARCH_OPERATION_NAME,
        search_operation_sha256: str = "",
        search_slot_path: str = _DEFAULT_SEARCH_SLOT_PATH,
        create_operation_name: str = _DEFAULT_CREATE_OPERATION_NAME,
        create_operation_sha256: str = "",
        create_path: str = _DEFAULT_CREATE_PATH,
        confirmation_operation_name: str = _DEFAULT_CONFIRMATION_OPERATION_NAME,
        confirmation_operation_sha256: str = "",
    ) -> None:
        self._transport = transport or HttpxTransport()
        self._base_url = base_url.rstrip("/")
        self._timeout_seconds = timeout_seconds
        self._auth_start_path = auth_start_path or _DEFAULT_AUTH_START_PATH
        self._auth_complete_path = auth_complete_path or _DEFAULT_AUTH_COMPLETE_PATH
        self._search_operation_name = search_operation_name or _DEFAULT_SEARCH_OPERATION_NAME
        self._search_operation_sha256 = search_operation_sha256.strip()
        self._search_slot_path = search_slot_path or _DEFAULT_SEARCH_SLOT_PATH
        self._create_operation_name = create_operation_name or _DEFAULT_CREATE_OPERATION_NAME
        self._create_operation_sha256 = create_operation_sha256.strip()
        self._create_path = create_path or _DEFAULT_CREATE_PATH
        self._confirmation_operation_name = confirmation_operation_name or _DEFAULT_CONFIRMATION_OPERATION_NAME
        self._confirmation_operation_sha256 = confirmation_operation_sha256.strip()

    def _headers(
        self,
        *,
        csrf_token: str | None = None,
        extra: dict[str, str] | None = None,
    ) -> dict[str, str]:
        headers = {
            "Accept": "*/*",
            "User-Agent": "Mozilla/5.0",
        }
        if csrf_token:
            headers["x-csrf-token"] = csrf_token
        if extra:
            headers.update(extra)
        return headers

    async def _request(
        self,
        *,
        method: str,
        path: str,
        headers: dict[str, str] | None = None,
        json_body: dict[str, Any] | None = None,
        data: dict[str, Any] | None = None,
        params: dict[str, Any] | None = None,
    ):
        return await self._transport.request(
            method=method,
            url=f"{self._base_url}{path}",
            headers=headers,
            json_body=json_body,
            data=data,
            params=params,
            timeout=self._timeout_seconds,
        )

    async def _gql_request(
        self,
        *,
        operation_type: str,
        operation_name: str,
        sha256_hash: str,
        variables: dict[str, Any],
        csrf_token: str | None = None,
        page_type: str = "home",
        page_group: str = "seo-landing-home",
        query_timeout_ms: int = 5000,
    ):
        query = urlencode({"optype": operation_type, "opname": operation_name})
        payload = {
            "operationName": operation_name,
            "variables": variables,
            "extensions": {
                "persistedQuery": {
                    "version": 1,
                    "sha256Hash": sha256_hash,
                }
            },
        }
        headers = self._headers(
            csrf_token=csrf_token,
            extra={
                "content-type": "application/json",
                "ot-page-type": page_type,
                "ot-page-group": page_group,
                "x-query-timeout": str(query_timeout_ms),
            },
        )
        return await self._request(
            method="POST",
            path=f"/dapi/fe/gql?{query}",
            headers=headers,
            json_body=payload,
        )

    async def authenticate_start(
        self,
        *,
        account_identifier: str,
        password: str | None = None,
        delivery_channel: str = "email",
    ) -> RestaurantProviderOutcome:
        if password:
            return RestaurantProviderOutcome(
                ok=False,
                retryable=False,
                error_code="auth_start_password_not_supported",
                error_message_safe="OpenTable OTP auth.start does not use password input.",
            )

        body = {
            "phoneNumberOrEmail": account_identifier,
            "channelType": _normalize_channel_type(delivery_channel),
            "isReauthentication": False,
        }

        response = await self._request(
            method="POST",
            path=self._auth_start_path,
            headers=self._headers(extra={"content-type": "application/json"}),
            json_body=body,
        )
        payload = _safe_json_loads(response.text)
        if response.status_code >= 400:
            error_code, retryable = _normalize_status_error("auth_start", response.status_code)
            return RestaurantProviderOutcome(
                ok=False,
                retryable=retryable,
                error_code=error_code,
                error_message_safe="OpenTable auth.start request failed.",
                data={"status_code": response.status_code},
            )

        challenge_ref = _find_first(
            payload,
            (
                "otpMfaToken",
                "challengeToken",
                "challenge_token",
                "verificationId",
                "verification_id",
                "requestId",
                "request_id",
            ),
        )
        if challenge_ref is None or (isinstance(challenge_ref, str) and not challenge_ref.strip()):
            return RestaurantProviderOutcome(
                ok=False,
                retryable=False,
                error_code="auth_start_challenge_missing",
                error_message_safe="OpenTable auth.start response missing challenge reference.",
            )
        phone_country_code = _find_first(payload, ("phoneCountryCode", "phone_country_code"))
        country_code = _find_first(payload, ("countryCode", "country_code"))
        suppress_validation_failure = _as_bool(
            _find_first(payload, ("suppressOtpMfaTokenValidationFailure", "suppress_otp_mfa_token_validation_failure")),
            default=False,
        )
        challenge_token = json.dumps(
            {
                "challenge_ref": str(challenge_ref),
                "phone_country_code": str(phone_country_code) if phone_country_code is not None else None,
                "country_code": str(country_code) if country_code is not None else None,
                "suppress_otp_mfa_token_validation_failure": suppress_validation_failure,
            },
            separators=(",", ":"),
        )

        return RestaurantProviderOutcome(
            ok=True,
            retryable=False,
            data={
                "challenge_token": challenge_token,
                "requires_otp": True,
                "challenge_ref_present": True,
                "phone_country_code": str(phone_country_code) if phone_country_code is not None else None,
                "country_code": str(country_code) if country_code is not None else None,
                "suppress_otp_mfa_token_validation_failure": suppress_validation_failure,
            },
        )

    async def authenticate_complete(
        self,
        *,
        account_identifier: str,
        challenge_token: str | None = None,
        otp_code: str | None = None,
    ) -> RestaurantProviderOutcome:
        if not otp_code:
            return RestaurantProviderOutcome(
                ok=False,
                retryable=False,
                error_code="auth_complete_otp_missing",
                error_message_safe="OpenTable auth.complete requires otp_code.",
            )

        challenge_payload = _decode_challenge_token(challenge_token)
        body: dict[str, Any] = {
            "phoneNumberOrEmail": account_identifier,
            "phoneCountryCode": str(challenge_payload.get("phone_country_code") or ""),
            "countryCode": str(challenge_payload.get("country_code") or ""),
            "otp": otp_code,
            "isReauthentication": False,
            "suppressOtpMfaTokenValidationFailure": bool(
                challenge_payload.get("suppress_otp_mfa_token_validation_failure", False)
            ),
        }
        challenge_ref = challenge_payload.get("challenge_ref")
        if challenge_ref:
            body["challengeToken"] = challenge_ref

        response = await self._request(
            method="POST",
            path=self._auth_complete_path,
            headers=self._headers(extra={"content-type": "application/json"}),
            json_body=body,
        )
        if response.status_code >= 400:
            error_code, retryable = _normalize_status_error("auth_complete", response.status_code)
            return RestaurantProviderOutcome(
                ok=False,
                retryable=retryable,
                error_code=error_code,
                error_message_safe="OpenTable auth.complete request failed.",
                data={"status_code": response.status_code},
            )

        payload = _safe_json_loads(response.text)
        if payload.get("errors") or payload.get("success") is False:
            provider_error_code = _find_first(payload, ("errorCode", "error_code", "code"))
            error_data: dict[str, Any] = {}
            if provider_error_code is not None:
                error_data["provider_error_code"] = str(provider_error_code)
            return RestaurantProviderOutcome(
                ok=False,
                retryable=False,
                error_code="auth_complete_failed",
                error_message_safe="OpenTable auth.complete response indicates failure.",
                data=error_data,
            )
        provider_account_ref = _find_first(payload, ("gpid", "userId", "user_id"))
        return RestaurantProviderOutcome(
            ok=True,
            retryable=False,
            data={
                "authenticated": True,
                "provider_account_ref": str(provider_account_ref) if provider_account_ref is not None else None,
            },
        )

    async def refresh_auth(self, *, account_ref: str) -> RestaurantProviderOutcome:
        _ = account_ref
        cpr_response, human_response, session_response = await asyncio.gather(
            self._request(method="GET", path="/_sec/cpr/params", headers=self._headers()),
            self._request(method="GET", path="/dapi/fe/human", headers=self._headers()),
            self._request(method="POST", path="/dapi/v1/session", headers=self._headers()),
        )

        if session_response.status_code >= 400:
            error_code, retryable = _normalize_status_error("auth_refresh", session_response.status_code)
            return RestaurantProviderOutcome(
                ok=False,
                retryable=retryable,
                error_code=error_code,
                error_message_safe="OpenTable session refresh request failed.",
                data={
                    "cpr_status_code": cpr_response.status_code,
                    "human_status_code": human_response.status_code,
                    "session_status_code": session_response.status_code,
                },
            )
        if human_response.status_code >= 400:
            error_code, retryable = _normalize_status_error("auth_refresh", human_response.status_code)
            return RestaurantProviderOutcome(
                ok=False,
                retryable=retryable,
                error_code=error_code,
                error_message_safe="OpenTable human heartbeat request failed.",
                data={
                    "cpr_status_code": cpr_response.status_code,
                    "human_status_code": human_response.status_code,
                    "session_status_code": session_response.status_code,
                },
            )

        payload = _safe_json_loads(session_response.text)
        return RestaurantProviderOutcome(
            ok=True,
            retryable=False,
            data={
                "cpr_status_code": cpr_response.status_code,
                "human_status_code": human_response.status_code,
                "session_status_code": session_response.status_code,
                "session_payload_safe": {
                    "facebook_present": bool(payload.get("facebook")),
                    "sojern_present": bool(payload.get("sojern")),
                },
            },
        )

    async def get_user_profile(self, *, account_ref: str) -> RestaurantProviderOutcome:
        _ = account_ref
        response = await self._gql_request(
            operation_type="query",
            operation_name=_HEADER_USER_PROFILE_OPERATION,
            sha256_hash=_HEADER_USER_PROFILE_SHA256,
            variables={
                "isAuthenticated": False,
                "isPrivilegedAccessEnabled": True,
                "tld": "com",
                "gpid": 0,
            },
        )
        payload = _safe_json_loads(response.text)
        if response.status_code >= 400:
            error_code, retryable = _normalize_status_error("profile_get", response.status_code)
            return RestaurantProviderOutcome(
                ok=False,
                retryable=retryable,
                error_code=error_code,
                error_message_safe="OpenTable profile query failed.",
                data={"status_code": response.status_code},
            )

        if payload.get("errors"):
            return RestaurantProviderOutcome(
                ok=False,
                retryable=False,
                error_code="profile_get_failed",
                error_message_safe="OpenTable profile query returned GraphQL errors.",
            )

        user_profile = _deep_get(payload, "data.userProfile")
        if not isinstance(user_profile, dict):
            return RestaurantProviderOutcome(
                ok=False,
                retryable=False,
                error_code="profile_missing",
                error_message_safe="OpenTable profile response missing user profile.",
            )

        return RestaurantProviderOutcome(
            ok=True,
            retryable=False,
            data={
                "first_name": user_profile.get("firstName"),
                "last_name": user_profile.get("lastName"),
                "email": user_profile.get("email"),
                "gpid": user_profile.get("gpid"),
                "mobile_phone_number": _deep_get(user_profile, "mobilePhoneNumber.number"),
                "country_id": user_profile.get("countryId"),
            },
        )

    async def search_availability(
        self,
        *,
        account_ref: str,
        restaurant_id: str,
        party_size: int,
        date_time_local: datetime,
        metadata: dict[str, Any] | None = None,
    ) -> RestaurantProviderOutcome:
        _ = account_ref
        metadata = metadata or {}
        if not self._search_operation_sha256:
            return RestaurantProviderOutcome(
                ok=False,
                retryable=False,
                error_code="search_contract_sha256_unconfigured",
                error_message_safe="OpenTable search operation sha256 is not configured.",
            )

        require_types = _normalize_list_of_strings(
            metadata.get("require_types"),
            fallback=["Standard", "Experience"],
        )
        privileged_access = _normalize_list_of_strings(
            metadata.get("privileged_access"),
            fallback=["UberOneDiningProgram", "VisaDiningProgram", "VisaEventsProgram", "ChaseDiningProgram"],
        )
        loyalty_redemption_tiers = _normalize_list_of_strings(
            metadata.get("loyalty_redemption_tiers"),
            fallback=[],
        )
        restaurant_availability_tokens = _normalize_list_of_strings(
            metadata.get("restaurant_availability_tokens"),
            fallback=[],
        )
        if not restaurant_availability_tokens:
            maybe_token = metadata.get("restaurant_availability_token") or metadata.get("availability_token")
            if isinstance(maybe_token, str) and maybe_token.strip():
                restaurant_availability_tokens = [maybe_token]

        variables: dict[str, Any] = {
            "onlyPop": bool(metadata.get("only_pop", False)),
            "forwardDays": _coerce_int(metadata.get("forward_days"), 0),
            "requireTimes": bool(metadata.get("require_times", False)),
            "requireTypes": require_types,
            "privilegedAccess": privileged_access,
            "restaurantIds": [_as_int_or_original(restaurant_id)],
            "date": date_time_local.strftime("%Y-%m-%d"),
            "time": date_time_local.strftime("%H:%M"),
            "partySize": party_size,
            "databaseRegion": str(metadata.get("database_region") or "EU"),
            "restaurantAvailabilityTokens": restaurant_availability_tokens,
            "loyaltyRedemptionTiers": loyalty_redemption_tiers,
        }
        attribution_token = metadata.get("attribution_token")
        if isinstance(attribution_token, str) and attribution_token.strip():
            variables["attributionToken"] = attribution_token
        correlation_id = metadata.get("correlation_id")
        if isinstance(correlation_id, str) and correlation_id.strip():
            variables["correlationId"] = correlation_id

        response = await self._gql_request(
            operation_type="query",
            operation_name=self._search_operation_name,
            sha256_hash=self._search_operation_sha256,
            variables=variables,
            page_type="search",
            page_group="booking",
            query_timeout_ms=5000,
        )
        payload = _safe_json_loads(response.text)
        if response.status_code >= 400:
            error_code, retryable = _normalize_status_error("search", response.status_code)
            return RestaurantProviderOutcome(
                ok=False,
                retryable=retryable,
                error_code=error_code,
                error_message_safe="OpenTable search query failed.",
                data={"status_code": response.status_code},
            )
        if payload.get("errors"):
            return RestaurantProviderOutcome(
                ok=False,
                retryable=False,
                error_code="search_graphql_error",
                error_message_safe="OpenTable search returned GraphQL errors.",
            )

        slot_rows: list[tuple[dict[str, Any], str | None, str]] = []
        slots_node: Any = None
        if self._search_slot_path:
            slots_node = _deep_get(payload, self._search_slot_path)
        if isinstance(slots_node, list):
            for restaurant_entry in slots_node:
                if not isinstance(restaurant_entry, dict):
                    continue
                resolved_restaurant_id = _find_first(restaurant_entry, ("restaurantId", "rid"))
                resolved_restaurant_id_str = (
                    str(resolved_restaurant_id) if resolved_restaurant_id is not None else restaurant_id
                )
                availability_days = restaurant_entry.get("availabilityDays")
                if isinstance(availability_days, list):
                    for day_entry in availability_days:
                        if not isinstance(day_entry, dict):
                            continue
                        day_date = str(day_entry.get("date")) if day_entry.get("date") else None
                        day_slots = day_entry.get("slots")
                        if not isinstance(day_slots, list):
                            continue
                        for day_slot in day_slots:
                            if isinstance(day_slot, dict):
                                slot_rows.append((day_slot, day_date, resolved_restaurant_id_str))

        if not slot_rows:
            fallback_slots = _find_first(payload, ("availableSlots", "slots", "availabilities"))
            if isinstance(fallback_slots, list):
                for fallback_slot in fallback_slots:
                    if isinstance(fallback_slot, dict):
                        slot_rows.append((fallback_slot, None, restaurant_id))

        slots: list[RestaurantSearchSlot] = []
        for raw_slot, day_date, resolved_restaurant_id in slot_rows:
            raw_slot_id = _find_first(raw_slot, ("slotHash", "slotId", "id", "slotAvailabilityToken", "availabilityToken"))
            if raw_slot_id is None:
                continue
            parsed_dt = _resolve_reservation_datetime(
                raw_slot=raw_slot,
                day_date=day_date,
                fallback=date_time_local,
            )
            availability_token = _find_first(raw_slot, ("slotAvailabilityToken", "availabilityToken", "token"))
            reservation_type = _find_first(raw_slot, ("type", "reservationType"))
            attributes = raw_slot.get("attributes")
            seating_option = "DEFAULT"
            if isinstance(attributes, list) and attributes and isinstance(attributes[0], str):
                seating_option = attributes[0].upper()

            dining_area_id: Any = _find_first(raw_slot, ("diningAreaId",))
            if dining_area_id is None:
                dining_areas = raw_slot.get("diningAreasBySeating")
                if isinstance(dining_areas, list):
                    for entry in dining_areas:
                        if isinstance(entry, dict):
                            dining_area_id = _find_first(entry, ("diningAreaId",))
                            if dining_area_id is not None:
                                break

            metadata_safe: dict[str, Any] = {
                "raw_slot_id": str(raw_slot_id),
                "slot_availability_token_present": bool(availability_token),
                "seating_option": seating_option,
                "reservation_type": str(reservation_type).upper() if reservation_type else "STANDARD",
            }
            if day_date:
                metadata_safe["day_date"] = day_date
            if dining_area_id is not None:
                metadata_safe["dining_area_id"] = dining_area_id
            slots.append(
                RestaurantSearchSlot(
                    provider_slot_id=str(raw_slot_id),
                    provider=RESTAURANT_PROVIDER_OPENTABLE,
                    restaurant_id=resolved_restaurant_id,
                    party_size=party_size,
                    date_time_local=parsed_dt,
                    availability_token=str(availability_token) if availability_token else None,
                    metadata_safe=metadata_safe,
                )
            )

        return RestaurantProviderOutcome(
            ok=True,
            retryable=False,
            data={
                "slots": slots,
                "slot_count": len(slots),
            },
        )

    async def create_reservation(
        self,
        *,
        account_ref: str,
        slot: RestaurantSearchSlot,
        metadata: dict[str, Any] | None = None,
    ) -> RestaurantProviderOutcome:
        _ = account_ref
        metadata = metadata or {}
        if not self._create_operation_sha256:
            return RestaurantProviderOutcome(
                ok=False,
                retryable=False,
                error_code="reservation_create_contract_sha256_unconfigured",
                error_message_safe="OpenTable reservation.create operation sha256 is not configured.",
            )
        if not slot.availability_token:
            return RestaurantProviderOutcome(
                ok=False,
                retryable=False,
                error_code="reservation_create_slot_token_missing",
                error_message_safe="OpenTable reservation.create requires slot availability token.",
            )
        required_contact_fields = ("email", "first_name", "last_name", "phone_number")
        missing_contact_fields = [field_name for field_name in required_contact_fields if not metadata.get(field_name)]
        if missing_contact_fields:
            return RestaurantProviderOutcome(
                ok=False,
                retryable=False,
                error_code="reservation_create_contact_missing",
                error_message_safe="OpenTable reservation.create requires diner contact profile fields.",
                data={"missing_fields": missing_contact_fields},
            )

        dining_area_id = _coerce_int(metadata.get("dining_area_id") or slot.metadata_safe.get("dining_area_id"), 1)
        seating_option = str(metadata.get("seating_option") or slot.metadata_safe.get("seating_option") or "DEFAULT")
        reservation_type_upper = str(
            metadata.get("reservation_type") or slot.metadata_safe.get("reservation_type") or "STANDARD"
        ).upper()
        reservation_type_display = str(metadata.get("reservation_type_display") or reservation_type_upper.title())
        database_region = str(metadata.get("database_region") or "NA")
        reservation_datetime_local = _format_local_datetime(slot.date_time_local)
        confirm_points = _as_bool(metadata.get("confirm_points"), default=False)
        marketing_opt_in_restaurant = _as_bool(metadata.get("opt_in_email_restaurant"), default=False)
        restaurant_policy_acknowledged = _as_bool(
            _first_dict_value(
                metadata,
                (
                    "restaurant_policy_acknowledged",
                    "policy_acknowledged",
                    "agreed_restaurant_policy",
                ),
            ),
            default=False,
        )

        lock_variables: dict[str, Any] = {
            "input": {
                "restaurantId": _as_int_or_original(slot.restaurant_id),
                "seatingOption": seating_option.upper(),
                "reservationDateTime": reservation_datetime_local,
                "partySize": slot.party_size,
                "databaseRegion": database_region,
                "slotHash": slot.provider_slot_id,
                "reservationType": reservation_type_upper,
                "diningAreaId": dining_area_id,
            }
        }

        lock_response = await self._gql_request(
            operation_type="mutation",
            operation_name=self._create_operation_name,
            sha256_hash=self._create_operation_sha256,
            variables=lock_variables,
            page_type="booking",
            page_group="booking",
            query_timeout_ms=5000,
        )
        lock_payload = _safe_json_loads(lock_response.text)
        if lock_response.status_code >= 400:
            error_code, retryable = _normalize_status_error("reservation_create", lock_response.status_code)
            return RestaurantProviderOutcome(
                ok=False,
                retryable=retryable,
                error_code=error_code,
                error_message_safe="OpenTable reservation.create slot lock failed.",
                data={"status_code": lock_response.status_code},
            )
        if lock_payload.get("errors"):
            return RestaurantProviderOutcome(
                ok=False,
                retryable=False,
                error_code="reservation_create_slot_lock_graphql_error",
                error_message_safe="OpenTable reservation.create slot lock returned GraphQL errors.",
            )

        lock_result = _deep_get(lock_payload, "data.lockSlot")
        lock_success = _find_first(lock_result, ("success",))
        if lock_success is not True:
            return RestaurantProviderOutcome(
                ok=False,
                retryable=False,
                error_code="reservation_create_slot_lock_failed",
                error_message_safe="OpenTable reservation.create slot lock did not succeed.",
            )

        slot_lock_id = _find_first(lock_result, ("slotLockId",))
        if slot_lock_id is None:
            slot_lock_id = _deep_get(lock_result, "slotLock.slotLockId")
        if slot_lock_id is None:
            return RestaurantProviderOutcome(
                ok=False,
                retryable=False,
                error_code="reservation_create_slot_lock_missing",
                error_message_safe="OpenTable reservation.create slot lock ID missing from response.",
            )

        make_reservation_payload: dict[str, Any] = {
            "additionalServiceFees": metadata.get("additional_service_fees")
            if isinstance(metadata.get("additional_service_fees"), list)
            else [],
            "country": str(metadata.get("country") or "US"),
            "diningAreaId": dining_area_id,
            "email": str(metadata["email"]),
            "firstName": str(metadata["first_name"]),
            "isModify": bool(metadata.get("is_modify", False)),
            "katakanaFirstName": str(metadata.get("katakana_first_name") or ""),
            "katakanaLastName": str(metadata.get("katakana_last_name") or ""),
            "lastName": str(metadata["last_name"]),
            "nonBookableExperiences": metadata.get("non_bookable_experiences")
            if isinstance(metadata.get("non_bookable_experiences"), list)
            else [],
            "partySize": slot.party_size,
            "points": _coerce_int(metadata.get("points"), 0),
            "pointsType": str(metadata.get("points_type") or "Standard"),
            "reservationAttribute": str(metadata.get("reservation_attribute") or "default"),
            "reservationDateTime": reservation_datetime_local,
            "reservationType": reservation_type_display,
            "restaurantId": _as_int_or_original(slot.restaurant_id),
            "slotAvailabilityToken": slot.availability_token,
            "slotHash": slot.provider_slot_id,
            "slotLockId": slot_lock_id,
            "tipAmount": _coerce_int(metadata.get("tip_amount"), 0),
            "tipPercent": _coerce_int(metadata.get("tip_percent"), 0),
            "phoneNumber": str(metadata["phone_number"]),
            "phoneNumberCountryId": str(metadata.get("phone_number_country_id") or "US"),
            "confirmPoints": confirm_points,
            "optInEmailRestaurant": marketing_opt_in_restaurant,
        }
        attribution_token = metadata.get("attribution_token")
        if isinstance(attribution_token, str) and attribution_token.strip():
            make_reservation_payload["attributionToken"] = attribution_token
        correlation_id = metadata.get("correlation_id")
        if isinstance(correlation_id, str) and correlation_id.strip():
            make_reservation_payload["correlationId"] = correlation_id

        create_response = await self._request(
            method="POST",
            path=self._create_path,
            headers=self._headers(extra={"content-type": "application/json"}),
            json_body=make_reservation_payload,
        )
        create_payload = _safe_json_loads(create_response.text)
        if create_response.status_code >= 400:
            error_code, retryable = _normalize_status_error("reservation_create", create_response.status_code)
            return RestaurantProviderOutcome(
                ok=False,
                retryable=retryable,
                error_code=error_code,
                error_message_safe="OpenTable reservation.create make-reservation request failed.",
                data={"status_code": create_response.status_code},
            )
        if create_payload.get("success") is False:
            return RestaurantProviderOutcome(
                ok=False,
                retryable=False,
                error_code="reservation_create_failed",
                error_message_safe="OpenTable reservation.create response did not return success.",
            )

        confirmation_number = _find_first(create_payload, ("confirmationNumber", "confirmation_number", "confNumber"))
        reservation_id = _find_first(create_payload, ("reservationId", "reservation_id"))
        security_token = _find_first(create_payload, ("securityToken", "security_token"))
        confirmation_enrichment_attempted = False
        confirmation_enrichment: dict[str, Any] | None = None
        confirmation_enrichment_error_code: str | None = None
        if self._confirmation_operation_sha256 and confirmation_number is not None and security_token:
            confirmation_enrichment_attempted = True
            try:
                confirmation_response = await self._gql_request(
                    operation_type="query",
                    operation_name=self._confirmation_operation_name,
                    sha256_hash=self._confirmation_operation_sha256,
                    variables={
                        "rid": _as_int_or_original(slot.restaurant_id),
                        "confirmationNumber": _as_int_or_original(str(confirmation_number)),
                        "databaseRegion": database_region,
                        "securityToken": str(security_token),
                        "isLoggedIn": True,
                    },
                    page_type="network_confirmation",
                    page_group="booking",
                    query_timeout_ms=5000,
                )
                confirmation_payload = _safe_json_loads(confirmation_response.text)
                if confirmation_response.status_code >= 400 or confirmation_payload.get("errors"):
                    confirmation_enrichment_error_code = "reservation_create_confirmation_failed"
                else:
                    confirmation_root = _deep_get(confirmation_payload, "data.bookingConfirmationPageInFlow")
                    if not isinstance(confirmation_root, dict):
                        confirmation_root = _deep_get(confirmation_payload, "data.bookingConfirmation")
                    if isinstance(confirmation_root, dict):
                        reservation_node = confirmation_root.get("reservation")
                        if not isinstance(reservation_node, dict):
                            reservation_node = confirmation_root
                        restaurant_node = confirmation_root.get("restaurant")
                        if not isinstance(restaurant_node, dict):
                            restaurant_node = {}

                        enrichment_data = {
                            "restaurant_id": str(
                                _find_first(
                                    restaurant_node,
                                    ("id", "restaurantId", "rid"),
                                )
                                or slot.restaurant_id
                            ),
                            "restaurant_name": _find_first(restaurant_node, ("name", "restaurantName")),
                            "confirmation_number": str(
                                _find_first(
                                    reservation_node,
                                    ("confirmationNumber", "confirmation_number"),
                                )
                                or confirmation_number
                            ),
                            "reservation_id": str(
                                _find_first(
                                    reservation_node,
                                    ("reservationId", "reservation_id"),
                                )
                                or reservation_id
                            )
                            if reservation_id is not None
                            else None,
                            "reservation_state": _find_first(
                                reservation_node,
                                ("reservationState", "state", "status"),
                            ),
                        }
                        confirmation_enrichment = {
                            key: value for key, value in enrichment_data.items() if value is not None
                        } or None
            except Exception:
                confirmation_enrichment_error_code = "reservation_create_confirmation_failed"
        policy_safe = {
            "confirm_points": confirm_points,
            "marketing_opt_in_restaurant": marketing_opt_in_restaurant,
            "restaurant_policy_acknowledged": restaurant_policy_acknowledged,
        }

        return RestaurantProviderOutcome(
            ok=True,
            retryable=False,
            data={
                "confirmation_number": str(confirmation_number) if confirmation_number is not None else None,
                "reservation_id": str(reservation_id) if reservation_id is not None else None,
                "security_token_present": bool(security_token),
                "slot_lock_id": str(slot_lock_id),
                "confirmation_enrichment_attempted": confirmation_enrichment_attempted,
                "confirmation_enrichment": confirmation_enrichment,
                "confirmation_enrichment_error_code": confirmation_enrichment_error_code,
                "policy_safe": policy_safe,
            },
        )

    async def cancel_reservation(
        self,
        *,
        account_ref: str,
        restaurant_id: str,
        confirmation_number: str,
        security_token: str | None = None,
        metadata: dict[str, Any] | None = None,
    ) -> RestaurantProviderOutcome:
        _ = account_ref
        metadata = metadata or {}

        variables = {
            "input": {
                "restaurantId": int(restaurant_id) if restaurant_id.isdigit() else restaurant_id,
                "confirmationNumber": (
                    int(confirmation_number) if confirmation_number.isdigit() else confirmation_number
                ),
                "securityToken": security_token,
                "databaseRegion": str(metadata.get("database_region") or "NA"),
                "reservationSource": str(metadata.get("reservation_source") or "Online"),
            }
        }
        response = await self._gql_request(
            operation_type="mutation",
            operation_name=_CANCEL_RESERVATION_OPERATION,
            sha256_hash=_CANCEL_RESERVATION_SHA256,
            variables=variables,
            csrf_token=str(metadata.get("csrf_token") or "") or None,
            page_type=str(metadata.get("page_type") or "network_confirmation"),
            page_group=str(metadata.get("page_group") or "booking"),
            query_timeout_ms=int(metadata.get("query_timeout_ms") or 5000),
        )
        payload = _safe_json_loads(response.text)
        if response.status_code >= 400:
            error_code, retryable = _normalize_status_error("reservation_cancel", response.status_code)
            return RestaurantProviderOutcome(
                ok=False,
                retryable=retryable,
                error_code=error_code,
                error_message_safe="OpenTable reservation cancel mutation failed.",
                data={"status_code": response.status_code},
            )
        if payload.get("errors"):
            return RestaurantProviderOutcome(
                ok=False,
                retryable=False,
                error_code="reservation_cancel_graphql_error",
                error_message_safe="OpenTable reservation cancel returned GraphQL errors.",
            )

        cancel_payload = _deep_get(payload, "data.cancelReservation")
        status_code = _find_first(cancel_payload, ("statusCode",))
        if isinstance(status_code, int) and status_code >= 400:
            return RestaurantProviderOutcome(
                ok=False,
                retryable=False,
                error_code="reservation_cancel_failed",
                error_message_safe="OpenTable reservation cancel status was not successful.",
                data={"status_code": status_code},
            )

        cancelled_data = _deep_get(cancel_payload, "data")
        return RestaurantProviderOutcome(
            ok=True,
            retryable=False,
            data={
                "status_code": status_code,
                "reservation_state": _find_first(cancelled_data, ("reservationState", "state")),
                "reservation_id": _find_first(cancelled_data, ("reservationId", "reservation_id")),
                "confirmation_number": _find_first(
                    cancelled_data,
                    ("confirmationNumber", "confirmation_number"),
                ),
            },
        )
