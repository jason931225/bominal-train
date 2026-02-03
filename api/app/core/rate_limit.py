from collections import defaultdict, deque
from datetime import datetime, timezone
from threading import Lock

from fastapi import HTTPException, status


class InMemoryRateLimiter:
    """Simple per-process limiter for local development.

    For production/distributed deployments, swap this for Redis.
    """

    def __init__(self) -> None:
        self._events: dict[str, deque[float]] = defaultdict(deque)
        self._lock = Lock()

    def check(self, key: str, limit: int, window_seconds: int) -> None:
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


rate_limiter = InMemoryRateLimiter()
