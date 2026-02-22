from __future__ import annotations

import asyncio
import sys
from dataclasses import dataclass, field
from types import ModuleType, SimpleNamespace
from typing import Any

import httpx
import pytest

import app.modules.train.providers.transport as transport_module
from app.modules.train.providers.transport import (
    CurlCffiTransport,
    HttpxTransport,
    OperationTimeout,
    ProviderTransportError,
    ResilientTransport,
    TransportResponse,
    _assert_allowed_host,
    _is_host_allowed,
    _normalize_allowed_hosts,
    is_retryable_exception,
    is_retryable_status_code,
)


def _patch_settings(
    monkeypatch: pytest.MonkeyPatch,
    *,
    allowed_hosts: tuple[str, ...] = ("app.srail.or.kr", "letskorail.com"),
    trust_env: bool = False,
    retry_attempts: int = 2,
    retry_backoff_seconds: float = 0.2,
    connect_timeout_seconds: float = 3.0,
    read_timeout_seconds: float = 8.0,
    total_timeout_seconds: float = 12.0,
) -> None:
    settings = SimpleNamespace(
        payment_provider_allowed_hosts=list(allowed_hosts),
        payment_transport_trust_env=trust_env,
        train_provider_retry_attempts=retry_attempts,
        train_provider_retry_backoff_seconds=retry_backoff_seconds,
        train_provider_timeout_connect_seconds=connect_timeout_seconds,
        train_provider_timeout_read_seconds=read_timeout_seconds,
        train_provider_timeout_total_seconds=total_timeout_seconds,
    )
    monkeypatch.setattr(transport_module, "get_settings", lambda: settings)


@dataclass(slots=True)
class _Action:
    status_code: int | None = None
    error: Exception | None = None


class _SequenceTransport:
    def __init__(self, actions: list[_Action]) -> None:
        self._actions = list(actions)
        self.requests: list[dict[str, Any]] = []
        self.close_called = False

    async def request(self, **kwargs) -> TransportResponse:
        self.requests.append(kwargs)
        if not self._actions:
            raise AssertionError("No queued action available for request")
        action = self._actions.pop(0)
        if action.error is not None:
            raise action.error
        status = action.status_code if action.status_code is not None else 200
        return TransportResponse(
            status_code=status,
            text="{}",
            headers={},
        )

    async def close(self) -> None:
        self.close_called = True


@dataclass(slots=True)
class _FakeHttpxResponse:
    status_code: int
    text: str = "{}"
    headers: dict[str, str] = field(default_factory=dict)


class _FakeHttpxClient:
    def __init__(self, responses: list[_FakeHttpxResponse]) -> None:
        self._responses = list(responses)
        self.calls: list[dict[str, Any]] = []
        self.closed = False

    async def request(self, method: str, url: str, **kwargs: Any) -> _FakeHttpxResponse:
        self.calls.append({"method": method, "url": url, **kwargs})
        if not self._responses:
            raise AssertionError("No queued response available for request")
        return self._responses.pop(0)

    async def aclose(self) -> None:
        self.closed = True


@dataclass(slots=True)
class _FakeCurlResponse:
    status_code: int
    text: str = "{}"
    headers: dict[str, str] = field(default_factory=dict)


class _FakeCurlSession:
    def __init__(
        self,
        *,
        responses: list[_FakeCurlResponse] | None = None,
        error: Exception | None = None,
    ) -> None:
        self._responses = list(responses or [])
        self._error = error
        self.headers: dict[str, str] = {}
        self.calls: list[dict[str, Any]] = []
        self.close_called = False
        self.close_error: Exception | None = None

    async def request(self, method: str, url: str, **kwargs: Any) -> _FakeCurlResponse:
        self.calls.append({"method": method, "url": url, **kwargs})
        if self._error is not None:
            raise self._error
        if not self._responses:
            raise AssertionError("No queued response available for curl session request")
        return self._responses.pop(0)

    async def close(self) -> None:
        self.close_called = True
        if self.close_error is not None:
            raise self.close_error


class _FallbackTransport:
    def __init__(self, response: TransportResponse) -> None:
        self._response = response
        self.calls: list[dict[str, Any]] = []
        self.closed = False

    async def request(self, **kwargs: Any) -> TransportResponse:
        self.calls.append(kwargs)
        return self._response

    async def close(self) -> None:
        self.closed = True


def _install_fake_curl_module(
    monkeypatch: pytest.MonkeyPatch,
    *,
    session: _FakeCurlSession,
    browser_type_attrs: dict[str, str] | None = None,
) -> dict[str, str]:
    captured: dict[str, str] = {}
    requests_module = ModuleType("curl_cffi.requests")

    def _async_session(*, impersonate: str) -> _FakeCurlSession:
        captured["impersonate"] = impersonate
        return session

    requests_module.AsyncSession = _async_session  # type: ignore[attr-defined]
    if browser_type_attrs is not None:
        requests_module.BrowserType = type("BrowserType", (), browser_type_attrs)  # type: ignore[attr-defined]

    curl_module = ModuleType("curl_cffi")
    curl_module.requests = requests_module  # type: ignore[attr-defined]

    monkeypatch.setitem(sys.modules, "curl_cffi", curl_module)
    monkeypatch.setitem(sys.modules, "curl_cffi.requests", requests_module)
    return captured


def test_host_allowlist_helpers_cover_exact_subdomain_and_invalid_cases():
    allowed = _normalize_allowed_hosts([" app.srail.or.kr ", "LETSKORAIL.COM", "", "   "])
    assert allowed == {"app.srail.or.kr", "letskorail.com"}

    assert _is_host_allowed(host="app.srail.or.kr", allowed_hosts=allowed) is True
    assert _is_host_allowed(host="api.letskorail.com", allowed_hosts=allowed) is True
    assert _is_host_allowed(host="", allowed_hosts=allowed) is False
    assert _is_host_allowed(host="evil.example.com", allowed_hosts=allowed) is False

    _assert_allowed_host("https://app.srail.or.kr/path", allowed_hosts=allowed)
    with pytest.raises(RuntimeError, match="allowlisted"):
        _assert_allowed_host("https://evil.example.com/path", allowed_hosts=allowed)
    with pytest.raises(RuntimeError, match="allowlisted"):
        _assert_allowed_host("not-a-valid-url", allowed_hosts=allowed)


def test_retryable_classification_helpers():
    assert is_retryable_status_code(429) is True
    assert is_retryable_status_code(500) is True
    assert is_retryable_status_code(400) is False

    assert is_retryable_exception(httpx.ReadTimeout("timed out")) is True
    assert is_retryable_exception(TimeoutError("timed out")) is True
    assert is_retryable_exception(asyncio.TimeoutError()) is True
    assert is_retryable_exception(RuntimeError("boom")) is False


@pytest.mark.asyncio
async def test_resilient_transport_operation_timeout_profiles_and_backoff(monkeypatch):
    _patch_settings(
        monkeypatch,
        allowed_hosts=("app.srail.or.kr",),
        retry_attempts=3,
        retry_backoff_seconds=0.5,
        connect_timeout_seconds=2.0,
        read_timeout_seconds=4.0,
        total_timeout_seconds=6.0,
    )
    sleep_calls: list[float] = []

    async def _sleep(delay: float) -> None:
        sleep_calls.append(delay)

    base = _SequenceTransport(actions=[_Action(status_code=200)])
    transport = ResilientTransport(base, provider="SRT", sleeper=_sleep)

    assert (
        transport._resolve_operation(
            method="POST",
            url="https://app.srail.or.kr:443/ata/selectListAta09036_n.do",
            explicit=None,
        )
        == "execute"
    )
    assert (
        transport._resolve_operation(
            method="POST",
            url="https://app.srail.or.kr:443/apb/selectListApb01080_n.do",
            explicit=None,
        )
        == "auth"
    )
    assert (
        transport._resolve_operation(
            method="GET",
            url="https://app.srail.or.kr:443/ara/selectListAra10007_n.do",
            explicit=None,
        )
        == "query"
    )
    assert transport._resolve_operation(method="GET", url="https://app.srail.or.kr/unknown", explicit=None) == "query"
    assert (
        transport._resolve_operation(
            method="POST",
            url="https://app.srail.or.kr/unknown",
            explicit=None,
        )
        == "default"
    )
    assert (
        transport._resolve_operation(
            method="GET",
            url="https://app.srail.or.kr/unknown",
            explicit="QUERY",
        )
        == "query"
    )

    query_profile = transport._timeout_profile_for_operation("query")
    assert query_profile.connect == pytest.approx(2.0)
    assert query_profile.read == pytest.approx(5.6)
    assert query_profile.total == pytest.approx(8.1)

    legacy_timeout = transport._resolve_timeout("query", transport_module.LEGACY_DEFAULT_TIMEOUT_SECONDS)
    assert legacy_timeout == query_profile

    custom_timeout = transport._resolve_timeout("query", 1.5)
    assert custom_timeout == OperationTimeout(connect=1.5, read=1.5, total=1.5)

    explicit_timeout = OperationTimeout(connect=0.9, read=1.1, total=1.3)
    assert transport._resolve_timeout("query", explicit_timeout) is explicit_timeout

    fallback_timeout = transport._resolve_timeout("query", "unexpected")
    assert fallback_timeout == query_profile

    assert transport._max_attempts_for_operation("execute") == 1
    assert transport._max_attempts_for_operation("default") == 1
    assert transport._max_attempts_for_operation("query") == 3

    await transport._sleep_before_retry(2)
    assert sleep_calls == [1.0]

    no_backoff = ResilientTransport(
        _SequenceTransport(actions=[_Action(status_code=200)]),
        provider="SRT",
        retry_backoff_seconds=0.0,
        sleeper=_sleep,
    )
    await no_backoff._sleep_before_retry(4)
    assert sleep_calls == [1.0]

    await transport.close()
    assert base.close_called is True


@pytest.mark.asyncio
async def test_retries_transient_5xx_for_query_operation():
    base = _SequenceTransport(actions=[_Action(status_code=503), _Action(status_code=200)])
    transport = ResilientTransport(
        base,
        provider="KTX",
        retry_attempts=2,
        retry_backoff_seconds=0.0,
    )

    response = await transport.request(
        method="GET",
        url="https://smart.letskorail.com/classes/com.korail.mobile.seatMovie.ScheduleView",
    )

    assert response.status_code == 200
    assert len(base.requests) == 2


@pytest.mark.asyncio
async def test_retryable_status_exhaustion_raises_retryable_provider_error():
    base = _SequenceTransport(actions=[_Action(status_code=503), _Action(status_code=503)])
    transport = ResilientTransport(
        base,
        provider="SRT",
        retry_attempts=2,
        retry_backoff_seconds=0.0,
    )

    with pytest.raises(ProviderTransportError) as exc_info:
        await transport.request(
            method="GET",
            url="https://app.srail.or.kr:443/ara/selectListAra10007_n.do",
        )

    assert exc_info.value.retryable is True
    assert exc_info.value.status_code == 503
    assert exc_info.value.error_code == "srt_provider_http_503"
    assert len(base.requests) == 2


@pytest.mark.asyncio
async def test_non_retryable_http_status_raises_non_retryable_provider_error():
    base = _SequenceTransport(actions=[_Action(status_code=400)])
    transport = ResilientTransport(
        base,
        provider="KTX",
        retry_attempts=3,
        retry_backoff_seconds=0.0,
    )

    with pytest.raises(ProviderTransportError) as exc_info:
        await transport.request(
            method="GET",
            url="https://smart.letskorail.com/classes/com.korail.mobile.seatMovie.ScheduleView",
        )

    assert exc_info.value.retryable is False
    assert exc_info.value.status_code == 400
    assert exc_info.value.error_code == "ktx_provider_http_400"
    assert len(base.requests) == 1


@pytest.mark.asyncio
async def test_no_retry_for_side_effecting_operation_prevents_duplicate_requests():
    base = _SequenceTransport(actions=[_Action(status_code=503), _Action(status_code=200)])
    transport = ResilientTransport(
        base,
        provider="KTX",
        retry_attempts=3,
        retry_backoff_seconds=0.0,
    )

    with pytest.raises(ProviderTransportError) as exc_info:
        await transport.request(
            method="POST",
            url="https://smart.letskorail.com/classes/com.korail.mobile.payment.ReservationPayment",
            data={"reservation_id": "PNR1"},
        )

    assert exc_info.value.retryable is True
    assert len(base.requests) == 1


@pytest.mark.asyncio
async def test_retries_timeout_for_query_operation():
    base = _SequenceTransport(
        actions=[
            _Action(error=httpx.ReadTimeout("timed out")),
            _Action(status_code=200),
        ]
    )
    transport = ResilientTransport(
        base,
        provider="SRT",
        retry_attempts=2,
        retry_backoff_seconds=0.0,
    )

    response = await transport.request(
        method="GET",
        url="https://app.srail.or.kr:443/ara/selectListAra10007_n.do",
    )

    assert response.status_code == 200
    assert len(base.requests) == 2


@pytest.mark.asyncio
async def test_retryable_timeout_exhaustion_raises_provider_timeout_error():
    base = _SequenceTransport(
        actions=[
            _Action(error=httpx.ReadTimeout("timed out")),
            _Action(error=httpx.ReadTimeout("timed out")),
        ]
    )
    transport = ResilientTransport(
        base,
        provider="SRT",
        retry_attempts=2,
        retry_backoff_seconds=0.0,
    )

    with pytest.raises(ProviderTransportError) as exc_info:
        await transport.request(
            method="GET",
            url="https://app.srail.or.kr:443/ara/selectListAra10007_n.do",
        )

    assert exc_info.value.retryable is True
    assert exc_info.value.error_code == "srt_provider_timeout"
    assert len(base.requests) == 2


@pytest.mark.asyncio
async def test_fail_closed_for_non_retryable_transport_error():
    base = _SequenceTransport(
        actions=[
            _Action(error=RuntimeError("boom")),
            _Action(status_code=200),
        ]
    )
    transport = ResilientTransport(
        base,
        provider="SRT",
        retry_attempts=3,
        retry_backoff_seconds=0.0,
    )

    with pytest.raises(ProviderTransportError) as exc_info:
        await transport.request(
            method="GET",
            url="https://app.srail.or.kr:443/ara/selectListAra10007_n.do",
        )

    assert exc_info.value.retryable is False
    assert len(base.requests) == 1


@pytest.mark.asyncio
async def test_blocks_non_allowlisted_host_before_transport_call():
    base = _SequenceTransport(actions=[_Action(status_code=200)])
    transport = ResilientTransport(
        base,
        provider="SRT",
        retry_attempts=2,
        retry_backoff_seconds=0.0,
    )

    with pytest.raises(ProviderTransportError) as exc_info:
        await transport.request(
            method="GET",
            url="https://evil.example.com/collect",
        )

    assert exc_info.value.retryable is False
    assert exc_info.value.error_code == "srt_provider_transport_error"
    assert len(base.requests) == 0


@pytest.mark.asyncio
async def test_httpx_transport_initializes_client_with_trust_env_and_handles_operation_timeout(monkeypatch):
    _patch_settings(monkeypatch, allowed_hosts=("app.srail.or.kr",), trust_env=True)
    init_args: dict[str, Any] = {}
    fake_client = _FakeHttpxClient(responses=[_FakeHttpxResponse(status_code=200, text='{"ok":true}')])

    def _fake_async_client(*, follow_redirects: bool, trust_env: bool) -> _FakeHttpxClient:
        init_args["follow_redirects"] = follow_redirects
        init_args["trust_env"] = trust_env
        return fake_client

    monkeypatch.setattr(transport_module.httpx, "AsyncClient", _fake_async_client)

    transport = HttpxTransport()
    response = await transport.request(
        method="POST",
        url="https://app.srail.or.kr:443/query",
        json_body={"a": 1},
        timeout=OperationTimeout(connect=1.0, read=2.0, total=3.0),
        operation="query",
    )

    assert init_args == {"follow_redirects": False, "trust_env": True}
    assert response.status_code == 200
    assert response.text == '{"ok":true}'
    assert response.headers == {}

    assert len(fake_client.calls) == 1
    timeout_obj = fake_client.calls[0]["timeout"]
    assert isinstance(timeout_obj, httpx.Timeout)
    assert timeout_obj.connect == pytest.approx(1.0)
    assert timeout_obj.read == pytest.approx(2.0)
    assert timeout_obj.write == pytest.approx(2.0)
    assert timeout_obj.pool == pytest.approx(1.0)

    await transport.close()
    assert fake_client.closed is True


@pytest.mark.asyncio
async def test_httpx_transport_redirect_query_allows_follow_and_post_to_get_conversion(monkeypatch):
    _patch_settings(monkeypatch, allowed_hosts=("app.srail.or.kr",), trust_env=False)
    fake_client = _FakeHttpxClient(
        responses=[
            _FakeHttpxResponse(status_code=302, headers={"location": "/next"}),
            _FakeHttpxResponse(status_code=200, text='{"ok":true}'),
        ]
    )
    monkeypatch.setattr(transport_module.httpx, "AsyncClient", lambda **_: fake_client)

    transport = HttpxTransport()
    response = await transport.request(
        method="POST",
        url="https://app.srail.or.kr:443/start",
        json_body={"k": "v"},
        data={"x": "1"},
        params={"page": 1},
        operation="query",
    )

    assert response.status_code == 200
    assert len(fake_client.calls) == 2
    first, second = fake_client.calls
    assert first["method"] == "POST"
    assert second["method"] == "GET"
    assert second["url"] == "https://app.srail.or.kr:443/next"
    assert second["json"] is None
    assert second["data"] is None
    assert second["params"] is None


@pytest.mark.asyncio
async def test_httpx_transport_redirect_missing_location_returns_original_redirect_response(monkeypatch):
    _patch_settings(monkeypatch, allowed_hosts=("app.srail.or.kr",), trust_env=False)
    fake_client = _FakeHttpxClient(responses=[_FakeHttpxResponse(status_code=302, text="redirect-no-location")])
    monkeypatch.setattr(transport_module.httpx, "AsyncClient", lambda **_: fake_client)

    transport = HttpxTransport()
    response = await transport.request(
        method="GET",
        url="https://app.srail.or.kr/start",
        operation="query",
    )

    assert response.status_code == 302
    assert response.text == "redirect-no-location"
    assert len(fake_client.calls) == 1


@pytest.mark.asyncio
async def test_httpx_transport_blocks_redirect_for_side_effecting_operations(monkeypatch):
    _patch_settings(monkeypatch, allowed_hosts=("app.srail.or.kr",), trust_env=False)
    fake_client = _FakeHttpxClient(
        responses=[_FakeHttpxResponse(status_code=302, headers={"location": "/next"})]
    )
    monkeypatch.setattr(transport_module.httpx, "AsyncClient", lambda **_: fake_client)

    transport = HttpxTransport()
    with pytest.raises(RuntimeError, match="redirect blocked"):
        await transport.request(
            method="POST",
            url="https://app.srail.or.kr/start",
            operation="execute",
        )


@pytest.mark.asyncio
async def test_httpx_transport_blocks_disallowed_hosts_before_network_call(monkeypatch):
    _patch_settings(monkeypatch, allowed_hosts=("app.srail.or.kr",), trust_env=False)
    fake_client = _FakeHttpxClient(responses=[_FakeHttpxResponse(status_code=200)])
    monkeypatch.setattr(transport_module.httpx, "AsyncClient", lambda **_: fake_client)

    transport = HttpxTransport()
    with pytest.raises(RuntimeError, match="allowlisted"):
        await transport.request(
            method="GET",
            url="https://evil.example.com/path",
            operation="query",
        )
    assert fake_client.calls == []


@pytest.mark.asyncio
async def test_httpx_transport_blocks_redirect_targets_outside_allowlist(monkeypatch):
    _patch_settings(monkeypatch, allowed_hosts=("app.srail.or.kr",), trust_env=False)
    fake_client = _FakeHttpxClient(
        responses=[_FakeHttpxResponse(status_code=302, headers={"location": "https://evil.example.com/next"})]
    )
    monkeypatch.setattr(transport_module.httpx, "AsyncClient", lambda **_: fake_client)

    transport = HttpxTransport()
    with pytest.raises(RuntimeError, match="allowlisted"):
        await transport.request(
            method="GET",
            url="https://app.srail.or.kr/start",
            operation="query",
        )
    assert len(fake_client.calls) == 1


@pytest.mark.asyncio
async def test_httpx_transport_enforces_redirect_hop_limit(monkeypatch):
    _patch_settings(monkeypatch, allowed_hosts=("app.srail.or.kr",), trust_env=False)
    hops = transport_module._MAX_REDIRECT_HOPS + 1
    fake_client = _FakeHttpxClient(
        responses=[_FakeHttpxResponse(status_code=302, headers={"location": "/loop"}) for _ in range(hops)]
    )
    monkeypatch.setattr(transport_module.httpx, "AsyncClient", lambda **_: fake_client)

    transport = HttpxTransport()
    with pytest.raises(RuntimeError, match="hop limit exceeded"):
        await transport.request(
            method="GET",
            url="https://app.srail.or.kr/start",
            operation="query",
        )
    assert len(fake_client.calls) == hops


@pytest.mark.asyncio
async def test_curl_transport_resolves_impersonate_from_available_browser_types(monkeypatch):
    _patch_settings(monkeypatch, allowed_hosts=("app.srail.or.kr",), trust_env=False)
    session = _FakeCurlSession(responses=[_FakeCurlResponse(status_code=200)])
    captured = _install_fake_curl_module(
        monkeypatch,
        session=session,
        browser_type_attrs={"chrome124": "chrome124"},
    )
    fallback = _FallbackTransport(response=TransportResponse(status_code=200, text="{}", headers={}))
    monkeypatch.setattr(transport_module, "HttpxTransport", lambda: fallback)

    transport = CurlCffiTransport(impersonate="chrome131_android")
    assert transport._impersonate == "chrome124"
    assert captured["impersonate"] == "chrome124"


@pytest.mark.asyncio
async def test_curl_transport_redirect_query_routes_to_httpx_fallback(monkeypatch):
    _patch_settings(monkeypatch, allowed_hosts=("app.srail.or.kr",), trust_env=False)
    session = _FakeCurlSession(responses=[_FakeCurlResponse(status_code=302, headers={"location": "/next"})])
    _install_fake_curl_module(monkeypatch, session=session, browser_type_attrs={"chrome124": "chrome124"})
    fallback = _FallbackTransport(response=TransportResponse(status_code=204, text="", headers={}))
    monkeypatch.setattr(transport_module, "HttpxTransport", lambda: fallback)

    transport = CurlCffiTransport(impersonate="chrome131_android")
    response = await transport.request(
        method="POST",
        url="https://app.srail.or.kr/start",
        data={"x": "1"},
        timeout=2.0,
        operation="query",
    )

    assert response.status_code == 204
    assert len(session.calls) == 1
    assert session.calls[0]["allow_redirects"] is False

    assert len(fallback.calls) == 1
    assert fallback.calls[0]["method"] == "GET"
    assert fallback.calls[0]["url"] == "https://app.srail.or.kr/next"
    assert fallback.calls[0]["params"] is None


@pytest.mark.asyncio
async def test_curl_transport_blocks_redirect_for_side_effecting_operations(monkeypatch):
    _patch_settings(monkeypatch, allowed_hosts=("app.srail.or.kr",), trust_env=False)
    session = _FakeCurlSession(responses=[_FakeCurlResponse(status_code=302, headers={"location": "/next"})])
    _install_fake_curl_module(monkeypatch, session=session, browser_type_attrs={"chrome124": "chrome124"})
    fallback = _FallbackTransport(response=TransportResponse(status_code=200, text="{}", headers={}))
    monkeypatch.setattr(transport_module, "HttpxTransport", lambda: fallback)

    transport = CurlCffiTransport(impersonate="chrome131_android")
    with pytest.raises(RuntimeError, match="redirect blocked"):
        await transport.request(
            method="POST",
            url="https://app.srail.or.kr/start",
            operation="execute",
        )
    assert fallback.calls == []


@pytest.mark.asyncio
async def test_curl_transport_impersonation_error_falls_back_to_httpx(monkeypatch):
    _patch_settings(monkeypatch, allowed_hosts=("app.srail.or.kr",), trust_env=False)
    session = _FakeCurlSession(error=RuntimeError("Impersonating chrome131_android is not supported"))
    _install_fake_curl_module(monkeypatch, session=session, browser_type_attrs={"chrome124": "chrome124"})
    fallback = _FallbackTransport(response=TransportResponse(status_code=202, text="{}", headers={}))
    monkeypatch.setattr(transport_module, "HttpxTransport", lambda: fallback)

    transport = CurlCffiTransport(impersonate="chrome131_android")
    response = await transport.request(
        method="GET",
        url="https://app.srail.or.kr/path",
        params={"page": 1},
        operation="query",
    )

    assert response.status_code == 202
    assert len(fallback.calls) == 1
    assert fallback.calls[0]["url"] == "https://app.srail.or.kr/path"
    assert fallback.calls[0]["params"] == {"page": 1}


@pytest.mark.asyncio
async def test_curl_transport_non_impersonation_errors_propagate(monkeypatch):
    _patch_settings(monkeypatch, allowed_hosts=("app.srail.or.kr",), trust_env=False)
    session = _FakeCurlSession(error=RuntimeError("network explosion"))
    _install_fake_curl_module(monkeypatch, session=session, browser_type_attrs={"chrome124": "chrome124"})
    fallback = _FallbackTransport(response=TransportResponse(status_code=200, text="{}", headers={}))
    monkeypatch.setattr(transport_module, "HttpxTransport", lambda: fallback)

    transport = CurlCffiTransport(impersonate="chrome131_android")
    with pytest.raises(RuntimeError, match="network explosion"):
        await transport.request(
            method="GET",
            url="https://app.srail.or.kr/path",
            operation="query",
        )
    assert fallback.calls == []


@pytest.mark.asyncio
async def test_curl_transport_close_best_effort_closes_fallback(monkeypatch):
    _patch_settings(monkeypatch, allowed_hosts=("app.srail.or.kr",), trust_env=False)
    session = _FakeCurlSession(responses=[_FakeCurlResponse(status_code=200)])
    session.close_error = RuntimeError("close failure")
    _install_fake_curl_module(monkeypatch, session=session, browser_type_attrs={"chrome124": "chrome124"})
    fallback = _FallbackTransport(response=TransportResponse(status_code=200, text="{}", headers={}))
    monkeypatch.setattr(transport_module, "HttpxTransport", lambda: fallback)

    transport = CurlCffiTransport(impersonate="chrome131_android")
    await transport.close()

    assert session.close_called is True
    assert fallback.closed is True
