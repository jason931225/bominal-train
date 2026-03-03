from __future__ import annotations

from redis.asyncio import Redis


async def acquire_payment_lease(
    redis: Redis,
    *,
    lease_key: str,
    holder_token: str,
    ttl_seconds: int,
) -> bool:
    acquired = await redis.set(lease_key, holder_token.encode("utf-8"), ex=ttl_seconds, nx=True)
    return bool(acquired)


async def release_payment_lease(
    redis: Redis,
    *,
    lease_key: str,
    holder_token: str,
) -> bool:
    current_holder = await redis.get(lease_key)
    if current_holder != holder_token.encode("utf-8"):
        return False
    deleted = await redis.delete(lease_key)
    return bool(deleted)
