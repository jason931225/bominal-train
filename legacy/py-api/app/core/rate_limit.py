from collections import defaultdict, deque
from datetime import datetime, timezone
from threading import Lock
from typing import Protocol

from fastapi import HTTPException, status

from app.core.config import get_settings

settings = get_settings()


class RateLimiter(Protocol):
    """Protocol for rate limiters."""
    async def check(self, key: str, limit: int, window_seconds: int) -> None:
        ...


class InMemoryRateLimiter:
    """Simple per-process limiter for local development.

    For production/distributed deployments, swap this for Redis.
    """

    def __init__(self) -> None:
        self._events: dict[str, deque[float]] = defaultdict(deque)
        self._lock = Lock()

    async def check(self, key: str, limit: int, window_seconds: int) -> None:
        """Async wrapper for sync check - allows consistent interface."""
        self._check_sync(key, limit, window_seconds)

    def _check_sync(self, key: str, limit: int, window_seconds: int) -> None:
        now = datetime.now(timezone.utc).timestamp()
        window_start = now - window_seconds

        with self._lock:
            timestamps = self._events[key]
            while timestamps and timestamps[0] < window_start:
                timestamps.popleft()

            if len(timestamps) >= limit:
                raise HTTPException(
                    status_code=status.HTTP_429_TOO_MANY_REQUESTS,
                    detail="Too many requests. Please try again shortly.",
                )

            timestamps.append(now)


class RedisRateLimiter:
    """Distributed rate limiter using Redis for multi-worker deployments.
    
    Uses a sliding window counter algorithm for accurate rate limiting
    across multiple uvicorn workers.
    """

    def __init__(self) -> None:
        self._redis = None

    async def _get_redis(self):
        if self._redis is None:
            from app.core.redis import get_redis_client
            self._redis = await get_redis_client()
        return self._redis

    async def check(self, key: str, limit: int, window_seconds: int) -> None:
        """Check if request is allowed under rate limit.
        
        Uses Redis sorted sets with timestamps as scores for sliding window.
        
        Args:
            key: Rate limit bucket key (e.g., "auth:192.168.1.1:/api/auth/login")
            limit: Maximum requests allowed in window
            window_seconds: Time window in seconds
            
        Raises:
            HTTPException: 429 if rate limit exceeded
        """
        redis = await self._get_redis()
        now = datetime.now(timezone.utc).timestamp()
        window_start = now - window_seconds
        
        redis_key = f"rate_limit:{key}"
        
        # Use pipeline for atomic operations
        pipe = redis.pipeline()
        # Remove expired entries
        pipe.zremrangebyscore(redis_key, 0, window_start)
        # Count current entries
        pipe.zcard(redis_key)
        # Add current request
        pipe.zadd(redis_key, {str(now): now})
        # Set expiry on key
        pipe.expire(redis_key, window_seconds + 1)
        
        results = await pipe.execute()
        current_count = results[1]  # zcard result
        
        if current_count >= limit:
            raise HTTPException(
                status_code=status.HTTP_429_TOO_MANY_REQUESTS,
                detail="Too many requests. Please try again shortly.",
            )


# Use Redis limiter if RATE_LIMIT_USE_REDIS env var is set, else fall back to in-memory
# For production with multiple workers, set RATE_LIMIT_USE_REDIS=1
_use_redis = settings.rate_limit_use_redis if hasattr(settings, 'rate_limit_use_redis') else False


def _create_rate_limiter():
    """Create appropriate rate limiter based on configuration."""
    if _use_redis and settings.resolved_redis_url_non_cde:
        return RedisRateLimiter()
    return InMemoryRateLimiter()


rate_limiter = _create_rate_limiter()
