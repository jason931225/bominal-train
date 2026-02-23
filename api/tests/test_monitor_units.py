from __future__ import annotations

import json
import runpy
import sys
from datetime import datetime, timedelta, timezone
from types import SimpleNamespace

import pytest

from app import monitor


class _ProcResult:
    def __init__(self, *, returncode: int, stdout: str):
        self.returncode = returncode
        self.stdout = stdout


class _FakeRedis:
    def __init__(self, *, fail_ping: bool = False):
        self.fail_ping = fail_ping
        self.closed = False

    async def ping(self):
        if self.fail_ping:
            raise RuntimeError("no redis")
        return True

    async def zcard(self, key: str):  # noqa: ARG002
        if key == "arq:queue":
            return 3
        if key == "arq:in-progress":
            return 1
        return 0

    async def scan_iter(self, pattern: str):
        if pattern == "arq:result:*":
            for item in [b"arq:result:1", b"arq:result:2"]:
                yield item
            return
        if pattern == "train_rate:*":
            for item in [b"train_rate:user1"]:
                yield item
            return
        if False:
            yield b""

    async def ttl(self, _key: bytes):
        return 10

    async def get(self, _key: bytes):
        return b"8"

    async def info(self, section: str):
        if section == "memory":
            return {"used_memory_human": "1M"}
        if section == "server":
            return {"uptime_in_seconds": 3661}
        if section == "clients":
            return {"connected_clients": 2}
        return {}

    async def aclose(self):
        self.closed = True


class _Result:
    def __init__(self, *, scalar_value=None, scalars_value=None):  # noqa: ANN001
        self._scalar = scalar_value
        self._scalars = scalars_value or []

    def scalar(self):  # noqa: ANN201
        return self._scalar

    def scalars(self):  # noqa: ANN201
        return SimpleNamespace(all=lambda: self._scalars)


class _DB:
    def __init__(self, results: list[_Result]):
        self.results = results

    async def execute(self, _query):  # noqa: ANN001
        return self.results.pop(0)


class _SessionCtx:
    def __init__(self, db: _DB):
        self.db = db

    async def __aenter__(self):
        return self.db

    async def __aexit__(self, *_args):
        return None


class _SessionFactory:
    def __init__(self, db: _DB):
        self.db = db

    def __call__(self):
        return _SessionCtx(self.db)


def test_get_container_status_success_and_fallback(monkeypatch):
    payload = json.dumps({"Name": "api-gateway", "State": "running", "Health": "healthy"})
    calls = {"n": 0}

    def _run(*_args, **_kwargs):  # noqa: ANN002, ANN003
        calls["n"] += 1
        if calls["n"] == 1:
            return _ProcResult(returncode=1, stdout="")
        return _ProcResult(returncode=0, stdout=f"{payload}\nnot-json")

    monkeypatch.setattr(monitor.subprocess, "run", _run)
    parsed = monitor.get_container_status()
    assert len(parsed) == 1
    assert parsed[0]["Name"] == "api-gateway"


def test_get_container_status_handles_subprocess_errors(monkeypatch):
    def _raise(*_args, **_kwargs):  # noqa: ANN002, ANN003
        raise FileNotFoundError("docker missing")

    monkeypatch.setattr(monitor.subprocess, "run", _raise)
    assert monitor.get_container_status() == []


def test_clear_screen_uses_platform_specific_command(monkeypatch):
    calls: list[str] = []
    monkeypatch.setattr(monitor.os, "system", lambda cmd: calls.append(cmd))

    monkeypatch.setattr(monitor.os, "name", "posix", raising=False)
    monitor.clear_screen()
    assert calls[-1] == "clear"

    monkeypatch.setattr(monitor.os, "name", "nt", raising=False)
    monitor.clear_screen()
    assert calls[-1] == "cls"


@pytest.mark.asyncio
async def test_get_redis_stats_success_and_error_paths():
    success = await monitor.get_redis_stats(_FakeRedis())
    assert success["connected"] is True
    assert success["queue_pending"] == 3
    assert success["queue_processing"] == 1
    assert success["queue_completed"] == 2
    assert success["rate_limiters"]

    error = await monitor.get_redis_stats(_FakeRedis(fail_ping=True))
    assert error["connected"] is False
    assert "error" in error


@pytest.mark.asyncio
async def test_get_db_stats_success_and_error_paths(monkeypatch):
    attempt_ok = SimpleNamespace(ok=True, error_code=None, started_at=datetime.now(timezone.utc))
    attempt_fail = SimpleNamespace(ok=False, error_code="E1", started_at=datetime.now(timezone.utc))
    task = SimpleNamespace(
        id="task-id-1",
        module="train",
        state="RUNNING",
        hidden_at=None,
        created_at=datetime.now(timezone.utc),
        updated_at=datetime.now(timezone.utc),
        attempts=[attempt_ok, attempt_fail],
    )

    db = _DB(
        [
            _Result(scalar_value=3),  # users
            _Result(scalar_value=9),  # total tasks
            _Result(scalar_value=2),  # active tasks
            _Result(scalar_value=4),  # tasks today
            _Result(scalar_value=1),  # completed today
            _Result(scalar_value=1),  # failed today
            _Result(scalar_value=10),  # attempts today
            _Result(scalar_value=2),  # failed attempts
            _Result(scalars_value=[task]),  # recent tasks
        ]
    )
    monkeypatch.setattr(monitor, "SessionLocal", _SessionFactory(db))

    stats = await monitor.get_db_stats()
    assert stats["connected"] is True
    assert stats["total_users"] == 3
    assert stats["error_rate_today"] == 20.0
    assert len(stats["recent_tasks"]) == 1

    class _BoomCtx:
        async def __aenter__(self):
            raise RuntimeError("db down")

        async def __aexit__(self, *_args):
            return None

    monkeypatch.setattr(monitor, "SessionLocal", lambda: _BoomCtx())
    failed = await monitor.get_db_stats()
    assert failed["connected"] is False
    assert "error" in failed


def test_formatters_and_printers(capsys):
    now = datetime.now(timezone.utc)
    assert monitor.format_time_ago(datetime.now(timezone.utc)).endswith("ago")
    assert monitor.format_time_ago(datetime.now(timezone.utc).replace(tzinfo=None)).endswith("ago")
    assert monitor.format_time_ago(now - timedelta(seconds=5)).endswith("ago")
    assert monitor.format_time_ago(now - timedelta(minutes=5)).endswith("ago")
    assert monitor.format_time_ago(now - timedelta(hours=5)).endswith("ago")
    assert monitor.format_time_ago(now - timedelta(days=2)).endswith("ago")

    assert monitor.state_color("QUEUED") == monitor.Colors.YELLOW
    assert monitor.state_color("unknown") == monitor.Colors.RESET

    assert monitor.format_uptime(10) == "10s"
    assert monitor.format_uptime(120) == "2m"
    assert monitor.format_uptime(3700).startswith("1h")
    assert monitor.format_uptime(90000).startswith("1d")

    monitor.print_containers([])
    monitor.print_redis_stats({"connected": False, "error": "down", "queue_pending": 0, "queue_processing": 0, "rate_limiters": [], "memory_used": "N/A"})
    monitor.print_db_stats({"connected": False, "error": "down"})
    monitor.print_recent_tasks([])
    output = capsys.readouterr().out
    assert "Container Status" in output
    assert "Redis disconnected" in output
    assert "Database disconnected" in output
    assert "No tasks found" in output


def test_printer_branches_for_nonempty_inputs(capsys):
    now = datetime.now(timezone.utc)
    monitor.print_containers(
        [
            {"Name": "api-gateway", "State": "running", "Health": "healthy"},
            {"Service": "worker-train", "State": "starting", "Health": "unhealthy"},
            {"Name": "redis", "State": "exited", "Health": ""},
            {"Name": "mailpit", "State": "created", "Health": ""},
        ]
    )
    monitor.print_redis_stats(
        {
            "connected": True,
            "queue_pending": 2,
            "queue_processing": 1,
            "queue_completed": 3,
            "rate_limiters": [{"key": "train_rate:user1", "tokens": "4", "ttl": 7}],
            "memory_used": "1M",
            "uptime_seconds": 3661,
            "total_connections": 4,
        }
    )
    monitor.print_db_stats(
        {
            "connected": True,
            "total_users": 3,
            "total_tasks": 6,
            "active_tasks": 2,
            "tasks_today": 4,
            "completed_today": 2,
            "failed_today": 1,
            "total_attempts_today": 5,
            "error_rate_today": 60.0,
        }
    )
    monitor.print_recent_tasks(
        [
            {
                "id": "abcd1234",
                "module": "train",
                "state": "FAILED",
                "hidden": True,
                "updated_at": now - timedelta(minutes=1),
                "attempts": 2,
                "last_error": "provider_error",
            }
        ]
    )
    output = capsys.readouterr().out
    assert "api-gateway" in output
    assert "Rate Limiters" in output
    assert "Attempts:" in output
    assert "FAILED(H)" in output


@pytest.mark.asyncio
async def test_run_monitor_single_snapshot_and_watch_interrupt(monkeypatch):
    fake_redis = _FakeRedis()
    monkeypatch.setattr(monitor.aioredis, "from_url", lambda *_args, **_kwargs: fake_redis)
    monkeypatch.setattr(monitor, "get_container_status", lambda: [])

    async def _redis_stats(_client):  # noqa: ANN001
        return {"connected": True, "queue_pending": 0, "queue_processing": 0, "queue_completed": 0, "rate_limiters": [], "memory_used": "1M", "uptime_seconds": 1, "total_connections": 1}

    async def _db_stats():
        return {"connected": True, "total_users": 0, "total_tasks": 0, "active_tasks": 0, "tasks_today": 0, "completed_today": 0, "failed_today": 0, "total_attempts_today": 0, "error_rate_today": 0.0, "recent_tasks": []}

    monkeypatch.setattr(monitor, "get_redis_stats", _redis_stats)
    monkeypatch.setattr(monitor, "get_db_stats", _db_stats)

    await monitor.run_monitor(watch=False, interval=1)
    assert fake_redis.closed is True

    fake_redis_watch = _FakeRedis()
    monkeypatch.setattr(monitor.aioredis, "from_url", lambda *_args, **_kwargs: fake_redis_watch)
    monkeypatch.setattr(monitor, "clear_screen", lambda: None)

    async def _sleep_interrupt(_seconds: int):
        raise KeyboardInterrupt

    monkeypatch.setattr(monitor.asyncio, "sleep", _sleep_interrupt)
    await monitor.run_monitor(watch=True, interval=1)
    assert fake_redis_watch.closed is True


def test_monitor_main_dispatch(monkeypatch):
    called = {}

    async def _fake_run_monitor(*, watch: bool, interval: int):
        called["watch"] = watch
        called["interval"] = interval

    def _fake_asyncio_run(coro):
        try:
            coro.send(None)
        except StopIteration:
            pass

    monkeypatch.setattr(monitor, "run_monitor", _fake_run_monitor)
    monkeypatch.setattr(monitor.asyncio, "run", _fake_asyncio_run)
    monkeypatch.setattr(monitor.sys, "argv", ["monitor", "--watch", "--interval", "5"])
    monitor.main()

    assert called == {"watch": True, "interval": 5}


def test_monitor_module_main_guard_executes(monkeypatch):
    called = {"ran": False}

    def _fake_asyncio_run(coro):
        called["ran"] = True
        coro.close()

    monkeypatch.setattr("asyncio.run", _fake_asyncio_run)
    monkeypatch.setattr("sys.argv", ["monitor"])
    sys.modules.pop("app.monitor", None)
    runpy.run_module("app.monitor", run_name="__main__")
    assert called["ran"] is True
