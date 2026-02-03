from __future__ import annotations

from dataclasses import dataclass
from typing import Any, Protocol

import httpx


@dataclass(slots=True)
class TransportResponse:
    status_code: int
    text: str
    headers: dict[str, str]


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
        timeout: float = 20.0,
    ) -> TransportResponse:
        ...


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
        timeout: float = 20.0,
    ) -> TransportResponse:
        response = await self._client.request(
            method,
            url,
            headers=headers,
            json=json_body,
            data=data,
            params=params,
            timeout=timeout,
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
        timeout: float = 20.0,
    ) -> TransportResponse:
        try:
            response = await self._session.request(
                method,
                url,
                headers=headers,
                json=json_body,
                data=data,
                params=params,
                timeout=timeout,
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
