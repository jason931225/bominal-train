from __future__ import annotations

from app.modules.restaurant.providers.base import RestaurantProviderClient
from app.modules.restaurant.providers.constants import (
    RESTAURANT_PROVIDER_OPENTABLE,
    RESTAURANT_PROVIDER_RESY,
)
from app.modules.restaurant.providers.opentable_adapter import OpenTableProviderClient
from app.modules.restaurant.providers.resy_adapter import ResyProviderClient


def _normalize_provider(provider: str) -> str:
    normalized = provider.strip().upper().replace("-", "").replace("_", "")
    if normalized == "OPENTABLE":
        return RESTAURANT_PROVIDER_OPENTABLE
    if normalized == "RESY":
        return RESTAURANT_PROVIDER_RESY
    raise ValueError(f"Unsupported restaurant provider: {provider}")


def get_restaurant_provider_client(provider: str) -> RestaurantProviderClient:
    normalized = _normalize_provider(provider)
    if normalized == RESTAURANT_PROVIDER_OPENTABLE:
        return OpenTableProviderClient()
    return ResyProviderClient()
