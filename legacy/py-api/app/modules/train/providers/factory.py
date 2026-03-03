from __future__ import annotations

from datetime import date
from typing import Any

from app.core.config import get_settings
from app.modules.train.providers.base import ProviderOutcome, TrainProviderClient
from app.modules.train.providers.hybrid import HybridKTXClient, HybridSRTClient
from app.modules.train.providers.ktx_client import KTXClient
from app.modules.train.providers.mock import MockKTXClient, MockSRTClient
from app.modules.train.providers.srt_client import SRTClient
from app.modules.train.providers.transport import (
    CurlCffiTransport,
    HttpxTransport,
    ProviderTransportError,
    ResilientTransport,
)


class FailSafeProviderClient(TrainProviderClient):
    def __init__(self, delegate: TrainProviderClient) -> None:
        self._delegate = delegate
        self.provider_name = delegate.provider_name

    def _from_transport_error(self, exc: ProviderTransportError) -> ProviderOutcome:
        return ProviderOutcome(
            ok=False,
            retryable=exc.retryable,
            error_code=exc.error_code,
            error_message_safe=exc.error_message_safe,
        )

    async def login(self, *, user_id: str, credentials: dict[str, str] | None = None) -> ProviderOutcome:
        try:
            return await self._delegate.login(user_id=user_id, credentials=credentials)
        except ProviderTransportError as exc:
            return self._from_transport_error(exc)

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
        try:
            return await self._delegate.search(
                dep=dep,
                arr=arr,
                date_value=date_value,
                time_window_start=time_window_start,
                time_window_end=time_window_end,
                user_id=user_id,
            )
        except ProviderTransportError as exc:
            return self._from_transport_error(exc)

    async def reserve(
        self,
        *,
        schedule_id: str,
        seat_class: str,
        passengers: dict[str, int],
        user_id: str,
    ) -> ProviderOutcome:
        try:
            return await self._delegate.reserve(
                schedule_id=schedule_id,
                seat_class=seat_class,
                passengers=passengers,
                user_id=user_id,
            )
        except ProviderTransportError as exc:
            return self._from_transport_error(exc)

    async def reserve_standby(
        self,
        *,
        schedule_id: str,
        seat_class: str,
        passengers: dict[str, int],
        user_id: str,
    ) -> ProviderOutcome:
        try:
            return await self._delegate.reserve_standby(
                schedule_id=schedule_id,
                seat_class=seat_class,
                passengers=passengers,
                user_id=user_id,
            )
        except ProviderTransportError as exc:
            return self._from_transport_error(exc)

    async def pay(
        self,
        *,
        reservation_id: str,
        user_id: str,
        payment_card: dict[str, Any] | None = None,
    ) -> ProviderOutcome:
        try:
            return await self._delegate.pay(
                reservation_id=reservation_id,
                user_id=user_id,
                payment_card=payment_card,
            )
        except ProviderTransportError as exc:
            return self._from_transport_error(exc)

    async def cancel(
        self,
        *,
        artifact_data: dict[str, Any],
        user_id: str,
    ) -> ProviderOutcome:
        try:
            return await self._delegate.cancel(artifact_data=artifact_data, user_id=user_id)
        except ProviderTransportError as exc:
            return self._from_transport_error(exc)

    async def get_reservations(
        self,
        *,
        user_id: str,
        paid_only: bool = False,
        reservation_id: str | None = None,
    ) -> ProviderOutcome:
        try:
            return await self._delegate.get_reservations(
                user_id=user_id,
                paid_only=paid_only,
                reservation_id=reservation_id,
            )
        except ProviderTransportError as exc:
            return self._from_transport_error(exc)

    async def ticket_info(
        self,
        *,
        reservation_id: str,
        user_id: str,
    ) -> ProviderOutcome:
        try:
            return await self._delegate.ticket_info(reservation_id=reservation_id, user_id=user_id)
        except ProviderTransportError as exc:
            return self._from_transport_error(exc)


def _build_transport(provider: str):
    settings = get_settings()
    mode = settings.train_provider_transport.lower()
    impersonate = "chrome" if provider == "SRT" else "chrome131_android"

    if mode in {"curl", "curl_cffi"}:
        base_transport = CurlCffiTransport(impersonate=impersonate)
    elif mode == "httpx":
        base_transport = HttpxTransport()
    else:
        # auto: try curl_cffi first, then fallback to httpx.
        try:
            base_transport = CurlCffiTransport(impersonate=impersonate)
        except RuntimeError:
            base_transport = HttpxTransport()

    return ResilientTransport(base_transport, provider=provider)


def _build_live_client(provider: str) -> TrainProviderClient:
    transport = _build_transport(provider)
    if provider == "SRT":
        return SRTClient(transport=transport)
    if provider == "KTX":
        return KTXClient(transport=transport)
    raise ValueError(f"Unsupported provider: {provider}")


def get_provider_client(provider: str) -> TrainProviderClient:
    settings = get_settings()
    mode = settings.train_provider_mode.lower()
    client: TrainProviderClient

    if mode == "mock":
        if provider == "SRT":
            client = MockSRTClient()
        elif provider == "KTX":
            client = MockKTXClient()
        else:
            raise ValueError(f"Unsupported provider: {provider}")
        return FailSafeProviderClient(client)

    if mode in {"hybrid", "real_search_mock_execute"}:
        if provider == "SRT":
            client = HybridSRTClient(live_client=_build_live_client("SRT"))
        elif provider == "KTX":
            client = HybridKTXClient(live_client=_build_live_client("KTX"))
        else:
            raise ValueError(f"Unsupported provider: {provider}")
        return FailSafeProviderClient(client)

    if provider in {"SRT", "KTX"}:
        return FailSafeProviderClient(_build_live_client(provider))

    raise ValueError(f"Unsupported provider: {provider}")
