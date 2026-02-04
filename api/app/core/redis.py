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

from redis.asyncio import Redis

from app.core.config import get_settings

settings = get_settings()

# Global connection pool (lazy initialized)
_redis_pool: Redis | None = None
_pool_lock: asyncio.Lock | None = None


def _get_pool_lock() -> asyncio.Lock:
    """Get or create the pool initialization lock."""
    global _pool_lock
    if _pool_lock is None:
        _pool_lock = asyncio.Lock()
    return _pool_lock


async def _create_pool() -> Redis:
    """Create a new Redis connection pool."""
    return Redis.from_url(
        settings.redis_url,
        decode_responses=False,
        # Connection pool settings for production use
        max_connections=20,
        socket_connect_timeout=5.0,
        socket_timeout=5.0,
    )


async def get_redis_client() -> Redis:
    """
    Get the shared Redis client instance.
    
    Uses a lock to prevent race conditions during initialization.
    The connection pool is reused across all requests.
    """
    global _redis_pool
    
    if _redis_pool is not None:
        return _redis_pool
    
    async with _get_pool_lock():
        # Double-check after acquiring lock
        if _redis_pool is not None:
            return _redis_pool
        _redis_pool = await _create_pool()
        return _redis_pool


@asynccontextmanager
async def get_redis_pool() -> AsyncIterator[Redis]:
    """
    Context manager for Redis operations.
    
    This returns the shared pool - the context manager is for
    compatibility and doesn't close the pool on exit.
    
    Usage:
        async with get_redis_pool() as redis:
            await redis.get("key")
    """
    redis = await get_redis_client()
    yield redis


async def close_redis_pool() -> None:
    """
    Close the Redis pool.
    
    Call this during application shutdown.
    """
    global _redis_pool
    if _redis_pool is not None:
        await _redis_pool.aclose()
        _redis_pool = None
