from datetime import datetime, timezone
from typing import Annotated

from fastapi import APIRouter, Cookie, Depends, HTTPException, Request, Response, status
from fastapi.responses import JSONResponse
from sqlalchemy import func, select
from sqlalchemy.ext.asyncio import AsyncSession
from sqlalchemy.orm import joinedload

from app.http.deps import auth_rate_limit, get_current_user
from app.core.config import get_settings
from app.core.security import hash_password, hash_token, new_session_token, session_expiry, verify_password
from app.db.models import Role, Session, User
from app.db.session import get_db
from app.schemas.auth import AccountUpdateRequest, AuthResponse, LoginRequest, MessageResponse, RegisterRequest
from app.services.auth import (
    clear_session_cookie,
    delete_account_data,
    request_ip,
    set_session_cookie,
    user_to_out,
    utc_now,
)

public_router = APIRouter()
user_router = APIRouter(dependencies=[Depends(get_current_user)])
settings = get_settings()

INVALID_LOGIN_DETAIL = "Invalid email or password"


@public_router.post(
    "/register",
    response_model=AuthResponse,
    status_code=status.HTTP_201_CREATED,
    dependencies=[Depends(auth_rate_limit)],
)
async def register(payload: RegisterRequest, db: AsyncSession = Depends(get_db)) -> AuthResponse:
    email = payload.email.lower()

    existing = await db.execute(select(User).where(User.email == email))
    if existing.scalar_one_or_none():
        raise HTTPException(status_code=status.HTTP_400_BAD_REQUEST, detail="Email already registered")

    existing_display_name = await db.execute(
        select(User)
        .where(User.display_name.is_not(None))
        .where(func.lower(User.display_name) == payload.display_name.lower())
    )
    if existing_display_name.scalar_one_or_none():
        raise HTTPException(status_code=status.HTTP_400_BAD_REQUEST, detail="Display name already registered")

    role_result = await db.execute(select(Role).where(Role.name == "user"))
    user_role = role_result.scalar_one_or_none()
    if user_role is None:
        raise HTTPException(status_code=status.HTTP_500_INTERNAL_SERVER_ERROR, detail="Role seed missing")

    user = User(
        email=email,
        password_hash=hash_password(payload.password),
        display_name=payload.display_name,
        role_id=user_role.id,
    )
    db.add(user)
    await db.commit()

    user_result = await db.execute(
        select(User).options(joinedload(User.role)).where(User.id == user.id)
    )
    created_user = user_result.scalar_one()

    return AuthResponse(user=user_to_out(created_user))


@public_router.post("/login", response_model=AuthResponse, dependencies=[Depends(auth_rate_limit)])
async def login(
    payload: LoginRequest,
    request: Request,
    db: AsyncSession = Depends(get_db),
) -> Response:
    email = payload.email.lower()

    result = await db.execute(select(User).options(joinedload(User.role)).where(User.email == email))
    user = result.scalar_one_or_none()

    if user is None or not verify_password(payload.password, user.password_hash):
        raise HTTPException(status_code=status.HTTP_401_UNAUTHORIZED, detail=INVALID_LOGIN_DETAIL)

    session_token = new_session_token()
    session = Session(
        user_id=user.id,
        token_hash=hash_token(session_token),
        expires_at=session_expiry(payload.remember_me),
        last_seen_at=datetime.now(timezone.utc),
        user_agent=request.headers.get("user-agent"),
        ip_address=request_ip(
            request.client.host if request.client else None,
            request.headers.get("x-forwarded-for"),
            request.headers.get("cf-connecting-ip"),
        ),
    )

    db.add(session)
    await db.commit()

    response = JSONResponse(content=AuthResponse(user=user_to_out(user)).model_dump(mode="json"))
    set_session_cookie(response, session_token, payload.remember_me)
    return response


async def get_current_session_optional(
    session_token: Annotated[str | None, Cookie(alias=settings.session_cookie_name)] = None,
    db: AsyncSession = Depends(get_db),
) -> Session | None:
    if not session_token:
        return None

    result = await db.execute(
        select(Session)
        .where(Session.token_hash == hash_token(session_token))
        .where(Session.revoked_at.is_(None))
        .where(Session.expires_at > utc_now())
    )
    return result.scalar_one_or_none()


@public_router.post("/logout", response_model=MessageResponse)
async def logout(
    response: Response,
    db: AsyncSession = Depends(get_db),
    auth_session: Session | None = Depends(get_current_session_optional),
) -> MessageResponse:
    if auth_session is not None:
        auth_session.revoked_at = utc_now()
        await db.commit()

    clear_session_cookie(response)
    return MessageResponse(message="Logged out")


@user_router.get("/me", response_model=AuthResponse)
async def me(current_user: User = Depends(get_current_user)) -> AuthResponse:
    return AuthResponse(user=user_to_out(current_user))


@user_router.patch("/account", response_model=AuthResponse, dependencies=[Depends(auth_rate_limit)])
async def update_account(
    payload: AccountUpdateRequest,
    current_user: User = Depends(get_current_user),
    db: AsyncSession = Depends(get_db),
) -> AuthResponse:
    provided = payload.model_fields_set
    updates: dict[str, object] = {}

    def normalize_optional(value: str | None) -> str | None:
        return (value or "").strip() or None

    if "email" in provided:
        if payload.email is None:
            raise HTTPException(status_code=status.HTTP_400_BAD_REQUEST, detail="Email cannot be empty")
        normalized_email = payload.email.lower()
        if normalized_email != current_user.email:
            existing = await db.execute(select(User).where(User.email == normalized_email))
            existing_user = existing.scalar_one_or_none()
            if existing_user is not None and existing_user.id != current_user.id:
                raise HTTPException(status_code=status.HTTP_400_BAD_REQUEST, detail="Email already registered")
            updates["email"] = normalized_email

    if "display_name" in provided:
        display_name = normalize_optional(payload.display_name)
        if display_name != current_user.display_name:
            if display_name is not None:
                existing_display_name = await db.execute(
                    select(User)
                    .where(User.id != current_user.id)
                    .where(User.display_name.is_not(None))
                    .where(func.lower(User.display_name) == display_name.lower())
                )
                if existing_display_name.scalar_one_or_none() is not None:
                    raise HTTPException(status_code=status.HTTP_400_BAD_REQUEST, detail="Display name already registered")
            updates["display_name"] = display_name

    if "phone_number" in provided:
        phone_number = normalize_optional(payload.phone_number)
        if phone_number != current_user.phone_number:
            updates["phone_number"] = phone_number

    if "billing_address" in provided:
        billing_address = normalize_optional(payload.billing_address)
        if billing_address != current_user.billing_address:
            updates["billing_address"] = billing_address

    if "billing_address_line1" in provided:
        billing_address_line1 = normalize_optional(payload.billing_address_line1)
        if billing_address_line1 != current_user.billing_address_line1:
            updates["billing_address_line1"] = billing_address_line1

    if "billing_address_line2" in provided:
        billing_address_line2 = normalize_optional(payload.billing_address_line2)
        if billing_address_line2 != current_user.billing_address_line2:
            updates["billing_address_line2"] = billing_address_line2

    if "billing_city" in provided:
        billing_city = normalize_optional(payload.billing_city)
        if billing_city != current_user.billing_city:
            updates["billing_city"] = billing_city

    if "billing_state_province" in provided:
        billing_state_province = normalize_optional(payload.billing_state_province)
        if billing_state_province != current_user.billing_state_province:
            updates["billing_state_province"] = billing_state_province

    if "billing_country" in provided:
        billing_country = normalize_optional(payload.billing_country)
        if billing_country != current_user.billing_country:
            updates["billing_country"] = billing_country

    if "billing_postal_code" in provided:
        billing_postal_code = normalize_optional(payload.billing_postal_code)
        if billing_postal_code != current_user.billing_postal_code:
            updates["billing_postal_code"] = billing_postal_code

    if "birthday" in provided and payload.birthday != current_user.birthday:
        updates["birthday"] = payload.birthday

    if "new_password" in provided:
        if payload.new_password is None:
            raise HTTPException(status_code=status.HTTP_400_BAD_REQUEST, detail="New password cannot be empty")
        updates["password_hash"] = hash_password(payload.new_password)

    if not updates:
        return AuthResponse(user=user_to_out(current_user))

    if not payload.current_password or not verify_password(payload.current_password, current_user.password_hash):
        raise HTTPException(
            status_code=status.HTTP_401_UNAUTHORIZED,
            detail="Current password is required and must be valid",
        )

    for key, value in updates.items():
        setattr(current_user, key, value)

    await db.commit()
    await db.refresh(current_user)
    return AuthResponse(user=user_to_out(current_user))


@user_router.delete("/account", response_model=MessageResponse, dependencies=[Depends(auth_rate_limit)])
async def delete_account(
    response: Response,
    current_user: User = Depends(get_current_user),
    db: AsyncSession = Depends(get_db),
) -> MessageResponse:
    await delete_account_data(db, user=current_user)
    clear_session_cookie(response)
    return MessageResponse(message="Account deleted")


@public_router.post("/request-email-verification", response_model=MessageResponse)
async def request_email_verification() -> MessageResponse:
    """Request email verification link. Coming soon."""
    return MessageResponse(message="Email verification is coming soon")


@public_router.post("/request-password-reset", response_model=MessageResponse)
async def request_password_reset() -> MessageResponse:
    """Request password reset link. Coming soon."""
    return MessageResponse(message="Password reset is coming soon")
