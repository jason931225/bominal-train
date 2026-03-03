from __future__ import annotations

import asyncio
import random
import time
from dataclasses import dataclass

from redis.asyncio import Redis
from redis.exceptions import NoScriptError, ResponseError

from app.modules.train.constants import DEFAULT_BUCKET_CONFIG

TOKEN_BUCKET_LUA = """
local key = KEYS[1]
local now_ms = tonumber(ARGV[1])
local capacity = tonumber(ARGV[2])
local refill_per_ms = tonumber(ARGV[3])
local requested = tonumber(ARGV[4])

local tokens = tonumber(redis.call('HGET', key, 'tokens'))
local ts = tonumber(redis.call('HGET', key, 'ts'))

if tokens == nil then
  tokens = capacity
end
if ts == nil then
  ts = now_ms
end

local delta = now_ms - ts
if delta > 0 then
  tokens = math.min(capacity, tokens + (delta * refill_per_ms))
end

local allowed = 0
local wait_ms = 0

if tokens >= requested then
  tokens = tokens - requested
  allowed = 1
else
  if refill_per_ms > 0 then
    wait_ms = math.ceil((requested - tokens) / refill_per_ms)
  else
    wait_ms = 1000
  end
end

redis.call('HSET', key, 'tokens', tokens, 'ts', now_ms)
local ttl_ms = math.max(1000, math.ceil((capacity / refill_per_ms) * 2))
redis.call('PEXPIRE', key, ttl_ms)

return {allowed, wait_ms, tokens}
"""


@dataclass(slots=True)
class RateLimitAcquireResult:
    waited_ms: int
    rounds: int


class RedisTokenBucketLimiter:
    def __init__(self, redis: Redis):
        self._redis = redis
        self._sha: str | None = None
        self._script_supported: bool | None = None
        self._fallback_lock = asyncio.Lock()

    async def _ensure_script(self) -> str | None:
        if self._script_supported is False:
            return None
        if self._sha is None:
            try:
                self._sha = await self._redis.script_load(TOKEN_BUCKET_LUA)
            except ResponseError as exc:
                if "unknown command" in str(exc).lower():
                    self._script_supported = False
                    self._sha = None
                    return None
                raise
            self._script_supported = True
        return self._sha

    @staticmethod
    def _to_float(value: object, *, default: float) -> float:
        if value is None:
            return default
        try:
            return float(value)
        except (TypeError, ValueError):
            return default

    async def _acquire_once_fallback(
        self,
        key: str,
        *,
        capacity: float,
        refill_per_second: float,
        requested: float = 1.0,
    ) -> tuple[bool, int]:
        now_ms = int(time.time() * 1000)
        refill_per_ms = refill_per_second / 1000.0

        async with self._fallback_lock:
            stored_tokens = await self._redis.hget(key, "tokens")
            stored_ts = await self._redis.hget(key, "ts")

            tokens = self._to_float(stored_tokens, default=capacity)
            ts_ms = self._to_float(stored_ts, default=float(now_ms))

            delta_ms = max(0.0, float(now_ms) - ts_ms)
            if delta_ms > 0:
                tokens = min(capacity, tokens + (delta_ms * refill_per_ms))

            allowed = False
            wait_ms = 0
            if tokens >= requested:
                tokens -= requested
                allowed = True
            elif refill_per_ms > 0:
                wait_ms = int(((requested - tokens) / refill_per_ms) + 0.999999)
            else:
                wait_ms = 1000

            await self._redis.hset(
                key,
                mapping={
                    "tokens": tokens,
                    "ts": now_ms,
                },
            )
            if refill_per_ms > 0:
                ttl_ms = max(1000, int(((capacity / refill_per_ms) * 2) + 0.999999))
            else:
                ttl_ms = 1000
            await self._redis.pexpire(key, ttl_ms)

        return allowed, wait_ms

    async def _acquire_once(
        self,
        key: str,
        *,
        capacity: float,
        refill_per_second: float,
        requested: float = 1.0,
    ) -> tuple[bool, int]:
        sha = await self._ensure_script()
        now_ms = int(time.time() * 1000)
        refill_per_ms = refill_per_second / 1000.0

        if sha is not None:
            try:
                allowed, wait_ms, _ = await self._redis.evalsha(
                    sha,
                    1,
                    key,
                    now_ms,
                    capacity,
                    refill_per_ms,
                    requested,
                )
                return bool(allowed), int(wait_ms)
            except NoScriptError:
                self._sha = None
            except ResponseError as exc:
                if "unknown command" not in str(exc).lower():
                    raise
                self._script_supported = False

        return await self._acquire_once_fallback(
            key,
            capacity=capacity,
            refill_per_second=refill_per_second,
            requested=requested,
        )

    async def acquire_provider_call(
        self,
        *,
        provider: str,
        user_bucket_key: str,
        host_bucket_key: str,
        requested: float = 1.0,
    ) -> RateLimitAcquireResult:
        total_wait = 0
        rounds = 0

        while True:
            rounds += 1
            checks = [
                (
                    f"rl:train:host:{host_bucket_key}",
                    DEFAULT_BUCKET_CONFIG["global"]["capacity"],
                    DEFAULT_BUCKET_CONFIG["global"]["refill_per_second"],
                ),
                (
                    f"rl:train:provider:{provider}",
                    DEFAULT_BUCKET_CONFIG["provider"]["capacity"],
                    DEFAULT_BUCKET_CONFIG["provider"]["refill_per_second"],
                ),
                (
                    f"rl:train:user:{provider}:{user_bucket_key}",
                    DEFAULT_BUCKET_CONFIG["credential"]["capacity"],
                    DEFAULT_BUCKET_CONFIG["credential"]["refill_per_second"],
                ),
            ]

            waits: list[int] = []
            for key, capacity, refill in checks:
                allowed, wait_ms = await self._acquire_once(
                    key,
                    capacity=capacity,
                    refill_per_second=refill,
                    requested=requested,
                )
                if not allowed:
                    waits.append(wait_ms)

            if not waits:
                return RateLimitAcquireResult(waited_ms=total_wait, rounds=rounds)

            base_wait_ms = max(waits)
            jitter_ms = int(random.uniform(10, 90))
            sleep_ms = max(20, base_wait_ms + jitter_ms)
            total_wait += sleep_ms
            await asyncio.sleep(sleep_ms / 1000)
