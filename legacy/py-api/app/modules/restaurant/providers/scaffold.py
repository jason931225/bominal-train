from __future__ import annotations

from datetime import datetime
from typing import Any

from app.modules.restaurant.providers.base import (
    RestaurantProviderClient,
    RestaurantProviderOutcome,
    RestaurantSearchSlot,
)


class ScaffoldRestaurantProviderClient(RestaurantProviderClient):
    provider_name = "UNKNOWN"

    def _not_implemented(self, operation: str) -> RestaurantProviderOutcome:
        return RestaurantProviderOutcome(
            ok=False,
            retryable=False,
            error_code="not_implemented",
            error_message_safe=f"{self.provider_name} {operation} adapter path not implemented yet.",
            data={"provider": self.provider_name, "operation": operation},
        )

    async def authenticate_start(
        self,
        *,
        account_identifier: str,
        password: str | None = None,
        delivery_channel: str = "email",
    ) -> RestaurantProviderOutcome:
        _ = (account_identifier, password, delivery_channel)
        return self._not_implemented("auth.start")

    async def authenticate_complete(
        self,
        *,
        account_identifier: str,
        challenge_token: str | None = None,
        otp_code: str | None = None,
    ) -> RestaurantProviderOutcome:
        _ = (account_identifier, challenge_token, otp_code)
        return self._not_implemented("auth.complete")

    async def refresh_auth(self, *, account_ref: str) -> RestaurantProviderOutcome:
        _ = account_ref
        return self._not_implemented("auth.refresh")

    async def get_user_profile(self, *, account_ref: str) -> RestaurantProviderOutcome:
        _ = account_ref
        return self._not_implemented("profile.get")

    async def search_availability(
        self,
        *,
        account_ref: str,
        restaurant_id: str,
        party_size: int,
        date_time_local: datetime,
        metadata: dict[str, Any] | None = None,
    ) -> RestaurantProviderOutcome:
        _ = (account_ref, restaurant_id, party_size, date_time_local, metadata)
        return self._not_implemented("search.availability")

    async def create_reservation(
        self,
        *,
        account_ref: str,
        slot: RestaurantSearchSlot,
        metadata: dict[str, Any] | None = None,
    ) -> RestaurantProviderOutcome:
        _ = (account_ref, slot, metadata)
        return self._not_implemented("reservation.create")

    async def cancel_reservation(
        self,
        *,
        account_ref: str,
        restaurant_id: str,
        confirmation_number: str,
        security_token: str | None = None,
        metadata: dict[str, Any] | None = None,
    ) -> RestaurantProviderOutcome:
        _ = (account_ref, restaurant_id, confirmation_number, security_token, metadata)
        return self._not_implemented("reservation.cancel")
