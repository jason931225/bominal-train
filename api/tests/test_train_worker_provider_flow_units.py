from __future__ import annotations

from datetime import datetime, timezone
from types import SimpleNamespace
from uuid import uuid4

import pytest

from app.modules.train import worker as train_worker
from app.modules.train.providers.base import ProviderOutcome, ProviderSchedule


@pytest.fixture(autouse=True)
def _enable_payment_for_worker_provider_flow(monkeypatch):
    monkeypatch.setattr(train_worker.settings, "payment_enabled", True)


class _Limiter:
    async def acquire_provider_call(self, **_kwargs):  # noqa: ANN003
        return SimpleNamespace(waited_ms=7, rounds=2)


def _spec(*, auto_pay: bool = True) -> dict:
    return {
        "dep": "수서",
        "arr": "부산",
        "date": "2026-02-23",
        "seat_class": "general",
        "passengers": {"adults": 1, "children": 0},
        "auto_pay": auto_pay,
    }


def _ranked(schedule_id: str = "SRT-1") -> list[dict]:
    return [{"schedule_id": schedule_id, "rank": 1}]


def _schedule(*, provider: str = "SRT", schedule_id: str = "SRT-1", general: bool = True) -> ProviderSchedule:
    return ProviderSchedule(
        schedule_id=schedule_id,
        provider=provider,
        dep="수서",
        arr="부산",
        departure_at=datetime(2026, 2, 23, 9, 0, tzinfo=timezone.utc),
        arrival_at=datetime(2026, 2, 23, 11, 0, tzinfo=timezone.utc),
        train_no="101",
        availability={"general": general, "special": False},
        metadata={},
    )


@pytest.mark.asyncio
async def test_provider_search_and_reserve_handles_login_and_search_failure_paths(monkeypatch):
    class _LoginBoomClient:
        async def login(self, **_kwargs):  # noqa: ANN003
            raise RuntimeError("transport")

    monkeypatch.setattr(train_worker, "get_provider_client", lambda _provider: _LoginBoomClient())
    result_login = await train_worker._provider_search_and_reserve(
        provider="SRT",
        ranked_for_provider=_ranked(),
        spec=_spec(),
        auto_pay_enabled=True,
        task_user_id=uuid4(),
        credentials={"username": "u", "password": "p"},
        limiter=_Limiter(),
    )
    assert result_login.candidate is None
    assert result_login.retryable is True
    assert result_login.attempts[0].error_code == "provider_login_transport_error"

    class _SearchFailClient:
        async def login(self, **_kwargs):  # noqa: ANN003
            return ProviderOutcome(ok=True)

        async def search(self, **_kwargs):  # noqa: ANN003
            return ProviderOutcome(ok=False, retryable=False, error_code="search_failed", error_message_safe="failed")

    monkeypatch.setattr(train_worker, "get_provider_client", lambda _provider: _SearchFailClient())
    result_search = await train_worker._provider_search_and_reserve(
        provider="SRT",
        ranked_for_provider=_ranked(),
        spec=_spec(),
        auto_pay_enabled=True,
        task_user_id=uuid4(),
        credentials={"username": "u", "password": "p"},
        limiter=_Limiter(),
    )
    assert result_search.candidate is None
    assert result_search.retryable is False
    assert result_search.attempts[-1].error_code == "search_failed"


@pytest.mark.asyncio
async def test_provider_search_and_reserve_handles_no_seat_and_reserve_exception(monkeypatch):
    class _NoSeatClient:
        async def login(self, **_kwargs):  # noqa: ANN003
            return ProviderOutcome(ok=True)

        async def search(self, **_kwargs):  # noqa: ANN003
            return ProviderOutcome(ok=True, data={"schedules": [_schedule(general=False)]})

    monkeypatch.setattr(train_worker, "get_provider_client", lambda _provider: _NoSeatClient())
    no_seat = await train_worker._provider_search_and_reserve(
        provider="SRT",
        ranked_for_provider=_ranked(),
        spec=_spec(),
        auto_pay_enabled=True,
        task_user_id=uuid4(),
        credentials={"username": "u", "password": "p"},
        limiter=_Limiter(),
    )
    assert no_seat.candidate is None
    assert no_seat.retryable is True
    assert no_seat.attempts[-1].error_code == "seat_unavailable"

    class _ReserveBoomClient:
        async def login(self, **_kwargs):  # noqa: ANN003
            return ProviderOutcome(ok=True)

        async def search(self, **_kwargs):  # noqa: ANN003
            return ProviderOutcome(ok=True, data={"schedules": [_schedule()]})

        async def reserve(self, **_kwargs):  # noqa: ANN003
            raise RuntimeError("reserve transport")

    monkeypatch.setattr(train_worker, "get_provider_client", lambda _provider: _ReserveBoomClient())
    reserve_boom = await train_worker._provider_search_and_reserve(
        provider="SRT",
        ranked_for_provider=_ranked(),
        spec=_spec(),
        auto_pay_enabled=True,
        task_user_id=uuid4(),
        credentials={"username": "u", "password": "p"},
        limiter=_Limiter(),
    )
    assert reserve_boom.candidate is None
    assert reserve_boom.retryable is True
    assert reserve_boom.attempts[-1].error_code == "provider_transport_error"


@pytest.mark.asyncio
async def test_provider_search_and_reserve_auth_relogin_and_non_payment_expiry_retry(monkeypatch):
    class _Client:
        def __init__(self) -> None:
            self.login_calls = 0
            self.reserve_calls = 0

        async def login(self, **_kwargs):  # noqa: ANN003
            self.login_calls += 1
            return ProviderOutcome(ok=True)

        async def search(self, **_kwargs):  # noqa: ANN003
            return ProviderOutcome(ok=True, data={"schedules": [_schedule()]})

        async def reserve(self, **_kwargs):  # noqa: ANN003
            self.reserve_calls += 1
            if self.reserve_calls == 1:
                return ProviderOutcome(ok=False, retryable=False, error_code="login_required", error_message_safe="login needed")
            return ProviderOutcome(ok=True, data={"reservation_id": "PNR-OK"})

    client = _Client()
    monkeypatch.setattr(train_worker, "get_provider_client", lambda _provider: client)
    relogin = await train_worker._provider_search_and_reserve(
        provider="SRT",
        ranked_for_provider=_ranked(),
        spec=_spec(),
        auto_pay_enabled=True,
        task_user_id=uuid4(),
        credentials={"username": "u", "password": "p"},
        limiter=_Limiter(),
    )
    assert relogin.candidate is not None
    assert relogin.candidate.reservation_id == "PNR-OK"
    assert relogin.attempts[-1].meta_json_safe["auth_relogin_retry"] is True

    class _ExpiryClient:
        async def login(self, **_kwargs):  # noqa: ANN003
            return ProviderOutcome(ok=True)

        async def search(self, **_kwargs):  # noqa: ANN003
            return ProviderOutcome(ok=True, data={"schedules": [_schedule()]})

        async def reserve(self, **_kwargs):  # noqa: ANN003
            return ProviderOutcome(
                ok=False,
                retryable=False,
                error_code="payment_expired",
                error_message_safe="non-payment window expired",
            )

    monkeypatch.setattr(train_worker, "get_provider_client", lambda _provider: _ExpiryClient())
    expiry_retry = await train_worker._provider_search_and_reserve(
        provider="SRT",
        ranked_for_provider=_ranked(),
        spec=_spec(auto_pay=False),
        auto_pay_enabled=False,
        task_user_id=uuid4(),
        credentials={"username": "u", "password": "p"},
        limiter=_Limiter(),
    )
    assert expiry_retry.candidate is None
    assert expiry_retry.retryable is True
    assert expiry_retry.attempts[-1].meta_json_safe["non_payment_expiry_retry"] is True


@pytest.mark.asyncio
async def test_worker_notification_and_mark_helpers_cover_terminal_paths(monkeypatch):
    commits = {"count": 0}

    class _DB:
        async def get(self, _model, _id):  # noqa: ANN001
            return SimpleNamespace(email="user@example.com")

        async def commit(self):
            commits["count"] += 1

    task = SimpleNamespace(
        id=uuid4(),
        user_id=uuid4(),
        spec_json={"notify": True, "dep": "수서", "arr": "부산", "item_code": "SRT101", "item_date": "2026-02-23"},
        updated_at=None,
        state="RUNNING",
        failed_at=None,
        completed_at=None,
    )

    async def _enqueue_raise(_payload):  # noqa: ANN001
        raise RuntimeError("mail queue down")

    monkeypatch.setattr(train_worker, "enqueue_template_email", _enqueue_raise)
    await train_worker._enqueue_terminal_notification(_DB(), task=task, final_state="FAILED")

    async def _enqueue_ok(_payload):  # noqa: ANN001
        return "job-1"

    monkeypatch.setattr(train_worker, "enqueue_template_email", _enqueue_ok)
    await train_worker._enqueue_terminal_notification(_DB(), task=task, final_state="COMPLETED")
    assert task.spec_json.get("notify_email_job_id") == "job-1"
    assert commits["count"] >= 1

    async def _noop_notify(_db, *, task, final_state):  # noqa: ANN001
        task.spec_json["final"] = final_state

    monkeypatch.setattr(train_worker, "_enqueue_terminal_notification", _noop_notify)
    monkeypatch.setattr(train_worker, "utc_now", lambda: datetime(2026, 2, 22, 12, 0, tzinfo=timezone.utc))
    await train_worker._mark_expired(_DB(), task)
    assert task.state == "EXPIRED"
    await train_worker._mark_failed(_DB(), task)
    assert task.state == "FAILED"
    await train_worker._mark_completed(_DB(), task)
    assert task.state == "COMPLETED"
