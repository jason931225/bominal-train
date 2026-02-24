from __future__ import annotations

import pytest
from httpx import ASGITransport, AsyncClient
from sqlalchemy import select

from app.db.models import User
from app.db.session import get_db
from app.http.deps import ACCESS_REVIEW_PENDING_DETAIL
from app.main import app as default_app
from app.main_gateway import app as gateway_app
from app.main_restaurant import app as restaurant_app
from app.main_train import app as train_app
from tests.conftest import CompatAsyncClient


def _paths(app) -> set[str]:
    return {route.path for route in app.routes}


def test_default_app_exposes_compatibility_route_surface():
    default_paths = _paths(default_app)
    assert "/api/auth/login" in default_paths
    assert "/api/train/stations" in default_paths


def test_gateway_routes_exclude_train_domain_routes():
    gateway_paths = _paths(gateway_app)
    assert "/api/auth/login" in gateway_paths
    assert "/api/train/stations" not in gateway_paths


def test_train_app_routes_include_train_domain_only():
    train_paths = _paths(train_app)
    assert "/api/train/stations" in train_paths
    assert "/api/auth/login" not in train_paths


def test_restaurant_app_exposes_restaurant_health_route():
    restaurant_paths = _paths(restaurant_app)
    assert "/api/restaurant/health" in restaurant_paths
    assert "/api/train/stations" not in restaurant_paths


@pytest.mark.asyncio
async def test_train_app_blocks_pending_user_and_allows_approved_user(db_session_factory):
    async def override_get_db():
        async with db_session_factory() as session:
            yield session

    default_app.dependency_overrides[get_db] = override_get_db
    train_app.dependency_overrides[get_db] = override_get_db
    try:
        async with CompatAsyncClient(transport=ASGITransport(app=default_app), base_url="http://testserver") as auth_client:
            register_response = await auth_client.post(
                "/api/auth/register",
                json={
                    "email": "split-train-review@example.com",
                    "password": "SuperSecret123",
                    "display_name": "Split Train Review User",
                },
            )
            assert register_response.status_code == 201

            login_response = await auth_client.post(
                "/api/auth/login",
                json={
                    "email": "split-train-review@example.com",
                    "password": "SuperSecret123",
                    "remember_me": False,
                },
            )
            assert login_response.status_code == 200
            session_cookie = login_response.cookies.get("bominal_session")
            assert session_cookie

        async with AsyncClient(transport=ASGITransport(app=train_app), base_url="http://testserver") as train_client:
            pending_response = await train_client.get(
                "/api/train/stations",
                headers={"cookie": f"bominal_session={session_cookie}"},
            )
            assert pending_response.status_code == 403
            assert pending_response.json()["detail"] == ACCESS_REVIEW_PENDING_DETAIL

        async with db_session_factory() as session:
            user = (
                await session.execute(
                    select(User).where(User.email == "split-train-review@example.com")
                )
            ).scalar_one()
            user.access_status = "approved"
            await session.commit()

        async with AsyncClient(transport=ASGITransport(app=train_app), base_url="http://testserver") as train_client:
            approved_response = await train_client.get(
                "/api/train/stations",
                headers={"cookie": f"bominal_session={session_cookie}"},
            )
            assert approved_response.status_code == 200
            payload = approved_response.json()
            assert "stations" in payload
            assert isinstance(payload["stations"], list)
    finally:
        default_app.dependency_overrides.clear()
        train_app.dependency_overrides.clear()
