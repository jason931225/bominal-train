from __future__ import annotations

from datetime import datetime, timedelta, timezone
from types import SimpleNamespace
from uuid import uuid4

import pytest
from fastapi import HTTPException, Response
from sqlalchemy import select

from app.core.security import hash_password
from app.db.models import PasswordResetToken, Secret, Session, Task, User, VerificationToken
from app.modules.train.constants import TASK_MODULE
from app.services import auth as auth_service


def _make_user(*, role_id: int = 2, email: str = "user@example.com") -> User:
    now = datetime.now(timezone.utc)
    return User(
        email=email,
        password_hash=hash_password("StrongPass123!"),
        display_name="User",
        phone_number="01000000000",
        role_id=role_id,
        ui_locale="en",
        billing_address="line",
        billing_address_line1="line1",
        billing_address_line2="line2",
        billing_city="city",
        billing_state_province="state",
        billing_country="KR",
        billing_postal_code="12345",
        birthday=datetime(2000, 1, 1, tzinfo=timezone.utc).date(),
        access_status="approved",
        access_reviewed_at=now,
        email_verified_at=now,
        created_at=now,
        updated_at=now,
    )


def test_user_to_out_cookie_helpers_and_ip_resolution(monkeypatch) -> None:
    user = SimpleNamespace(
        id=uuid4(),
        supabase_user_id="supa-001",
        email="a@example.com",
        display_name="A",
        phone_number="010",
        ui_locale="en",
        billing_address="addr",
        billing_address_line1="line1",
        billing_address_line2="line2",
        billing_city="city",
        billing_state_province="state",
        billing_country="KR",
        billing_postal_code="12345",
        birthday=None,
        role=SimpleNamespace(name="user"),
        access_status="approved",
        access_reviewed_at=None,
        created_at=datetime.now(timezone.utc),
    )
    out = auth_service.user_to_out(user)
    assert out.role == "user"
    assert out.email == "a@example.com"

    monkeypatch.setattr(auth_service.settings, "session_cookie_name", "bominal_session")
    monkeypatch.setattr(auth_service.settings, "session_days_default", 1)
    monkeypatch.setattr(auth_service.settings, "session_days_remember", 7)
    monkeypatch.setattr(auth_service.settings, "app_env", "development")

    resp_default = Response()
    auth_service.set_session_cookie(resp_default, "tok-default", remember_me=False)
    header_default = resp_default.headers.get("set-cookie", "")
    assert "bominal_session=tok-default" in header_default
    assert "HttpOnly" in header_default
    assert "SameSite=lax" in header_default
    assert "Max-Age=86400" in header_default
    assert "Secure" not in header_default

    monkeypatch.setattr(auth_service.settings, "app_env", "production")
    resp_remember = Response()
    auth_service.set_session_cookie(resp_remember, "tok-remember", remember_me=True)
    header_remember = resp_remember.headers.get("set-cookie", "")
    assert "Max-Age=604800" in header_remember
    assert "Secure" in header_remember

    resp_clear = Response()
    auth_service.clear_session_cookie(resp_clear)
    header_clear = resp_clear.headers.get("set-cookie", "")
    assert "Max-Age=0" in header_clear
    assert "SameSite=lax" in header_clear

    assert auth_service.request_ip("10.0.0.1", None, None) == "10.0.0.1"
    assert auth_service.request_ip("10.0.0.1", "20.0.0.1, 30.0.0.1", None) == "20.0.0.1"
    assert auth_service.request_ip("10.0.0.1", "20.0.0.1", " 40.0.0.1 ") == "40.0.0.1"


def test_session_activity_and_marker_helpers() -> None:
    now = datetime.now(timezone.utc)
    last_seen = now - timedelta(seconds=30)

    assert auth_service.should_update_session_activity(
        last_seen_at=last_seen,
        now=now,
        debounce_seconds=0,
    )
    assert not auth_service.should_update_session_activity(
        last_seen_at=last_seen,
        now=now,
        debounce_seconds=60,
    )
    assert auth_service.should_update_session_activity(
        last_seen_at=last_seen.replace(tzinfo=None),
        now=now.replace(tzinfo=None),
        debounce_seconds=10,
    )

    marker = auth_service._task_removal_marker(now=now)
    assert marker["reason"] == "account_deleted"
    assert auth_service._deleted_email_for_user(uuid4()).endswith("@deleted.bominal.local")


@pytest.mark.asyncio
async def test_delete_account_data_blocks_when_active_tasks_exist(db_session) -> None:
    user = _make_user()
    db_session.add(user)
    await db_session.flush()
    db_session.add(
        Task(
            user_id=user.id,
            module=TASK_MODULE,
            state="RUNNING",
            deadline_at=datetime.now(timezone.utc) + timedelta(hours=1),
            spec_json={"provider": "SRT"},
            idempotency_key="active-task",
        )
    )
    await db_session.commit()

    with pytest.raises(HTTPException) as exc:
        await auth_service.delete_account_data(db_session, user=user)

    assert exc.value.status_code == 409


@pytest.mark.asyncio
async def test_delete_account_data_scrubs_user_and_related_rows(db_session, monkeypatch) -> None:
    user = _make_user(email="wipe-me@example.com")
    db_session.add(user)
    await db_session.flush()

    finished_task = Task(
        user_id=user.id,
        module=TASK_MODULE,
        state="COMPLETED",
        deadline_at=datetime.now(timezone.utc) + timedelta(hours=1),
        spec_json={"provider": "SRT"},
        idempotency_key="finished-task",
    )
    db_session.add(finished_task)
    db_session.add(
        Session(
            user_id=user.id,
            token_hash=hash_password("session"),
            expires_at=datetime.now(timezone.utc) + timedelta(days=1),
            last_seen_at=datetime.now(timezone.utc),
        )
    )
    db_session.add(
        VerificationToken(
            user_id=user.id,
            token_hash=hash_password("verify"),
            expires_at=datetime.now(timezone.utc) + timedelta(minutes=15),
        )
    )
    db_session.add(
        PasswordResetToken(
            user_id=user.id,
            token_hash=hash_password("reset"),
            expires_at=datetime.now(timezone.utc) + timedelta(minutes=15),
        )
    )
    db_session.add(
        Secret(
            user_id=user.id,
            kind="payment.card",
            ciphertext="ciphertext",
            nonce="nonce",
            wrapped_dek="wrapped",
            dek_nonce="dek-nonce",
            aad="aad",
            kek_version=1,
        )
    )
    await db_session.commit()

    cleared = {"user_id": None}

    async def _fake_clear_payment_card_cache(*, user_id):  # noqa: ANN001
        cleared["user_id"] = user_id

    monkeypatch.setattr(auth_service, "clear_payment_card_cache", _fake_clear_payment_card_cache)
    await auth_service.delete_account_data(db_session, user=user)
    await db_session.refresh(user)
    await db_session.refresh(finished_task)

    assert user.email.endswith("@deleted.bominal.local")
    assert user.display_name is None
    assert user.phone_number is None
    assert user.access_status == "rejected"
    assert user.access_reviewed_at is not None
    assert cleared["user_id"] == user.id

    assert finished_task.hidden_at is not None
    marker = finished_task.spec_json.get(auth_service.ACCOUNT_TASK_REMOVAL_MARKER_KEY)
    assert marker["reason"] == "account_deleted"

    assert (await db_session.execute(select(Session).where(Session.user_id == user.id))).scalars().all() == []
    assert (
        await db_session.execute(select(VerificationToken).where(VerificationToken.user_id == user.id))
    ).scalars().all() == []
    assert (
        await db_session.execute(select(PasswordResetToken).where(PasswordResetToken.user_id == user.id))
    ).scalars().all() == []
    assert (await db_session.execute(select(Secret).where(Secret.user_id == user.id))).scalars().all() == []
