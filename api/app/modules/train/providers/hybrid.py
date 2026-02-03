from __future__ import annotations

from datetime import date
from typing import Any

from app.modules.train.providers.base import ProviderOutcome, TrainProviderClient
from app.modules.train.providers.ktx_client import KTXClient
from app.modules.train.providers.mock import MockKTXClient, MockSRTClient
from app.modules.train.providers.srt_client import SRTClient


class _HybridProviderClient(TrainProviderClient):
    provider_name = "HYBRID"

    def __init__(self, *, live_client: TrainProviderClient, mock_client: TrainProviderClient) -> None:
        self._live_client = live_client
        self._mock_client = mock_client

    async def login(self, *, user_id: str, credentials: dict[str, str] | None = None) -> ProviderOutcome:
        live = await self._live_client.login(user_id=user_id, credentials=credentials)
        if live.ok or credentials is not None:
            return live
        return await self._mock_client.login(user_id=user_id, credentials=credentials)

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
        live = await self._live_client.search(
            dep=dep,
            arr=arr,
            date_value=date_value,
            time_window_start=time_window_start,
            time_window_end=time_window_end,
            user_id=user_id,
        )

        if live.ok:
            schedules = live.data.get("schedules", [])
            for schedule in schedules:
                schedule.metadata = {**schedule.metadata, "source": "live"}
            return live

        fallback = await self._mock_client.search(
            dep=dep,
            arr=arr,
            date_value=date_value,
            time_window_start=time_window_start,
            time_window_end=time_window_end,
            user_id=user_id,
        )
        if fallback.ok:
            schedules = fallback.data.get("schedules", [])
            for schedule in schedules:
                schedule.metadata = {
                    **schedule.metadata,
                    "source": "mock-fallback",
                    "live_error_code": live.error_code,
                    "live_error_message": live.error_message_safe,
                }
        return fallback

    async def reserve(
        self,
        *,
        schedule_id: str,
        seat_class: str,
        passengers: dict[str, int],
        user_id: str,
    ) -> ProviderOutcome:
        return await self._mock_client.reserve(
            schedule_id=schedule_id,
            seat_class=seat_class,
            passengers=passengers,
            user_id=user_id,
        )

    async def reserve_standby(
        self,
        *,
        schedule_id: str,
        seat_class: str,
        passengers: dict[str, int],
        user_id: str,
    ) -> ProviderOutcome:
        return await self._mock_client.reserve_standby(
            schedule_id=schedule_id,
            seat_class=seat_class,
            passengers=passengers,
            user_id=user_id,
        )

    async def pay(
        self,
        *,
        reservation_id: str,
        user_id: str,
        payment_card: dict[str, Any] | None = None,
    ) -> ProviderOutcome:
        return await self._mock_client.pay(
            reservation_id=reservation_id,
            user_id=user_id,
            payment_card=payment_card,
        )

    async def cancel(
        self,
        *,
        artifact_data: dict[str, Any],
        user_id: str,
    ) -> ProviderOutcome:
        return await self._mock_client.cancel(
            artifact_data=artifact_data,
            user_id=user_id,
        )

    async def get_reservations(
        self,
        *,
        user_id: str,
        paid_only: bool = False,
        reservation_id: str | None = None,
    ) -> ProviderOutcome:
        live = await self._live_client.get_reservations(
            user_id=user_id,
            paid_only=paid_only,
            reservation_id=reservation_id,
        )
        if live.ok:
            return live
        return await self._mock_client.get_reservations(
            user_id=user_id,
            paid_only=paid_only,
            reservation_id=reservation_id,
        )

    async def ticket_info(
        self,
        *,
        reservation_id: str,
        user_id: str,
    ) -> ProviderOutcome:
        live = await self._live_client.ticket_info(
            reservation_id=reservation_id,
            user_id=user_id,
        )
        if live.ok:
            return live
        return await self._mock_client.ticket_info(
            reservation_id=reservation_id,
            user_id=user_id,
        )


class HybridSRTClient(_HybridProviderClient):
    provider_name = "SRT"

    def __init__(
        self,
        *,
        live_client: TrainProviderClient | None = None,
        mock_client: TrainProviderClient | None = None,
    ) -> None:
        super().__init__(live_client=live_client or SRTClient(), mock_client=mock_client or MockSRTClient())


class HybridKTXClient(_HybridProviderClient):
    provider_name = "KTX"

    def __init__(
        self,
        *,
        live_client: TrainProviderClient | None = None,
        mock_client: TrainProviderClient | None = None,
    ) -> None:
        super().__init__(live_client=live_client or KTXClient(), mock_client=mock_client or MockKTXClient())
