from __future__ import annotations

import pytest
from fastapi import FastAPI
from starlette.requests import Request

from app import main as main_mod


def test_redis_guard_helpers() -> None:
    assert main_mod._redis_save_is_disabled({"save": ""})
    assert not main_mod._redis_save_is_disabled({"save": "900 1"})
    assert main_mod._redis_appendonly_is_disabled({"appendonly": "no"})
    assert main_mod._redis_appendonly_is_disabled({"appendonly": "0"})
    assert main_mod._redis_appendonly_is_disabled({"appendonly": ""})
    assert not main_mod._redis_appendonly_is_disabled({"appendonly": "yes"})


@pytest.mark.asyncio
async def test_enforce_production_security_guards_paths(monkeypatch) -> None:
    monkeypatch.setattr(main_mod.settings, "app_env", "development")
    await main_mod._enforce_production_security_guards()

    monkeypatch.setattr(main_mod.settings, "app_env", "production")
    monkeypatch.setattr(main_mod.settings, "payment_enabled", False)

    async def _should_not_run():
        raise AssertionError("CDE redis guard should be skipped when payment is disabled")

    monkeypatch.setattr("app.core.redis.get_cde_redis_client", _should_not_run)
    await main_mod._enforce_production_security_guards()
    monkeypatch.setattr(main_mod.settings, "payment_enabled", True)

    class _RedisOK:
        async def config_get(self, key: str):
            if key == "save":
                return {"save": ""}
            return {"appendonly": "no"}

    async def _redis_ok_client():
        return _RedisOK()

    monkeypatch.setattr("app.core.redis.get_cde_redis_client", _redis_ok_client)
    await main_mod._enforce_production_security_guards()

    class _RedisFail:
        async def config_get(self, _key: str):
            raise RuntimeError("boom")

    async def _redis_fail_client():
        return _RedisFail()

    monkeypatch.setattr("app.core.redis.get_cde_redis_client", _redis_fail_client)
    with pytest.raises(RuntimeError, match="Failed to verify Redis persistence security guard"):
        await main_mod._enforce_production_security_guards()

    class _RedisPersistent:
        async def config_get(self, key: str):
            if key == "save":
                return {"save": "900 1"}
            return {"appendonly": "yes"}

    async def _redis_persistent_client():
        return _RedisPersistent()

    monkeypatch.setattr("app.core.redis.get_cde_redis_client", _redis_persistent_client)
    with pytest.raises(RuntimeError, match="Redis persistence must be disabled"):
        await main_mod._enforce_production_security_guards()


@pytest.mark.asyncio
async def test_lifespan_calls_setup_and_logs(monkeypatch) -> None:
    called = {"setup": 0, "guard": 0, "logs": []}

    def _fake_setup_logging() -> None:
        called["setup"] += 1

    async def _fake_guard() -> None:
        called["guard"] += 1

    class _Logger:
        def info(self, message: str, **_kwargs):  # noqa: ANN003
            called["logs"].append(message)

    monkeypatch.setattr(main_mod, "setup_logging", _fake_setup_logging)
    monkeypatch.setattr(main_mod, "_enforce_production_security_guards", _fake_guard)
    monkeypatch.setattr(main_mod, "logger", _Logger())

    async with main_mod.lifespan(FastAPI()):
        pass

    assert called["setup"] == 1
    assert called["guard"] == 1
    assert called["logs"] == ["Application starting", "Application shutting down"]


@pytest.mark.asyncio
async def test_global_exception_handler_redacts_and_returns_500(monkeypatch) -> None:
    captured = {}

    def _fake_redact(payload: dict):  # noqa: ANN001
        return {"safe": True, **payload}

    class _Logger:
        def exception(self, _message: str, **kwargs):  # noqa: ANN003
            captured["extra"] = kwargs.get("extra")

    monkeypatch.setattr(main_mod, "redact_sensitive", _fake_redact)
    monkeypatch.setattr(main_mod, "logger", _Logger())

    request = Request({"type": "http", "method": "GET", "path": "/boom", "headers": []})
    response = await main_mod.global_exception_handler(request, RuntimeError("boom"))

    assert response.status_code == 500
    assert b"Internal server error" in response.body
    assert captured["extra"]["safe"] is True
    assert captured["extra"]["path"] == "/boom"


@pytest.mark.asyncio
async def test_healthcheck_success_and_degraded_paths(monkeypatch) -> None:
    class _SessionCtx:
        async def __aenter__(self):
            return self

        async def __aexit__(self, *_args):
            return None

        async def execute(self, _query):  # noqa: ANN001
            return None

    class _Redis:
        def __init__(self, *, fail: bool):
            self.fail = fail

        async def ping(self):
            if self.fail:
                raise RuntimeError("redis failed")
            return True

    monkeypatch.setattr(main_mod, "SessionLocal", lambda: _SessionCtx())

    async def _redis_ok():
        return _Redis(fail=False)

    monkeypatch.setattr("app.core.redis.get_redis_client", _redis_ok)
    monkeypatch.setattr("app.core.redis.get_cde_redis_client", _redis_ok)
    healthy = await main_mod.healthcheck()
    assert healthy["status"] == "ok"
    assert healthy["redis_non_cde"] is True
    assert healthy["redis_cde"] is True

    class _BrokenSessionCtx:
        async def __aenter__(self):
            raise RuntimeError("db down")

        async def __aexit__(self, *_args):
            return None

    monkeypatch.setattr(main_mod, "SessionLocal", lambda: _BrokenSessionCtx())
    degraded_db = await main_mod.healthcheck()
    assert degraded_db["status"] == "degraded"
    assert degraded_db["db"] is False

    monkeypatch.setattr(main_mod, "SessionLocal", lambda: _SessionCtx())

    async def _redis_fail():
        return _Redis(fail=True)

    monkeypatch.setattr("app.core.redis.get_redis_client", _redis_fail)
    monkeypatch.setattr("app.core.redis.get_cde_redis_client", _redis_fail)
    degraded_redis = await main_mod.healthcheck()
    assert degraded_redis["status"] == "degraded"
    assert degraded_redis["redis"] is False


@pytest.mark.asyncio
async def test_version_info_reports_runtime_versions(monkeypatch) -> None:
    monkeypatch.setenv("APP_VERSION", " 9.8.7 ")
    monkeypatch.setenv("BUILD_VERSION", " commit-abc123 ")

    payload = await main_mod.version_info()
    assert payload == {
        "app": main_mod.settings.app_name,
        "app_version": "9.8.7",
        "build_version": "commit-abc123",
    }

    monkeypatch.setenv("APP_VERSION", " ")
    monkeypatch.delenv("BUILD_VERSION", raising=False)

    default_payload = await main_mod.version_info()
    assert default_payload["app_version"] == "0.0.0"
    assert default_payload["build_version"] == "unknown"


def test_main_app_exposes_version_routes() -> None:
    paths = {route.path for route in main_mod.app.routes}
    assert "/version" in paths
    assert "/api/version" in paths


@pytest.mark.asyncio
async def test_admin_docs_endpoints_return_schema() -> None:
    docs_response = await main_mod.admin_swagger_ui(None)
    assert docs_response.status_code == 200
    assert "/api/openapi.json" in docs_response.body.decode("utf-8")

    schema = await main_mod.admin_openapi(None)
    assert isinstance(schema, dict)
    assert "paths" in schema
