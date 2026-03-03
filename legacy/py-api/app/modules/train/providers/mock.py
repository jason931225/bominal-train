from __future__ import annotations

import hashlib
from datetime import date, datetime, timedelta
from typing import Any
from uuid import uuid4

from app.modules.train.providers.base import ProviderOutcome, ProviderSchedule
from app.modules.train.timezone import KST


def _stable_schedule_id(provider: str, dep: str, arr: str, departure_at: datetime, train_no: str) -> str:
    raw = f"{provider}|{dep}|{arr}|{departure_at.isoformat()}|{train_no}"
    return hashlib.sha256(raw.encode("utf-8")).hexdigest()[:24]


def _parse_window_start(date_value: date, start: str) -> datetime:
    hour, minute = [int(piece) for piece in start.split(":")]
    return datetime(date_value.year, date_value.month, date_value.day, hour, minute, tzinfo=KST)


class MockProviderBase:
    provider_name = "MOCK"

    async def login(self, *, user_id: str, credentials: dict[str, str] | None = None) -> ProviderOutcome:
        username = credentials.get("username") if credentials else None
        return ProviderOutcome(ok=True, data={"user_id": user_id, "username": username})

    async def search(
        self,
        *,
        dep: str,
        arr: str,
        date_value: date,
        time_window_start: str,
        time_window_end: str,
        user_id: str,
    ) -> ProviderOutcome:
        start_dt = _parse_window_start(date_value, time_window_start)
        schedules: list[ProviderSchedule] = []
        for idx in range(4):
            departure = start_dt + timedelta(minutes=idx * 35)
            arrival = departure + timedelta(minutes=65 + idx * 3)
            train_no = f"{self.provider_name[:1]}{200 + idx}"
            schedule_id = _stable_schedule_id(self.provider_name, dep, arr, departure, train_no)
            schedules.append(
                ProviderSchedule(
                    schedule_id=schedule_id,
                    provider=self.provider_name,
                    dep=dep,
                    arr=arr,
                    departure_at=departure,
                    arrival_at=arrival,
                    train_no=train_no,
                    availability={"general": idx % 2 == 0, "special": idx % 3 == 0},
                    metadata={
                        "mock": True,
                        "source": "provider-mock",
                        "provider": self.provider_name,
                    },
                )
            )

        return ProviderOutcome(
            ok=True,
            data={
                "schedules": schedules,
                "window_end": time_window_end,
                "user_id_hash": hashlib.sha256(user_id.encode("utf-8")).hexdigest()[:12],
            },
        )

    async def reserve(
        self,
        *,
        schedule_id: str,
        seat_class: str,
        passengers: dict[str, int],
        user_id: str,
    ) -> ProviderOutcome:
        reservation_id = f"{self.provider_name}-rsv-{uuid4().hex[:12]}"
        return ProviderOutcome(
            ok=True,
            data={
                "reservation_id": reservation_id,
                "schedule_id": schedule_id,
                "seat_class": seat_class,
                "passengers": passengers,
                "http_trace": {
                    "endpoint": "reserve",
                    "request": {
                        "schedule_id": schedule_id,
                        "seat_class": seat_class,
                        "passengers": passengers,
                    },
                    "response": {"reservation_id": reservation_id},
                },
            },
        )

    async def reserve_standby(
        self,
        *,
        schedule_id: str,
        seat_class: str,
        passengers: dict[str, int],
        user_id: str,
    ) -> ProviderOutcome:
        return ProviderOutcome(
            ok=True,
            data={
                "reservation_id": f"{self.provider_name}-std-{uuid4().hex[:12]}",
                "schedule_id": schedule_id,
                "seat_class": seat_class,
                "passengers": passengers,
                "status": "waiting",
            },
        )

    async def pay(
        self,
        *,
        reservation_id: str,
        user_id: str,
        payment_card: dict[str, Any] | None = None,
    ) -> ProviderOutcome:
        payment_id = f"{self.provider_name}-pay-{uuid4().hex[:12]}"
        ticket_no = f"{self.provider_name}{uuid4().hex[:8].upper()}"
        return ProviderOutcome(
            ok=True,
            data={
                "payment_id": payment_id,
                "ticket_no": ticket_no,
                "reservation_id": reservation_id,
                "http_trace": {
                    "endpoint": "pay",
                    "request": {"reservation_id": reservation_id},
                    "response": {"payment_id": payment_id, "ticket_no": ticket_no},
                },
            },
        )

    async def cancel(
        self,
        *,
        artifact_data: dict[str, Any],
        user_id: str,
    ) -> ProviderOutcome:
        return ProviderOutcome(
            ok=False,
            retryable=False,
            error_code="not_supported",
            error_message_safe="Ticket cancel is not supported by mock provider",
            data={
                "artifact_id": artifact_data.get("artifact_id"),
                "http_trace": {
                    "endpoint": "cancel",
                    "request": artifact_data,
                    "response": {"status": "not_supported"},
                },
            },
        )

    async def get_reservations(
        self,
        *,
        user_id: str,
        paid_only: bool = False,
        reservation_id: str | None = None,
    ) -> ProviderOutcome:
        return ProviderOutcome(
            ok=True,
            data={
                "reservations": [],
                "http_trace": {
                    "endpoint": "get_reservations",
                    "request": {"paid_only": paid_only, "reservation_id": reservation_id},
                    "response": {"reservations": []},
                },
            },
        )

    async def ticket_info(
        self,
        *,
        reservation_id: str,
        user_id: str,
    ) -> ProviderOutcome:
        return ProviderOutcome(
            ok=True,
            data={
                "reservation_id": reservation_id,
                "tickets": [],
                "http_trace": {
                    "endpoint": "ticket_info",
                    "request": {"reservation_id": reservation_id},
                    "response": {"tickets": []},
                },
            },
        )


class MockSRTClient(MockProviderBase):
    provider_name = "SRT"


class MockKTXClient(MockProviderBase):
    provider_name = "KTX"
