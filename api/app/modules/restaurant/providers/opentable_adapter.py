from __future__ import annotations

import json
from datetime import datetime
from typing import Any
from urllib.parse import urlencode

from app.modules.restaurant.providers.base import RestaurantProviderOutcome, RestaurantSearchSlot
from app.modules.restaurant.providers.constants import RESTAURANT_PROVIDER_OPENTABLE
from app.modules.train.providers.transport import AsyncTransport, HttpxTransport

_DEFAULT_OPENTABLE_BASE_URL = "https://www.opentable.com"
_DEFAULT_TIMEOUT_SECONDS = 20.0

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


def _normalize_status_error(prefix: str, status_code: int) -> tuple[str, bool]:
    if status_code in {401, 403}:
        return f"{prefix}_auth_required", False
    if status_code == 429:
        return f"{prefix}_rate_limited", True
    if status_code >= 500:
        return f"{prefix}_provider_unavailable", True
    return f"{prefix}_failed", False


class OpenTableProviderClient:
    provider_name = RESTAURANT_PROVIDER_OPENTABLE

    def __init__(
        self,
        *,
        transport: AsyncTransport | None = None,
        base_url: str = _DEFAULT_OPENTABLE_BASE_URL,
        timeout_seconds: float = _DEFAULT_TIMEOUT_SECONDS,
        auth_start_path: str | None = None,
        auth_complete_path: str | None = None,
    ) -> None:
        self._transport = transport or HttpxTransport()
        self._base_url = base_url.rstrip("/")
        self._timeout_seconds = timeout_seconds
        self._auth_start_path = auth_start_path
        self._auth_complete_path = auth_complete_path

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
        if not self._auth_start_path:
            return RestaurantProviderOutcome(
                ok=False,
                retryable=False,
                error_code="auth_start_endpoint_unconfigured",
                error_message_safe="OpenTable auth.start endpoint is not configured.",
            )

        body = {
            "accountIdentifier": account_identifier,
            "deliveryChannel": delivery_channel,
        }
        if password:
            body["password"] = password

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

        challenge_token = _find_first(
            payload,
            ("challengeToken", "challenge_token", "verificationId", "verification_id", "requestId", "request_id"),
        )
        return RestaurantProviderOutcome(
            ok=True,
            retryable=False,
            data={
                "challenge_token": str(challenge_token) if challenge_token else None,
                "requires_otp": bool(challenge_token),
            },
        )

    async def authenticate_complete(
        self,
        *,
        account_identifier: str,
        challenge_token: str | None = None,
        otp_code: str | None = None,
    ) -> RestaurantProviderOutcome:
        if not self._auth_complete_path:
            return RestaurantProviderOutcome(
                ok=False,
                retryable=False,
                error_code="auth_complete_endpoint_unconfigured",
                error_message_safe="OpenTable auth.complete endpoint is not configured.",
            )

        body: dict[str, Any] = {"accountIdentifier": account_identifier}
        if challenge_token is not None:
            body["challengeToken"] = challenge_token
        if otp_code is not None:
            body["otpCode"] = otp_code

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
        return RestaurantProviderOutcome(
            ok=True,
            retryable=False,
            data={
                "authenticated": True,
                "provider_account_ref": _find_first(payload, ("gpid", "userId", "user_id")),
            },
        )

    async def refresh_auth(self, *, account_ref: str) -> RestaurantProviderOutcome:
        _ = account_ref
        cpr_response = await self._request(method="GET", path="/_sec/cpr/params", headers=self._headers())
        human_response = await self._request(method="GET", path="/dapi/fe/human", headers=self._headers())
        session_response = await self._request(method="POST", path="/dapi/v1/session", headers=self._headers())

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
        operation_name = str(metadata.get("operation_name") or "").strip()
        sha256_hash = str(metadata.get("sha256_hash") or "").strip()
        if not operation_name or not sha256_hash:
            return RestaurantProviderOutcome(
                ok=False,
                retryable=False,
                error_code="search_contract_unconfigured",
                error_message_safe="OpenTable search contract is not configured.",
            )

        variables = metadata.get("variables")
        if not isinstance(variables, dict):
            variables = {
                "restaurantId": int(restaurant_id) if restaurant_id.isdigit() else restaurant_id,
                "partySize": party_size,
                "dateTime": date_time_local.isoformat(),
            }

        response = await self._gql_request(
            operation_type="query",
            operation_name=operation_name,
            sha256_hash=sha256_hash,
            variables=variables,
            page_type=str(metadata.get("page_type") or "home"),
            page_group=str(metadata.get("page_group") or "seo-landing-home"),
            query_timeout_ms=int(metadata.get("query_timeout_ms") or 5000),
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

        slots_node: Any = None
        slot_path = metadata.get("slot_path")
        if isinstance(slot_path, str) and slot_path:
            slots_node = _deep_get(payload, slot_path)
        if not isinstance(slots_node, list):
            slots_node = _find_first(payload, ("availableSlots", "slots", "availabilities"))
        if not isinstance(slots_node, list):
            slots_node = []

        slots: list[RestaurantSearchSlot] = []
        for raw_slot in slots_node:
            if not isinstance(raw_slot, dict):
                continue
            raw_slot_id = _find_first(raw_slot, ("slotHash", "slotId", "id", "availabilityToken"))
            if raw_slot_id is None:
                continue
            raw_dt = _find_first(raw_slot, ("dateTime", "dateTimeLocal", "time"))
            parsed_dt = _parse_datetime(str(raw_dt) if raw_dt else None, fallback=date_time_local)
            availability_token = _find_first(raw_slot, ("availabilityToken", "token"))
            slots.append(
                RestaurantSearchSlot(
                    provider_slot_id=str(raw_slot_id),
                    provider=RESTAURANT_PROVIDER_OPENTABLE,
                    restaurant_id=restaurant_id,
                    party_size=party_size,
                    date_time_local=parsed_dt,
                    availability_token=str(availability_token) if availability_token else None,
                    metadata_safe={
                        "raw_slot_id": str(raw_slot_id),
                    },
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
        operation_name = str(metadata.get("operation_name") or "").strip()
        sha256_hash = str(metadata.get("sha256_hash") or "").strip()
        if not operation_name or not sha256_hash:
            return RestaurantProviderOutcome(
                ok=False,
                retryable=False,
                error_code="reservation_create_contract_unconfigured",
                error_message_safe="OpenTable reservation.create contract is not configured.",
            )

        variables = metadata.get("variables")
        if not isinstance(variables, dict):
            variables = {
                "input": {
                    "restaurantId": int(slot.restaurant_id) if slot.restaurant_id.isdigit() else slot.restaurant_id,
                    "partySize": slot.party_size,
                    "dateTime": slot.date_time_local.isoformat(),
                    "slotHash": slot.provider_slot_id,
                    "availabilityToken": slot.availability_token,
                }
            }

        response = await self._gql_request(
            operation_type="mutation",
            operation_name=operation_name,
            sha256_hash=sha256_hash,
            variables=variables,
            page_type=str(metadata.get("page_type") or "booking"),
            page_group=str(metadata.get("page_group") or "booking"),
            query_timeout_ms=int(metadata.get("query_timeout_ms") or 5000),
        )
        payload = _safe_json_loads(response.text)
        if response.status_code >= 400:
            error_code, retryable = _normalize_status_error("reservation_create", response.status_code)
            return RestaurantProviderOutcome(
                ok=False,
                retryable=retryable,
                error_code=error_code,
                error_message_safe="OpenTable reservation create mutation failed.",
                data={"status_code": response.status_code},
            )
        if payload.get("errors"):
            return RestaurantProviderOutcome(
                ok=False,
                retryable=False,
                error_code="reservation_create_graphql_error",
                error_message_safe="OpenTable reservation create returned GraphQL errors.",
            )

        confirmation_number = _find_first(payload, ("confirmationNumber", "confirmation_number", "confNumber"))
        reservation_id = _find_first(payload, ("reservationId", "reservation_id"))
        security_token = _find_first(payload, ("securityToken", "security_token"))

        return RestaurantProviderOutcome(
            ok=True,
            retryable=False,
            data={
                "confirmation_number": str(confirmation_number) if confirmation_number is not None else None,
                "reservation_id": str(reservation_id) if reservation_id is not None else None,
                "security_token_present": bool(security_token),
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
