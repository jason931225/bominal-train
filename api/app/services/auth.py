from datetime import datetime, timedelta, timezone
from uuid import UUID

from fastapi import HTTPException, Response, status
from sqlalchemy import delete, select
from sqlalchemy.ext.asyncio import AsyncSession

from app.core.config import get_settings
from app.core.security import hash_password, new_session_token
from app.db.models import AuthChallenge, PasswordResetToken, PasskeyCredential, Secret, Session, Task, User, VerificationToken
from app.modules.train.constants import ACTIVE_TASK_STATES
from app.schemas.auth import UserOut
from app.services.wallet import clear_payment_card_cache

settings = get_settings()
ACCOUNT_TASK_REMOVAL_RETENTION_DAYS = 365
ACCOUNT_TASK_REMOVAL_MARKER_KEY = "account_removal_safe"


def user_to_out(user: User) -> UserOut:
    return UserOut(
        id=user.id,
        supabase_user_id=user.supabase_user_id,
        email=user.email,
        display_name=user.display_name,
        phone_number=user.phone_number,
        ui_locale=user.ui_locale,
        billing_address=user.billing_address,
        billing_address_line1=user.billing_address_line1,
        billing_address_line2=user.billing_address_line2,
        billing_city=user.billing_city,
        billing_state_province=user.billing_state_province,
        billing_country=user.billing_country,
        billing_postal_code=user.billing_postal_code,
        birthday=user.birthday,
        role=user.role.name,
        access_status=user.access_status,
        access_reviewed_at=user.access_reviewed_at,
        created_at=user.created_at,
    )


def set_session_cookie(response: Response, token: str, remember_me: bool) -> None:
    days = settings.session_days_remember if remember_me else settings.session_days_default
    max_age = days * 24 * 60 * 60

    response.set_cookie(
        key=settings.session_cookie_name,
        value=token,
        max_age=max_age,
        expires=max_age,
        httponly=True,
        secure=settings.is_production,
        samesite="lax",
        path="/",
    )


def clear_session_cookie(response: Response) -> None:
    response.delete_cookie(
        key=settings.session_cookie_name,
        httponly=True,
        secure=settings.is_production,
        samesite="lax",
        path="/",
    )


def request_ip(remote: str | None, forwarded: str | None, cf_connecting_ip: str | None = None) -> str | None:
    """Extract client IP, preferring Cloudflare header if present."""
    if cf_connecting_ip:
        return cf_connecting_ip.strip()
    if forwarded:
        return forwarded.split(",")[0].strip()
    return remote


def utc_now() -> datetime:
    return datetime.now(timezone.utc)


def _as_utc(value: datetime) -> datetime:
    return value if value.tzinfo is not None else value.replace(tzinfo=timezone.utc)


def should_update_session_activity(*, last_seen_at: datetime, now: datetime, debounce_seconds: int) -> bool:
    if debounce_seconds <= 0:
        return True
    return (_as_utc(now) - _as_utc(last_seen_at)) >= timedelta(seconds=debounce_seconds)


def _deleted_email_for_user(user_id: UUID) -> str:
    return f"deleted-{user_id}@deleted.bominal.local"


def _task_removal_marker(*, now: datetime) -> dict[str, str]:
    return {
        "marked_for_removal_at": now.isoformat(),
        "remove_after_at": (now + timedelta(days=ACCOUNT_TASK_REMOVAL_RETENTION_DAYS)).isoformat(),
        "reason": "account_deleted",
    }


async def delete_account_data(db: AsyncSession, *, user: User) -> None:
    outstanding_task = (
        await db.execute(
            select(Task.id)
            .where(Task.user_id == user.id)
            .where(Task.state.in_(ACTIVE_TASK_STATES))
            .limit(1)
        )
    ).scalar_one_or_none()
    if outstanding_task is not None:
        raise HTTPException(
            status_code=status.HTTP_409_CONFLICT,
            detail="Cannot delete account while outstanding worker instances exist.",
        )

    now = utc_now()
    removal_safe = _task_removal_marker(now=now)
    tasks = (await db.execute(select(Task).where(Task.user_id == user.id))).scalars().all()
    for task in tasks:
        spec_json = dict(task.spec_json) if isinstance(task.spec_json, dict) else {}
        spec_json[ACCOUNT_TASK_REMOVAL_MARKER_KEY] = dict(removal_safe)
        task.spec_json = spec_json
        task.hidden_at = task.hidden_at or now
        task.updated_at = now

    await db.execute(delete(Session).where(Session.user_id == user.id))
    await db.execute(delete(AuthChallenge).where(AuthChallenge.user_id == user.id))
    await db.execute(delete(PasskeyCredential).where(PasskeyCredential.user_id == user.id))
    await db.execute(delete(VerificationToken).where(VerificationToken.user_id == user.id))
    await db.execute(delete(PasswordResetToken).where(PasswordResetToken.user_id == user.id))
    await db.execute(delete(Secret).where(Secret.user_id == user.id))

    user.email = _deleted_email_for_user(user.id)
    user.password_hash = hash_password(new_session_token())
    user.display_name = None
    user.phone_number = None
    user.ui_locale = "en"
    user.billing_address = None
    user.billing_address_line1 = None
    user.billing_address_line2 = None
    user.billing_city = None
    user.billing_state_province = None
    user.billing_country = None
    user.billing_postal_code = None
    user.birthday = None
    user.email_verified_at = None
    user.access_status = "rejected"
    user.access_reviewed_at = now
    user.updated_at = now

    await clear_payment_card_cache(user_id=user.id)
    await db.commit()
