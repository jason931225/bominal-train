from __future__ import annotations

import json
from datetime import date, datetime, timedelta

import pytest

from app.modules.train.providers.base import ProviderOutcome, ProviderSchedule
from app.modules.train.providers.hybrid import _HybridProviderClient
from app.modules.train.providers.ktx_client import KTXClient
from app.modules.train.providers.transport import TransportResponse
from app.modules.train.timezone import KST


class _CaptureTransport:
    def __init__(self, payload: dict):
        self.payload = payload
        self.last_request: dict | None = None

    async def request(self, **kwargs) -> TransportResponse:
        self.last_request = kwargs
        return TransportResponse(
            status_code=200,
            text=json.dumps(self.payload),
            headers={},
        )


class _LiveFailingClient:
    provider_name = "SRT"

    async def login(self, *, user_id: str, credentials: dict[str, str] | None = None) -> ProviderOutcome:
        return ProviderOutcome(ok=True)

    async def search(self, **kwargs) -> ProviderOutcome:
        return ProviderOutcome(
            ok=False,
            retryable=True,
            error_code="provider_unreachable",
            error_message_safe="temporary failure",
        )

    async def reserve(self, **kwargs) -> ProviderOutcome:
        return ProviderOutcome(ok=False)

    async def pay(self, **kwargs) -> ProviderOutcome:
        return ProviderOutcome(ok=False)

    async def cancel(self, **kwargs) -> ProviderOutcome:
        return ProviderOutcome(ok=False)

    async def reserve_standby(self, **kwargs) -> ProviderOutcome:
        return ProviderOutcome(ok=False)

    async def get_reservations(self, **kwargs) -> ProviderOutcome:
        return ProviderOutcome(ok=False)

    async def ticket_info(self, **kwargs) -> ProviderOutcome:
        return ProviderOutcome(ok=False)


class _MockSuccessClient:
    provider_name = "SRT"

    async def login(self, *, user_id: str, credentials: dict[str, str] | None = None) -> ProviderOutcome:
        return ProviderOutcome(ok=True)

    async def search(self, **kwargs) -> ProviderOutcome:
        departure = datetime(2026, 2, 3, 8, 0, tzinfo=KST)
        schedule = ProviderSchedule(
            schedule_id="mock-1",
            provider="SRT",
            dep=kwargs["dep"],
            arr=kwargs["arr"],
            departure_at=departure,
            arrival_at=departure + timedelta(hours=2),
            train_no="S200",
            availability={"general": True, "special": False},
            metadata={},
        )
        return ProviderOutcome(ok=True, data={"schedules": [schedule]})

    async def reserve(self, **kwargs) -> ProviderOutcome:
        return ProviderOutcome(ok=True, data={"reservation_id": "r-1"})

    async def pay(self, **kwargs) -> ProviderOutcome:
        return ProviderOutcome(ok=True, data={"payment_id": "p-1"})

    async def cancel(self, **kwargs) -> ProviderOutcome:
        return ProviderOutcome(ok=False, error_code="not_supported")

    async def reserve_standby(self, **kwargs) -> ProviderOutcome:
        return ProviderOutcome(ok=True, data={"reservation_id": "std-1"})

    async def get_reservations(self, **kwargs) -> ProviderOutcome:
        return ProviderOutcome(ok=True, data={"reservations": []})

    async def ticket_info(self, **kwargs) -> ProviderOutcome:
        return ProviderOutcome(ok=True, data={"reservation_id": "r-1", "tickets": []})


@pytest.mark.asyncio
async def test_ktx_search_uses_query_params():
    payload = {
        "strResult": "SUCC",
        "trn_infos": {
            "trn_info": [
                {
                    "h_trn_no": "305",
                    "h_dpt_dt": "20260203",
                    "h_dpt_tm": "090000",
                    "h_arv_dt": "20260203",
                    "h_arv_tm": "112000",
                    "h_gen_rsv_cd": "11",
                    "h_spe_rsv_cd": "00",
                }
            ]
        },
    }
    transport = _CaptureTransport(payload)
    client = KTXClient(transport=transport)

    result = await client.search(
        dep="서울",
        arr="부산",
        date_value=date(2026, 2, 3),
        time_window_start="08:00",
        time_window_end="12:00",
        user_id="u1",
    )

    assert result.ok is True
    assert transport.last_request is not None
    assert transport.last_request.get("params", {}).get("txtGoStart") == "서울"
    assert transport.last_request.get("data") is None


@pytest.mark.asyncio
async def test_hybrid_provider_falls_back_to_mock_search():
    hybrid = _HybridProviderClient(
        live_client=_LiveFailingClient(),
        mock_client=_MockSuccessClient(),
    )

    result = await hybrid.search(
        dep="수서",
        arr="마산",
        date_value=date(2026, 2, 3),
        time_window_start="08:00",
        time_window_end="12:00",
        user_id="u1",
    )

    assert result.ok is True
    schedules = result.data["schedules"]
    assert len(schedules) == 1
    assert schedules[0].metadata["source"] == "mock-fallback"
    assert schedules[0].metadata["live_error_code"] == "provider_unreachable"
