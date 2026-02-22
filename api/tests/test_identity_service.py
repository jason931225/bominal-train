from __future__ import annotations

from datetime import datetime, timezone

import pytest
from sqlalchemy import delete, select

from app.core.security import hash_password
from app.db.models import Role, User
from app.services import identity


def _new_user(*, email: str, role_id: int, supabase_user_id: str | None = None, display_name: str | None = None) -> User:
    now = datetime.now(timezone.utc)
    return User(
        email=email,
        password_hash=hash_password("StrongPass123!"),
        display_name=display_name,
        role_id=role_id,
        ui_locale="en",
        supabase_user_id=supabase_user_id,
        created_at=now,
        updated_at=now,
    )


def test_display_name_and_fallback_helpers() -> None:
    assert identity._fallback_email_for_sub("abc") == "supabase-abc@auth.bominal.local"
    assert (
        identity._display_name_from_claims({"user_metadata": {"display_name": "  Alice  "}}, "a@example.com")
        == "Alice"
    )
    assert (
        identity._display_name_from_claims({"user_metadata": {"full_name": "  Alice Kim  "}}, "a@example.com")
        == "Alice Kim"
    )
    assert identity._display_name_from_claims({"name": "FromClaim"}, "a@example.com") == "FromClaim"
    assert identity._display_name_from_claims({}, "fallback@example.com") == "fallback"


@pytest.mark.asyncio
async def test_get_or_create_requires_sub(db_session) -> None:
    with pytest.raises(ValueError, match="missing sub"):
        await identity.get_or_create_local_user_from_supabase_claims(db_session, claims={"email": "a@example.com"})


@pytest.mark.asyncio
async def test_get_or_create_returns_existing_by_supabase_sub(db_session) -> None:
    user = _new_user(email="existing@example.com", role_id=2, supabase_user_id="sub-001", display_name="Existing")
    db_session.add(user)
    await db_session.commit()

    resolved = await identity.get_or_create_local_user_from_supabase_claims(
        db_session,
        claims={"sub": "sub-001", "email": "changed@example.com"},
    )

    assert resolved.id == user.id
    assert resolved.email == "existing@example.com"


@pytest.mark.asyncio
async def test_get_or_create_links_existing_email(db_session) -> None:
    user = _new_user(email="link-me@example.com", role_id=2, display_name="LinkMe")
    db_session.add(user)
    await db_session.commit()

    resolved = await identity.get_or_create_local_user_from_supabase_claims(
        db_session,
        claims={"sub": "supa-002", "email": "link-me@example.com"},
    )

    assert resolved.id == user.id
    assert resolved.supabase_user_id == "supa-002"


@pytest.mark.asyncio
async def test_get_or_create_creates_new_user_and_handles_display_name_collision(db_session) -> None:
    db_session.add(_new_user(email="taken@example.com", role_id=2, display_name="TakenDisplay"))
    await db_session.commit()

    created = await identity.get_or_create_local_user_from_supabase_claims(
        db_session,
        claims={
            "sub": "supa-003",
            "email": "new-user@example.com",
            "user_metadata": {"display_name": "TakenDisplay"},
        },
    )

    assert created.email == "new-user@example.com"
    assert created.supabase_user_id == "supa-003"
    # Collision is dropped to avoid unique-constraint violations on auto-provision.
    assert created.display_name is None

    fallback_created = await identity.get_or_create_local_user_from_supabase_claims(
        db_session,
        claims={"sub": "supa-no-email"},
    )
    assert fallback_created.email == "supabase-supa-no-email@auth.bominal.local"


@pytest.mark.asyncio
async def test_get_or_create_raises_when_user_role_seed_missing(db_session) -> None:
    await db_session.execute(delete(Role).where(Role.name == "user"))
    await db_session.commit()

    with pytest.raises(RuntimeError, match="Role seed missing"):
        await identity.get_or_create_local_user_from_supabase_claims(
            db_session,
            claims={"sub": "supa-004", "email": "role-missing@example.com"},
        )

    users = (await db_session.execute(select(User))).scalars().all()
    assert users == []
