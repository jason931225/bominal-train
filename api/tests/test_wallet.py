import fakeredis.aioredis
import pytest
from sqlalchemy import select

from app.db.models import Secret, User
from app.services.wallet import (
    LEGACY_PAYMENT_CVV_REDIS_KEY_PREFIX,
    PAYMENT_CVV_REDIS_KEY_PREFIX,
    SECRET_KIND_PAYMENT_CARD,
)
from tests.conftest import MockRedisContextManager


async def _register_and_login(client, *, email: str) -> str:
    await client.post(
        "/api/auth/register",
        json={"email": email, "password": "SuperSecret123", "display_name": "Wallet User"},
    )
    login_res = await client.post(
        "/api/auth/login",
        json={"email": email, "password": "SuperSecret123", "remember_me": True},
    )
    assert login_res.status_code == 200
    cookie = login_res.cookies.get("bominal_session")
    assert cookie
    return cookie


@pytest.mark.asyncio
async def test_remove_payment_settings_wipes_saved_data(client, db_session, monkeypatch):
    fake_redis = fakeredis.aioredis.FakeRedis()

    async def _get_fake_redis():
        return fake_redis

    monkeypatch.setattr("app.services.wallet.get_redis_pool", lambda: MockRedisContextManager(fake_redis))

    email = "wallet-remove@example.com"
    cookie = await _register_and_login(client, email=email)

    save_res = await client.post(
        "/api/wallet/payment-card",
        cookies={"bominal_session": cookie},
        json={
            "card_number": "1234 5678 9012 3456",
            "expiry_month": 12,
            "expiry_year": 2099,
            "cvv": "123",
            "birth_date": "1990-01-01",
            "pin2": "12",
        },
    )
    assert save_res.status_code == 200
    assert save_res.json()["configured"] is True

    user = (await db_session.execute(select(User).where(User.email == email))).scalar_one()
    secret = (
        await db_session.execute(
            select(Secret)
            .where(Secret.user_id == user.id)
            .where(Secret.kind == SECRET_KIND_PAYMENT_CARD)
            .limit(1)
        )
    ).scalar_one_or_none()
    assert secret is not None

    redis_key = f"{PAYMENT_CVV_REDIS_KEY_PREFIX}:{user.id}"
    legacy_key = f"{LEGACY_PAYMENT_CVV_REDIS_KEY_PREFIX}:{user.id}"
    assert await fake_redis.get(redis_key) is not None

    remove_res = await client.delete("/api/wallet/payment-card", cookies={"bominal_session": cookie})
    assert remove_res.status_code == 200
    assert remove_res.json()["configured"] is False
    assert "removed" in str(remove_res.json()["detail"]).lower()

    secret_after = (
        await db_session.execute(
            select(Secret)
            .where(Secret.user_id == user.id)
            .where(Secret.kind == SECRET_KIND_PAYMENT_CARD)
            .limit(1)
        )
    ).scalar_one_or_none()
    assert secret_after is None
    assert await fake_redis.get(redis_key) is None
    assert await fake_redis.get(legacy_key) is None

    status_res = await client.get("/api/wallet/payment-card", cookies={"bominal_session": cookie})
    assert status_res.status_code == 200
    assert status_res.json()["configured"] is False


@pytest.mark.asyncio
async def test_remove_payment_settings_is_idempotent(client, monkeypatch):
    fake_redis = fakeredis.aioredis.FakeRedis()
    monkeypatch.setattr("app.services.wallet.get_redis_pool", lambda: MockRedisContextManager(fake_redis))

    cookie = await _register_and_login(client, email="wallet-remove-idempotent@example.com")

    remove_res = await client.delete("/api/wallet/payment-card", cookies={"bominal_session": cookie})
    assert remove_res.status_code == 200
    assert remove_res.json()["configured"] is False

