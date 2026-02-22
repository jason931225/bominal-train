"""
Shared Redis connection pool for the application.

This module provides a centralized Redis connection pool to avoid creating
new connections per-call, which can exhaust connection limits under load.

Usage:
    from app.core.redis import get_redis_pool

    async with get_redis_pool() as redis:
        await redis.get("key")
        await redis.set("key", "value")
"""
from __future__ import annotations

import asyncio
from contextlib import asynccontextmanager
from typing import AsyncIterator
from typing import Literal

from redis.asyncio import Redis

from app.core.config import get_settings

settings = get_settings()
RedisPurpose = Literal["non_cde", "cde"]

# Global connection pools (lazy initialized)
_redis_pool_non_cde: Redis | None = None
_redis_pool_cde: Redis | None = None
_pool_lock_non_cde: asyncio.Lock | None = None
_pool_lock_cde: asyncio.Lock | None = None


def _get_pool_lock(*, purpose: RedisPurpose) -> asyncio.Lock:
    """Get or create the pool initialization lock for the purpose."""
    global _pool_lock_non_cde
    global _pool_lock_cde
    if purpose == "cde":
        if _pool_lock_cde is None:
            _pool_lock_cde = asyncio.Lock()
        return _pool_lock_cde
    if _pool_lock_non_cde is None:
        _pool_lock_non_cde = asyncio.Lock()
    return _pool_lock_non_cde


def _redis_url_for_purpose(*, purpose: RedisPurpose) -> str:
    if purpose == "cde":
        return settings.resolved_redis_url_cde
    return settings.resolved_redis_url_non_cde


async def _create_pool(*, purpose: RedisPurpose) -> Redis:
    """Create a new Redis connection pool for the specified purpose."""
    return Redis.from_url(
        _redis_url_for_purpose(purpose=purpose),
        decode_responses=False,
        # Connection pool settings for production use
        max_connections=20,
        socket_connect_timeout=5.0,
        socket_timeout=5.0,
    )


def _get_existing_pool(*, purpose: RedisPurpose) -> Redis | None:
    if purpose == "cde":
        return _redis_pool_cde
    return _redis_pool_non_cde


def _set_existing_pool(*, purpose: RedisPurpose, pool: Redis) -> None:
    global _redis_pool_non_cde
    global _redis_pool_cde
    if purpose == "cde":
        _redis_pool_cde = pool
    else:
        _redis_pool_non_cde = pool


async def get_redis_client(*, purpose: RedisPurpose = "non_cde") -> Redis:
    """
    Get the shared Redis client instance.
    
    Uses a lock to prevent race conditions during initialization.
    The connection pool is reused across all requests.
    """
    pool = _get_existing_pool(purpose=purpose)
    if pool is not None:
        return pool

    async with _get_pool_lock(purpose=purpose):
        # Double-check after acquiring lock
        pool = _get_existing_pool(purpose=purpose)
        if pool is not None:
            return pool
        created = await _create_pool(purpose=purpose)
        _set_existing_pool(purpose=purpose, pool=created)
        return created


@asynccontextmanager
async def get_redis_pool(*, purpose: RedisPurpose = "non_cde") -> AsyncIterator[Redis]:
    """
    Context manager for Redis operations.
    
    This returns the shared pool - the context manager is for
    compatibility and doesn't close the pool on exit.
    
    Usage:
        async with get_redis_pool() as redis:
            await redis.get("key")
    """
    redis = await get_redis_client(purpose=purpose)
    yield redis


async def get_cde_redis_client() -> Redis:
    return await get_redis_client(purpose="cde")


@asynccontextmanager
async def get_cde_redis_pool() -> AsyncIterator[Redis]:
    async with get_redis_pool(purpose="cde") as redis:
        yield redis


async def close_redis_pool() -> None:
    """
    Close all Redis pools.
    
    Call this during application shutdown.
    """
    global _redis_pool_non_cde
    global _redis_pool_cde
    if _redis_pool_non_cde is not None:
        await _redis_pool_non_cde.aclose()
        _redis_pool_non_cde = None
    if _redis_pool_cde is not None:
        await _redis_pool_cde.aclose()
        _redis_pool_cde = None
