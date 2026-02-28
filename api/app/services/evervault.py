from __future__ import annotations

import asyncio
import time
from dataclasses import dataclass
from typing import Any, Mapping
from urllib.parse import urlparse

import httpx

from app.core.config import get_settings

settings = get_settings()

_PROVIDER_DESTINATION_DOMAIN: dict[str, str] = {
    "KTX": "smart.letskorail.com",
    "SRT": "app.srail.or.kr",
}
_PROVIDER_DECRYPT_FORM_FIELDS: dict[str, tuple[str, ...]] = {
    "KTX": (
        "hidStlCrCrdNo1",
        "hidVanPwd1",
        "hidAthnVal1",
        "hidCrdVlidTrm1",
    ),
    "SRT": (
        "stlCrCrdNo1",
        "vanPwd1",
        "athnVal1",
        "crdVlidTrm1",
    ),
}
_RELAY_DOMAIN_SUFFIX = ".relay.evervault.app"
_DEFAULT_MANAGEMENT_TIMEOUT_SECONDS = 10.0
_RELAY_CACHE_LOCK = asyncio.Lock()
_RELAY_CACHE: dict[str, tuple[float, "RelayRuntimeConfig"]] = {}


@dataclass(slots=True, frozen=True)
class RelayRuntimeConfig:
    relay_id: str
    relay_domain: str


@dataclass(slots=True, frozen=True)
class EvervaultRelayHttpResponse:
    status_code: int
    text: str
    relay_url: str
    relay_id: str
    relay_domain: str


class EvervaultRelayError(RuntimeError):
    def __init__(self, *, retryable: bool, error_code: str, error_message_safe: str) -> None:
        super().__init__(error_message_safe)
        self.retryable = retryable
        self.error_code = error_code
        self.error_message_safe = error_message_safe


def _normalize_provider(provider: str) -> str:
    normalized = str(provider or "").strip().upper()
    if normalized not in _PROVIDER_DESTINATION_DOMAIN:
        raise ValueError(f"Unsupported provider: {provider}")
    return normalized


def _relay_id_override(provider: str) -> str | None:
    if provider == "KTX":
        value = str(settings.evervault_ktx_payment_relay_id or "").strip()
        return value or None
    if provider == "SRT":
        value = str(settings.evervault_srt_payment_relay_id or "").strip()
        return value or None
    return None


def _relay_domain_override(provider: str) -> str | None:
    if provider == "KTX":
        value = str(settings.evervault_ktx_payment_relay_domain or "").strip().lower()
    elif provider == "SRT":
        value = str(settings.evervault_srt_payment_relay_domain or "").strip().lower()
    else:
        return None
    if not value:
        return None
    if value.startswith("http://") or value.startswith("https://"):
        parsed = urlparse(value)
        value = str(parsed.netloc or "").strip().lower()
    if "/" in value:
        value = value.split("/", 1)[0]
    value = value.rstrip(".")
    if not value or not value.endswith(_RELAY_DOMAIN_SUFFIX):
        raise EvervaultRelayError(
            retryable=False,
            error_code="evervault_relay_invalid_domain",
            error_message_safe="Evervault relay domain is invalid",
        )
    return value


def _extract_payment_path(payment_url: str) -> str:
    parsed = urlparse(payment_url)
    path = str(parsed.path or "").strip()
    if not path:
        raise ValueError("payment_url must include a path")
    if not path.startswith("/"):
        path = f"/{path}"
    return path


def _route_for_payment(path: str, selectors: tuple[str, ...]) -> dict[str, Any]:
    return {
        "method": "POST",
        "path": path,
        "request": [
            {
                "action": "decrypt",
                "selections": [{"type": "form", "selector": selector} for selector in selectors],
            }
        ],
        "response": [],
    }


def build_payment_relay_definition(*, provider: str, payment_url: str) -> dict[str, Any]:
    normalized_provider = _normalize_provider(provider)
    path = _extract_payment_path(payment_url)
    return {
        "destinationDomain": _PROVIDER_DESTINATION_DOMAIN[normalized_provider],
        "authentication": "api-key",
        "encryptEmptyStrings": True,
        "routes": [_route_for_payment(path, _PROVIDER_DECRYPT_FORM_FIELDS[normalized_provider])],
    }


def _normalized_route_signature(routes: Any) -> list[dict[str, Any]]:
    signatures: list[dict[str, Any]] = []
    rows = routes if isinstance(routes, list) else []
    for row in rows:
        if not isinstance(row, dict):
            continue
        request_items = row.get("request") if isinstance(row.get("request"), list) else []
        request_action = request_items[0] if request_items and isinstance(request_items[0], dict) else {}
        selections = request_action.get("selections") if isinstance(request_action.get("selections"), list) else []
        signatures.append(
            {
                "method": str(row.get("method") or "").upper(),
                "path": str(row.get("path") or ""),
                "action": str(request_action.get("action") or "").lower(),
                "selections": [
                    {
                        "type": str(selection.get("type") or "").lower(),
                        "selector": str(selection.get("selector") or ""),
                    }
                    for selection in selections
                    if isinstance(selection, dict)
                ],
            }
        )
    return signatures


def _relay_matches_definition(relay: dict[str, Any], expected: dict[str, Any]) -> bool:
    destination = str(relay.get("destinationDomain") or "").strip().lower()
    expected_destination = str(expected.get("destinationDomain") or "").strip().lower()
    if destination != expected_destination:
        return False

    auth_value = relay.get("authentication")
    normalized_auth = str(auth_value).strip().lower() if auth_value is not None else None
    if normalized_auth != "api-key":
        return False

    return _normalized_route_signature(relay.get("routes")) == _normalized_route_signature(expected.get("routes"))


def _runtime_from_relay(relay: dict[str, Any]) -> RelayRuntimeConfig:
    relay_id = str(relay.get("id") or "").strip()
    relay_domain = str(relay.get("evervaultDomain") or "").strip().lower()
    if not relay_id:
        raise EvervaultRelayError(
            retryable=False,
            error_code="evervault_relay_invalid_response",
            error_message_safe="Evervault relay configuration is missing relay id",
        )
    if not relay_domain or not relay_domain.endswith(_RELAY_DOMAIN_SUFFIX):
        raise EvervaultRelayError(
            retryable=False,
            error_code="evervault_relay_invalid_domain",
            error_message_safe="Evervault relay domain is invalid",
        )
    return RelayRuntimeConfig(relay_id=relay_id, relay_domain=relay_domain)


def _runtime_credentials() -> tuple[str, str]:
    app_id = str(settings.evervault_app_id or "").strip()
    api_key = str(settings.evervault_api_key or "").strip()
    if not app_id or not api_key:
        raise EvervaultRelayError(
            retryable=False,
            error_code="evervault_config_missing",
            error_message_safe="Evervault runtime credentials are not configured",
        )
    return app_id, api_key


async def _management_request(
    *,
    method: str,
    path: str,
    json_body: dict[str, Any] | None = None,
) -> dict[str, Any]:
    app_id, api_key = _runtime_credentials()
    base_url = str(settings.evervault_api_base_url or "").strip().rstrip("/")
    if not base_url:
        raise EvervaultRelayError(
            retryable=False,
            error_code="evervault_config_missing",
            error_message_safe="Evervault API base URL is not configured",
        )

    url = f"{base_url}{path}"
    timeout = max(float(_DEFAULT_MANAGEMENT_TIMEOUT_SECONDS), 1.0)
    try:
        async with httpx.AsyncClient(
            follow_redirects=False,
            trust_env=settings.payment_transport_trust_env,
            timeout=timeout,
        ) as client:
            response = await client.request(
                method,
                url,
                auth=httpx.BasicAuth(app_id, api_key),
                headers={"Accept": "application/json"},
                json=json_body,
            )
    except (httpx.TimeoutException, TimeoutError) as exc:
        raise EvervaultRelayError(
            retryable=True,
            error_code="evervault_relay_management_timeout",
            error_message_safe="Evervault relay management request timed out",
        ) from exc
    except Exception as exc:
        raise EvervaultRelayError(
            retryable=False,
            error_code="evervault_relay_management_transport_error",
            error_message_safe=f"Evervault relay management transport error: {type(exc).__name__}",
        ) from exc

    if response.status_code >= 500:
        raise EvervaultRelayError(
            retryable=True,
            error_code="evervault_relay_management_unavailable",
            error_message_safe="Evervault relay management service is unavailable",
        )
    if response.status_code >= 400:
        raise EvervaultRelayError(
            retryable=False,
            error_code=f"evervault_relay_management_http_{response.status_code}",
            error_message_safe="Evervault relay management request failed",
        )

    if not response.content:
        return {}
    try:
        parsed = response.json()
    except ValueError as exc:
        raise EvervaultRelayError(
            retryable=False,
            error_code="evervault_relay_management_invalid_json",
            error_message_safe="Evervault relay management response was not valid JSON",
        ) from exc
    if isinstance(parsed, dict):
        return parsed
    raise EvervaultRelayError(
        retryable=False,
        error_code="evervault_relay_management_invalid_shape",
        error_message_safe="Evervault relay management response had an invalid shape",
    )


async def _resolve_relay_runtime(*, provider: str, payment_url: str) -> RelayRuntimeConfig:
    normalized_provider = _normalize_provider(provider)
    relay_id_override = _relay_id_override(normalized_provider)
    relay_domain_override = _relay_domain_override(normalized_provider)

    if relay_id_override and relay_domain_override:
        return RelayRuntimeConfig(
            relay_id=relay_id_override,
            relay_domain=relay_domain_override,
        )

    expected_definition = build_payment_relay_definition(provider=normalized_provider, payment_url=payment_url)

    list_payload = await _management_request(method="GET", path="/relays")
    relay_rows = list_payload.get("data")
    relays = relay_rows if isinstance(relay_rows, list) else []
    relays = [row for row in relays if isinstance(row, dict)]

    selected_relay: dict[str, Any] | None = None
    if relay_id_override:
        for relay in relays:
            if str(relay.get("id") or "").strip() == relay_id_override:
                selected_relay = relay
                break
        if selected_relay is None:
            relay_payload = await _management_request(
                method="GET",
                path=f"/relays/{relay_id_override}",
            )
            selected_relay = relay_payload
    else:
        for relay in relays:
            destination = str(relay.get("destinationDomain") or "").strip().lower()
            if destination == str(expected_definition["destinationDomain"]).lower():
                selected_relay = relay
                break

    if selected_relay is None:
        selected_relay = await _management_request(
            method="POST",
            path="/relays",
            json_body=expected_definition,
        )
        return _runtime_from_relay(selected_relay)

    if not _relay_matches_definition(selected_relay, expected_definition):
        relay_id = str(selected_relay.get("id") or "").strip()
        if not relay_id:
            raise EvervaultRelayError(
                retryable=False,
                error_code="evervault_relay_invalid_response",
                error_message_safe="Evervault relay configuration is missing relay id",
            )
        destination = str(selected_relay.get("destinationDomain") or "").strip().lower()
        if destination != str(expected_definition["destinationDomain"]).lower():
            selected_relay = await _management_request(
                method="POST",
                path="/relays",
                json_body=expected_definition,
            )
            return _runtime_from_relay(selected_relay)

        selected_relay = await _management_request(
            method="PATCH",
            path=f"/relays/{relay_id}",
            json_body={
                "routes": expected_definition["routes"],
                "authentication": expected_definition["authentication"],
                "encryptEmptyStrings": bool(expected_definition.get("encryptEmptyStrings", True)),
            },
        )

    return _runtime_from_relay(selected_relay)


async def _get_cached_or_resolved_relay_runtime(*, provider: str, payment_url: str) -> RelayRuntimeConfig:
    cache_ttl_seconds = max(int(settings.evervault_relay_cache_seconds), 0)
    cache_key = f"{_normalize_provider(provider)}:{payment_url}"
    if cache_ttl_seconds <= 0:
        return await _resolve_relay_runtime(provider=provider, payment_url=payment_url)

    now_mono = time.monotonic()
    cached = _RELAY_CACHE.get(cache_key)
    if cached and cached[0] > now_mono:
        return cached[1]

    async with _RELAY_CACHE_LOCK:
        cached = _RELAY_CACHE.get(cache_key)
        now_mono = time.monotonic()
        if cached and cached[0] > now_mono:
            return cached[1]

        resolved = await _resolve_relay_runtime(provider=provider, payment_url=payment_url)
        _RELAY_CACHE[cache_key] = (now_mono + cache_ttl_seconds, resolved)
        return resolved


async def submit_payment_via_evervault_relay(
    *,
    provider: str,
    payment_url: str,
    provider_headers: Mapping[str, str],
    form_data: Mapping[str, Any],
    timeout: float,
) -> EvervaultRelayHttpResponse:
    normalized_provider = _normalize_provider(provider)
    relay_runtime = await _get_cached_or_resolved_relay_runtime(
        provider=normalized_provider,
        payment_url=payment_url,
    )
    path = _extract_payment_path(payment_url)
    relay_url = f"https://{relay_runtime.relay_domain}{path}"

    app_id, api_key = _runtime_credentials()
    request_headers = {str(k): str(v) for k, v in dict(provider_headers or {}).items() if str(k).lower() != "host"}
    request_headers["X-Evervault-App-Id"] = app_id
    request_headers["X-Evervault-Api-Key"] = api_key

    try:
        async with httpx.AsyncClient(
            follow_redirects=False,
            trust_env=settings.payment_transport_trust_env,
            timeout=max(float(timeout), 1.0),
        ) as client:
            response = await client.post(
                relay_url,
                headers=request_headers,
                data=dict(form_data),
            )
    except (httpx.TimeoutException, TimeoutError) as exc:
        raise EvervaultRelayError(
            retryable=True,
            error_code=f"{normalized_provider.lower()}_evervault_relay_timeout",
            error_message_safe=f"{normalized_provider} payment relay timed out",
        ) from exc
    except Exception as exc:
        raise EvervaultRelayError(
            retryable=False,
            error_code=f"{normalized_provider.lower()}_evervault_relay_transport_error",
            error_message_safe=f"{normalized_provider} payment relay transport error: {type(exc).__name__}",
        ) from exc

    if response.status_code in {401, 403}:
        raise EvervaultRelayError(
            retryable=False,
            error_code=f"{normalized_provider.lower()}_evervault_relay_auth_failed",
            error_message_safe=f"{normalized_provider} payment relay authentication failed",
        )

    return EvervaultRelayHttpResponse(
        status_code=response.status_code,
        text=response.text,
        relay_url=relay_url,
        relay_id=relay_runtime.relay_id,
        relay_domain=relay_runtime.relay_domain,
    )


async def submit_ktx_payment_via_evervault_relay(
    *,
    payment_url: str,
    provider_headers: Mapping[str, str],
    form_data: Mapping[str, Any],
    timeout: float,
) -> EvervaultRelayHttpResponse:
    return await submit_payment_via_evervault_relay(
        provider="KTX",
        payment_url=payment_url,
        provider_headers=provider_headers,
        form_data=form_data,
        timeout=timeout,
    )


async def submit_srt_payment_via_evervault_relay(
    *,
    payment_url: str,
    provider_headers: Mapping[str, str],
    form_data: Mapping[str, Any],
    timeout: float,
) -> EvervaultRelayHttpResponse:
    return await submit_payment_via_evervault_relay(
        provider="SRT",
        payment_url=payment_url,
        provider_headers=provider_headers,
        form_data=form_data,
        timeout=timeout,
    )
