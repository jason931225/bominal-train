from __future__ import annotations

from dataclasses import dataclass, field
from datetime import datetime
from typing import Any, Protocol, runtime_checkable


@dataclass(slots=True)
class RestaurantProviderOutcome:
    ok: bool
    retryable: bool = False
    error_code: str | None = None
    error_message_safe: str | None = None
    data: dict[str, Any] = field(default_factory=dict)


@dataclass(slots=True)
class RestaurantSearchSlot:
    provider_slot_id: str
    provider: str
    restaurant_id: str
    party_size: int
    date_time_local: datetime
    availability_token: str | None = None
    metadata_safe: dict[str, Any] = field(default_factory=dict)


@runtime_checkable
class RestaurantProviderClient(Protocol):
    provider_name: str

    async def authenticate_start(
        self,
        *,
        account_identifier: str,
        password: str | None = None,
        delivery_channel: str = "email",
    ) -> RestaurantProviderOutcome:
        ...

    async def authenticate_complete(
        self,
        *,
        account_identifier: str,
        challenge_token: str | None = None,
        otp_code: str | None = None,
    ) -> RestaurantProviderOutcome:
        ...

    async def refresh_auth(self, *, account_ref: str) -> RestaurantProviderOutcome:
        ...

    async def get_user_profile(self, *, account_ref: str) -> RestaurantProviderOutcome:
        ...

    async def search_availability(
        self,
        *,
        account_ref: str,
        restaurant_id: str,
        party_size: int,
        date_time_local: datetime,
        metadata: dict[str, Any] | None = None,
    ) -> RestaurantProviderOutcome:
        ...

    async def create_reservation(
        self,
        *,
        account_ref: str,
        slot: RestaurantSearchSlot,
        metadata: dict[str, Any] | None = None,
    ) -> RestaurantProviderOutcome:
        ...

    async def cancel_reservation(
        self,
        *,
        account_ref: str,
        restaurant_id: str,
        confirmation_number: str,
        security_token: str | None = None,
        metadata: dict[str, Any] | None = None,
    ) -> RestaurantProviderOutcome:
        ...
