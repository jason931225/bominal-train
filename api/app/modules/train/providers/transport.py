from __future__ import annotations

import asyncio
from dataclasses import dataclass
from typing import Any, Awaitable, Callable, Protocol

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
    def __init__(self) -> None:
        self._client = httpx.AsyncClient(follow_redirects=True)

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
        del operation  # transport compatibility hook
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
        response = await self._client.request(
            method,
            url,
            headers=headers,
            json=json_body,
            data=data,
            params=params,
            timeout=resolved_timeout,
        )
        return TransportResponse(
            status_code=response.status_code,
            text=response.text,
            headers=dict(response.headers),
        )

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
    ) -> None:
        try:
            import curl_cffi.requests  # type: ignore
        except Exception as exc:  # pragma: no cover - optional dependency
            raise RuntimeError("curl_cffi is not installed") from exc

        self._requests = curl_cffi.requests
        self._impersonate = self._resolve_impersonate(impersonate)
        self._session = self._requests.AsyncSession(impersonate=self._impersonate)
        if default_headers:
            self._session.headers.update(default_headers)
        self._fallback_transport = HttpxTransport()

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
        del operation  # transport compatibility hook
        curl_timeout = timeout.total if isinstance(timeout, OperationTimeout) else timeout
        try:
            response = await self._session.request(
                method,
                url,
                headers=headers,
                json=json_body,
                data=data,
                params=params,
                timeout=curl_timeout,
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
            )
            return fallback

    async def close(self) -> None:
        try:  # pragma: no cover - best effort cleanup
            await self._session.close()
        except Exception:
            pass
        await self._fallback_transport.close()
