import json

import fakeredis.aioredis
import pytest
from sqlalchemy import select

from app.core.config import get_settings
from app.db.models import Secret, User
from app.services.wallet import (
    LEGACY_PAYMENT_CVV_REDIS_KEY_PREFIX,
    PAYMENT_CVV_REDIS_KEY_PREFIX,
    SECRET_KIND_PAYMENT_CARD,
    purge_all_saved_payment_data,
)
from tests.conftest import MockRedisContextManager


@pytest.fixture(autouse=True)
def _enable_payment_routes_for_wallet_tests(monkeypatch):
    monkeypatch.setattr("app.http.deps.settings.payment_enabled", True)
    monkeypatch.setattr("app.services.wallet.settings.payment_enabled", True)
    monkeypatch.setattr("app.modules.train.service.settings.payment_enabled", True)


async def _register_and_login(client, db_session, *, email: str) -> str:
    register_res = await client.post(
        "/api/auth/register",
        json={
            "email": email,
            "password": "SuperSecret123",
            "display_name": f"Wallet User {email.split('@', 1)[0]}",
        },
    )
    assert register_res.status_code == 201

    user = (await db_session.execute(select(User).where(User.email == email))).scalar_one()
    user.access_status = "approved"
    await db_session.commit()

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

    monkeypatch.setattr("app.services.wallet.get_cde_redis_pool", lambda: MockRedisContextManager(fake_redis))

    email = "wallet-remove@example.com"
    cookie = await _register_and_login(client, db_session, email=email)

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
async def test_remove_payment_settings_is_idempotent(client, db_session, monkeypatch):
    fake_redis = fakeredis.aioredis.FakeRedis()
    monkeypatch.setattr("app.services.wallet.get_cde_redis_pool", lambda: MockRedisContextManager(fake_redis))

    cookie = await _register_and_login(client, db_session, email="wallet-remove-idempotent@example.com")

    remove_res = await client.delete("/api/wallet/payment-card", cookies={"bominal_session": cookie})
    assert remove_res.status_code == 200
    assert remove_res.json()["configured"] is False


@pytest.mark.asyncio
async def test_payment_card_configured_endpoint_returns_minimal_boolean(client, db_session, monkeypatch):
    fake_redis = fakeredis.aioredis.FakeRedis()
    monkeypatch.setattr("app.services.wallet.get_cde_redis_pool", lambda: MockRedisContextManager(fake_redis))

    cookie = await _register_and_login(client, db_session, email="wallet-configured-only@example.com")

    before_res = await client.get("/api/wallet/payment-card/configured", cookies={"bominal_session": cookie})
    assert before_res.status_code == 200
    assert before_res.json() == {"configured": False}

    save_res = await client.post(
        "/api/wallet/payment-card",
        cookies={"bominal_session": cookie},
        json={
            "card_number": "4111 1111 1111 1111",
            "expiry_month": 12,
            "expiry_year": 2099,
            "cvv": "123",
            "birth_date": "1990-01-01",
            "pin2": "12",
        },
    )
    assert save_res.status_code == 200

    after_res = await client.get("/api/wallet/payment-card/configured", cookies={"bominal_session": cookie})
    assert after_res.status_code == 200
    assert after_res.json() == {"configured": True}


@pytest.mark.asyncio
async def test_payment_card_cache_blob_has_kek_version_and_bounded_ttl(client, db_session, monkeypatch):
    fake_redis = fakeredis.aioredis.FakeRedis()
    monkeypatch.setattr("app.services.wallet.get_cde_redis_pool", lambda: MockRedisContextManager(fake_redis))

    cookie = await _register_and_login(client, db_session, email="wallet-cache-kek@example.com")

    res = await client.post(
        "/api/wallet/payment-card",
        cookies={"bominal_session": cookie},
        json={
            "card_number": "4111 1111 1111 1111",
            "expiry_month": 12,
            "expiry_year": 2099,
            "cvv": "123",
            "birth_date": "1990-01-01",
            "pin2": "12",
        },
    )
    assert res.status_code == 200

    user = (await db_session.execute(select(User).where(User.email == "wallet-cache-kek@example.com"))).scalar_one()
    redis_key = f"{PAYMENT_CVV_REDIS_KEY_PREFIX}:{user.id}"
    raw_blob = await fake_redis.get(redis_key)
    assert raw_blob is not None

    if isinstance(raw_blob, bytes):
        raw_blob = raw_blob.decode("utf-8")
    parsed = json.loads(raw_blob)
    assert "kek_version" in parsed
    assert isinstance(parsed["kek_version"], int)

    ttl = await fake_redis.ttl(redis_key)
    settings = get_settings()
    assert settings.payment_cvv_ttl_min_seconds <= ttl <= settings.payment_cvv_ttl_max_seconds


@pytest.mark.asyncio
async def test_payment_card_status_returns_not_configured_on_kek_version_mismatch(client, db_session, monkeypatch):
    fake_redis = fakeredis.aioredis.FakeRedis()
    monkeypatch.setattr("app.services.wallet.get_cde_redis_pool", lambda: MockRedisContextManager(fake_redis))

    email = "wallet-kek-mismatch@example.com"
    cookie = await _register_and_login(client, db_session, email=email)

    save_res = await client.post(
        "/api/wallet/payment-card",
        cookies={"bominal_session": cookie},
        json={
            "card_number": "4111 1111 1111 1111",
            "expiry_month": 12,
            "expiry_year": 2099,
            "cvv": "123",
            "birth_date": "1990-01-01",
            "pin2": "12",
        },
    )
    assert save_res.status_code == 200

    user = (await db_session.execute(select(User).where(User.email == email))).scalar_one()
    secret = (
        await db_session.execute(
            select(Secret)
            .where(Secret.user_id == user.id)
            .where(Secret.kind == SECRET_KIND_PAYMENT_CARD)
            .limit(1)
        )
    ).scalar_one()
    secret.kek_version = secret.kek_version + 1
    await db_session.commit()

    status_res = await client.get("/api/wallet/payment-card", cookies={"bominal_session": cookie})
    assert status_res.status_code == 200
    payload = status_res.json()
    assert payload["configured"] is False
    assert payload["detail"] == "Payment card could not be loaded"


@pytest.mark.asyncio
async def test_purge_all_saved_payment_data_wipes_wallet_secrets_and_cvv_cache(client, db_session, monkeypatch):
    fake_redis = fakeredis.aioredis.FakeRedis()
    monkeypatch.setattr("app.services.wallet.get_redis_pool", lambda: MockRedisContextManager(fake_redis))

    cookie_a = await _register_and_login(client, db_session, email="wallet-purge-a@example.com")
    cookie_b = await _register_and_login(client, db_session, email="wallet-purge-b@example.com")

    payload = {
        "card_number": "4111 1111 1111 1111",
        "expiry_month": 12,
        "expiry_year": 2099,
        "cvv": "123",
        "birth_date": "1990-01-01",
        "pin2": "12",
    }
    res_a = await client.post("/api/wallet/payment-card", cookies={"bominal_session": cookie_a}, json=payload)
    res_b = await client.post("/api/wallet/payment-card", cookies={"bominal_session": cookie_b}, json=payload)
    assert res_a.status_code == 200
    assert res_b.status_code == 200

    user_a = (await db_session.execute(select(User).where(User.email == "wallet-purge-a@example.com"))).scalar_one()
    user_b = (await db_session.execute(select(User).where(User.email == "wallet-purge-b@example.com"))).scalar_one()

    # Legacy compatibility key should also be removed by purge.
    await fake_redis.set(f"{LEGACY_PAYMENT_CVV_REDIS_KEY_PREFIX}:{user_a.id}", "legacy-cvv")

    summary = await purge_all_saved_payment_data(db_session)

    remaining_payment_secrets = (
        await db_session.execute(select(Secret).where(Secret.kind == SECRET_KIND_PAYMENT_CARD))
    ).scalars().all()
    assert remaining_payment_secrets == []

    assert await fake_redis.get(f"{PAYMENT_CVV_REDIS_KEY_PREFIX}:{user_a.id}") is None
    assert await fake_redis.get(f"{PAYMENT_CVV_REDIS_KEY_PREFIX}:{user_b.id}") is None
    assert await fake_redis.get(f"{LEGACY_PAYMENT_CVV_REDIS_KEY_PREFIX}:{user_a.id}") is None

    assert summary["db_payment_card_secrets_deleted"] == 2
    assert summary["redis_cvv_keys_deleted_total"] >= 3
