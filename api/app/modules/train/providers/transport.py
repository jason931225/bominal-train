from __future__ import annotations

import asyncio
from dataclasses import dataclass
from typing import Any, Awaitable, Callable, Mapping, Protocol
from urllib.parse import urljoin, urlparse

import httpx

from app.core.config import get_settings

LEGACY_DEFAULT_TIMEOUT_SECONDS = 20.0

_AUTH_OPERATION = "auth"
_QUERY_OPERATION = "query"
_EXECUTE_OPERATION = "execute"
_DEFAULT_OPERATION = "default"

_AUTH_URL_TOKENS = ("apb01080", "app.login", "common.code")
_QUERY_URL_TOKENS = (
    "ara10007",
    "scheduleview",
    "atc14016",
    "ard02019",
    "getlistatc14087",
    "myticket.myticketlist",
    "refunds.selticketinfo",
    "reservation.reservationview",
    "certification.reservationlist",
)
_EXECUTE_URL_TOKENS = (
    "arc05013",
    "ticketreservation",
    "ata01135",
    "ata09036",
    "reservationpayment",
    "ard02045",
    "reservationcancel",
    "atc02063",
    "refundsrequest",
)

_REDIRECT_STATUS_CODES = {301, 302, 303, 307, 308}
_MAX_REDIRECT_HOPS = 3

_EGRESS_DOMAIN_SET_TRAIN = "train"
_EGRESS_DOMAIN_SET_RESTAURANT = "restaurant"

_TRAIN_EGRESS_PROXY_PATH_BY_HOST: dict[str, str] = {
    "app.srail.or.kr": "srt",
    "smart.letskorail.com": "korail",
    "nf.letskorail.com": "netfunnel",
    # NetFunnel can rotate among hostnames like nf5.letskorail.com.
    # Keep a domain fallback so those hosts route to the same egress path.
    "letskorail.com": "netfunnel",
}

_RESTAURANT_EGRESS_PROXY_PATH_BY_HOST: dict[str, str] = {
    "opentable.com": "opentable",
    "www.opentable.com": "opentable",
    "api.resy.com": "resy",
    "resy.com": "resy",
}


def _normalize_allowed_hosts(hosts: list[str]) -> set[str]:
    return {str(host).strip().lower() for host in hosts if str(host).strip()}


def _is_host_allowed(*, host: str, allowed_hosts: set[str]) -> bool:
    if not host:
        return False
    if host in allowed_hosts:
        return True
    return any(host.endswith(f".{allowed}") for allowed in allowed_hosts)


def _assert_allowed_host(url: str, *, allowed_hosts: set[str]) -> None:
    parsed = urlparse(url)
    host = (parsed.hostname or "").lower()
    if not _is_host_allowed(host=host, allowed_hosts=allowed_hosts):
        raise RuntimeError(f"provider egress host is not allowlisted: {host or 'unknown'}")


def _resolve_proxy_path_prefix(host: str, *, host_path_map: Mapping[str, str]) -> str | None:
    host_lower = host.lower().strip()
    best_match: tuple[int, str] | None = None
    for mapped_host, mapped_path in host_path_map.items():
        normalized_host = str(mapped_host).lower().strip()
        if not normalized_host:
            continue
        if host_lower == normalized_host or host_lower.endswith(f".{normalized_host}"):
            score = len(normalized_host)
            if best_match is None or score > best_match[0]:
                best_match = (score, str(mapped_path))
    return best_match[1] if best_match else None


def _rewrite_url_for_egress_proxy(
    url: str,
    *,
    proxy_base_url: str | None,
    host_path_map: Mapping[str, str],
) -> str:
    if not proxy_base_url:
        return url
    parsed = urlparse(url)
    host = (parsed.hostname or "").lower().strip()
    if not host:
        raise RuntimeError("provider egress proxy requires absolute URL host")

    route_prefix = _resolve_proxy_path_prefix(host, host_path_map=host_path_map)
    if route_prefix is None:
        raise RuntimeError(f"provider egress proxy route is not configured for host: {host}")

    normalized_prefix = f"/{route_prefix.strip('/')}"
    path = parsed.path or "/"
    if not path.startswith("/"):
        path = f"/{path}"
    query_suffix = f"?{parsed.query}" if parsed.query else ""
    return f"{proxy_base_url.rstrip('/')}{normalized_prefix}{path}{query_suffix}"


def _resolve_egress_proxy_config(*, settings: Any, domain_set: str) -> tuple[str | None, dict[str, str]]:
    normalized = domain_set.strip().lower()
    if normalized == _EGRESS_DOMAIN_SET_TRAIN:
        proxy_url = str(settings.train_provider_egress_proxy_url or "").strip() or None
        return proxy_url, dict(_TRAIN_EGRESS_PROXY_PATH_BY_HOST)
    if normalized == _EGRESS_DOMAIN_SET_RESTAURANT:
        proxy_url = str(settings.restaurant_provider_egress_proxy_url or "").strip() or None
        return proxy_url, dict(_RESTAURANT_EGRESS_PROXY_PATH_BY_HOST)
    return None, {}


@dataclass(slots=True)
class TransportResponse:
    status_code: int
    text: str
    headers: dict[str, str]


@dataclass(slots=True, frozen=True)
class OperationTimeout:
    connect: float
    read: float
    total: float


class ProviderTransportError(RuntimeError):
    def __init__(
        self,
        *,
        retryable: bool,
        error_code: str,
        error_message_safe: str,
        status_code: int | None = None,
    ) -> None:
        super().__init__(error_message_safe)
        self.retryable = retryable
        self.error_code = error_code
        self.error_message_safe = error_message_safe
        self.status_code = status_code


class AsyncTransport(Protocol):
    async def request(
        self,
        *,
        method: str,
        url: str,
        headers: dict[str, str] | None = None,
        json_body: dict[str, Any] | None = None,
        data: dict[str, Any] | None = None,
        params: dict[str, Any] | None = None,
        timeout: float | OperationTimeout = LEGACY_DEFAULT_TIMEOUT_SECONDS,
        operation: str | None = None,
    ) -> TransportResponse:
        ...


def is_retryable_status_code(status_code: int) -> bool:
    return status_code == 429 or status_code >= 500


def is_retryable_exception(exc: Exception) -> bool:
    return isinstance(exc, (httpx.TimeoutException, TimeoutError, asyncio.TimeoutError))


class ResilientTransport:
    """Retry + timeout policy wrapper around provider egress transports."""

    def __init__(
        self,
        transport: AsyncTransport,
        *,
        provider: str,
        retry_attempts: int | None = None,
        retry_backoff_seconds: float | None = None,
        connect_timeout_seconds: float | None = None,
        read_timeout_seconds: float | None = None,
        total_timeout_seconds: float | None = None,
        sleeper: Callable[[float], Awaitable[None]] = asyncio.sleep,
    ) -> None:
        settings = get_settings()
        self._transport = transport
        self._provider = provider
        self._retry_attempts = max(1, retry_attempts or settings.train_provider_retry_attempts)
        self._retry_backoff_seconds = max(
            0.0,
            retry_backoff_seconds
            if retry_backoff_seconds is not None
            else settings.train_provider_retry_backoff_seconds,
        )
        self._connect_timeout_seconds = max(
            0.1,
            connect_timeout_seconds
            if connect_timeout_seconds is not None
            else settings.train_provider_timeout_connect_seconds,
        )
        self._read_timeout_seconds = max(
            0.1,
            read_timeout_seconds
            if read_timeout_seconds is not None
            else settings.train_provider_timeout_read_seconds,
        )
        self._total_timeout_seconds = max(
            0.1,
            total_timeout_seconds
            if total_timeout_seconds is not None
            else settings.train_provider_timeout_total_seconds,
        )
        self._allowed_hosts = _normalize_allowed_hosts(settings.payment_provider_allowed_hosts)
        self._sleeper = sleeper

    async def request(
        self,
        *,
        method: str,
        url: str,
        headers: dict[str, str] | None = None,
        json_body: dict[str, Any] | None = None,
        data: dict[str, Any] | None = None,
        params: dict[str, Any] | None = None,
        timeout: float | OperationTimeout = LEGACY_DEFAULT_TIMEOUT_SECONDS,
        operation: str | None = None,
    ) -> TransportResponse:
        resolved_operation = self._resolve_operation(method=method, url=url, explicit=operation)
        max_attempts = self._max_attempts_for_operation(resolved_operation)
        resolved_timeout = self._resolve_timeout(resolved_operation, timeout)

        for attempt in range(1, max_attempts + 1):
            try:
                _assert_allowed_host(url, allowed_hosts=self._allowed_hosts)
                response = await self._transport.request(
                    method=method,
                    url=url,
                    headers=headers,
                    json_body=json_body,
                    data=data,
                    params=params,
                    timeout=resolved_timeout,
                    operation=resolved_operation,
                )
            except Exception as exc:
                retryable = is_retryable_exception(exc)
                if retryable and attempt < max_attempts:
                    await self._sleep_before_retry(attempt)
                    continue
                if retryable:
                    raise ProviderTransportError(
                        retryable=True,
                        error_code=f"{self._provider.lower()}_provider_timeout",
                        error_message_safe=f"{self._provider} provider timeout during {resolved_operation}",
                    ) from exc
                raise ProviderTransportError(
                    retryable=False,
                    error_code=f"{self._provider.lower()}_provider_transport_error",
                    error_message_safe=(
                        f"{self._provider} provider transport failure during {resolved_operation}: "
                        f"{type(exc).__name__}"
                    ),
                ) from exc

            if response.status_code >= 400:
                if is_retryable_status_code(response.status_code):
                    if attempt < max_attempts:
                        await self._sleep_before_retry(attempt)
                        continue
                    raise ProviderTransportError(
                        retryable=True,
                        error_code=f"{self._provider.lower()}_provider_http_{response.status_code}",
                        error_message_safe=(
                            f"{self._provider} provider transient HTTP {response.status_code} during "
                            f"{resolved_operation}"
                        ),
                        status_code=response.status_code,
                    )
                raise ProviderTransportError(
                    retryable=False,
                    error_code=f"{self._provider.lower()}_provider_http_{response.status_code}",
                    error_message_safe=(
                        f"{self._provider} provider non-retryable HTTP {response.status_code} during "
                        f"{resolved_operation}"
                    ),
                    status_code=response.status_code,
                )
            return response

        raise ProviderTransportError(
            retryable=True,
            error_code=f"{self._provider.lower()}_provider_retry_exhausted",
            error_message_safe=f"{self._provider} provider retries exhausted",
        )

    async def close(self) -> None:
        close_fn = getattr(self._transport, "close", None)
        if callable(close_fn):
            await close_fn()

    def _resolve_operation(self, *, method: str, url: str, explicit: str | None) -> str:
        if explicit:
            return explicit.lower()

        lower_url = url.lower()
        if any(token in lower_url for token in _EXECUTE_URL_TOKENS):
            return _EXECUTE_OPERATION
        if any(token in lower_url for token in _AUTH_URL_TOKENS):
            return _AUTH_OPERATION
        if any(token in lower_url for token in _QUERY_URL_TOKENS):
            return _QUERY_OPERATION

        if method.upper() == "GET":
            return _QUERY_OPERATION
        return _DEFAULT_OPERATION

    def _resolve_timeout(self, operation: str, timeout: float | OperationTimeout) -> OperationTimeout:
        profile = self._timeout_profile_for_operation(operation)
        if isinstance(timeout, OperationTimeout):
            return timeout
        if isinstance(timeout, (int, float)):
            timeout_value = float(timeout)
            if abs(timeout_value - LEGACY_DEFAULT_TIMEOUT_SECONDS) < 1e-6:
                return profile
            bounded_total = max(0.1, timeout_value)
            return OperationTimeout(
                connect=min(profile.connect, bounded_total),
                read=min(profile.read, bounded_total),
                total=bounded_total,
            )
        return profile

    def _timeout_profile_for_operation(self, operation: str) -> OperationTimeout:
        multipliers: dict[str, tuple[float, float, float]] = {
            _AUTH_OPERATION: (1.0, 1.0, 1.0),
            _QUERY_OPERATION: (1.0, 1.4, 1.35),
            _EXECUTE_OPERATION: (1.0, 1.0, 1.0),
            _DEFAULT_OPERATION: (1.0, 1.0, 1.0),
        }
        connect_mult, read_mult, total_mult = multipliers.get(operation, multipliers[_DEFAULT_OPERATION])
        connect_timeout = max(0.1, self._connect_timeout_seconds * connect_mult)
        read_timeout = max(0.1, self._read_timeout_seconds * read_mult)
        total_timeout = max(0.1, self._total_timeout_seconds * total_mult)
        total_timeout = max(total_timeout, connect_timeout, read_timeout)
        return OperationTimeout(connect=connect_timeout, read=read_timeout, total=total_timeout)

    def _max_attempts_for_operation(self, operation: str) -> int:
        if operation == _EXECUTE_OPERATION:
            # No auto-retry for side-effecting provider operations.
            return 1
        if operation == _DEFAULT_OPERATION:
            # Unknown write paths fail closed by default.
            return 1
        return self._retry_attempts

    async def _sleep_before_retry(self, attempt: int) -> None:
        if self._retry_backoff_seconds <= 0.0:
            return
        await self._sleeper(self._retry_backoff_seconds * attempt)


class HttpxTransport:
    def __init__(self, *, egress_domain_set: str = _EGRESS_DOMAIN_SET_TRAIN) -> None:
        settings = get_settings()
        self._allowed_hosts = _normalize_allowed_hosts(settings.payment_provider_allowed_hosts)
        self._egress_proxy_url, self._egress_proxy_path_by_host = _resolve_egress_proxy_config(
            settings=settings,
            domain_set=egress_domain_set,
        )
        self._client = httpx.AsyncClient(
            follow_redirects=False,
            trust_env=settings.payment_transport_trust_env,
        )

    async def request(
        self,
        *,
        method: str,
        url: str,
        headers: dict[str, str] | None = None,
        json_body: dict[str, Any] | None = None,
        data: dict[str, Any] | None = None,
        params: dict[str, Any] | None = None,
        timeout: float | OperationTimeout = LEGACY_DEFAULT_TIMEOUT_SECONDS,
        operation: str | None = None,
    ) -> TransportResponse:
        resolved_operation = (operation or _DEFAULT_OPERATION).lower()
        if isinstance(timeout, OperationTimeout):
            resolved_timeout: float | httpx.Timeout = httpx.Timeout(
                timeout=timeout.total,
                connect=timeout.connect,
                read=timeout.read,
                write=timeout.read,
                pool=timeout.connect,
            )
        else:
            resolved_timeout = timeout

        current_url = url
        current_method = method
        current_json = json_body
        current_data = data
        current_params = params

        for _ in range(_MAX_REDIRECT_HOPS + 1):
            _assert_allowed_host(current_url, allowed_hosts=self._allowed_hosts)
            target_url = _rewrite_url_for_egress_proxy(
                current_url,
                proxy_base_url=self._egress_proxy_url,
                host_path_map=self._egress_proxy_path_by_host,
            )
            response = await self._client.request(
                current_method,
                target_url,
                headers=headers,
                json=current_json,
                data=current_data,
                params=current_params,
                timeout=resolved_timeout,
            )
            if response.status_code not in _REDIRECT_STATUS_CODES:
                return TransportResponse(
                    status_code=response.status_code,
                    text=response.text,
                    headers=dict(response.headers),
                )

            location = response.headers.get("location")
            if not location:
                return TransportResponse(
                    status_code=response.status_code,
                    text=response.text,
                    headers=dict(response.headers),
                )

            if resolved_operation in {_EXECUTE_OPERATION, _DEFAULT_OPERATION}:
                raise RuntimeError("redirect blocked for side-effecting provider operation")

            next_url = urljoin(current_url, location)
            _assert_allowed_host(next_url, allowed_hosts=self._allowed_hosts)

            current_url = next_url
            current_params = None
            if response.status_code == 303 or (
                response.status_code in {301, 302} and current_method.upper() not in {"GET", "HEAD"}
            ):
                current_method = "GET"
                current_json = None
                current_data = None

        raise RuntimeError("provider redirect hop limit exceeded")

    async def close(self) -> None:
        await self._client.aclose()


class CurlCffiTransport:
    """Optional transport for providers that require browser-like TLS behavior.

    This adapter is intentionally lightweight and only used when curl_cffi is installed.
    """

    def __init__(
        self,
        *,
        impersonate: str = "chrome131_android",
        default_headers: dict[str, str] | None = None,
        egress_domain_set: str = _EGRESS_DOMAIN_SET_TRAIN,
    ) -> None:
        settings = get_settings()
        try:
            import curl_cffi.requests  # type: ignore
        except Exception as exc:
            raise RuntimeError("curl_cffi is not installed") from exc

        self._requests = curl_cffi.requests
        self._allowed_hosts = _normalize_allowed_hosts(settings.payment_provider_allowed_hosts)
        self._egress_proxy_url, self._egress_proxy_path_by_host = _resolve_egress_proxy_config(
            settings=settings,
            domain_set=egress_domain_set,
        )
        self._impersonate = self._resolve_impersonate(impersonate)
        self._session = self._requests.AsyncSession(impersonate=self._impersonate)
        if default_headers:
            self._session.headers.update(default_headers)
        self._fallback_transport = HttpxTransport(egress_domain_set=egress_domain_set)

    def _resolve_impersonate(self, desired: str) -> str:
        browser_type = getattr(self._requests, "BrowserType", None)
        if browser_type is None:
            return desired

        available: set[str] = set()
        for name in dir(browser_type):
            if name.startswith("_"):
                continue
            available.add(name)
            value = getattr(browser_type, name, None)
            if isinstance(value, str):
                available.add(value)

        if desired in available:
            return desired

        for candidate in ("chrome99_android", "chrome124", "chrome120", "chrome119", "chrome110"):
            if candidate in available:
                return candidate

        return desired

    async def request(
        self,
        *,
        method: str,
        url: str,
        headers: dict[str, str] | None = None,
        json_body: dict[str, Any] | None = None,
        data: dict[str, Any] | None = None,
        params: dict[str, Any] | None = None,
        timeout: float | OperationTimeout = LEGACY_DEFAULT_TIMEOUT_SECONDS,
        operation: str | None = None,
    ) -> TransportResponse:
        resolved_operation = (operation or _DEFAULT_OPERATION).lower()
        _assert_allowed_host(url, allowed_hosts=self._allowed_hosts)
        target_url = _rewrite_url_for_egress_proxy(
            url,
            proxy_base_url=self._egress_proxy_url,
            host_path_map=self._egress_proxy_path_by_host,
        )
        curl_timeout = timeout.total if isinstance(timeout, OperationTimeout) else timeout
        try:
            response = await self._session.request(
                method,
                target_url,
                headers=headers,
                json=json_body,
                data=data,
                params=params,
                timeout=curl_timeout,
                allow_redirects=False,
            )
            if response.status_code in _REDIRECT_STATUS_CODES:
                location = response.headers.get("location")
                if location:
                    if resolved_operation in {_EXECUTE_OPERATION, _DEFAULT_OPERATION}:
                        raise RuntimeError("redirect blocked for side-effecting provider operation")
                    redirected_url = urljoin(url, location)
                    _assert_allowed_host(redirected_url, allowed_hosts=self._allowed_hosts)
                    return await self._fallback_transport.request(
                        method="GET",
                        url=redirected_url,
                        headers=headers,
                        params=None,
                        timeout=timeout,
                        operation=resolved_operation,
                    )
            return TransportResponse(
                status_code=response.status_code,
                text=response.text,
                headers=dict(response.headers),
            )
        except Exception as exc:
            # Some curl_cffi builds do not support newer impersonation presets.
            if "Impersonating" not in str(exc):
                raise

            fallback = await self._fallback_transport.request(
                method=method,
                url=url,
                headers=headers,
                json_body=json_body,
                data=data,
                params=params,
                timeout=timeout,
                operation=resolved_operation,
            )
            return fallback

    async def close(self) -> None:
        try:
            await self._session.close()
        except Exception:
            pass
        await self._fallback_transport.close()
