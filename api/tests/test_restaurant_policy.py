from __future__ import annotations

import fakeredis.aioredis
import pytest

from app.modules.restaurant.lease import acquire_payment_lease, release_payment_lease
from app.modules.restaurant.policy import (
    build_payment_lease_key,
    is_non_committing_phase,
    resolve_auth_fallback_step,
)
from app.modules.restaurant.types import RestaurantAuthStep


def test_auth_fallback_order_is_refresh_then_bootstrap_then_fail():
    assert (
        resolve_auth_fallback_step(refresh_attempts=0, bootstrap_attempted=False, max_refresh_retries=2)
        == RestaurantAuthStep.REFRESH
    )
    assert (
        resolve_auth_fallback_step(refresh_attempts=1, bootstrap_attempted=False, max_refresh_retries=2)
        == RestaurantAuthStep.REFRESH
    )
    assert (
        resolve_auth_fallback_step(refresh_attempts=2, bootstrap_attempted=False, max_refresh_retries=2)
        == RestaurantAuthStep.BOOTSTRAP
    )
    assert (
        resolve_auth_fallback_step(refresh_attempts=2, bootstrap_attempted=True, max_refresh_retries=2)
        == RestaurantAuthStep.FAIL
    )


def test_payment_lease_key_is_provider_account_restaurant():
    lease_key = build_payment_lease_key(provider="Resy", account_ref="acct-123", restaurant_id="rest-999")
    assert lease_key == "bominal:restaurant:payment-lease:resy:acct-123:rest-999"


def test_non_committing_actions_mark_concurrency_safe():
    assert is_non_committing_phase("search") is True
    assert is_non_committing_phase(" availability ") is True
    assert is_non_committing_phase("payment") is False


@pytest.mark.asyncio
async def test_payment_lease_acquire_blocks_second_holder():
    redis = fakeredis.aioredis.FakeRedis()
    lease_key = build_payment_lease_key(provider="resy", account_ref="acct", restaurant_id="rest")

    first = await acquire_payment_lease(redis, lease_key=lease_key, holder_token="holder-1", ttl_seconds=30)
    second = await acquire_payment_lease(redis, lease_key=lease_key, holder_token="holder-2", ttl_seconds=30)

    assert first is True
    assert second is False


@pytest.mark.asyncio
async def test_payment_lease_release_restores_availability():
    redis = fakeredis.aioredis.FakeRedis()
    lease_key = build_payment_lease_key(provider="resy", account_ref="acct", restaurant_id="rest")

    acquired = await acquire_payment_lease(redis, lease_key=lease_key, holder_token="holder-1", ttl_seconds=30)
    assert acquired is True

    released = await release_payment_lease(redis, lease_key=lease_key, holder_token="holder-1")
    assert released is True

    acquired_again = await acquire_payment_lease(redis, lease_key=lease_key, holder_token="holder-2", ttl_seconds=30)
    assert acquired_again is True
