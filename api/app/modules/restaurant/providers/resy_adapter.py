from __future__ import annotations

import json
from typing import Any

from app.modules.restaurant.providers.base import RestaurantProviderOutcome
from app.modules.restaurant.providers.constants import RESTAURANT_PROVIDER_RESY
from app.modules.restaurant.providers.scaffold import ScaffoldRestaurantProviderClient
from app.modules.train.providers.transport import AsyncTransport, HttpxTransport

_DEFAULT_RESY_BASE_URL = "https://api.resy.com"
_DEFAULT_TIMEOUT_SECONDS = 20.0
_DEFAULT_AUTH_PASSWORD_PATH = "/4/auth/password"
_DEFAULT_X_ORIGIN = "https://resy.com"


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


def _normalize_status_error(prefix: str, status_code: int) -> tuple[str, bool]:
    if status_code in {401, 403}:
        return f"{prefix}_auth_required", False
    if status_code == 429:
        return f"{prefix}_rate_limited", True
    if status_code >= 500:
        return f"{prefix}_provider_unavailable", True
    return f"{prefix}_failed", False


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
    ) -> None:
        self._transport = transport or HttpxTransport()
        self._base_url = base_url.rstrip("/")
        self._timeout_seconds = timeout_seconds
        self._auth_password_path = auth_password_path or _DEFAULT_AUTH_PASSWORD_PATH
        self._auth_api_key = (auth_api_key or "").strip()
        self._x_origin = x_origin or _DEFAULT_X_ORIGIN

    async def _request(
        self,
        *,
        method: str,
        path: str,
        headers: dict[str, str] | None = None,
        data: dict[str, Any] | None = None,
    ):
        return await self._transport.request(
            method=method,
            url=f"{self._base_url}{path}",
            headers=headers,
            data=data,
            timeout=self._timeout_seconds,
        )

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
