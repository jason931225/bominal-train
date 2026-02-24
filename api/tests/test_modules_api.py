from __future__ import annotations

from uuid import uuid4

import pytest
from sqlalchemy import select

from app.db.models import User
from app.modules.restaurant.capabilities import (
    RESTAURANT_CAPABILITIES_COMING_SOON,
    RESTAURANT_CAPABILITIES_EXPOSED,
)


async def _register_and_login(client, db_session, *, email: str, display_name: str) -> str:
    register_response = await client.post(
        "/api/auth/register",
        json={"email": email, "password": "SuperSecret123", "display_name": display_name},
    )
    assert register_response.status_code == 201

    user = (await db_session.execute(select(User).where(User.email == email))).scalar_one()
    user.access_status = "approved"
    await db_session.commit()

    login_response = await client.post(
        "/api/auth/login",
        json={"email": email, "password": "SuperSecret123", "remember_me": False},
    )
    assert login_response.status_code == 200
    session_cookie = login_response.cookies.get("bominal_session")
    assert session_cookie
    return session_cookie


@pytest.mark.asyncio
async def test_modules_response_includes_enabled_and_capabilities(client, db_session):
    session_cookie = await _register_and_login(
        client,
        db_session,
        email=f"module-user-{uuid4().hex[:8]}@example.com",
        display_name=f"Module User {uuid4().hex[:6]}",
    )

    response = await client.get("/api/modules", cookies={"bominal_session": session_cookie})
    assert response.status_code == 200

    modules = response.json()["modules"]
    assert isinstance(modules, list)
    assert len(modules) >= 3

    by_slug = {module["slug"]: module for module in modules}
    assert "train" in by_slug
    assert "restaurant" in by_slug
    assert "calendar" in by_slug

    for module in modules:
        assert "enabled" in module
        assert isinstance(module["enabled"], bool)
        assert "capabilities" in module
        assert isinstance(module["capabilities"], list)

    train = by_slug["train"]
    assert train["coming_soon"] is False
    assert train["enabled"] is True
    assert train["capabilities"]


@pytest.mark.asyncio
async def test_modules_omits_wallet_capability_when_payment_disabled(client, db_session, monkeypatch):
    session_cookie = await _register_and_login(
        client,
        db_session,
        email=f"module-payment-off-{uuid4().hex[:8]}@example.com",
        display_name=f"Module Payment Off {uuid4().hex[:6]}",
    )

    monkeypatch.setattr("app.http.routes.modules.settings.payment_enabled", False)

    response = await client.get("/api/modules", cookies={"bominal_session": session_cookie})
    assert response.status_code == 200

    modules = response.json()["modules"]
    by_slug = {module["slug"]: module for module in modules}
    train = by_slug["train"]
    assert "wallet.payment_card" not in train["capabilities"]


@pytest.mark.asyncio
async def test_modules_includes_wallet_capability_when_payment_enabled(client, db_session, monkeypatch):
    session_cookie = await _register_and_login(
        client,
        db_session,
        email=f"module-payment-on-{uuid4().hex[:8]}@example.com",
        display_name=f"Module Payment On {uuid4().hex[:6]}",
    )

    monkeypatch.setattr("app.http.routes.modules.settings.payment_enabled", True)

    response = await client.get("/api/modules", cookies={"bominal_session": session_cookie})
    assert response.status_code == 200

    modules = response.json()["modules"]
    by_slug = {module["slug"]: module for module in modules}
    train = by_slug["train"]
    assert "wallet.payment_card" in train["capabilities"]


@pytest.mark.asyncio
async def test_restaurant_capabilities_are_safe_subset(client, db_session):
    session_cookie = await _register_and_login(
        client,
        db_session,
        email=f"restaurant-module-{uuid4().hex[:8]}@example.com",
        display_name=f"Restaurant Module {uuid4().hex[:6]}",
    )

    response = await client.get("/api/modules", cookies={"bominal_session": session_cookie})
    assert response.status_code == 200
    modules = response.json()["modules"]
    by_slug = {module["slug"]: module for module in modules}

    restaurant = by_slug["restaurant"]
    assert restaurant["coming_soon"] is True
    assert restaurant["enabled"] is False

    capabilities = set(restaurant["capabilities"])
    assert capabilities.issubset(set(RESTAURANT_CAPABILITIES_EXPOSED))
    assert capabilities.isdisjoint(set(RESTAURANT_CAPABILITIES_COMING_SOON))
