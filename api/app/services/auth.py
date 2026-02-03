from datetime import datetime, timezone

from fastapi import Response

from app.core.config import get_settings
from app.db.models import User
from app.schemas.auth import UserOut

settings = get_settings()


def user_to_out(user: User) -> UserOut:
    return UserOut(
        id=user.id,
        email=user.email,
        display_name=user.display_name,
        phone_number=user.phone_number,
        billing_address=user.billing_address,
        billing_address_line1=user.billing_address_line1,
        billing_address_line2=user.billing_address_line2,
        billing_city=user.billing_city,
        billing_state_province=user.billing_state_province,
        billing_country=user.billing_country,
        billing_postal_code=user.billing_postal_code,
        birthday=user.birthday,
        role=user.role.name,
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


def request_ip(remote: str | None, forwarded: str | None) -> str | None:
    if forwarded:
        return forwarded.split(",")[0].strip()
    return remote


def utc_now() -> datetime:
    return datetime.now(timezone.utc)
