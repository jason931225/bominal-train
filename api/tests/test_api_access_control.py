from __future__ import annotations

import pytest
from sqlalchemy import select
from starlette.requests import Request

from app.db.models import Role, User


async def _register_and_login(client, *, email: str, display_name: str) -> str:
    register_response = await client.post(
        "/api/auth/register",
        json={"email": email, "password": "SuperSecret123", "display_name": display_name},
    )
    assert register_response.status_code == 201

    login_response = await client.post(
        "/api/auth/login",
        json={"email": email, "password": "SuperSecret123", "remember_me": False},
    )
    assert login_response.status_code == 200
    session_cookie = login_response.cookies.get("bominal_session")
    assert session_cookie
    return session_cookie


@pytest.mark.asyncio
async def test_public_auth_routes_allow_unauthenticated_access(client):
    password_reset = await client.post("/api/auth/request-password-reset")
    assert password_reset.status_code == 200

    email_verification = await client.post("/api/auth/request-email-verification")
    assert email_verification.status_code == 200


@pytest.mark.asyncio
async def test_authenticated_routes_require_session_cookie(client):
    unauthenticated_paths = [
        "/api/auth/me",
        "/api/modules",
        "/api/train/stations",
        "/api/wallet/payment-card",
        "/api/notifications/email/status",
    ]

    for path in unauthenticated_paths:
        response = await client.get(path)
        assert response.status_code == 401


@pytest.mark.asyncio
async def test_admin_route_requires_admin_role(client, db_session):
    session_cookie = await _register_and_login(
        client,
        email="access-admin@example.com",
        display_name="Access Admin User",
    )

    client.cookies.clear()
    unauthorized = await client.get("/api/admin")
    assert unauthorized.status_code == 401

    forbidden = await client.get("/api/admin", cookies={"bominal_session": session_cookie})
    assert forbidden.status_code == 403

    user = (await db_session.execute(select(User).where(User.email == "access-admin@example.com"))).scalar_one()
    admin_role = (await db_session.execute(select(Role).where(Role.name == "admin"))).scalar_one()
    user.role_id = admin_role.id
    await db_session.commit()

    allowed = await client.get("/api/admin", cookies={"bominal_session": session_cookie})
    assert allowed.status_code == 200
    assert allowed.json()["message"] == "Admin access granted"


@pytest.mark.asyncio
async def test_internal_route_requires_internal_api_key(client, monkeypatch):
    monkeypatch.setattr("app.http.deps.settings.internal_api_key", "internal-test-key")

    missing = await client.get("/api/internal/health")
    assert missing.status_code == 403

    wrong = await client.get("/api/internal/health", headers={"X-Internal-Api-Key": "wrong-key"})
    assert wrong.status_code == 403

    valid = await client.get("/api/internal/health", headers={"X-Internal-Api-Key": "internal-test-key"})
    assert valid.status_code == 200
    assert "internal access granted" in valid.json()["message"]


@pytest.mark.asyncio
async def test_internal_route_returns_503_when_not_configured(client, monkeypatch):
    monkeypatch.setattr("app.http.deps.settings.internal_api_key", None)

    response = await client.get("/api/internal/health", headers={"X-Internal-Api-Key": "ignored"})
    assert response.status_code == 503


def _build_request_for_rate_limit(
    *,
    path: str = "/api/auth/login",
    headers: dict[str, str] | None = None,
    client: tuple[str, int] | None = ("198.51.100.10", 54321),
) -> Request:
    raw_headers = []
    for key, value in (headers or {}).items():
        raw_headers.append((key.lower().encode("utf-8"), value.encode("utf-8")))
    scope = {
        "type": "http",
        "http_version": "1.1",
        "method": "POST",
        "scheme": "http",
        "path": path,
        "raw_path": path.encode("utf-8"),
        "query_string": b"",
        "headers": raw_headers,
        "client": client,
        "server": ("testserver", 80),
        "root_path": "",
    }
    return Request(scope)


@pytest.mark.asyncio
async def test_auth_rate_limit_prefers_cf_connecting_ip(monkeypatch):
    from app.http.deps import auth_rate_limit

    captured: dict[str, str] = {}

    async def _fake_check(*, key: str, limit: int, window_seconds: int) -> None:
        captured["key"] = key

    monkeypatch.setattr("app.http.deps.rate_limiter.check", _fake_check)
    request = _build_request_for_rate_limit(headers={"cf-connecting-ip": "203.0.113.2"})
    await auth_rate_limit(request)
    assert captured["key"] == "auth:203.0.113.2:/api/auth/login"


@pytest.mark.asyncio
async def test_auth_rate_limit_uses_first_forwarded_for_ip(monkeypatch):
    from app.http.deps import auth_rate_limit

    captured: dict[str, str] = {}

    async def _fake_check(*, key: str, limit: int, window_seconds: int) -> None:
        captured["key"] = key

    monkeypatch.setattr("app.http.deps.rate_limiter.check", _fake_check)
    request = _build_request_for_rate_limit(headers={"x-forwarded-for": "203.0.113.9, 10.0.0.1"})
    await auth_rate_limit(request)
    assert captured["key"] == "auth:203.0.113.9:/api/auth/login"


@pytest.mark.asyncio
async def test_auth_rate_limit_falls_back_to_client_host(monkeypatch):
    from app.http.deps import auth_rate_limit

    captured: dict[str, str] = {}

    async def _fake_check(*, key: str, limit: int, window_seconds: int) -> None:
        captured["key"] = key

    monkeypatch.setattr("app.http.deps.rate_limiter.check", _fake_check)
    request = _build_request_for_rate_limit(client=("198.51.100.44", 54321))
    await auth_rate_limit(request)
    assert captured["key"] == "auth:198.51.100.44:/api/auth/login"


@pytest.mark.asyncio
async def test_auth_rate_limit_uses_unknown_when_no_ip(monkeypatch):
    from app.http.deps import auth_rate_limit

    captured: dict[str, str] = {}

    async def _fake_check(*, key: str, limit: int, window_seconds: int) -> None:
        captured["key"] = key

    monkeypatch.setattr("app.http.deps.rate_limiter.check", _fake_check)
    request = _build_request_for_rate_limit(client=None)
    await auth_rate_limit(request)
    assert captured["key"] == "auth:unknown:/api/auth/login"
