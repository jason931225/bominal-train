from __future__ import annotations

from dataclasses import dataclass, field
from datetime import date, datetime
from typing import Any, Protocol


@dataclass(slots=True)
class ProviderOutcome:
    ok: bool
    retryable: bool = False
    error_code: str | None = None
    error_message_safe: str | None = None
    data: dict[str, Any] = field(default_factory=dict)


@dataclass(slots=True)
class ProviderSchedule:
    schedule_id: str
    provider: str
    dep: str
    arr: str
    departure_at: datetime
    arrival_at: datetime
    train_no: str
    availability: dict[str, bool]
    metadata: dict[str, Any] = field(default_factory=dict)


class TrainProviderClient(Protocol):
    provider_name: str

    async def login(self, *, user_id: str, credentials: dict[str, str] | None = None) -> ProviderOutcome:
        ...

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
        ...

    async def reserve(
        self,
        *,
        schedule_id: str,
        seat_class: str,
        passengers: dict[str, int],
        user_id: str,
    ) -> ProviderOutcome:
        ...

    async def reserve_standby(
        self,
        *,
        schedule_id: str,
        seat_class: str,
        passengers: dict[str, int],
        user_id: str,
    ) -> ProviderOutcome:
        ...

    async def pay(
        self,
        *,
        reservation_id: str,
        user_id: str,
        payment_card: dict[str, Any] | None = None,
    ) -> ProviderOutcome:
        ...

    async def cancel(
        self,
        *,
        artifact_data: dict[str, Any],
        user_id: str,
    ) -> ProviderOutcome:
        ...

    async def get_reservations(
        self,
        *,
        user_id: str,
        paid_only: bool = False,
        reservation_id: str | None = None,
    ) -> ProviderOutcome:
        ...

    async def ticket_info(
        self,
        *,
        reservation_id: str,
        user_id: str,
    ) -> ProviderOutcome:
        ...
