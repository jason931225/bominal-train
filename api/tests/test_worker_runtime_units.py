from __future__ import annotations

import asyncio
import signal
from uuid import uuid4

import pytest

from app import worker as worker_mod


def test_register_unregister_and_shutdown_signal_sets_event():
    worker_mod._in_flight_tasks.clear()
    worker_mod._register_in_flight("task-1")
    worker_mod._register_in_flight("task-2")
    assert worker_mod._in_flight_tasks == {"task-1", "task-2"}

    worker_mod._unregister_in_flight("task-1")
    assert worker_mod._in_flight_tasks == {"task-2"}

    worker_mod._shutdown_event = asyncio.Event()
    worker_mod._handle_shutdown_signal(signal.SIGTERM)
    assert worker_mod._shutdown_event.is_set() is True


@pytest.mark.asyncio
async def test_on_startup_populates_context_and_registers_handlers(monkeypatch):
    class _SessionCtx:
        async def __aenter__(self):
            return object()

        async def __aexit__(self, *_args):
            return None

    class _Loop:
        def __init__(self):
            self.registered: list[tuple[signal.Signals, object, signal.Signals]] = []

        def add_signal_handler(self, sig, handler, arg):  # noqa: ANN001
            self.registered.append((sig, handler, arg))

    loop = _Loop()
    task_sentinel = object()

    async def _fake_enqueue_recoverable_tasks(_db):  # noqa: ANN001
        return 3

    async def _fake_compact_and_prune_task_attempts(_db):  # noqa: ANN001
        return {
            "deleted_sync_rows": 0,
            "deleted_repetitive_rows": 0,
            "deleted_retention_rows": 0,
        }

    monkeypatch.setattr(worker_mod, "SessionLocal", lambda: _SessionCtx())
    monkeypatch.setattr(worker_mod, "enqueue_recoverable_tasks", _fake_enqueue_recoverable_tasks)
    monkeypatch.setattr(worker_mod, "compact_and_prune_task_attempts", _fake_compact_and_prune_task_attempts)
    monkeypatch.setattr(worker_mod.asyncio, "get_running_loop", lambda: loop)

    def _fake_create_task(coro):  # noqa: ANN001
        coro.close()
        return task_sentinel

    monkeypatch.setattr(worker_mod.asyncio, "create_task", _fake_create_task)

    ctx: dict = {}
    await worker_mod.on_startup(ctx)

    assert ctx["db_factory"] is worker_mod.SessionLocal
    assert ctx["register_in_flight"] is worker_mod._register_in_flight
    assert ctx["unregister_in_flight"] is worker_mod._unregister_in_flight
    assert ctx["heartbeat_task"] is task_sentinel
    assert worker_mod._shutdown_event is ctx["shutdown_event"]
    assert {item[0] for item in loop.registered} == {signal.SIGTERM, signal.SIGINT}


@pytest.mark.asyncio
async def test_on_shutdown_waits_for_inflight_and_recoveries(monkeypatch):
    worker_mod._in_flight_tasks.clear()
    worker_mod._in_flight_tasks.add("task-1")

    slept = {"count": 0}
    recovered = {"count": 0}

    async def _fake_sleep(_seconds: float):
        slept["count"] += 1

    async def _fake_recover():
        recovered["count"] += 1
        worker_mod._in_flight_tasks.clear()
        return 2

    monkeypatch.setattr(worker_mod.asyncio, "sleep", _fake_sleep)
    monkeypatch.setattr(worker_mod, "_recover_in_flight_tasks", _fake_recover)
    await worker_mod.on_shutdown({})

    assert slept["count"] == 1
    assert recovered["count"] == 1


@pytest.mark.asyncio
async def test_heartbeat_loop_handles_redis_failures_and_timeout(monkeypatch):
    shutdown_event = asyncio.Event()
    called = {"redis": 0, "wait_for": 0}

    async def _failing_get_redis_client():
        called["redis"] += 1
        raise RuntimeError("redis down")

    async def _fake_wait_for(awaitable, timeout: float):  # noqa: ANN001
        called["wait_for"] += 1
        # Prevent "coroutine was never awaited" warnings for shutdown_event.wait().
        awaitable.close()
        shutdown_event.set()
        raise asyncio.TimeoutError

    monkeypatch.setattr(worker_mod, "get_redis_client", _failing_get_redis_client)
    monkeypatch.setattr(worker_mod.asyncio, "wait_for", _fake_wait_for)

    await worker_mod._heartbeat_loop(shutdown_event)
    assert called["redis"] == 1
    assert called["wait_for"] == 1


@pytest.mark.asyncio
async def test_recover_in_flight_returns_zero_when_nothing_tracked():
    worker_mod._in_flight_tasks.clear()
    assert await worker_mod._recover_in_flight_tasks() == 0


@pytest.mark.asyncio
async def test_recover_in_flight_logs_warning_and_discards_on_error(monkeypatch):
    worker_mod._in_flight_tasks.clear()
    bad_task_id = str(uuid4())
    worker_mod._in_flight_tasks.add(bad_task_id)

    class _SessionCtx:
        async def __aenter__(self):
            class _DB:
                async def get(self, *_args, **_kwargs):  # noqa: ANN002, ANN003
                    raise RuntimeError("db exploded")

            return _DB()

        async def __aexit__(self, *_args):
            return None

    warnings: list[str] = []

    monkeypatch.setattr(worker_mod, "SessionLocal", lambda: _SessionCtx())
    monkeypatch.setattr(worker_mod.logger, "warning", lambda msg, *_args: warnings.append(str(msg)))

    recovered = await worker_mod._recover_in_flight_tasks()
    assert recovered == 0
    assert worker_mod._in_flight_tasks == set()
    assert warnings
