from datetime import datetime, timedelta, timezone

import fakeredis.aioredis
import pytest
from sqlalchemy import select

from app.db.models import Secret, Session, Task, User
from app.services.wallet import LEGACY_PAYMENT_CVV_REDIS_KEY_PREFIX, PAYMENT_CVV_REDIS_KEY_PREFIX


@pytest.mark.asyncio
async def test_register_login_me_logout_flow(client):
    register_payload = {
        "email": "user@example.com",
        "password": "SuperSecret123",
        "display_name": "Bloom User",
    }
    register_res = await client.post("/api/auth/register", json=register_payload)
    assert register_res.status_code == 201
    assert register_res.json()["user"]["email"] == "user@example.com"

    login_res = await client.post(
        "/api/auth/login",
        json={"email": "user@example.com", "password": "SuperSecret123", "remember_me": True},
    )
    assert login_res.status_code == 200
    set_cookie = login_res.headers.get("set-cookie", "")
    assert "Max-Age=7776000" in set_cookie

    session_cookie = login_res.cookies.get("bominal_session")
    assert session_cookie

    me_res = await client.get("/api/auth/me", cookies={"bominal_session": session_cookie})
    assert me_res.status_code == 200
    me_json = me_res.json()
    assert me_json["user"]["email"] == "user@example.com"
    assert me_json["user"]["role"] == "user"

    logout_res = await client.post("/api/auth/logout", cookies={"bominal_session": session_cookie})
    assert logout_res.status_code == 200

    me_after_logout = await client.get("/api/auth/me", cookies={"bominal_session": session_cookie})
    assert me_after_logout.status_code == 401


@pytest.mark.asyncio
async def test_login_returns_generic_error_for_unknown_email(client):
    login_res = await client.post(
        "/api/auth/login",
        json={"email": "notfound@example.com", "password": "WrongPass123", "remember_me": False},
    )

    assert login_res.status_code == 401
    assert login_res.json()["detail"] == "Invalid email or password"


@pytest.mark.asyncio
async def test_register_requires_display_name(client):
    missing_name = await client.post(
        "/api/auth/register",
        json={"email": "missing-name@example.com", "password": "SuperSecret123"},
    )
    assert missing_name.status_code == 422

    blank_name = await client.post(
        "/api/auth/register",
        json={"email": "blank-name@example.com", "password": "SuperSecret123", "display_name": "   "},
    )
    assert blank_name.status_code == 422


@pytest.mark.asyncio
async def test_register_rejects_duplicate_email_and_display_name(client):
    first = await client.post(
        "/api/auth/register",
        json={"email": "duplicate@example.com", "password": "SuperSecret123", "display_name": "Duplicate Name"},
    )
    assert first.status_code == 201

    duplicate_email = await client.post(
        "/api/auth/register",
        json={"email": "DUPLICATE@example.com", "password": "SuperSecret123", "display_name": "Another Name"},
    )
    assert duplicate_email.status_code == 400
    assert duplicate_email.json()["detail"] == "Email already registered"

    duplicate_display_name = await client.post(
        "/api/auth/register",
        json={"email": "another@example.com", "password": "SuperSecret123", "display_name": "duplicate name"},
    )
    assert duplicate_display_name.status_code == 400
    assert duplicate_display_name.json()["detail"] == "Display name already registered"


@pytest.mark.asyncio
async def test_account_update_requires_current_password_for_changes(client):
    await client.post(
        "/api/auth/register",
        json={"email": "account-user@example.com", "password": "SuperSecret123", "display_name": "User"},
    )
    login_res = await client.post(
        "/api/auth/login",
        json={"email": "account-user@example.com", "password": "SuperSecret123", "remember_me": False},
    )
    session_cookie = login_res.cookies.get("bominal_session")
    assert session_cookie

    update_res = await client.patch(
        "/api/auth/account",
        cookies={"bominal_session": session_cookie},
        json={"display_name": "Updated User"},
    )
    assert update_res.status_code == 401
    assert "Current password is required" in update_res.json()["detail"]


@pytest.mark.asyncio
async def test_account_update_updates_optional_fields_and_password(client):
    await client.post(
        "/api/auth/register",
        json={"email": "account-update@example.com", "password": "SuperSecret123", "display_name": "Old Name"},
    )
    login_res = await client.post(
        "/api/auth/login",
        json={"email": "account-update@example.com", "password": "SuperSecret123", "remember_me": False},
    )
    session_cookie = login_res.cookies.get("bominal_session")
    assert session_cookie

    update_res = await client.patch(
        "/api/auth/account",
        cookies={"bominal_session": session_cookie},
        json={
            "email": "account-update-new@example.com",
            "display_name": "New Name",
            "phone_number": "010-1234-5678",
            "billing_address_line1": "123 Blossom St",
            "billing_address_line2": "Apt 402",
            "billing_city": "Seoul",
            "billing_state_province": "Seoul",
            "billing_country": "KR",
            "billing_postal_code": "04524",
            "birthday": "1990-01-01",
            "new_password": "EvenMoreSecret123",
            "current_password": "SuperSecret123",
        },
    )
    assert update_res.status_code == 200
    updated_user = update_res.json()["user"]
    assert updated_user["email"] == "account-update-new@example.com"
    assert updated_user["display_name"] == "New Name"
    assert updated_user["phone_number"] == "010-1234-5678"
    assert updated_user["billing_address_line1"] == "123 Blossom St"
    assert updated_user["billing_address_line2"] == "Apt 402"
    assert updated_user["billing_city"] == "Seoul"
    assert updated_user["billing_state_province"] == "Seoul"
    assert updated_user["billing_country"] == "KR"
    assert updated_user["billing_postal_code"] == "04524"
    assert updated_user["birthday"] == "1990-01-01"

    old_password_login = await client.post(
        "/api/auth/login",
        json={"email": "account-update-new@example.com", "password": "SuperSecret123", "remember_me": False},
    )
    assert old_password_login.status_code == 401

    new_password_login = await client.post(
        "/api/auth/login",
        json={"email": "account-update-new@example.com", "password": "EvenMoreSecret123", "remember_me": False},
    )
    assert new_password_login.status_code == 200


@pytest.mark.asyncio
async def test_account_update_rejects_duplicate_email_and_display_name(client):
    await client.post(
        "/api/auth/register",
        json={"email": "primary@example.com", "password": "SuperSecret123", "display_name": "Primary User"},
    )
    await client.post(
        "/api/auth/register",
        json={"email": "secondary@example.com", "password": "SuperSecret123", "display_name": "Secondary User"},
    )

    login_res = await client.post(
        "/api/auth/login",
        json={"email": "secondary@example.com", "password": "SuperSecret123", "remember_me": False},
    )
    session_cookie = login_res.cookies.get("bominal_session")
    assert session_cookie

    duplicate_email = await client.patch(
        "/api/auth/account",
        cookies={"bominal_session": session_cookie},
        json={"email": "PRIMARY@example.com", "current_password": "SuperSecret123"},
    )
    assert duplicate_email.status_code == 400
    assert duplicate_email.json()["detail"] == "Email already registered"

    duplicate_display_name = await client.patch(
        "/api/auth/account",
        cookies={"bominal_session": session_cookie},
        json={"display_name": "primary user", "current_password": "SuperSecret123"},
    )
    assert duplicate_display_name.status_code == 400
    assert duplicate_display_name.json()["detail"] == "Display name already registered"


@pytest.mark.asyncio
async def test_delete_account_blocks_when_outstanding_worker_instances_exist(client, db_session):
    email = "account-delete-blocked@example.com"
    await client.post(
        "/api/auth/register",
        json={"email": email, "password": "SuperSecret123", "display_name": "Delete Blocked"},
    )
    login_res = await client.post(
        "/api/auth/login",
        json={"email": email, "password": "SuperSecret123", "remember_me": False},
    )
    session_cookie = login_res.cookies.get("bominal_session")
    assert session_cookie

    user = (await db_session.execute(select(User).where(User.email == email))).scalar_one()
    db_session.add(
        Task(
            user_id=user.id,
            module="train",
            state="RUNNING",
            deadline_at=datetime.now(timezone.utc) + timedelta(hours=1),
            spec_json={"dep": "수서", "arr": "마산"},
            idempotency_key="active-delete-block",
        )
    )
    await db_session.commit()

    delete_res = await client.delete("/api/auth/account", cookies={"bominal_session": session_cookie})
    assert delete_res.status_code == 409
    assert "outstanding worker instances" in delete_res.json()["detail"].lower()

    user_after = (await db_session.execute(select(User).where(User.id == user.id))).scalar_one_or_none()
    assert user_after is not None
    assert user_after.email == email


@pytest.mark.asyncio
async def test_delete_account_scrubs_user_and_marks_tasks_for_removal(client, db_session, monkeypatch):
    fake_redis = fakeredis.aioredis.FakeRedis()
    monkeypatch.setattr("app.services.wallet.Redis.from_url", lambda *_args, **_kwargs: fake_redis)

    email = "account-delete-success@example.com"
    password = "SuperSecret123"
    await client.post(
        "/api/auth/register",
        json={"email": email, "password": password, "display_name": "Delete Success"},
    )
    login_res = await client.post(
        "/api/auth/login",
        json={"email": email, "password": password, "remember_me": False},
    )
    session_cookie = login_res.cookies.get("bominal_session")
    assert session_cookie

    user = (await db_session.execute(select(User).where(User.email == email))).scalar_one()
    now = datetime.now(timezone.utc)
    db_session.add(
        Task(
            user_id=user.id,
            module="train",
            state="COMPLETED",
            deadline_at=now + timedelta(hours=2),
            spec_json={"dep": "수서", "arr": "부산"},
            idempotency_key="completed-delete-success",
            completed_at=now - timedelta(hours=1),
        )
    )
    db_session.add(
        Secret(
            user_id=user.id,
            kind="payment_card",
            ciphertext="ciphertext",
            nonce="nonce",
            wrapped_dek="wrapped",
            dek_nonce="dek-nonce",
            aad="aad",
            kek_version=1,
        )
    )
    await db_session.commit()
    await fake_redis.set(f"{PAYMENT_CVV_REDIS_KEY_PREFIX}:{user.id}", "encrypted-cvv")
    await fake_redis.set(f"{LEGACY_PAYMENT_CVV_REDIS_KEY_PREFIX}:{user.id}", "legacy-cvv")

    delete_res = await client.delete("/api/auth/account", cookies={"bominal_session": session_cookie})
    assert delete_res.status_code == 200
    assert delete_res.json()["message"] == "Account deleted"
    assert "bominal_session=" in (delete_res.headers.get("set-cookie") or "")

    me_res = await client.get("/api/auth/me", cookies={"bominal_session": session_cookie})
    assert me_res.status_code == 401

    deleted_user = (await db_session.execute(select(User).where(User.id == user.id))).scalar_one()
    assert deleted_user.email.startswith(f"deleted-{user.id}")
    assert deleted_user.email.endswith("@deleted.bominal.local")
    assert deleted_user.display_name is None
    assert deleted_user.phone_number is None
    assert deleted_user.billing_address_line1 is None
    assert deleted_user.billing_address_line2 is None
    assert deleted_user.billing_city is None
    assert deleted_user.billing_state_province is None
    assert deleted_user.billing_country is None
    assert deleted_user.billing_postal_code is None
    assert deleted_user.birthday is None
    assert deleted_user.email_verified_at is None

    sessions = (await db_session.execute(select(Session).where(Session.user_id == user.id))).scalars().all()
    assert sessions == []

    secrets = (await db_session.execute(select(Secret).where(Secret.user_id == user.id))).scalars().all()
    assert secrets == []
    assert await fake_redis.get(f"{PAYMENT_CVV_REDIS_KEY_PREFIX}:{user.id}") is None
    assert await fake_redis.get(f"{LEGACY_PAYMENT_CVV_REDIS_KEY_PREFIX}:{user.id}") is None

    tasks = (await db_session.execute(select(Task).where(Task.user_id == user.id))).scalars().all()
    assert len(tasks) == 1
    assert tasks[0].hidden_at is not None
    removal_safe = tasks[0].spec_json.get("account_removal_safe")
    assert isinstance(removal_safe, dict)
    assert removal_safe.get("reason") == "account_deleted"
    marked_for_removal_at = datetime.fromisoformat(str(removal_safe["marked_for_removal_at"]))
    remove_after_at = datetime.fromisoformat(str(removal_safe["remove_after_at"]))
    assert remove_after_at > marked_for_removal_at
    assert (remove_after_at - marked_for_removal_at) >= timedelta(days=365)
