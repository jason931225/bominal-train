import logging
import secrets
from datetime import datetime, timedelta, timezone
from typing import Annotated
from urllib.parse import urlencode
from uuid import UUID

from fastapi import APIRouter, Cookie, Depends, HTTPException, Request, Response, status
from fastapi.responses import JSONResponse
from sqlalchemy import func, select
from sqlalchemy.exc import IntegrityError
from sqlalchemy.ext.asyncio import AsyncSession
from sqlalchemy.orm import joinedload

from app.http.deps import auth_rate_limit, get_current_approved_user, get_current_user
from app.core.config import get_settings
from app.core.security import (
    hash_password,
    hash_token,
    new_session_token,
    password_needs_rehash,
    session_expiry,
    verify_password,
)
from app.db.models import PasswordResetToken, Role, Session, User, VerificationToken
from app.db.session import get_db
from app.schemas.auth import (
    AccountUpdateRequest,
    AuthResponse,
    EmailChangeConfirmRequest,
    EmailVerificationConfirmRequest,
    EmailVerificationRequest,
    LoginRequest,
    MessageResponse,
    PasskeyAuthenticationOptionsRequest,
    PasskeyAuthenticationOptionsResponse,
    PasskeyStepUpVerifyRequest,
    PasskeyStepUpVerifyResponse,
    PasskeyAuthenticationVerifyRequest,
    PasskeyCredentialListResponse,
    PasskeyCredentialOut,
    PasskeyRegistrationOptionsResponse,
    PasskeyRegistrationVerifyRequest,
    PasswordVerifyRequest,
    PasswordResetConfirmRequest,
    PasswordResetRequest,
    RegisterRequest,
)
from app.schemas.notification import EmailTemplateBlock, EmailTemplateJobPayload
from app.services.email_queue import enqueue_template_email
from app.services.passkeys import (
    begin_passkey_authentication,
    begin_passkey_registration,
    complete_passkey_authentication,
    complete_passkey_registration,
    consume_passkey_step_up_token,
    delete_passkey,
    ensure_passkeys_enabled,
    issue_passkey_step_up_token,
    list_passkeys,
)
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
logger = logging.getLogger(__name__)

INVALID_LOGIN_DETAIL = "Invalid email or password"
EMAIL_OTP_TTL_MINUTES = 10
PASSWORD_RESET_OTP_TTL_MINUTES = 15
VERIFICATION_PURPOSE_EMAIL = "email_verify"
VERIFICATION_PURPOSE_EMAIL_CHANGE = "email_change"


def _new_otp_code() -> str:
    return f"{secrets.randbelow(1_000_000):06d}"


def _integrity_conflict_detail(exc: IntegrityError) -> str:
    message = str(exc).lower()
    if "display_name" in message:
        return "Display name already registered"
    if "email" in message:
        return "Email already registered"
    return "Account already exists"


def _public_base_url() -> str:
    return settings.app_public_base_url.rstrip("/")


async def _issue_verification_token(
    db: AsyncSession,
    *,
    user_id,
    purpose: str = VERIFICATION_PURPOSE_EMAIL,
    target_email: str | None = None,
) -> tuple[str, datetime]:
    now = utc_now()
    active_tokens = (
        await db.execute(
            select(VerificationToken)
            .where(VerificationToken.user_id == user_id)
            .where(VerificationToken.purpose == purpose)
            .where(VerificationToken.used_at.is_(None))
            .where(VerificationToken.expires_at > now)
        )
    ).scalars().all()
    for token in active_tokens:
        token.used_at = now

    code = _new_otp_code()
    expires_at = now + timedelta(minutes=EMAIL_OTP_TTL_MINUTES)
    db.add(
        VerificationToken(
            user_id=user_id,
            token_hash=hash_token(code),
            purpose=purpose,
            target_email=target_email,
            expires_at=expires_at,
        )
    )
    await db.commit()
    return code, expires_at


async def _issue_password_reset_token(db: AsyncSession, *, user_id) -> tuple[str, datetime]:
    now = utc_now()
    active_tokens = (
        await db.execute(
            select(PasswordResetToken)
            .where(PasswordResetToken.user_id == user_id)
            .where(PasswordResetToken.used_at.is_(None))
            .where(PasswordResetToken.expires_at > now)
        )
    ).scalars().all()
    for token in active_tokens:
        token.used_at = now

    code = _new_otp_code()
    expires_at = now + timedelta(minutes=PASSWORD_RESET_OTP_TTL_MINUTES)
    db.add(
        PasswordResetToken(
            user_id=user_id,
            token_hash=hash_token(code),
            expires_at=expires_at,
        )
    )
    await db.commit()
    return code, expires_at


def _verification_template_payload(*, email: str, display_name: str | None, code: str) -> EmailTemplateJobPayload:
    verify_query = urlencode({"email": email, "code": code})
    verify_url = f"{_public_base_url()}/api/auth/verify-email?{verify_query}"
    return EmailTemplateJobPayload(
        to_email=email,
        subject="Verify your email for bominal",
        preheader="Verify with the button or enter the code.",
        theme="spring",
        blocks=[
            EmailTemplateBlock(type="hero", data={"title": "Welcome to bominal", "subtitle": "Verify your email to finish setup."}),
            EmailTemplateBlock(
                type="cta",
                data={
                    "label": "Verify email",
                    "url": {"$ref": "verify.url"},
                    "helper_text": "Link expires in {{ verify.ttl_minutes }} minutes.",
                },
            ),
            EmailTemplateBlock(
                type="otp",
                data={
                    "code": {"$ref": "verify.code"},
                    "ttl_minutes": {"$ref": "verify.ttl_minutes"},
                },
            ),
            EmailTemplateBlock(type="divider", data={}),
            EmailTemplateBlock(
                type="bullets",
                data={
                    "items": [
                        "Use either the button or the code - both work.",
                        "If this wasn't you, ignore this email.",
                    ]
                },
            ),
        ],
        context={
            "user": {"display_name": display_name or email},
            "verify": {"url": verify_url, "code": code, "ttl_minutes": EMAIL_OTP_TTL_MINUTES},
        },
        tags=["onboarding", "verify"],
        metadata={"kind": "onboarding_verify"},
    )


def _password_reset_template_payload(*, email: str, code: str) -> EmailTemplateJobPayload:
    reset_query = urlencode({"email": email, "code": code})
    reset_url = f"{_public_base_url()}/reset-password?{reset_query}"
    return EmailTemplateJobPayload(
        to_email=email,
        subject="Reset your bominal password",
        preheader="Use the button or OTP to reset your password.",
        theme="winter",
        blocks=[
            EmailTemplateBlock(type="hero", data={"title": "Password reset request", "subtitle": "Confirm this request to set a new password."}),
            EmailTemplateBlock(
                type="cta",
                data={
                    "label": "Reset password",
                    "url": {"$ref": "reset.url"},
                    "helper_text": "Link expires in {{ reset.ttl_minutes }} minutes.",
                },
            ),
            EmailTemplateBlock(
                type="otp",
                data={
                    "code": {"$ref": "reset.code"},
                    "ttl_minutes": {"$ref": "reset.ttl_minutes"},
                    "label": "Password reset code",
                },
            ),
            EmailTemplateBlock(
                type="paragraph",
                data={"text": "If you didn't request this, you can ignore this email."},
            ),
        ],
        context={"reset": {"url": reset_url, "code": code, "ttl_minutes": PASSWORD_RESET_OTP_TTL_MINUTES}},
        tags=["auth", "password-reset"],
        metadata={"kind": "password_reset"},
    )


def _email_change_template_payload(*, email: str, code: str) -> EmailTemplateJobPayload:
    verify_query = urlencode({"email": email, "code": code, "email_change": "1"})
    verify_url = f"{_public_base_url()}/settings/account?{verify_query}"
    return EmailTemplateJobPayload(
        to_email=email,
        subject="Confirm your new email for bominal",
        preheader="Use the button or OTP to complete your email change.",
        theme="spring",
        blocks=[
            EmailTemplateBlock(
                type="hero",
                data={"title": "Confirm your new email", "subtitle": "Email updates take effect only after verification."},
            ),
            EmailTemplateBlock(
                type="cta",
                data={
                    "label": "Confirm email change",
                    "url": {"$ref": "verify.url"},
                    "helper_text": "Link expires in {{ verify.ttl_minutes }} minutes.",
                },
            ),
            EmailTemplateBlock(
                type="otp",
                data={
                    "code": {"$ref": "verify.code"},
                    "ttl_minutes": {"$ref": "verify.ttl_minutes"},
                    "label": "Email change code",
                },
            ),
            EmailTemplateBlock(
                type="paragraph",
                data={"text": "If you did not request this change, ignore this email and keep your current address."},
            ),
        ],
        context={"verify": {"url": verify_url, "code": code, "ttl_minutes": EMAIL_OTP_TTL_MINUTES}},
        tags=["account", "email-change"],
        metadata={"kind": "email_change_verify"},
    )


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
        ui_locale="en",
        access_status="pending",
        role_id=user_role.id,
    )
    db.add(user)
    try:
        await db.commit()
    except IntegrityError as exc:
        await db.rollback()
        raise HTTPException(status_code=status.HTTP_409_CONFLICT, detail=_integrity_conflict_detail(exc)) from exc

    user_result = await db.execute(
        select(User).options(joinedload(User.role)).where(User.id == user.id)
    )
    created_user = user_result.scalar_one()

    try:
        code, _ = await _issue_verification_token(
            db,
            user_id=created_user.id,
            purpose=VERIFICATION_PURPOSE_EMAIL,
        )
        await enqueue_template_email(
            _verification_template_payload(
                email=created_user.email,
                display_name=created_user.display_name,
                code=code,
            )
        )
    except Exception as exc:
        logger.warning("Failed to queue onboarding verification email for user %s: %s", created_user.id, type(exc).__name__)

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

    if user is None:
        raise HTTPException(status_code=status.HTTP_401_UNAUTHORIZED, detail=INVALID_LOGIN_DETAIL)
    if not verify_password(payload.password, user.password_hash):
        raise HTTPException(status_code=status.HTTP_401_UNAUTHORIZED, detail=INVALID_LOGIN_DETAIL)

    # Opportunistic migration: rehash on successful login when policy changed.
    if password_needs_rehash(user.password_hash):
        user.password_hash = hash_password(payload.password)

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
    current_user: User = Depends(get_current_approved_user),
    db: AsyncSession = Depends(get_db),
) -> AuthResponse:
    provided = payload.model_fields_set
    updates: dict[str, object] = {}
    requested_email_change_to: str | None = None

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
            requested_email_change_to = normalized_email

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

    if "ui_locale" in provided:
        if payload.ui_locale is None:
            raise HTTPException(status_code=status.HTTP_400_BAD_REQUEST, detail="ui_locale cannot be empty")
        if payload.ui_locale != current_user.ui_locale:
            updates["ui_locale"] = payload.ui_locale

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

    if not updates and requested_email_change_to is None:
        return AuthResponse(user=user_to_out(current_user))

    sensitive_update = requested_email_change_to is not None or "password_hash" in updates
    if sensitive_update:
        has_valid_password = bool(payload.current_password) and verify_password(
            payload.current_password or "",
            current_user.password_hash,
        )
        has_valid_step_up = bool(payload.passkey_step_up_token) and await consume_passkey_step_up_token(
            db,
            user_id=current_user.id,
            token=payload.passkey_step_up_token or "",
        )
        if not (has_valid_password or has_valid_step_up):
            raise HTTPException(
                status_code=status.HTTP_401_UNAUTHORIZED,
                detail="Current password is required and must be valid, or verify with passkey",
            )

    for key, value in updates.items():
        setattr(current_user, key, value)

    notice: str | None = None
    try:
        await db.commit()
    except IntegrityError as exc:
        await db.rollback()
        raise HTTPException(status_code=status.HTTP_409_CONFLICT, detail=_integrity_conflict_detail(exc)) from exc

    if requested_email_change_to is not None:
        try:
            code, _ = await _issue_verification_token(
                db,
                user_id=current_user.id,
                purpose=VERIFICATION_PURPOSE_EMAIL_CHANGE,
                target_email=requested_email_change_to,
            )
            await enqueue_template_email(
                _email_change_template_payload(
                    email=requested_email_change_to,
                    code=code,
                )
            )
            notice = "Email change requested. Verify the new address to apply it."
        except Exception as exc:
            logger.warning(
                "Failed to queue email-change verification for user %s: %s",
                current_user.id,
                type(exc).__name__,
            )
            raise HTTPException(
                status_code=status.HTTP_500_INTERNAL_SERVER_ERROR,
                detail="Could not send email verification for address change",
            ) from exc

    await db.refresh(current_user)
    return AuthResponse(user=user_to_out(current_user), notice=notice, pending_email_change_to=requested_email_change_to)


@user_router.post("/account/verify-password", response_model=MessageResponse, dependencies=[Depends(auth_rate_limit)])
async def verify_current_password(
    payload: PasswordVerifyRequest,
    current_user: User = Depends(get_current_approved_user),
) -> MessageResponse:
    if not verify_password(payload.current_password, current_user.password_hash):
        raise HTTPException(status_code=status.HTTP_401_UNAUTHORIZED, detail="Current password is invalid")
    return MessageResponse(message="Password verified")


@user_router.post("/account/email-change/confirm", response_model=AuthResponse, dependencies=[Depends(auth_rate_limit)])
async def confirm_email_change(
    payload: EmailChangeConfirmRequest,
    current_user: User = Depends(get_current_approved_user),
    db: AsyncSession = Depends(get_db),
) -> AuthResponse:
    requested_email = payload.email.lower()
    token_hash = hash_token(payload.code)
    now = utc_now()
    token = (
        await db.execute(
            select(VerificationToken)
            .where(VerificationToken.user_id == current_user.id)
            .where(VerificationToken.purpose == VERIFICATION_PURPOSE_EMAIL_CHANGE)
            .where(VerificationToken.target_email == requested_email)
            .where(VerificationToken.token_hash == token_hash)
            .where(VerificationToken.used_at.is_(None))
            .where(VerificationToken.expires_at > now)
        )
    ).scalar_one_or_none()
    if token is None:
        raise HTTPException(status_code=status.HTTP_400_BAD_REQUEST, detail="Invalid or expired verification code")

    existing = (await db.execute(select(User).where(User.email == requested_email))).scalar_one_or_none()
    if existing is not None and existing.id != current_user.id:
        raise HTTPException(status_code=status.HTTP_400_BAD_REQUEST, detail="Email already registered")

    token.used_at = now
    current_user.email = requested_email
    current_user.email_verified_at = now
    await db.commit()
    await db.refresh(current_user)
    return AuthResponse(user=user_to_out(current_user), notice="Email address updated")


@user_router.get("/passkeys", response_model=PasskeyCredentialListResponse)
async def get_passkeys(
    current_user: User = Depends(get_current_approved_user),
    db: AsyncSession = Depends(get_db),
) -> PasskeyCredentialListResponse:
    ensure_passkeys_enabled()
    credentials = await list_passkeys(db, user_id=current_user.id)
    return PasskeyCredentialListResponse(
        credentials=[
            PasskeyCredentialOut(id=item.id, created_at=item.created_at, last_used_at=item.last_used_at)
            for item in credentials
        ]
    )


@user_router.post("/passkeys/register/options", response_model=PasskeyRegistrationOptionsResponse)
async def passkey_register_options(
    current_user: User = Depends(get_current_approved_user),
    db: AsyncSession = Depends(get_db),
) -> PasskeyRegistrationOptionsResponse:
    ensure_passkeys_enabled()
    challenge_id, public_key = await begin_passkey_registration(db, user=current_user)
    return PasskeyRegistrationOptionsResponse(challenge_id=challenge_id, public_key=public_key)


@user_router.post("/passkeys/register/verify", response_model=PasskeyCredentialOut)
async def passkey_register_verify(
    payload: PasskeyRegistrationVerifyRequest,
    current_user: User = Depends(get_current_approved_user),
    db: AsyncSession = Depends(get_db),
) -> PasskeyCredentialOut:
    ensure_passkeys_enabled()
    credential = await complete_passkey_registration(
        db,
        user=current_user,
        challenge_id=payload.challenge_id,
        credential=payload.credential,
    )
    return PasskeyCredentialOut(id=credential.id, created_at=credential.created_at, last_used_at=credential.last_used_at)


@user_router.post("/passkeys/step-up/options", response_model=PasskeyAuthenticationOptionsResponse)
async def passkey_step_up_options(
    current_user: User = Depends(get_current_approved_user),
    db: AsyncSession = Depends(get_db),
) -> PasskeyAuthenticationOptionsResponse:
    ensure_passkeys_enabled()
    challenge_id, public_key = await begin_passkey_authentication(db, email=current_user.email, user=current_user)
    return PasskeyAuthenticationOptionsResponse(challenge_id=challenge_id, public_key=public_key)


@user_router.post("/passkeys/step-up/verify", response_model=PasskeyStepUpVerifyResponse)
async def passkey_step_up_verify(
    payload: PasskeyStepUpVerifyRequest,
    current_user: User = Depends(get_current_approved_user),
    db: AsyncSession = Depends(get_db),
) -> PasskeyStepUpVerifyResponse:
    ensure_passkeys_enabled()
    await complete_passkey_authentication(
        db,
        email=current_user.email,
        user=current_user,
        challenge_id=payload.challenge_id,
        credential=payload.credential,
    )
    token = await issue_passkey_step_up_token(db, user_id=current_user.id)
    return PasskeyStepUpVerifyResponse(step_up_token=token)


@user_router.delete("/passkeys/{passkey_id}", response_model=MessageResponse)
async def remove_passkey(
    passkey_id: UUID,
    current_user: User = Depends(get_current_approved_user),
    db: AsyncSession = Depends(get_db),
) -> MessageResponse:
    ensure_passkeys_enabled()
    deleted = await delete_passkey(db, user_id=current_user.id, passkey_id=passkey_id)
    if not deleted:
        raise HTTPException(status_code=status.HTTP_404_NOT_FOUND, detail="Passkey not found")
    return MessageResponse(message="Passkey removed")


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
async def request_email_verification(
    payload: EmailVerificationRequest | None = None,
    db: AsyncSession = Depends(get_db),
) -> MessageResponse:
    if payload is None:
        return MessageResponse(message="If eligible, a verification email has been sent")

    email = payload.email.lower()
    user = (await db.execute(select(User).where(User.email == email))).scalar_one_or_none()
    if user is None:
        return MessageResponse(message="If eligible, a verification email has been sent")

    try:
        code, _ = await _issue_verification_token(
            db,
            user_id=user.id,
            purpose=VERIFICATION_PURPOSE_EMAIL,
        )
        await enqueue_template_email(
            _verification_template_payload(
                email=user.email,
                display_name=user.display_name,
                code=code,
            )
        )
    except Exception as exc:
        logger.warning("Failed to queue verification email for user %s: %s", user.id, type(exc).__name__)

    return MessageResponse(message="If eligible, a verification email has been sent")


@public_router.post("/verify-email", response_model=MessageResponse)
async def verify_email(
    payload: EmailVerificationConfirmRequest,
    db: AsyncSession = Depends(get_db),
) -> MessageResponse:
    email = payload.email.lower()
    user = (await db.execute(select(User).where(User.email == email))).scalar_one_or_none()
    if user is None:
        raise HTTPException(status_code=status.HTTP_400_BAD_REQUEST, detail="Invalid or expired verification code")

    token_hash = hash_token(payload.code)
    now = utc_now()
    token = (
        await db.execute(
            select(VerificationToken)
            .where(VerificationToken.user_id == user.id)
            .where(VerificationToken.token_hash == token_hash)
            .where(VerificationToken.purpose == VERIFICATION_PURPOSE_EMAIL)
            .where(VerificationToken.used_at.is_(None))
            .where(VerificationToken.expires_at > now)
        )
    ).scalar_one_or_none()
    if token is None:
        raise HTTPException(status_code=status.HTTP_400_BAD_REQUEST, detail="Invalid or expired verification code")

    token.used_at = now
    user.email_verified_at = now
    await db.commit()
    return MessageResponse(message="Email verified successfully")


@public_router.post("/request-password-reset", response_model=MessageResponse)
async def request_password_reset(
    payload: PasswordResetRequest | None = None,
    db: AsyncSession = Depends(get_db),
) -> MessageResponse:
    if payload is None:
        return MessageResponse(message="If eligible, a password reset email has been sent")

    email = payload.email.lower()
    user = (await db.execute(select(User).where(User.email == email))).scalar_one_or_none()
    if user is None:
        return MessageResponse(message="If eligible, a password reset email has been sent")

    try:
        code, _ = await _issue_password_reset_token(db, user_id=user.id)
        await enqueue_template_email(_password_reset_template_payload(email=user.email, code=code))
    except Exception as exc:
        logger.warning("Failed to queue password reset email for user %s: %s", user.id, type(exc).__name__)

    return MessageResponse(message="If eligible, a password reset email has been sent")


@public_router.post("/reset-password", response_model=MessageResponse)
async def reset_password(
    payload: PasswordResetConfirmRequest,
    db: AsyncSession = Depends(get_db),
) -> MessageResponse:
    email = payload.email.lower()
    user = (await db.execute(select(User).where(User.email == email))).scalar_one_or_none()
    if user is None:
        raise HTTPException(status_code=status.HTTP_400_BAD_REQUEST, detail="Invalid or expired reset code")

    token_hash = hash_token(payload.code)
    now = utc_now()
    token = (
        await db.execute(
            select(PasswordResetToken)
            .where(PasswordResetToken.user_id == user.id)
            .where(PasswordResetToken.token_hash == token_hash)
            .where(PasswordResetToken.used_at.is_(None))
            .where(PasswordResetToken.expires_at > now)
        )
    ).scalar_one_or_none()
    if token is None:
        raise HTTPException(status_code=status.HTTP_400_BAD_REQUEST, detail="Invalid or expired reset code")

    token.used_at = now
    user.password_hash = hash_password(payload.new_password)
    await db.commit()
    return MessageResponse(message="Password reset complete")


@public_router.post("/passkeys/auth/options", response_model=PasskeyAuthenticationOptionsResponse)
async def passkey_auth_options(
    payload: PasskeyAuthenticationOptionsRequest,
    db: AsyncSession = Depends(get_db),
) -> PasskeyAuthenticationOptionsResponse:
    ensure_passkeys_enabled()
    email = payload.email.lower()
    user = (
        await db.execute(select(User).options(joinedload(User.role)).where(User.email == email))
    ).scalar_one_or_none()
    if user is None:
        raise HTTPException(status_code=status.HTTP_400_BAD_REQUEST, detail="No passkey registered for this account")
    challenge_id, public_key = await begin_passkey_authentication(db, email=email, user=user)
    return PasskeyAuthenticationOptionsResponse(challenge_id=challenge_id, public_key=public_key)


@public_router.post("/passkeys/auth/verify", response_model=AuthResponse, dependencies=[Depends(auth_rate_limit)])
async def passkey_auth_verify(
    payload: PasskeyAuthenticationVerifyRequest,
    request: Request,
    db: AsyncSession = Depends(get_db),
) -> Response:
    ensure_passkeys_enabled()
    email = payload.email.lower()
    user = (
        await db.execute(select(User).options(joinedload(User.role)).where(User.email == email))
    ).scalar_one_or_none()
    if user is None:
        raise HTTPException(status_code=status.HTTP_400_BAD_REQUEST, detail="Passkey authentication failed")

    await complete_passkey_authentication(
        db,
        email=email,
        user=user,
        challenge_id=payload.challenge_id,
        credential=payload.credential,
    )

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
