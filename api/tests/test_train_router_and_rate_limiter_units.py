from __future__ import annotations

from types import SimpleNamespace
from uuid import uuid4

import pytest
from redis.exceptions import NoScriptError, ResponseError

from app.modules.train import router as train_router
from app.modules.train.rate_limiter import RedisTokenBucketLimiter


class _FakeRedis:
    def __init__(
        self,
        *,
        script_sha: str = "sha-1",
        script_load_exc: Exception | None = None,
        evalsha_result: tuple[int, int, float] = (1, 0, 0.0),
        evalsha_exc: Exception | None = None,
    ) -> None:
        self.script_sha = script_sha
        self.script_load_exc = script_load_exc
        self.script_load_calls = 0
        self.evalsha_result = evalsha_result
        self.evalsha_exc = evalsha_exc
        self.hmap: dict[tuple[str, str], object] = {}
        self.hset_calls: list[tuple[str, dict[str, object]]] = []
        self.pexpire_calls: list[tuple[str, int]] = []

    async def script_load(self, _script: str) -> str:
        self.script_load_calls += 1
        if self.script_load_exc is not None:
            raise self.script_load_exc
        return self.script_sha

    async def evalsha(self, *_args):  # noqa: ANN002
        if self.evalsha_exc is not None:
            raise self.evalsha_exc
        return self.evalsha_result

    async def hget(self, key: str, field: str):  # noqa: ANN201
        return self.hmap.get((key, field))

    async def hset(self, key: str, mapping: dict[str, object]) -> None:
        self.hset_calls.append((key, mapping))
        for field, value in mapping.items():
            self.hmap[(key, field)] = value

    async def pexpire(self, key: str, ttl_ms: int) -> None:
        self.pexpire_calls.append((key, ttl_ms))


def _stub_return(tag: str):
    async def _inner(*_args, **_kwargs):  # noqa: ANN002, ANN003
        return {"tag": tag}

    return _inner


@pytest.mark.asyncio
async def test_train_router_wrappers_delegate_to_service_functions(monkeypatch):
    user = SimpleNamespace(id=uuid4())
    db = object()
    task_id = uuid4()
    artifact_id = uuid4()

    monkeypatch.setattr(train_router, "get_srt_credential_status", _stub_return("get_srt"))
    monkeypatch.setattr(train_router, "get_ktx_credential_status", _stub_return("get_ktx"))
    monkeypatch.setattr(train_router, "clear_srt_credentials", _stub_return("clear_srt"))
    monkeypatch.setattr(train_router, "clear_ktx_credentials", _stub_return("clear_ktx"))
    monkeypatch.setattr(train_router, "pause_task", _stub_return("pause"))
    monkeypatch.setattr(train_router, "resume_task", _stub_return("resume"))
    monkeypatch.setattr(train_router, "retry_task_now", _stub_return("retry"))
    monkeypatch.setattr(train_router, "cancel_task", _stub_return("cancel"))
    monkeypatch.setattr(train_router, "delete_task", _stub_return("delete"))
    monkeypatch.setattr(train_router, "list_provider_reservations", _stub_return("reservations"))
    monkeypatch.setattr(train_router, "get_provider_ticket_info", _stub_return("ticket_info"))
    monkeypatch.setattr(train_router, "cancel_provider_reservation", _stub_return("cancel_reservation"))

    assert (await train_router.get_srt_credentials(user=user, db=db))["tag"] == "get_srt"
    assert (await train_router.get_ktx_credentials(user=user, db=db))["tag"] == "get_ktx"
    assert (await train_router.signout_srt_credentials(user=user, db=db))["tag"] == "clear_srt"
    assert (await train_router.signout_ktx_credentials(user=user, db=db))["tag"] == "clear_ktx"
    assert (await train_router.pause_train_task(task_id=task_id, user=user, db=db))["tag"] == "pause"
    assert (await train_router.resume_train_task(task_id=task_id, user=user, db=db))["tag"] == "resume"
    assert (await train_router.retry_train_task(task_id=task_id, user=user, db=db))["tag"] == "retry"
    assert (await train_router.cancel_train_task(task_id=task_id, user=user, db=db))["tag"] == "cancel"
    assert (await train_router.delete_train_task(task_id=task_id, user=user, db=db))["tag"] == "delete"
    assert (
        await train_router.list_train_provider_reservations(
            provider="SRT",
            paid_only=False,
            user=user,
            db=db,
        )
    )["tag"] == "reservations"
    assert (
        await train_router.get_train_provider_ticket_info(
            provider="KTX",
            reservation_id="PNR-1",
            user=user,
            db=db,
        )
    )["tag"] == "ticket_info"
    assert (
        await train_router.cancel_train_provider_reservation(
            provider="SRT",
            reservation_id="PNR-2",
            user=user,
            db=db,
        )
    )["tag"] == "cancel_reservation"


@pytest.mark.asyncio
async def test_rate_limiter_script_loading_and_script_support_detection():
    limiter = RedisTokenBucketLimiter(_FakeRedis())
    sha = await limiter._ensure_script()
    assert sha == "sha-1"
    assert limiter._script_supported is True
    # The cached SHA should avoid a second script-load.
    assert await limiter._ensure_script() == "sha-1"
    assert limiter._redis.script_load_calls == 1

    unsupported = RedisTokenBucketLimiter(
        _FakeRedis(script_load_exc=ResponseError("ERR unknown command 'SCRIPT'"))
    )
    assert await unsupported._ensure_script() is None
    assert unsupported._script_supported is False

    exploding = RedisTokenBucketLimiter(_FakeRedis(script_load_exc=ResponseError("ERR generic")))
    with pytest.raises(ResponseError):
        await exploding._ensure_script()

    assert RedisTokenBucketLimiter._to_float(None, default=1.2) == 1.2
    assert RedisTokenBucketLimiter._to_float("bad", default=3.4) == 3.4


@pytest.mark.asyncio
async def test_rate_limiter_fallback_branch_with_zero_refill_sets_min_wait_and_ttl():
    redis = _FakeRedis()
    limiter = RedisTokenBucketLimiter(redis)

    allowed, wait_ms = await limiter._acquire_once_fallback(
        "rl:key",
        capacity=1.0,
        refill_per_second=0.0,
        requested=2.0,
    )

    assert allowed is False
    assert wait_ms == 1000
    assert redis.pexpire_calls
    assert redis.pexpire_calls[-1] == ("rl:key", 1000)


@pytest.mark.asyncio
async def test_rate_limiter_acquire_once_handles_evalsha_noscript_and_response_errors(monkeypatch):
    redis = _FakeRedis(evalsha_exc=NoScriptError("NOSCRIPT"))
    limiter = RedisTokenBucketLimiter(redis)
    limiter._sha = "cached-sha"
    limiter._script_supported = True

    async def _fallback(*_args, **_kwargs):  # noqa: ANN002, ANN003
        return True, 0

    monkeypatch.setattr(limiter, "_acquire_once_fallback", _fallback)
    assert await limiter._acquire_once("rl:key", capacity=1.0, refill_per_second=1.0, requested=1.0) == (True, 0)
    assert limiter._sha is None

    redis_unknown = _FakeRedis(evalsha_exc=ResponseError("ERR unknown command"))
    limiter_unknown = RedisTokenBucketLimiter(redis_unknown)
    limiter_unknown._sha = "cached-sha"
    limiter_unknown._script_supported = True
    monkeypatch.setattr(limiter_unknown, "_acquire_once_fallback", _fallback)
    assert await limiter_unknown._acquire_once("rl:key", capacity=1.0, refill_per_second=1.0, requested=1.0) == (True, 0)
    assert limiter_unknown._script_supported is False

    redis_generic = _FakeRedis(evalsha_exc=ResponseError("ERR generic"))
    limiter_generic = RedisTokenBucketLimiter(redis_generic)
    limiter_generic._sha = "cached-sha"
    limiter_generic._script_supported = True
    with pytest.raises(ResponseError):
        await limiter_generic._acquire_once("rl:key", capacity=1.0, refill_per_second=1.0, requested=1.0)


@pytest.mark.asyncio
async def test_rate_limiter_acquire_once_uses_evalsha_result_when_available():
    redis = _FakeRedis(evalsha_result=(1, 12, 0.5))
    limiter = RedisTokenBucketLimiter(redis)
    limiter._sha = "cached-sha"
    limiter._script_supported = True

    allowed, wait_ms = await limiter._acquire_once(
        "rl:key",
        capacity=2.0,
        refill_per_second=1.5,
        requested=1.0,
    )

    assert allowed is True
    assert wait_ms == 12
