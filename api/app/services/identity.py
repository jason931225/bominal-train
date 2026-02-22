from __future__ import annotations

from typing import Any

from sqlalchemy import select
from sqlalchemy.ext.asyncio import AsyncSession
from sqlalchemy.orm import joinedload

from app.core.security import hash_password, new_session_token
from app.db.models import Role, User


def _fallback_email_for_sub(supabase_sub: str) -> str:
    return f"supabase-{supabase_sub}@auth.bominal.local"


def _display_name_from_claims(claims: dict[str, Any], email: str) -> str:
    user_meta = claims.get("user_metadata") if isinstance(claims.get("user_metadata"), dict) else {}
    candidates = [
        user_meta.get("display_name"),
        user_meta.get("full_name"),
        claims.get("name"),
        email.split("@", 1)[0],
    ]
    for candidate in candidates:
        text = str(candidate or "").strip()
        if text:
            return text[:255]
    return "user"


async def get_or_create_local_user_from_supabase_claims(
    db: AsyncSession,
    *,
    claims: dict[str, Any],
) -> User:
    supabase_sub = str(claims.get("sub") or "").strip()
    if not supabase_sub:
        raise ValueError("supabase claims missing sub")

    email = str(claims.get("email") or "").strip().lower()
    if not email:
        email = _fallback_email_for_sub(supabase_sub)

    stmt = (
        select(User)
        .options(joinedload(User.role))
        .where(User.supabase_user_id == supabase_sub)
        .limit(1)
    )
    existing = (await db.execute(stmt)).scalar_one_or_none()
    if existing is not None:
        # Keep profile fields in sync opportunistically for non-sensitive attributes.
        if existing.email != email:
            existing.email = email
            await db.commit()
            await db.refresh(existing)
        return existing

    by_email_stmt = (
        select(User)
        .options(joinedload(User.role))
        .where(User.email == email)
        .limit(1)
    )
    by_email = (await db.execute(by_email_stmt)).scalar_one_or_none()
    if by_email is not None:
        by_email.supabase_user_id = supabase_sub
        await db.commit()
        await db.refresh(by_email)
        return by_email

    role = (await db.execute(select(Role).where(Role.name == "user").limit(1))).scalar_one_or_none()
    if role is None:
        raise RuntimeError("Role seed missing")

    user = User(
        email=email,
        password_hash=hash_password(new_session_token()),
        display_name=_display_name_from_claims(claims, email),
        ui_locale="en",
        role_id=role.id,
        supabase_user_id=supabase_sub,
    )
    db.add(user)
    await db.commit()

    user_with_role = (
        await db.execute(select(User).options(joinedload(User.role)).where(User.id == user.id).limit(1))
    ).scalar_one()
    return user_with_role
