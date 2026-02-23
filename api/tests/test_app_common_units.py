from __future__ import annotations

from fastapi.testclient import TestClient

from app.http import app_common


class _SessionContext:
    def __init__(self, *, should_fail: bool):
        self.should_fail = should_fail

    async def __aenter__(self):
        return self

    async def __aexit__(self, *_args):
        return None

    async def execute(self, _query):  # noqa: ANN001
        if self.should_fail:
            raise RuntimeError("db failed")
        return None


class _RedisClient:
    def __init__(self, *, should_fail: bool):
        self.should_fail = should_fail

    async def ping(self):
        if self.should_fail:
            raise RuntimeError("redis failed")
        return True


def test_create_base_app_healthcheck_happy_path(monkeypatch):
    monkeypatch.setattr(app_common, "SessionLocal", lambda: _SessionContext(should_fail=False))

    async def _fake_get_redis_client():
        return _RedisClient(should_fail=False)

    monkeypatch.setattr("app.core.redis.get_redis_client", _fake_get_redis_client)

    app = app_common.create_base_app(description="test app")
    client = TestClient(app)
    response = client.get("/health")
    payload = response.json()

    assert response.status_code == 200
    assert payload["status"] == "ok"
    assert payload["db"] is True
    assert payload["redis"] is True


def test_create_base_app_healthcheck_degraded_paths(monkeypatch):
    # DB failure path.
    monkeypatch.setattr(app_common, "SessionLocal", lambda: _SessionContext(should_fail=True))

    async def _redis_ok():
        return _RedisClient(should_fail=False)

    monkeypatch.setattr("app.core.redis.get_redis_client", _redis_ok)

    app = app_common.create_base_app(description="db-fail")
    client = TestClient(app)
    payload = client.get("/health").json()
    assert payload["status"] == "degraded"
    assert payload["db"] is False
    assert payload["redis"] is True

    # Redis failure path.
    monkeypatch.setattr(app_common, "SessionLocal", lambda: _SessionContext(should_fail=False))

    async def _redis_fail():
        return _RedisClient(should_fail=True)

    monkeypatch.setattr("app.core.redis.get_redis_client", _redis_fail)
    app_redis_fail = app_common.create_base_app(description="redis-fail")
    client_redis_fail = TestClient(app_redis_fail)
    payload_redis_fail = client_redis_fail.get("/health").json()
    assert payload_redis_fail["status"] == "degraded"
    assert payload_redis_fail["db"] is True
    assert payload_redis_fail["redis"] is False


def test_global_exception_handler_returns_internal_server_error():
    app = app_common.create_base_app(description="errors")

    @app.get("/boom")
    async def _boom():
        raise RuntimeError("boom")

    client = TestClient(app, raise_server_exceptions=False)
    response = client.get("/boom")
    assert response.status_code == 500
    assert response.json() == {"detail": "Internal server error"}


def test_add_admin_docs_registers_routes():
    app = app_common.create_base_app(description="docs")
    app_common.add_admin_docs(app)
    paths = {route.path for route in app.routes}
    assert "/api/docs" in paths
    assert "/api/openapi.json" in paths


def test_lifespan_and_admin_docs_handlers(monkeypatch):
    log_events: list[str] = []

    class _Logger:
        def info(self, message: str, **_kwargs):  # noqa: ANN003
            log_events.append(message)

    monkeypatch.setattr(app_common, "logger", _Logger())
    monkeypatch.setattr(app_common, "setup_logging", lambda: log_events.append("setup"))

    app = app_common.create_base_app(description="docs-live")
    app.dependency_overrides[app_common.get_current_admin] = lambda: object()
    app_common.add_admin_docs(app)

    with TestClient(app) as client:
        docs_response = client.get("/api/docs")
        openapi_response = client.get("/api/openapi.json")

    assert docs_response.status_code == 200
    assert openapi_response.status_code == 200
    assert "openapi" in openapi_response.json()
    assert "setup" in log_events
    assert "Application starting" in log_events
    assert "Application shutting down" in log_events
