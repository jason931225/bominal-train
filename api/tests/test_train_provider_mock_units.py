from __future__ import annotations

from datetime import date

import pytest

from app.modules.train.providers.mock import MockKTXClient, MockSRTClient


@pytest.mark.asyncio
async def test_mock_provider_operations_include_expected_safe_shapes():
    client = MockSRTClient()

    search = await client.search(
        dep="수서",
        arr="부산",
        date_value=date(2026, 2, 23),
        time_window_start="08:00",
        time_window_end="12:00",
        user_id="user-1",
    )
    assert search.ok is True
    schedules = search.data["schedules"]
    assert len(schedules) == 4

    reserve = await client.reserve(
        schedule_id=schedules[0].schedule_id,
        seat_class="general",
        passengers={"adults": 1, "children": 1},
        user_id="user-1",
    )
    assert reserve.ok is True
    assert reserve.data["reservation_id"].startswith("SRT-rsv-")
    assert reserve.data["http_trace"]["endpoint"] == "reserve"

    standby = await client.reserve_standby(
        schedule_id=schedules[0].schedule_id,
        seat_class="special",
        passengers={"adults": 1},
        user_id="user-1",
    )
    assert standby.ok is True
    assert standby.data["reservation_id"].startswith("SRT-std-")
    assert standby.data["status"] == "waiting"

    pay = await client.pay(
        reservation_id=reserve.data["reservation_id"],
        user_id="user-1",
        payment_card={"token": "opaque"},
    )
    assert pay.ok is True
    assert pay.data["payment_id"].startswith("SRT-pay-")
    assert pay.data["ticket_no"].startswith("SRT")
    assert pay.data["http_trace"]["endpoint"] == "pay"

    cancel = await client.cancel(
        artifact_data={"artifact_id": "artifact-1", "reservation_id": reserve.data["reservation_id"]},
        user_id="user-1",
    )
    assert cancel.ok is False
    assert cancel.retryable is False
    assert cancel.error_code == "not_supported"
    assert cancel.data["artifact_id"] == "artifact-1"
    assert cancel.data["http_trace"]["endpoint"] == "cancel"

    reservations = await client.get_reservations(
        user_id="user-1",
        paid_only=True,
        reservation_id=reserve.data["reservation_id"],
    )
    assert reservations.ok is True
    assert reservations.data["reservations"] == []
    assert reservations.data["http_trace"]["request"]["paid_only"] is True

    ticket_info = await client.ticket_info(reservation_id=reserve.data["reservation_id"], user_id="user-1")
    assert ticket_info.ok is True
    assert ticket_info.data["reservation_id"] == reserve.data["reservation_id"]
    assert ticket_info.data["tickets"] == []


@pytest.mark.asyncio
async def test_mock_ktx_provider_uses_ktx_prefixes():
    client = MockKTXClient()

    reserve = await client.reserve(
        schedule_id="sched-1",
        seat_class="general",
        passengers={"adults": 1},
        user_id="user-1",
    )
    pay = await client.pay(
        reservation_id=reserve.data["reservation_id"],
        user_id="user-1",
        payment_card={"token": "opaque"},
    )

    assert reserve.data["reservation_id"].startswith("KTX-rsv-")
    assert pay.data["payment_id"].startswith("KTX-pay-")
    assert pay.data["ticket_no"].startswith("KTX")
