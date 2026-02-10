from __future__ import annotations

import asyncio

import fakeredis.aioredis
import pytest


@pytest.mark.asyncio
async def test_worker_heartbeat_sets_key_and_ttl(monkeypatch):
    from app import worker as worker_mod

    fake_redis = fakeredis.aioredis.FakeRedis()

    async def _fake_get_redis_client():
        return fake_redis

    monkeypatch.setattr(worker_mod, "get_redis_client", _fake_get_redis_client)

    shutdown = asyncio.Event()
    task = asyncio.create_task(worker_mod._heartbeat_loop(shutdown))
    try:
        # Heartbeat loop writes immediately, then sleeps.
        await asyncio.sleep(0.05)
        value = await fake_redis.get(worker_mod.HEARTBEAT_KEY)
        assert value is not None
        ttl = await fake_redis.ttl(worker_mod.HEARTBEAT_KEY)
        assert ttl > 0
    finally:
        shutdown.set()
        await task

