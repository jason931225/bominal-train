from __future__ import annotations

from app.core.config import get_settings
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
    settings = get_settings()
    normalized = _normalize_provider(provider)
    if normalized == RESTAURANT_PROVIDER_OPENTABLE:
        return OpenTableProviderClient(
            base_url=settings.restaurant_opentable_base_url,
            timeout_seconds=settings.restaurant_opentable_timeout_seconds,
            auth_start_path=settings.restaurant_opentable_auth_start_path,
            auth_complete_path=settings.restaurant_opentable_auth_complete_path,
            search_operation_name=settings.restaurant_opentable_search_operation_name,
            search_operation_sha256=settings.restaurant_opentable_search_operation_sha256,
            search_slot_path=settings.restaurant_opentable_search_slot_path,
            create_operation_name=settings.restaurant_opentable_create_operation_name,
            create_operation_sha256=settings.restaurant_opentable_create_operation_sha256,
            create_path=settings.restaurant_opentable_create_path,
            confirmation_operation_name=settings.restaurant_opentable_confirmation_operation_name,
            confirmation_operation_sha256=settings.restaurant_opentable_confirmation_operation_sha256,
        )
    return ResyProviderClient(
        base_url=settings.restaurant_resy_base_url,
        timeout_seconds=settings.restaurant_resy_timeout_seconds,
        auth_password_path=settings.restaurant_resy_auth_password_path,
        auth_api_key=settings.restaurant_resy_auth_api_key,
        x_origin=settings.restaurant_resy_x_origin,
        profile_path=settings.restaurant_resy_profile_path,
        search_path=settings.restaurant_resy_search_path,
        create_details_path=settings.restaurant_resy_create_details_path,
        create_book_path=settings.restaurant_resy_create_book_path,
        cancel_path=settings.restaurant_resy_cancel_path,
        source_id=settings.restaurant_resy_source_id,
        refresh_path=settings.restaurant_resy_refresh_path,
        logout_path=settings.restaurant_resy_logout_path,
    )
