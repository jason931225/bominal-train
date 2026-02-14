from __future__ import annotations

from app.modules.restaurant.types import RestaurantAuthStep

PAYMENT_LEASE_PREFIX = "bominal:restaurant:payment-lease"

NON_COMMITTING_PHASES = {"search", "availability", "quote", "hold"}


def _normalize_key_component(value: str) -> str:
    return value.strip().lower().replace(" ", "_")


def build_payment_lease_key(*, provider: str, account_ref: str, restaurant_id: str) -> str:
    return (
        f"{PAYMENT_LEASE_PREFIX}:"
        f"{_normalize_key_component(provider)}:"
        f"{_normalize_key_component(account_ref)}:"
        f"{_normalize_key_component(restaurant_id)}"
    )


def resolve_auth_fallback_step(
    *,
    refresh_attempts: int,
    bootstrap_attempted: bool,
    max_refresh_retries: int,
) -> RestaurantAuthStep:
    if refresh_attempts < max_refresh_retries:
        return RestaurantAuthStep.REFRESH
    if not bootstrap_attempted:
        return RestaurantAuthStep.BOOTSTRAP
    return RestaurantAuthStep.FAIL


def is_non_committing_phase(action: str) -> bool:
    return action.strip().lower() in NON_COMMITTING_PHASES
