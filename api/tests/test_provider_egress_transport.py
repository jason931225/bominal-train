from __future__ import annotations

from dataclasses import dataclass
from typing import Any

import httpx
import pytest

from app.modules.train.providers.transport import (
    ProviderTransportError,
    ResilientTransport,
    TransportResponse,
    is_retryable_status_code,
)


@dataclass(slots=True)
class _Action:
    status_code: int | None = None
    error: Exception | None = None


class _SequenceTransport:
    def __init__(self, actions: list[_Action]) -> None:
        self._actions = list(actions)
        self.requests: list[dict[str, Any]] = []

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


def test_retryable_status_code_classification():
    assert is_retryable_status_code(429) is True
    assert is_retryable_status_code(500) is True
    assert is_retryable_status_code(400) is False
