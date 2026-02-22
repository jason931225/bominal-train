from __future__ import annotations

from datetime import date
from types import SimpleNamespace

import pytest

from app.modules.train.providers import factory
from app.modules.train.providers.base import ProviderOutcome, TrainProviderClient
from app.modules.train.providers.transport import ProviderTransportError


class _Delegate(TrainProviderClient):
    provider_name = "SRT"

    def __init__(self, *, raise_methods: set[str] | None = None):
        self.raise_methods = raise_methods or set()

    def _maybe_raise(self, method: str) -> None:
        if method in self.raise_methods:
            raise ProviderTransportError(
                retryable=True,
                error_code=f"{method}_timeout",
                error_message_safe=f"{method} timed out",
            )

    async def login(self, *, user_id: str, credentials: dict[str, str] | None = None) -> ProviderOutcome:
        self._maybe_raise("login")
        return ProviderOutcome(ok=True, data={"method": "login", "user_id": user_id, "credentials": credentials or {}})

    async def search(
        self,
        *,
        dep: str,
        arr: str,
        date_value: date,  # noqa: ARG002
        time_window_start: str,  # noqa: ARG002
        time_window_end: str,  # noqa: ARG002
        user_id: str,
    ) -> ProviderOutcome:
        self._maybe_raise("search")
        return ProviderOutcome(ok=True, data={"method": "search", "dep": dep, "arr": arr, "user_id": user_id})

    async def reserve(
        self,
        *,
        schedule_id: str,
        seat_class: str,
        passengers: dict[str, int],
        user_id: str,
    ) -> ProviderOutcome:
        self._maybe_raise("reserve")
        return ProviderOutcome(ok=True, data={"schedule_id": schedule_id, "seat_class": seat_class, "passengers": passengers, "user_id": user_id})

    async def reserve_standby(
        self,
        *,
        schedule_id: str,
        seat_class: str,
        passengers: dict[str, int],
        user_id: str,
    ) -> ProviderOutcome:
        self._maybe_raise("reserve_standby")
        return ProviderOutcome(ok=True, data={"schedule_id": schedule_id, "seat_class": seat_class, "passengers": passengers, "user_id": user_id})

    async def pay(
        self,
        *,
        reservation_id: str,
        user_id: str,
        payment_card: dict | None = None,
    ) -> ProviderOutcome:
        self._maybe_raise("pay")
        return ProviderOutcome(ok=True, data={"reservation_id": reservation_id, "user_id": user_id, "payment_card": payment_card or {}})

    async def cancel(self, *, artifact_data: dict, user_id: str) -> ProviderOutcome:
        self._maybe_raise("cancel")
        return ProviderOutcome(ok=True, data={"artifact_data": artifact_data, "user_id": user_id})

    async def get_reservations(
        self,
        *,
        user_id: str,
        paid_only: bool = False,
        reservation_id: str | None = None,
    ) -> ProviderOutcome:
        self._maybe_raise("get_reservations")
        return ProviderOutcome(ok=True, data={"user_id": user_id, "paid_only": paid_only, "reservation_id": reservation_id})

    async def ticket_info(self, *, reservation_id: str, user_id: str) -> ProviderOutcome:
        self._maybe_raise("ticket_info")
        return ProviderOutcome(ok=True, data={"reservation_id": reservation_id, "user_id": user_id})


@pytest.mark.asyncio
async def test_failsafe_provider_client_passthrough_and_error_mapping():
    wrapped = factory.FailSafeProviderClient(_Delegate())
    assert wrapped.provider_name == "SRT"

    ok_login = await wrapped.login(user_id="u1", credentials={"id": "user"})
    ok_search = await wrapped.search(
        dep="수서",
        arr="부산",
        date_value=date(2026, 2, 22),
        time_window_start="08:00",
        time_window_end="12:00",
        user_id="u1",
    )
    ok_reserve = await wrapped.reserve(
        schedule_id="s1",
        seat_class="general",
        passengers={"adult": 1},
        user_id="u1",
    )
    ok_standby = await wrapped.reserve_standby(
        schedule_id="s2",
        seat_class="special",
        passengers={"adult": 1},
        user_id="u1",
    )
    ok_pay = await wrapped.pay(reservation_id="r1", user_id="u1", payment_card={"token": "x"})
    ok_cancel = await wrapped.cancel(artifact_data={"reservation_id": "r1"}, user_id="u1")
    ok_reservations = await wrapped.get_reservations(user_id="u1", paid_only=True, reservation_id="r1")
    ok_ticket_info = await wrapped.ticket_info(reservation_id="r1", user_id="u1")

    assert all(
        outcome.ok
        for outcome in [
            ok_login,
            ok_search,
            ok_reserve,
            ok_standby,
            ok_pay,
            ok_cancel,
            ok_reservations,
            ok_ticket_info,
        ]
    )

    for method in [
        "login",
        "search",
        "reserve",
        "reserve_standby",
        "pay",
        "cancel",
        "get_reservations",
        "ticket_info",
    ]:
        error_wrapped = factory.FailSafeProviderClient(_Delegate(raise_methods={method}))
        call = {
            "login": lambda: error_wrapped.login(user_id="u1"),
            "search": lambda: error_wrapped.search(
                dep="수서",
                arr="부산",
                date_value=date(2026, 2, 22),
                time_window_start="08:00",
                time_window_end="12:00",
                user_id="u1",
            ),
            "reserve": lambda: error_wrapped.reserve(
                schedule_id="s1", seat_class="general", passengers={"adult": 1}, user_id="u1"
            ),
            "reserve_standby": lambda: error_wrapped.reserve_standby(
                schedule_id="s2", seat_class="special", passengers={"adult": 1}, user_id="u1"
            ),
            "pay": lambda: error_wrapped.pay(reservation_id="r1", user_id="u1"),
            "cancel": lambda: error_wrapped.cancel(artifact_data={"reservation_id": "r1"}, user_id="u1"),
            "get_reservations": lambda: error_wrapped.get_reservations(user_id="u1", paid_only=False),
            "ticket_info": lambda: error_wrapped.ticket_info(reservation_id="r1", user_id="u1"),
        }[method]

        outcome = await call()
        assert outcome.ok is False
        assert outcome.retryable is True
        assert outcome.error_code == f"{method}_timeout"


def test_build_transport_mode_selection(monkeypatch):
    monkeypatch.setattr(
        factory,
        "get_settings",
        lambda: SimpleNamespace(train_provider_transport="curl_cffi"),
    )
    monkeypatch.setattr(factory, "CurlCffiTransport", lambda impersonate: ("curl", impersonate))
    monkeypatch.setattr(factory, "HttpxTransport", lambda: ("httpx", None))
    monkeypatch.setattr(factory, "ResilientTransport", lambda base_transport, provider: ("resilient", base_transport, provider))

    assert factory._build_transport("SRT") == ("resilient", ("curl", "chrome"), "SRT")
    assert factory._build_transport("KTX") == ("resilient", ("curl", "chrome131_android"), "KTX")

    monkeypatch.setattr(
        factory,
        "get_settings",
        lambda: SimpleNamespace(train_provider_transport="httpx"),
    )
    assert factory._build_transport("SRT") == ("resilient", ("httpx", None), "SRT")

    monkeypatch.setattr(
        factory,
        "get_settings",
        lambda: SimpleNamespace(train_provider_transport="auto"),
    )

    def _raise_curl(*_args, **_kwargs):  # noqa: ANN002, ANN003
        raise RuntimeError("curl unavailable")

    monkeypatch.setattr(factory, "CurlCffiTransport", _raise_curl)
    assert factory._build_transport("SRT") == ("resilient", ("httpx", None), "SRT")


def test_build_live_client_dispatch(monkeypatch):
    monkeypatch.setattr(factory, "_build_transport", lambda provider: f"transport:{provider}")
    monkeypatch.setattr(factory, "SRTClient", lambda transport: ("srt", transport))
    monkeypatch.setattr(factory, "KTXClient", lambda transport: ("ktx", transport))

    assert factory._build_live_client("SRT") == ("srt", "transport:SRT")
    assert factory._build_live_client("KTX") == ("ktx", "transport:KTX")

    with pytest.raises(ValueError, match="Unsupported provider"):
        factory._build_live_client("UNKNOWN")


def test_get_provider_client_mode_selection(monkeypatch):
    monkeypatch.setattr(factory, "FailSafeProviderClient", lambda client: ("safe", client))
    monkeypatch.setattr(factory, "MockSRTClient", lambda: "mock-srt")
    monkeypatch.setattr(factory, "MockKTXClient", lambda: "mock-ktx")
    monkeypatch.setattr(factory, "HybridSRTClient", lambda live_client: ("hybrid-srt", live_client))
    monkeypatch.setattr(factory, "HybridKTXClient", lambda live_client: ("hybrid-ktx", live_client))
    monkeypatch.setattr(factory, "_build_live_client", lambda provider: f"live:{provider}")

    monkeypatch.setattr(factory, "get_settings", lambda: SimpleNamespace(train_provider_mode="mock"))
    assert factory.get_provider_client("SRT") == ("safe", "mock-srt")
    assert factory.get_provider_client("KTX") == ("safe", "mock-ktx")
    with pytest.raises(ValueError, match="Unsupported provider"):
        factory.get_provider_client("OTHER")

    monkeypatch.setattr(factory, "get_settings", lambda: SimpleNamespace(train_provider_mode="hybrid"))
    assert factory.get_provider_client("SRT") == ("safe", ("hybrid-srt", "live:SRT"))
    assert factory.get_provider_client("KTX") == ("safe", ("hybrid-ktx", "live:KTX"))
    with pytest.raises(ValueError, match="Unsupported provider"):
        factory.get_provider_client("OTHER")

    monkeypatch.setattr(factory, "get_settings", lambda: SimpleNamespace(train_provider_mode="live"))
    assert factory.get_provider_client("SRT") == ("safe", "live:SRT")
    assert factory.get_provider_client("KTX") == ("safe", "live:KTX")
    with pytest.raises(ValueError, match="Unsupported provider"):
        factory.get_provider_client("OTHER")
