from __future__ import annotations

import pytest
from fastapi import HTTPException

from app.core import rate_limit as rate_limit_mod


def test_inmemory_rate_limiter_prunes_old_events_and_enforces_limit() -> None:
    limiter = rate_limit_mod.InMemoryRateLimiter()

    # Prime deque with one stale and one fresh event to exercise pruning.
    now = rate_limit_mod.datetime.now(rate_limit_mod.timezone.utc).timestamp()
    limiter._events["bucket"].extend([now - 20, now - 1])

    limiter._check_sync("bucket", limit=2, window_seconds=5)
    assert len(limiter._events["bucket"]) == 2

    with pytest.raises(HTTPException) as exc:
        limiter._check_sync("bucket", limit=2, window_seconds=5)
    assert exc.value.status_code == 429


@pytest.mark.asyncio
async def test_inmemory_rate_limiter_async_wrapper_calls_sync(monkeypatch) -> None:
    limiter = rate_limit_mod.InMemoryRateLimiter()
    called = {"value": False}

    def _fake_check_sync(_key: str, _limit: int, _window: int) -> None:
        called["value"] = True

    monkeypatch.setattr(limiter, "_check_sync", _fake_check_sync)
    await limiter.check("bucket", 1, 60)
    assert called["value"] is True


class _FakePipeline:
    def __init__(self, *, count: int):
        self.count = count
        self.ops: list[tuple[str, tuple, dict]] = []

    def zremrangebyscore(self, *args, **kwargs):  # noqa: ANN002, ANN003
        self.ops.append(("zremrangebyscore", args, kwargs))
        return self

    def zcard(self, *args, **kwargs):  # noqa: ANN002, ANN003
        self.ops.append(("zcard", args, kwargs))
        return self

    def zadd(self, *args, **kwargs):  # noqa: ANN002, ANN003
        self.ops.append(("zadd", args, kwargs))
        return self

    def expire(self, *args, **kwargs):  # noqa: ANN002, ANN003
        self.ops.append(("expire", args, kwargs))
        return self

    async def execute(self):
        return [None, self.count, None, None]


class _FakeRedis:
    def __init__(self, *, count: int):
        self.pipe = _FakePipeline(count=count)

    def pipeline(self):
        return self.pipe


@pytest.mark.asyncio
async def test_redis_rate_limiter_check_and_limit_paths() -> None:
    limiter_ok = rate_limit_mod.RedisRateLimiter()
    limiter_ok._redis = _FakeRedis(count=1)
    await limiter_ok.check("ip:route", limit=2, window_seconds=60)
    ops = [name for (name, _args, _kwargs) in limiter_ok._redis.pipe.ops]
    assert ops == ["zremrangebyscore", "zcard", "zadd", "expire"]

    limiter_blocked = rate_limit_mod.RedisRateLimiter()
    limiter_blocked._redis = _FakeRedis(count=2)
    with pytest.raises(HTTPException) as exc:
        await limiter_blocked.check("ip:route", limit=2, window_seconds=60)
    assert exc.value.status_code == 429


@pytest.mark.asyncio
async def test_redis_rate_limiter_get_redis_caches_client(monkeypatch) -> None:
    calls = {"count": 0}
    shared = _FakeRedis(count=0)

    async def _fake_get_redis_client():
        calls["count"] += 1
        return shared

    monkeypatch.setattr("app.core.redis.get_redis_client", _fake_get_redis_client)
    limiter = rate_limit_mod.RedisRateLimiter()
    assert await limiter._get_redis() is shared
    assert await limiter._get_redis() is shared
    assert calls["count"] == 1


def test_create_rate_limiter_selects_expected_backend(monkeypatch) -> None:
    monkeypatch.setattr(rate_limit_mod, "_use_redis", True)
    monkeypatch.setattr(rate_limit_mod.settings, "redis_url_non_cde", "redis://test:6379/0")
    assert isinstance(rate_limit_mod._create_rate_limiter(), rate_limit_mod.RedisRateLimiter)

    monkeypatch.setattr(rate_limit_mod, "_use_redis", False)
    assert isinstance(rate_limit_mod._create_rate_limiter(), rate_limit_mod.InMemoryRateLimiter)
