from __future__ import annotations

import pytest
from sqlalchemy import select

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
