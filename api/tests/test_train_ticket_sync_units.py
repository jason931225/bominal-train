from __future__ import annotations

from types import SimpleNamespace
from uuid import uuid4

import pytest

from app.modules.train.providers.base import ProviderOutcome
from app.modules.train import ticket_sync


class _Limiter:
    def __init__(self, waited_ms: int = 5) -> None:
        self.waited_ms = waited_ms
        self.calls = 0

    async def acquire_provider_call(self, **_kwargs):  # noqa: ANN003
        self.calls += 1
        return SimpleNamespace(waited_ms=self.waited_ms)


class _Client:
    def __init__(self, *, reservations_outcome):
        self._reservations_outcome = reservations_outcome

    async def get_reservations(self, **_kwargs):  # noqa: ANN003
        return self._reservations_outcome


def test_status_from_snapshot_branch_matrix():
    assert ticket_sync._status_from_snapshot(paid=True, waiting=False, expired=False, reservation_found=True) == "paid"
    assert ticket_sync._status_from_snapshot(paid=False, waiting=False, expired=True, reservation_found=True) == "expired"
    assert ticket_sync._status_from_snapshot(paid=False, waiting=True, expired=False, reservation_found=True) == "waiting"
    assert (
        ticket_sync._status_from_snapshot(paid=False, waiting=False, expired=False, reservation_found=True)
        == "awaiting_payment"
    )
    assert (
        ticket_sync._status_from_snapshot(paid=False, waiting=False, expired=False, reservation_found=False)
        == "reservation_not_found"
    )


@pytest.mark.asyncio
async def test_fetch_ticket_sync_snapshot_uses_reservation_fallback_ticket_rows_and_optional_fields(monkeypatch):
    monkeypatch.setattr(ticket_sync, "utc_now", lambda: SimpleNamespace(isoformat=lambda: "2026-02-22T12:00:00+00:00"))

    reservation = {
        "reservation_id": "PNR-100",
        "paid": False,
        "waiting": False,
        "expired": False,
        "payment_deadline_at": "2026-02-22T13:00:00+09:00",
        "dep": "수서",
        "arr": "부산",
        "train_no": "SRT-101",
        "journey_no": "1",
        "journey_cnt": "1",
        "rsv_chg_no": "0",
        "wct_no": "W1",
        "seat_count": 2,
        "tickets": [{"car": "3", "seat": "4A"}],
    }
    reservations_outcome = ProviderOutcome(
        ok=True,
        data={"reservations": [reservation], "http_trace": {"authorization": "Bearer secret"}},
    )
    client = _Client(reservations_outcome=reservations_outcome)
    limiter = _Limiter(waited_ms=12)

    snapshot = await ticket_sync.fetch_ticket_sync_snapshot(
        client=client,
        provider="SRT",
        reservation_id="PNR-100",
        user_id=uuid4(),
        limiter=limiter,
    )

    assert snapshot["status"] == "awaiting_payment"
    assert snapshot["payment_deadline_at"] == reservation["payment_deadline_at"]
    assert snapshot["dep"] == "수서"
    assert snapshot["arr"] == "부산"
    assert snapshot["train_no"] == "SRT-101"
    assert snapshot["journey_no"] == "1"
    assert snapshot["journey_cnt"] == "1"
    assert snapshot["rsv_chg_no"] == "0"
    assert snapshot["wct_no"] == "W1"
    assert snapshot["seat_count"] == 2
    assert snapshot["tickets"]
    assert snapshot["provider_sync"]["reservations_rate_limit_wait_ms"] == 12
    assert "provider_http" in snapshot
    assert limiter.calls == 1


@pytest.mark.asyncio
async def test_fetch_ticket_sync_snapshot_handles_provider_failures_and_transport_errors():
    reservations_outcome = ProviderOutcome(
        ok=False,
        error_code="reservation_failure",
        error_message_safe="reservation failed",
        data={"http_trace": {"token": "abc"}},
    )
    client = _Client(reservations_outcome=reservations_outcome)

    snapshot = await ticket_sync.fetch_ticket_sync_snapshot(
        client=client,
        provider="KTX",
        reservation_id="PNR-MISSING",
        user_id=uuid4(),
        limiter=None,
    )

    assert snapshot["status"] == "reservation_not_found"
    assert snapshot["provider_sync"]["reservations_ok"] is False
    assert snapshot["provider_sync"]["reservations_error"] == "reservation failed"
