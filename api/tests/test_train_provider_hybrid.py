from __future__ import annotations

import json
from datetime import date, datetime, timedelta

import pytest

from app.modules.train.providers.base import ProviderOutcome, ProviderSchedule
from app.modules.train.providers import hybrid as hybrid_mod
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


class _LiveNonRetryableFailingClient:
    provider_name = "SRT"

    async def login(self, *, user_id: str, credentials: dict[str, str] | None = None) -> ProviderOutcome:
        return ProviderOutcome(ok=True)

    async def search(self, **kwargs) -> ProviderOutcome:
        return ProviderOutcome(
            ok=False,
            retryable=False,
            error_code="invalid_credentials",
            error_message_safe="invalid credentials",
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


@pytest.mark.asyncio
async def test_hybrid_provider_does_not_fallback_on_non_retryable_search_error():
    hybrid = _HybridProviderClient(
        live_client=_LiveNonRetryableFailingClient(),
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

    assert result.ok is False
    assert result.retryable is False
    assert result.error_code == "invalid_credentials"


class _DynamicOutcomeClient:
    provider_name = "SRT"

    def __init__(self, outcomes: dict[str, ProviderOutcome]):
        self.outcomes = outcomes
        self.calls: list[str] = []

    def _result(self, method: str) -> ProviderOutcome:
        return self.outcomes.get(method, ProviderOutcome(ok=True))

    async def login(self, **kwargs) -> ProviderOutcome:  # noqa: ANN003
        self.calls.append("login")
        return self._result("login")

    async def search(self, **kwargs) -> ProviderOutcome:  # noqa: ANN003
        self.calls.append("search")
        return self._result("search")

    async def reserve(self, **kwargs) -> ProviderOutcome:  # noqa: ANN003
        self.calls.append("reserve")
        return self._result("reserve")

    async def reserve_standby(self, **kwargs) -> ProviderOutcome:  # noqa: ANN003
        self.calls.append("reserve_standby")
        return self._result("reserve_standby")

    async def pay(self, **kwargs) -> ProviderOutcome:  # noqa: ANN003
        self.calls.append("pay")
        return self._result("pay")

    async def cancel(self, **kwargs) -> ProviderOutcome:  # noqa: ANN003
        self.calls.append("cancel")
        return self._result("cancel")

    async def get_reservations(self, **kwargs) -> ProviderOutcome:  # noqa: ANN003
        self.calls.append("get_reservations")
        return self._result("get_reservations")

    async def ticket_info(self, **kwargs) -> ProviderOutcome:  # noqa: ANN003
        self.calls.append("ticket_info")
        return self._result("ticket_info")


@pytest.mark.asyncio
async def test_hybrid_login_fallback_and_credentials_behavior():
    live_retryable_fail = _DynamicOutcomeClient({"login": ProviderOutcome(ok=False, retryable=True)})
    mock_success = _DynamicOutcomeClient({"login": ProviderOutcome(ok=True)})
    hybrid = _HybridProviderClient(live_client=live_retryable_fail, mock_client=mock_success)

    fallback_login = await hybrid.login(user_id="u1", credentials=None)
    assert fallback_login.ok is True
    assert mock_success.calls == ["login"]

    live_only = _DynamicOutcomeClient({"login": ProviderOutcome(ok=False, retryable=True)})
    mock_unused = _DynamicOutcomeClient({"login": ProviderOutcome(ok=True)})
    hybrid_live_only = _HybridProviderClient(live_client=live_only, mock_client=mock_unused)
    no_fallback = await hybrid_live_only.login(user_id="u1", credentials={"id": "account"})
    assert no_fallback.ok is False
    assert mock_unused.calls == []


@pytest.mark.asyncio
async def test_hybrid_search_live_success_marks_source():
    departure = datetime(2026, 2, 3, 8, 0, tzinfo=KST)
    schedule = ProviderSchedule(
        schedule_id="live-1",
        provider="SRT",
        dep="수서",
        arr="부산",
        departure_at=departure,
        arrival_at=departure + timedelta(hours=2),
        train_no="S123",
        availability={"general": True, "special": False},
        metadata={},
    )
    live = _DynamicOutcomeClient({"search": ProviderOutcome(ok=True, data={"schedules": [schedule]})})
    mock = _DynamicOutcomeClient({})
    hybrid = _HybridProviderClient(live_client=live, mock_client=mock)

    result = await hybrid.search(
        dep="수서",
        arr="부산",
        date_value=date(2026, 2, 3),
        time_window_start="08:00",
        time_window_end="12:00",
        user_id="u1",
    )

    assert result.ok is True
    assert result.data["schedules"][0].metadata["source"] == "live"
    assert mock.calls == []


@pytest.mark.asyncio
async def test_hybrid_execute_methods_delegate_to_mock():
    live = _DynamicOutcomeClient({})
    mock = _DynamicOutcomeClient(
        {
            "reserve": ProviderOutcome(ok=True, data={"id": "r1"}),
            "reserve_standby": ProviderOutcome(ok=True, data={"id": "std1"}),
            "pay": ProviderOutcome(ok=True, data={"id": "p1"}),
            "cancel": ProviderOutcome(ok=True, data={"id": "c1"}),
        }
    )
    hybrid = _HybridProviderClient(live_client=live, mock_client=mock)

    assert (await hybrid.reserve(schedule_id="s", seat_class="general", passengers={"adult": 1}, user_id="u1")).ok
    assert (await hybrid.reserve_standby(schedule_id="s", seat_class="general", passengers={"adult": 1}, user_id="u1")).ok
    assert (await hybrid.pay(reservation_id="r", user_id="u1", payment_card={"token": "x"})).ok
    assert (await hybrid.cancel(artifact_data={"reservation_id": "r"}, user_id="u1")).ok
    assert mock.calls == ["reserve", "reserve_standby", "pay", "cancel"]


@pytest.mark.asyncio
async def test_hybrid_reservation_and_ticket_fallback_rules():
    live_retryable = _DynamicOutcomeClient(
        {
            "get_reservations": ProviderOutcome(ok=False, retryable=True, error_code="timeout"),
            "ticket_info": ProviderOutcome(ok=False, retryable=True, error_code="timeout"),
        }
    )
    mock = _DynamicOutcomeClient(
        {
            "get_reservations": ProviderOutcome(ok=True, data={"reservations": []}),
            "ticket_info": ProviderOutcome(ok=True, data={"tickets": []}),
        }
    )
    hybrid = _HybridProviderClient(live_client=live_retryable, mock_client=mock)
    assert (await hybrid.get_reservations(user_id="u1")).ok is True
    assert (await hybrid.ticket_info(reservation_id="r1", user_id="u1")).ok is True
    assert mock.calls == ["get_reservations", "ticket_info"]

    live_non_retryable = _DynamicOutcomeClient(
        {
            "get_reservations": ProviderOutcome(ok=False, retryable=False, error_code="invalid"),
            "ticket_info": ProviderOutcome(ok=False, retryable=False, error_code="invalid"),
        }
    )
    mock_unused = _DynamicOutcomeClient({})
    hybrid_no_fallback = _HybridProviderClient(live_client=live_non_retryable, mock_client=mock_unused)
    assert (await hybrid_no_fallback.get_reservations(user_id="u1")).retryable is False
    assert (await hybrid_no_fallback.ticket_info(reservation_id="r1", user_id="u1")).retryable is False
    assert mock_unused.calls == []


@pytest.mark.asyncio
async def test_hybrid_reservation_and_ticket_live_success_short_circuits():
    live = _DynamicOutcomeClient(
        {
            "get_reservations": ProviderOutcome(ok=True, data={"reservations": [{"reservation_id": "r1"}]}),
            "ticket_info": ProviderOutcome(ok=True, data={"reservation_id": "r1", "tickets": [{"seat": "1A"}]}),
        }
    )
    mock = _DynamicOutcomeClient(
        {
            "get_reservations": ProviderOutcome(ok=True, data={"reservations": []}),
            "ticket_info": ProviderOutcome(ok=True, data={"reservation_id": "r1", "tickets": []}),
        }
    )
    hybrid = _HybridProviderClient(live_client=live, mock_client=mock)

    reservations = await hybrid.get_reservations(user_id="u1")
    ticket_info = await hybrid.ticket_info(reservation_id="r1", user_id="u1")

    assert reservations.ok is True
    assert reservations.data["reservations"][0]["reservation_id"] == "r1"
    assert ticket_info.ok is True
    assert ticket_info.data["tickets"][0]["seat"] == "1A"
    assert mock.calls == []


def test_hybrid_provider_default_constructors(monkeypatch):
    sentinel_live_srt = object()
    sentinel_mock_srt = object()
    sentinel_live_ktx = object()
    sentinel_mock_ktx = object()

    monkeypatch.setattr(hybrid_mod, "SRTClient", lambda: sentinel_live_srt)
    monkeypatch.setattr(hybrid_mod, "MockSRTClient", lambda: sentinel_mock_srt)
    monkeypatch.setattr(hybrid_mod, "KTXClient", lambda: sentinel_live_ktx)
    monkeypatch.setattr(hybrid_mod, "MockKTXClient", lambda: sentinel_mock_ktx)

    srt_client = hybrid_mod.HybridSRTClient()
    ktx_client = hybrid_mod.HybridKTXClient()

    assert srt_client._live_client is sentinel_live_srt
    assert srt_client._mock_client is sentinel_mock_srt
    assert ktx_client._live_client is sentinel_live_ktx
    assert ktx_client._mock_client is sentinel_mock_ktx
