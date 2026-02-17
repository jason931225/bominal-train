from __future__ import annotations

import json
from datetime import datetime
from typing import Any

from app.modules.restaurant.providers.base import RestaurantProviderOutcome, RestaurantSearchSlot
from app.modules.restaurant.providers.constants import RESTAURANT_PROVIDER_RESY
from app.modules.restaurant.providers.scaffold import ScaffoldRestaurantProviderClient
from app.modules.train.providers.transport import AsyncTransport, HttpxTransport

_DEFAULT_RESY_BASE_URL = "https://api.resy.com"
_DEFAULT_TIMEOUT_SECONDS = 20.0
_DEFAULT_AUTH_PASSWORD_PATH = "/4/auth/password"
_DEFAULT_X_ORIGIN = "https://resy.com"
_DEFAULT_PROFILE_PATH = "/2/user"
_DEFAULT_SEARCH_PATH = "/4/find"
_DEFAULT_CREATE_DETAILS_PATH = "/3/details"
_DEFAULT_CREATE_BOOK_PATH = "/3/book"
_DEFAULT_CANCEL_PATH = "/3/cancel"
_DEFAULT_SOURCE_ID = "resy.com-venue-details"
_DEFAULT_REFRESH_PATH = "/3/auth/refresh"
_DEFAULT_LOGOUT_PATH = "/3/auth/logout"


def _safe_json_loads(raw: str) -> dict[str, Any]:
    try:
        loaded = json.loads(raw)
    except Exception:
        return {}
    return loaded if isinstance(loaded, dict) else {}


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


def _deep_get(data: Any, dotted_path: str) -> Any:
    node = data
    for part in dotted_path.split("."):
        if isinstance(node, dict) and part in node:
            node = node[part]
            continue
        return None
    return node


def _normalize_status_error(prefix: str, status_code: int) -> tuple[str, bool]:
    if status_code in {401, 403}:
        return f"{prefix}_auth_required", False
    if status_code == 429:
        return f"{prefix}_rate_limited", True
    if status_code >= 500:
        return f"{prefix}_provider_unavailable", True
    return f"{prefix}_failed", False


def _parse_datetime(value: str | None, fallback: datetime) -> datetime:
    if not value:
        return fallback
    normalized = value.replace("T", " ").replace("Z", "")
    for fmt in ("%Y-%m-%d %H:%M:%S", "%Y-%m-%d %H:%M", "%Y-%m-%d"):
        try:
            parsed = datetime.strptime(normalized, fmt)
        except ValueError:
            continue
        if fmt == "%Y-%m-%d":
            return datetime.combine(parsed.date(), fallback.time())
        return parsed
    return fallback


def _as_bool(raw: Any) -> bool:
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
    return bool(raw)


def _as_int(raw: Any) -> int | None:
    if isinstance(raw, bool):
        return None
    if isinstance(raw, int):
        return raw
    if isinstance(raw, str):
        try:
            return int(raw.strip())
        except Exception:
            return None
    return None


def _extract_book_token(payload: dict[str, Any]) -> str | None:
    candidate = _find_first(payload, ("book_token", "bookToken"))
    if isinstance(candidate, str) and candidate.strip():
        return candidate.strip()
    if isinstance(candidate, dict):
        nested = _find_first(candidate, ("value", "token"))
        if isinstance(nested, str) and nested.strip():
            return nested.strip()
    fallback = _find_first(payload, ("book_token_value", "bookTokenValue"))
    if isinstance(fallback, str) and fallback.strip():
        return fallback.strip()
    return None


class ResyProviderClient(ScaffoldRestaurantProviderClient):
    provider_name = RESTAURANT_PROVIDER_RESY

    def __init__(
        self,
        *,
        transport: AsyncTransport | None = None,
        base_url: str = _DEFAULT_RESY_BASE_URL,
        timeout_seconds: float = _DEFAULT_TIMEOUT_SECONDS,
        auth_password_path: str = _DEFAULT_AUTH_PASSWORD_PATH,
        auth_api_key: str | None = None,
        x_origin: str = _DEFAULT_X_ORIGIN,
        profile_path: str = _DEFAULT_PROFILE_PATH,
        search_path: str = _DEFAULT_SEARCH_PATH,
        create_details_path: str = _DEFAULT_CREATE_DETAILS_PATH,
        create_book_path: str = _DEFAULT_CREATE_BOOK_PATH,
        cancel_path: str = _DEFAULT_CANCEL_PATH,
        source_id: str = _DEFAULT_SOURCE_ID,
        refresh_path: str = _DEFAULT_REFRESH_PATH,
        logout_path: str = _DEFAULT_LOGOUT_PATH,
    ) -> None:
        self._transport = transport or HttpxTransport()
        self._base_url = base_url.rstrip("/")
        self._timeout_seconds = timeout_seconds
        self._auth_password_path = auth_password_path or _DEFAULT_AUTH_PASSWORD_PATH
        self._auth_api_key = (auth_api_key or "").strip()
        self._x_origin = x_origin or _DEFAULT_X_ORIGIN
        self._profile_path = profile_path or _DEFAULT_PROFILE_PATH
        self._search_path = search_path or _DEFAULT_SEARCH_PATH
        self._create_details_path = create_details_path or _DEFAULT_CREATE_DETAILS_PATH
        self._create_book_path = create_book_path or _DEFAULT_CREATE_BOOK_PATH
        self._cancel_path = cancel_path or _DEFAULT_CANCEL_PATH
        self._source_id = source_id or _DEFAULT_SOURCE_ID
        self._refresh_path = refresh_path.strip()
        self._logout_path = logout_path.strip()

    async def _request(
        self,
        *,
        method: str,
        path: str,
        headers: dict[str, str] | None = None,
        data: dict[str, Any] | None = None,
        params: dict[str, Any] | None = None,
    ):
        return await self._transport.request(
            method=method,
            url=f"{self._base_url}{path}",
            headers=headers,
            data=data,
            params=params,
            timeout=self._timeout_seconds,
        )

    def _auth_headers(
        self,
        *,
        account_ref: str,
        operation_prefix: str,
        form_encoded: bool = False,
        extra: dict[str, str] | None = None,
    ) -> tuple[dict[str, str] | None, RestaurantProviderOutcome | None]:
        if not self._auth_api_key:
            return None, RestaurantProviderOutcome(
                ok=False,
                retryable=False,
                error_code=f"{operation_prefix}_api_key_unconfigured",
                error_message_safe=f"Resy {operation_prefix} API key is not configured.",
            )
        auth_token = account_ref.strip()
        if not auth_token:
            return None, RestaurantProviderOutcome(
                ok=False,
                retryable=False,
                error_code=f"{operation_prefix}_account_ref_missing",
                error_message_safe=f"Resy {operation_prefix} requires account_ref.",
            )
        headers = {
            "Accept": "application/json, text/plain, */*",
            "Authorization": f'ResyAPI api_key="{self._auth_api_key}"',
            "X-Resy-Auth-Token": auth_token,
            "X-Resy-Universal-Auth": auth_token,
            "X-Origin": self._x_origin,
        }
        if form_encoded:
            headers["Content-Type"] = "application/x-www-form-urlencoded"
        if extra:
            headers.update(extra)
        return headers, None

    async def authenticate_start(
        self,
        *,
        account_identifier: str,
        password: str | None = None,
        delivery_channel: str = "email",
    ) -> RestaurantProviderOutcome:
        _ = delivery_channel
        if not password:
            return RestaurantProviderOutcome(
                ok=False,
                retryable=False,
                error_code="auth_start_password_required",
                error_message_safe="Resy auth.start requires password.",
            )
        if not self._auth_api_key:
            return RestaurantProviderOutcome(
                ok=False,
                retryable=False,
                error_code="auth_start_api_key_unconfigured",
                error_message_safe="Resy auth.start API key is not configured.",
            )

        headers = {
            "Accept": "application/json, text/plain, */*",
            "Content-Type": "application/x-www-form-urlencoded",
            "Authorization": f'ResyAPI api_key="{self._auth_api_key}"',
            "Cache-Control": "no-cache",
            "X-Origin": self._x_origin,
        }
        response = await self._request(
            method="POST",
            path=self._auth_password_path,
            headers=headers,
            data={
                "email": account_identifier,
                "password": password,
            },
        )
        payload = _safe_json_loads(response.text)
        if response.status_code >= 400:
            error_code, retryable = _normalize_status_error("auth_start", response.status_code)
            return RestaurantProviderOutcome(
                ok=False,
                retryable=retryable,
                error_code=error_code,
                error_message_safe="Resy auth.start request failed.",
                data={"status_code": response.status_code},
            )
        if payload.get("errors") or payload.get("success") is False:
            provider_error_code = _find_first(payload, ("errorCode", "error_code", "code"))
            error_data: dict[str, Any] = {}
            if provider_error_code is not None:
                error_data["provider_error_code"] = str(provider_error_code)
            return RestaurantProviderOutcome(
                ok=False,
                retryable=False,
                error_code="auth_start_failed",
                error_message_safe="Resy auth.start response indicates failure.",
                data=error_data,
            )

        provider_account_ref = _find_first(
            payload,
            ("id", "userId", "user_id", "gpid", "resyId", "resy_id", "memberId", "member_id", "uid"),
        )
        provider_account_ref_str = str(provider_account_ref) if provider_account_ref is not None else None
        challenge_token = json.dumps(
            {
                "password_flow_complete": True,
                "provider_account_ref": provider_account_ref_str,
            },
            separators=(",", ":"),
        )
        return RestaurantProviderOutcome(
            ok=True,
            retryable=False,
            data={
                "requires_otp": False,
                "password_flow_complete": True,
                "challenge_token": challenge_token,
                "provider_account_ref": provider_account_ref_str,
            },
        )

    async def authenticate_complete(
        self,
        *,
        account_identifier: str,
        challenge_token: str | None = None,
        otp_code: str | None = None,
    ) -> RestaurantProviderOutcome:
        _ = (account_identifier, otp_code)
        payload = _safe_json_loads(challenge_token or "")
        if not bool(payload.get("password_flow_complete")):
            return RestaurantProviderOutcome(
                ok=False,
                retryable=False,
                error_code="auth_complete_challenge_missing",
                error_message_safe="Resy auth.complete requires password-flow challenge token.",
            )
        provider_account_ref = payload.get("provider_account_ref")
        return RestaurantProviderOutcome(
            ok=True,
            retryable=False,
            data={
                "authenticated": True,
                "provider_account_ref": str(provider_account_ref) if provider_account_ref else None,
            },
        )

    async def refresh_auth(self, *, account_ref: str) -> RestaurantProviderOutcome:
        if not self._refresh_path:
            return RestaurantProviderOutcome(
                ok=False,
                retryable=False,
                error_code="auth_refresh_path_unconfigured",
                error_message_safe="Resy auth.refresh path is not configured.",
            )
        headers, error_outcome = self._auth_headers(account_ref=account_ref, operation_prefix="auth_refresh")
        if error_outcome is not None:
            return error_outcome

        response = await self._request(
            method="POST",
            path=self._refresh_path,
            headers=headers,
        )
        payload = _safe_json_loads(response.text)
        if response.status_code >= 400:
            error_code, retryable = _normalize_status_error("auth_refresh", response.status_code)
            return RestaurantProviderOutcome(
                ok=False,
                retryable=retryable,
                error_code=error_code,
                error_message_safe="Resy auth.refresh request failed.",
                data={"status_code": response.status_code},
            )
        if payload.get("errors") or payload.get("success") is False:
            provider_error_code = _find_first(payload, ("errorCode", "error_code", "code"))
            data: dict[str, Any] = {}
            if provider_error_code is not None:
                data["provider_error_code"] = str(provider_error_code)
            return RestaurantProviderOutcome(
                ok=False,
                retryable=False,
                error_code="auth_refresh_failed",
                error_message_safe="Resy auth.refresh response indicates failure.",
                data=data,
            )

        refreshed = _as_bool(_find_first(payload, ("refreshed", "success", "ok")))
        expires_in = _as_int(_find_first(payload, ("expires_in", "expiresIn", "eik")))
        token_value = _find_first(payload, ("token", "auth_token", "access_token", "x_resy_auth_token"))
        return RestaurantProviderOutcome(
            ok=True,
            retryable=False,
            data={
                "refreshed": refreshed,
                "expires_in": expires_in,
                "token_present": bool(token_value),
            },
        )

    async def logout(self, *, account_ref: str) -> RestaurantProviderOutcome:
        if not self._logout_path:
            return RestaurantProviderOutcome(
                ok=False,
                retryable=False,
                error_code="logout_path_unconfigured",
                error_message_safe="Resy logout path is not configured.",
            )
        headers, error_outcome = self._auth_headers(account_ref=account_ref, operation_prefix="logout")
        if error_outcome is not None:
            return error_outcome

        response = await self._request(
            method="POST",
            path=self._logout_path,
            headers=headers,
        )
        payload = _safe_json_loads(response.text)
        if response.status_code >= 400:
            error_code, retryable = _normalize_status_error("logout", response.status_code)
            return RestaurantProviderOutcome(
                ok=False,
                retryable=retryable,
                error_code=error_code,
                error_message_safe="Resy logout request failed.",
                data={"status_code": response.status_code},
            )
        if payload.get("errors") or payload.get("success") is False:
            provider_error_code = _find_first(payload, ("errorCode", "error_code", "code"))
            data: dict[str, Any] = {}
            if provider_error_code is not None:
                data["provider_error_code"] = str(provider_error_code)
            return RestaurantProviderOutcome(
                ok=False,
                retryable=False,
                error_code="logout_failed",
                error_message_safe="Resy logout response indicates failure.",
                data=data,
            )

        status_value = _find_first(payload, ("status", "message", "result"))
        return RestaurantProviderOutcome(
            ok=True,
            retryable=False,
            data={
                "logged_out": True,
                "status": str(status_value) if status_value is not None else "ok",
            },
        )

    async def get_user_profile(self, *, account_ref: str) -> RestaurantProviderOutcome:
        headers, error_outcome = self._auth_headers(account_ref=account_ref, operation_prefix="profile_get")
        if error_outcome is not None:
            return error_outcome

        response = await self._request(
            method="GET",
            path=self._profile_path,
            headers=headers,
        )
        payload = _safe_json_loads(response.text)
        if response.status_code >= 400:
            error_code, retryable = _normalize_status_error("profile_get", response.status_code)
            return RestaurantProviderOutcome(
                ok=False,
                retryable=retryable,
                error_code=error_code,
                error_message_safe="Resy profile query failed.",
                data={"status_code": response.status_code},
            )
        if payload.get("errors") or payload.get("success") is False:
            provider_error_code = _find_first(payload, ("errorCode", "error_code", "code"))
            data: dict[str, Any] = {}
            if provider_error_code is not None:
                data["provider_error_code"] = str(provider_error_code)
            return RestaurantProviderOutcome(
                ok=False,
                retryable=False,
                error_code="profile_get_failed",
                error_message_safe="Resy profile response indicates failure.",
                data=data,
            )

        reservations = payload.get("reservations")
        payment_methods = payload.get("payment_methods")
        user_id = _find_first(payload, ("id", "userId", "user_id"))
        return RestaurantProviderOutcome(
            ok=True,
            retryable=False,
            data={
                "provider_account_ref": str(user_id) if user_id is not None else None,
                "user_id": user_id,
                "email": _find_first(payload, ("email",)),
                "first_name": _find_first(payload, ("first_name", "firstName")),
                "last_name": _find_first(payload, ("last_name", "lastName")),
                "reservation_count": len(reservations) if isinstance(reservations, list) else 0,
                "payment_method_count": len(payment_methods) if isinstance(payment_methods, list) else 0,
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
        headers, error_outcome = self._auth_headers(account_ref=account_ref, operation_prefix="search")
        if error_outcome is not None:
            return error_outcome

        metadata = metadata or {}
        params: dict[str, Any] = {
            "venue_id": restaurant_id,
            "day": date_time_local.strftime("%Y-%m-%d"),
            "party_size": party_size,
        }
        lat = metadata.get("lat")
        long = metadata.get("long")
        if isinstance(lat, (int, float)) and isinstance(long, (int, float)):
            params["lat"] = float(lat)
            params["long"] = float(long)

        response = await self._request(
            method="GET",
            path=self._search_path,
            headers=headers,
            params=params,
        )
        payload = _safe_json_loads(response.text)
        if response.status_code >= 400:
            error_code, retryable = _normalize_status_error("search", response.status_code)
            return RestaurantProviderOutcome(
                ok=False,
                retryable=retryable,
                error_code=error_code,
                error_message_safe="Resy availability search failed.",
                data={"status_code": response.status_code},
            )
        if payload.get("errors") or payload.get("success") is False:
            return RestaurantProviderOutcome(
                ok=False,
                retryable=False,
                error_code="search_failed",
                error_message_safe="Resy availability response indicates failure.",
            )

        venues = _deep_get(payload, "results.venues")
        slots: list[RestaurantSearchSlot] = []
        if isinstance(venues, list):
            for venue_entry in venues:
                if not isinstance(venue_entry, dict):
                    continue
                venue_name = _deep_get(venue_entry, "venue.name")
                venue_id = _deep_get(venue_entry, "venue.id.resy")
                if venue_id is None:
                    venue_id = _deep_get(venue_entry, "venue.id")
                resolved_restaurant_id = str(venue_id) if venue_id is not None else restaurant_id
                raw_slots = venue_entry.get("slots")
                if not isinstance(raw_slots, list):
                    continue
                for raw_slot in raw_slots:
                    if not isinstance(raw_slot, dict):
                        continue
                    token = _deep_get(raw_slot, "config.token")
                    if not isinstance(token, str) or not token.strip():
                        continue
                    start_time = _deep_get(raw_slot, "date.start")
                    slot_type = _deep_get(raw_slot, "config.type")
                    parsed_dt = _parse_datetime(str(start_time) if isinstance(start_time, str) else None, date_time_local)
                    metadata_safe: dict[str, Any] = {
                        "slot_type": str(slot_type) if isinstance(slot_type, str) else "Standard",
                    }
                    if isinstance(venue_name, str) and venue_name.strip():
                        metadata_safe["venue_name"] = venue_name
                    if isinstance(start_time, str) and start_time.strip():
                        metadata_safe["raw_start"] = start_time
                    slots.append(
                        RestaurantSearchSlot(
                            provider_slot_id=token.strip(),
                            provider=RESTAURANT_PROVIDER_RESY,
                            restaurant_id=resolved_restaurant_id,
                            party_size=party_size,
                            date_time_local=parsed_dt,
                            availability_token=token.strip(),
                            metadata_safe=metadata_safe,
                        )
                    )

        return RestaurantProviderOutcome(
            ok=True,
            retryable=False,
            data={"slots": slots, "slot_count": len(slots)},
        )

    async def create_reservation(
        self,
        *,
        account_ref: str,
        slot: RestaurantSearchSlot,
        metadata: dict[str, Any] | None = None,
    ) -> RestaurantProviderOutcome:
        headers, error_outcome = self._auth_headers(
            account_ref=account_ref,
            operation_prefix="reservation_create",
            form_encoded=True,
        )
        if error_outcome is not None:
            return error_outcome
        metadata = metadata or {}
        config_id = slot.availability_token or slot.provider_slot_id
        if not config_id:
            return RestaurantProviderOutcome(
                ok=False,
                retryable=False,
                error_code="reservation_create_slot_token_missing",
                error_message_safe="Resy reservation.create requires slot token.",
            )

        details_data: dict[str, Any] = {
            "commit": 1,
            "config_id": config_id,
            "day": slot.date_time_local.strftime("%Y-%m-%d"),
            "party_size": slot.party_size,
        }
        notes = metadata.get("notes")
        if isinstance(notes, str) and notes.strip():
            details_data["notes"] = notes.strip()

        details_response = await self._request(
            method="POST",
            path=self._create_details_path,
            headers=headers,
            data=details_data,
        )
        details_payload = _safe_json_loads(details_response.text)
        if details_response.status_code >= 400:
            error_code, retryable = _normalize_status_error("reservation_create", details_response.status_code)
            return RestaurantProviderOutcome(
                ok=False,
                retryable=retryable,
                error_code=error_code,
                error_message_safe="Resy reservation.create details request failed.",
                data={"status_code": details_response.status_code},
            )
        if details_payload.get("errors") or details_payload.get("success") is False:
            return RestaurantProviderOutcome(
                ok=False,
                retryable=False,
                error_code="reservation_create_details_failed",
                error_message_safe="Resy reservation.create details response indicates failure.",
            )

        book_token = _extract_book_token(details_payload)
        if not book_token:
            return RestaurantProviderOutcome(
                ok=False,
                retryable=False,
                error_code="reservation_create_book_token_missing",
                error_message_safe="Resy reservation.create details response missing book token.",
            )

        source_id = str(metadata.get("source_id") or self._source_id)
        book_headers = dict(headers)
        idempotency_key = metadata.get("idempotency_key")
        if isinstance(idempotency_key, str) and idempotency_key.strip():
            book_headers["Idempotency-Key"] = idempotency_key.strip()

        book_data: dict[str, Any] = {
            "book_token": book_token,
            "source_id": source_id,
        }
        struct_payment_method = metadata.get("struct_payment_method")
        if struct_payment_method is not None:
            if isinstance(struct_payment_method, str) and struct_payment_method.strip():
                book_data["struct_payment_method"] = struct_payment_method.strip()
            elif isinstance(struct_payment_method, dict):
                book_data["struct_payment_method"] = json.dumps(struct_payment_method, separators=(",", ":"))
        else:
            payment_method_id = metadata.get("payment_method_id")
            if payment_method_id is not None:
                book_data["struct_payment_method"] = json.dumps(
                    {"id": str(payment_method_id)},
                    separators=(",", ":"),
                )
        if "venue_marketing_opt_in" in metadata:
            book_data["venue_marketing_opt_in"] = 1 if _as_bool(metadata.get("venue_marketing_opt_in")) else 0

        book_response = await self._request(
            method="POST",
            path=self._create_book_path,
            headers=book_headers,
            data=book_data,
        )
        book_payload = _safe_json_loads(book_response.text)
        if book_response.status_code >= 400:
            error_code, retryable = _normalize_status_error("reservation_create", book_response.status_code)
            return RestaurantProviderOutcome(
                ok=False,
                retryable=retryable,
                error_code=error_code,
                error_message_safe="Resy reservation.create book request failed.",
                data={"status_code": book_response.status_code},
            )
        if book_payload.get("errors") or book_payload.get("success") is False:
            return RestaurantProviderOutcome(
                ok=False,
                retryable=False,
                error_code="reservation_create_failed",
                error_message_safe="Resy reservation.create response indicates failure.",
            )

        reservation_id = _find_first(book_payload, ("reservation_id", "reservationId", "id"))
        resy_token = _find_first(book_payload, ("resy_token", "resyToken", "token"))
        confirmation_number = _find_first(book_payload, ("confirmation_number", "confirmationNumber"))
        if reservation_id is None and resy_token is None:
            return RestaurantProviderOutcome(
                ok=False,
                retryable=False,
                error_code="reservation_create_missing_reference",
                error_message_safe="Resy reservation.create response missing reservation reference.",
            )

        return RestaurantProviderOutcome(
            ok=True,
            retryable=False,
            data={
                "reservation_id": str(reservation_id) if reservation_id is not None else None,
                "confirmation_number": str(confirmation_number) if confirmation_number is not None else None,
                "resy_token_present": bool(resy_token),
                "payment_required": bool(_deep_get(details_payload, "payment.required")),
                "source_id": source_id,
                "display_date": _find_first(book_payload, ("display_date", "displayDate")),
                "display_time": _find_first(book_payload, ("display_time", "displayTime")),
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
        _ = (restaurant_id, metadata)
        headers, error_outcome = self._auth_headers(
            account_ref=account_ref,
            operation_prefix="reservation_cancel",
            form_encoded=True,
        )
        if error_outcome is not None:
            return error_outcome

        used_fallback = False
        cancel_response = await self._request(
            method="POST",
            path=self._cancel_path,
            headers=headers,
            data={"reservation_id": confirmation_number},
        )
        if cancel_response.status_code >= 400 and security_token:
            used_fallback = True
            cancel_response = await self._request(
                method="POST",
                path=self._cancel_path,
                headers=headers,
                data={"reservation_id": confirmation_number, "resy_token": security_token},
            )
        if cancel_response.status_code >= 400:
            error_code, retryable = _normalize_status_error("reservation_cancel", cancel_response.status_code)
            return RestaurantProviderOutcome(
                ok=False,
                retryable=retryable,
                error_code=error_code,
                error_message_safe="Resy reservation.cancel request failed.",
                data={
                    "status_code": cancel_response.status_code,
                    "used_resy_token_fallback": used_fallback,
                },
            )

        payload = _safe_json_loads(cancel_response.text)
        if payload.get("errors") or payload.get("success") is False:
            return RestaurantProviderOutcome(
                ok=False,
                retryable=False,
                error_code="reservation_cancel_failed",
                error_message_safe="Resy reservation.cancel response indicates failure.",
                data={"used_resy_token_fallback": used_fallback},
            )
        status_value = _find_first(payload, ("status", "reservationState", "reservation_state"))
        return RestaurantProviderOutcome(
            ok=True,
            retryable=False,
            data={
                "reservation_id": confirmation_number,
                "status": str(status_value) if status_value is not None else "unknown",
                "used_resy_token_fallback": used_fallback,
            },
        )
