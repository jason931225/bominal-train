import logging
import hmac
import json
import secrets
import time
from datetime import datetime, timedelta, timezone
from typing import Annotated
from urllib.parse import urlencode
from uuid import UUID

from fastapi import APIRouter, BackgroundTasks, Cookie, Depends, HTTPException, Request, Response, status
from fastapi.responses import JSONResponse
from sqlalchemy import func, select
from sqlalchemy.exc import IntegrityError
from sqlalchemy.ext.asyncio import AsyncSession
from sqlalchemy.orm import joinedload

from app.http.deps import auth_rate_limit, get_current_approved_user, get_current_user
from app.core.config import get_settings
from app.core.redis import get_redis_client
from app.core.security import (
    async_hash_password,
    async_password_needs_rehash,
    async_verify_password,
    hash_token,
    new_session_token,
    session_expiry,
)
from app.db.models import AuthChallenge, PasswordResetToken, Role, Session, User, VerificationToken
from app.db.session import SessionLocal, get_db
from app.schemas.auth import (
    AccountUpdateRequest,
    AuthMethodsResponse,
    AuthResponse,
    DevDemoPasskeySignInRequest,
    EmailChangeConfirmRequest,
    EmailVerificationConfirmRequest,
    EmailVerificationRequest,
    LoginRequest,
    MagicLinkConfirmRequest,
    MagicLinkRequest,
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
    SignInOtpRequest,
    SignInOtpVerifyRequest,
    SupabaseConfirmRequest,
    SupabaseConfirmResponse,
    SupabasePasswordResetConfirmRequest,
)
from app.schemas.notification import EmailTemplateBlock, EmailTemplateJobPayload
from app.modules.train.service import refresh_train_reservations_after_sign_in
from app.services.email_queue import enqueue_template_email
from app.services.email_worker import deliver_email_job
from app.services.identity import get_or_create_local_user_from_supabase_claims
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
from app.services.supabase_auth import (
    exchange_supabase_token_hash_detailed,
    send_supabase_magic_link,
    send_supabase_password_recovery,
    send_supabase_signin_otp,
    update_supabase_password_detailed,
    verify_supabase_signin_otp,
    verify_supabase_password,
)

public_router = APIRouter()
user_router = APIRouter(dependencies=[Depends(get_current_user)])
settings = get_settings()
logger = logging.getLogger(__name__)

INVALID_LOGIN_DETAIL = "Invalid email or password"
SIGNUP_DISABLED_DETAIL = "Sign up is currently disabled"
EMAIL_OTP_TTL_MINUTES = 10
PASSWORD_RESET_OTP_TTL_MINUTES = 15
VERIFICATION_PURPOSE_EMAIL = "email_verify"
VERIFICATION_PURPOSE_EMAIL_CHANGE = "email_change"
VERIFICATION_PURPOSE_MAGIC_LOGIN = "magic_login"
PASSKEY_SETUP_CONTEXT_PURPOSE = "passkey_setup"
PASSKEY_SETUP_CONTEXT_COOKIE = "bominal_passkey_setup_ctx"
PASSKEY_SETUP_CONTEXT_TTL_SECONDS = 10 * 60
PASSKEY_SETUP_ALLOWED_SOURCES = {"signup", "reset", "magiclink"}
SUPABASE_RECOVERY_CONTEXT_COOKIE = "bominal_supabase_recovery_ctx"
SUPABASE_RECOVERY_CONTEXT_REDIS_PREFIX = "auth:supabase:recovery_ctx:"
SUPABASE_RECOVERY_CONTEXT_TTL_SECONDS = 15 * 60


def _new_otp_code() -> str:
    return f"{secrets.randbelow(1_000_000):06d}"


def _integrity_conflict_detail(exc: IntegrityError) -> str:
    message = str(exc).lower()
    if "display_name" in message:
        return "Display name already registered"
    if "email" in message:
        return "Email already registered"
    return "Account already exists"


async def _is_idempotent_register_retry(*, existing_user: User, payload: RegisterRequest) -> bool:
    existing_display_name = str(existing_user.display_name or "").strip().lower()
    requested_display_name = payload.display_name.strip().lower()
    if existing_display_name != requested_display_name:
        return False
    return await async_verify_password(payload.password, existing_user.password_hash)


def _public_base_url() -> str:
    return settings.app_public_base_url.rstrip("/")


def _extract_bearer_token(authorization_header: str | None) -> str | None:
    if not authorization_header:
        return None
    parts = authorization_header.strip().split(None, 1)
    if len(parts) != 2:
        return None
    if parts[0].lower() != "bearer":
        return None
    token = parts[1].strip()
    return token or None


def _is_dev_demo_mode_enabled() -> bool:
    return settings.dev_demo_auth_enabled and not settings.is_production


def _normalized_dev_demo_email() -> str:
    return settings.dev_demo_email.strip().lower()


def _is_dev_demo_email(email: str) -> bool:
    normalized_candidate = email.strip().lower()
    normalized_target = _normalized_dev_demo_email()
    if not normalized_candidate or not normalized_target:
        return False
    return hmac.compare_digest(normalized_candidate, normalized_target)


async def _resolve_or_create_env_user(
    db: AsyncSession,
    *,
    email: str,
    role_name: str,
    display_name: str,
    password_seed: str,
) -> User:
    normalized_email = email.strip().lower()
    if not normalized_email:
        raise HTTPException(status_code=status.HTTP_503_SERVICE_UNAVAILABLE, detail="Configured auth email is invalid")

    role_result = await db.execute(select(Role).where(Role.name == role_name))
    role = role_result.scalar_one_or_none()
    if role is None:
        raise HTTPException(
            status_code=status.HTTP_503_SERVICE_UNAVAILABLE,
            detail=f"Missing role seed: {role_name}",
        )

    user = (
        await db.execute(
            select(User).options(joinedload(User.role)).where(User.email == normalized_email)
        )
    ).scalar_one_or_none()
    if user is None:
        now = utc_now()
        user = User(
            email=normalized_email,
            password_hash=await async_hash_password(password_seed),
            display_name=display_name,
            role_id=role.id,
            access_status="approved",
            email_verified_at=now,
        )
        db.add(user)
        await db.commit()
        return (
            await db.execute(select(User).options(joinedload(User.role)).where(User.email == normalized_email))
        ).scalar_one()

    should_commit = False
    if user.role_id != role.id:
        user.role_id = role.id
        should_commit = True
    if user.access_status != "approved":
        user.access_status = "approved"
        should_commit = True
    if user.email_verified_at is None:
        user.email_verified_at = utc_now()
        should_commit = True
    if not await async_verify_password(password_seed, user.password_hash):
        user.password_hash = await async_hash_password(password_seed)
        should_commit = True
    if should_commit:
        await db.commit()

    return (
        await db.execute(select(User).options(joinedload(User.role)).where(User.id == user.id))
    ).scalar_one()


async def _resolve_dev_demo_user(db: AsyncSession) -> User:
    return await _resolve_or_create_env_user(
        db,
        email=settings.dev_demo_email,
        role_name=settings.dev_demo_role,
        display_name="Dev Demo",
        password_seed=settings.dev_demo_password,
    )


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


def _set_passkey_setup_context_cookie(response: Response, token: str) -> None:
    response.set_cookie(
        key=PASSKEY_SETUP_CONTEXT_COOKIE,
        value=token,
        max_age=PASSKEY_SETUP_CONTEXT_TTL_SECONDS,
        expires=PASSKEY_SETUP_CONTEXT_TTL_SECONDS,
        httponly=True,
        secure=settings.is_production,
        samesite="lax",
        path="/",
    )


def _clear_passkey_setup_context_cookie(response: Response) -> None:
    response.delete_cookie(
        key=PASSKEY_SETUP_CONTEXT_COOKIE,
        httponly=True,
        secure=settings.is_production,
        samesite="lax",
        path="/",
    )


def _supabase_recovery_context_key(token: str) -> str:
    return f"{SUPABASE_RECOVERY_CONTEXT_REDIS_PREFIX}{token}"


def _set_supabase_recovery_context_cookie(response: Response, *, token: str) -> None:
    response.set_cookie(
        key=SUPABASE_RECOVERY_CONTEXT_COOKIE,
        value=token,
        max_age=SUPABASE_RECOVERY_CONTEXT_TTL_SECONDS,
        expires=SUPABASE_RECOVERY_CONTEXT_TTL_SECONDS,
        httponly=True,
        secure=settings.is_production,
        samesite="lax",
        path="/",
    )


def _clear_supabase_recovery_context_cookie(response: Response) -> None:
    response.delete_cookie(
        key=SUPABASE_RECOVERY_CONTEXT_COOKIE,
        httponly=True,
        secure=settings.is_production,
        samesite="lax",
        path="/",
    )


def _supabase_recovery_error_response(*, status_code: int, detail: str) -> JSONResponse:
    response = JSONResponse(status_code=status_code, content={"detail": detail})
    _clear_supabase_recovery_context_cookie(response)
    return response


async def _issue_supabase_recovery_context(*, access_token: str, refresh_token: str | None) -> str | None:
    normalized_access_token = str(access_token or "").strip()
    if not normalized_access_token:
        return None
    token = secrets.token_urlsafe(32)
    payload: dict[str, str] = {"access_token": normalized_access_token}
    normalized_refresh_token = str(refresh_token or "").strip()
    if normalized_refresh_token:
        payload["refresh_token"] = normalized_refresh_token

    serialized = json.dumps(payload, separators=(",", ":"), ensure_ascii=True).encode("utf-8")
    key = _supabase_recovery_context_key(token)
    try:
        redis = await get_redis_client()
        await redis.set(key, serialized, ex=SUPABASE_RECOVERY_CONTEXT_TTL_SECONDS)
    except Exception as exc:  # noqa: BLE001
        logger.warning("Supabase recovery context store failed: %s", type(exc).__name__)
        return None
    return token


async def _load_supabase_recovery_context(*, token: str) -> tuple[str, str | None] | None:
    normalized_token = str(token or "").strip()
    if not normalized_token:
        return None
    key = _supabase_recovery_context_key(normalized_token)

    try:
        redis = await get_redis_client()
        raw_payload = await redis.get(key)
    except Exception as exc:  # noqa: BLE001
        logger.warning("Supabase recovery context load failed: %s", type(exc).__name__)
        return None

    if raw_payload is None:
        return None
    if isinstance(raw_payload, bytes):
        serialized = raw_payload.decode("utf-8", errors="ignore")
    elif isinstance(raw_payload, str):
        serialized = raw_payload
    else:
        return None

    try:
        payload = json.loads(serialized)
    except ValueError:
        return None
    if not isinstance(payload, dict):
        return None

    access_token = str(payload.get("access_token") or "").strip()
    refresh_token_raw = payload.get("refresh_token")
    refresh_token = str(refresh_token_raw).strip() if isinstance(refresh_token_raw, str) else ""
    if not access_token:
        return None
    return access_token, (refresh_token or None)


async def _delete_supabase_recovery_context(*, token: str) -> None:
    normalized_token = str(token or "").strip()
    if not normalized_token:
        return
    key = _supabase_recovery_context_key(normalized_token)
    try:
        redis = await get_redis_client()
        await redis.delete(key)
    except Exception as exc:  # noqa: BLE001
        logger.warning("Supabase recovery context delete failed: %s", type(exc).__name__)


def _is_supabase_password_policy_failure(*, status_code: int | None, error_message: str | None) -> bool:
    if status_code == 422:
        return True
    if status_code == 400 and "password" in str(error_message or "").lower():
        return True
    return False


async def _issue_passkey_setup_context(db: AsyncSession, *, user_id: UUID, source: str) -> str:
    normalized_source = source.strip().lower()
    if normalized_source not in PASSKEY_SETUP_ALLOWED_SOURCES:
        normalized_source = "magiclink"
    now = utc_now()
    active = (
        await db.execute(
            select(AuthChallenge)
            .where(AuthChallenge.user_id == user_id)
            .where(AuthChallenge.purpose == PASSKEY_SETUP_CONTEXT_PURPOSE)
            .where(AuthChallenge.used_at.is_(None))
            .where(AuthChallenge.expires_at > now)
        )
    ).scalars().all()
    for item in active:
        item.used_at = now
    token = secrets.token_urlsafe(32)
    db.add(
        AuthChallenge(
            user_id=user_id,
            email=None,
            purpose=PASSKEY_SETUP_CONTEXT_PURPOSE,
            challenge_hash=hash_token(token),
            challenge_b64url=normalized_source,
            expires_at=now + timedelta(seconds=PASSKEY_SETUP_CONTEXT_TTL_SECONDS),
        )
    )
    await db.commit()
    return token


async def _consume_passkey_setup_context(db: AsyncSession, *, user_id: UUID, token: str, source: str) -> bool:
    normalized_token = token.strip()
    if not normalized_token:
        return False
    normalized_source = source.strip().lower()
    now = utc_now()
    challenge = (
        await db.execute(
            select(AuthChallenge)
            .where(AuthChallenge.user_id == user_id)
            .where(AuthChallenge.purpose == PASSKEY_SETUP_CONTEXT_PURPOSE)
            .where(AuthChallenge.challenge_hash == hash_token(normalized_token))
            .where(AuthChallenge.used_at.is_(None))
            .where(AuthChallenge.expires_at > now)
        )
    ).scalar_one_or_none()
    if challenge is None:
        return False
    if normalized_source and challenge.challenge_b64url.strip().lower() != normalized_source:
        return False
    challenge.used_at = now
    await db.commit()
    return True


def _supabase_signin_otp_enabled() -> bool:
    return settings.auth_mode == "supabase" and settings.supabase_auth_enabled and settings.supabase_signin_otp_enabled


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


def _magic_link_template_payload(*, email: str, code: str) -> EmailTemplateJobPayload:
    link_query = urlencode({"email": email, "code": code})
    link_url = f"{_public_base_url()}/auth/magic-link?{link_query}"
    return EmailTemplateJobPayload(
        to_email=email,
        subject="Your bominal sign-in link",
        preheader="Use the link below to sign in without a password.",
        theme="spring",
        blocks=[
            EmailTemplateBlock(
                type="hero",
                data={"title": "Sign in to bominal", "subtitle": "Use this one-time link to sign in."},
            ),
            EmailTemplateBlock(
                type="cta",
                data={
                    "label": "Sign in",
                    "url": {"$ref": "magic.url"},
                    "helper_text": "Link expires in {{ magic.ttl_minutes }} minutes.",
                },
            ),
            EmailTemplateBlock(
                type="otp",
                data={
                    "code": {"$ref": "magic.code"},
                    "ttl_minutes": {"$ref": "magic.ttl_minutes"},
                    "label": "Magic link code",
                },
            ),
            EmailTemplateBlock(
                type="paragraph",
                data={"text": "If you did not request this sign-in link, you can ignore this email."},
            ),
        ],
        context={"magic": {"url": link_url, "code": code, "ttl_minutes": EMAIL_OTP_TTL_MINUTES}},
        tags=["auth", "magic-link"],
        metadata={"kind": "magic_link"},
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
    if not settings.auth_registration_enabled:
        raise HTTPException(status_code=status.HTTP_403_FORBIDDEN, detail=SIGNUP_DISABLED_DETAIL)

    email = payload.email.lower()

    existing = await db.execute(select(User).options(joinedload(User.role)).where(User.email == email))
    existing_user = existing.scalar_one_or_none()
    if existing_user is not None:
        if await _is_idempotent_register_retry(existing_user=existing_user, payload=payload):
            return AuthResponse(
                user=user_to_out(existing_user),
                notice="Account already exists. Continuing with existing account.",
            )
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
        password_hash=await async_hash_password(payload.password),
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
        detail = _integrity_conflict_detail(exc)
        if detail == "Email already registered":
            existing_after_conflict = (
                await db.execute(select(User).options(joinedload(User.role)).where(User.email == email))
            ).scalar_one_or_none()
            if existing_after_conflict is not None and await _is_idempotent_register_retry(
                existing_user=existing_after_conflict,
                payload=payload,
            ):
                return AuthResponse(
                    user=user_to_out(existing_after_conflict),
                    notice="Account already exists. Continuing with existing account.",
                )
        raise HTTPException(status_code=status.HTTP_409_CONFLICT, detail=detail) from exc

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


async def _refresh_train_reservations_after_sign_in_background(*, user_id: UUID) -> None:
    # Best-effort background refresh. This must never make login fail.
    try:
        async with SessionLocal() as background_db:
            user = (
                await background_db.execute(
                    select(User).options(joinedload(User.role)).where(User.id == user_id)
                )
            ).scalar_one_or_none()
            if user is None:
                logger.warning("Skipped sign-in reservation refresh for unknown user", extra={"user_id": str(user_id)})
                return
            try:
                await refresh_train_reservations_after_sign_in(background_db, user=user)
            except Exception:
                await background_db.rollback()
                logger.warning("Failed background train reservation refresh after sign-in", extra={"user_id": str(user_id)})
    except Exception:
        logger.warning(
            "Skipped background train reservation refresh after sign-in due to background session error",
            extra={"user_id": str(user_id)},
        )


def _new_user_session(user: User, request: Request, *, remember_me: bool) -> tuple[str, Session]:
    session_token = new_session_token()
    session = Session(
        user_id=user.id,
        token_hash=hash_token(session_token),
        expires_at=session_expiry(remember_me),
        last_seen_at=datetime.now(timezone.utc),
        user_agent=request.headers.get("user-agent"),
        ip_address=request_ip(
            request.client.host if request.client else None,
            request.headers.get("x-forwarded-for"),
            request.headers.get("cf-connecting-ip"),
        ),
    )
    return session_token, session


@public_router.post("/login", response_model=AuthResponse, dependencies=[Depends(auth_rate_limit)])
async def login(
    payload: LoginRequest,
    request: Request,
    background_tasks: BackgroundTasks,
    db: AsyncSession = Depends(get_db),
) -> Response:
    started_at = time.perf_counter()
    outcome = "unauthenticated"
    email = payload.email.lower()
    supabase_mode = settings.auth_mode == "supabase" and settings.supabase_auth_enabled

    try:
        if (
            _is_dev_demo_mode_enabled()
            and _is_dev_demo_email(email)
            and hmac.compare_digest(str(payload.password), str(settings.dev_demo_password))
        ):
            user = await _resolve_dev_demo_user(db=db)
            session_token, session = _new_user_session(user, request, remember_me=payload.remember_me)
            db.add(session)
            await db.commit()
            background_tasks.add_task(_refresh_train_reservations_after_sign_in_background, user_id=user.id)
            response = JSONResponse(content=AuthResponse(user=user_to_out(user)).model_dump(mode="json"))
            if background_tasks.tasks:
                response.background = background_tasks
            set_session_cookie(response, session_token, payload.remember_me)
            outcome = "success_dev_demo"
            return response

        result = await db.execute(select(User).options(joinedload(User.role)).where(User.email == email))
        user = result.scalar_one_or_none()
        local_password_ok = False
        if user is not None:
            local_password_ok = await async_verify_password(payload.password, user.password_hash)

        supabase_password_ok = False
        if not local_password_ok and supabase_mode:
            supabase_identity = await verify_supabase_password(email=email, password=payload.password)
            if supabase_identity is not None:
                supabase_password_ok = True
                if user is None:
                    user = await get_or_create_local_user_from_supabase_claims(
                        db,
                        claims={"sub": supabase_identity.user_id, "email": supabase_identity.email},
                    )
                elif user.supabase_user_id != supabase_identity.user_id:
                    user.supabase_user_id = supabase_identity.user_id

        if user is None:
            outcome = "user_missing"
            raise HTTPException(status_code=status.HTTP_401_UNAUTHORIZED, detail=INVALID_LOGIN_DETAIL)
        if not local_password_ok and not supabase_password_ok:
            outcome = "invalid_password"
            raise HTTPException(status_code=status.HTTP_401_UNAUTHORIZED, detail=INVALID_LOGIN_DETAIL)

        # Opportunistic migration: rehash on successful login when policy changed.
        if local_password_ok and await async_password_needs_rehash(user.password_hash):
            user.password_hash = await async_hash_password(payload.password)

        session_token, session = _new_user_session(user, request, remember_me=payload.remember_me)

        db.add(session)
        await db.commit()
        background_tasks.add_task(_refresh_train_reservations_after_sign_in_background, user_id=user.id)

        response = JSONResponse(content=AuthResponse(user=user_to_out(user)).model_dump(mode="json"))
        if background_tasks.tasks:
            response.background = background_tasks
        set_session_cookie(response, session_token, payload.remember_me)
        flow_source = (request.headers.get("x-bominal-flow-source") or "").strip().lower()
        if flow_source in PASSKEY_SETUP_ALLOWED_SOURCES:
            passkey_setup_context_token = await _issue_passkey_setup_context(db, user_id=user.id, source=flow_source)
            _set_passkey_setup_context_cookie(response, passkey_setup_context_token)
        outcome = "success"
        return response
    finally:
        duration_ms = int((time.perf_counter() - started_at) * 1000)
        logger.info("Auth login processed", extra={"outcome": outcome, "duration_ms": duration_ms})


@public_router.get("/methods", response_model=AuthMethodsResponse)
async def auth_methods() -> AuthMethodsResponse:
    return AuthMethodsResponse(
        password=True,
        passkey=settings.passkey_enabled or _is_dev_demo_mode_enabled(),
        magic_link=True,
        otp=_supabase_signin_otp_enabled(),
    )


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
        updates["password_hash"] = await async_hash_password(payload.new_password)

    if not updates and requested_email_change_to is None:
        return AuthResponse(user=user_to_out(current_user))

    sensitive_update = requested_email_change_to is not None or "password_hash" in updates
    if sensitive_update:
        has_valid_password = False
        if payload.current_password:
            has_valid_password = await async_verify_password(
                payload.current_password,
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
    current_user: User = Depends(get_current_user),
) -> MessageResponse:
    if not await async_verify_password(payload.current_password, current_user.password_hash):
        raise HTTPException(status_code=status.HTTP_401_UNAUTHORIZED, detail="Current password is invalid")
    return MessageResponse(message="Password verified")


@user_router.post("/account/email-change/confirm", response_model=AuthResponse, dependencies=[Depends(auth_rate_limit)])
async def confirm_email_change(
    payload: EmailChangeConfirmRequest,
    current_user: User = Depends(get_current_user),
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
    current_user: User = Depends(get_current_user),
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
    current_user: User = Depends(get_current_user),
    db: AsyncSession = Depends(get_db),
) -> PasskeyRegistrationOptionsResponse:
    ensure_passkeys_enabled()
    challenge_id, public_key = await begin_passkey_registration(db, user=current_user)
    return PasskeyRegistrationOptionsResponse(challenge_id=challenge_id, public_key=public_key)


@user_router.post("/passkeys/register/verify", response_model=PasskeyCredentialOut)
async def passkey_register_verify(
    payload: PasskeyRegistrationVerifyRequest,
    current_user: User = Depends(get_current_user),
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
    current_user: User = Depends(get_current_user),
    db: AsyncSession = Depends(get_db),
) -> PasskeyAuthenticationOptionsResponse:
    ensure_passkeys_enabled()
    challenge_id, public_key = await begin_passkey_authentication(db, email=current_user.email, user=current_user)
    return PasskeyAuthenticationOptionsResponse(challenge_id=challenge_id, public_key=public_key)


@user_router.post("/passkeys/step-up/verify", response_model=PasskeyStepUpVerifyResponse)
async def passkey_step_up_verify(
    payload: PasskeyStepUpVerifyRequest,
    current_user: User = Depends(get_current_user),
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
    current_user: User = Depends(get_current_user),
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


@public_router.post("/request-magic-link", response_model=MessageResponse)
async def request_magic_link(
    payload: MagicLinkRequest | None = None,
    db: AsyncSession = Depends(get_db),
) -> MessageResponse:
    if payload is None:
        return MessageResponse(message="If eligible, a sign-in link has been sent")

    email = payload.email.lower()
    supabase_mode = settings.auth_mode == "supabase" and settings.supabase_auth_enabled
    user = (await db.execute(select(User).where(User.email == email))).scalar_one_or_none()
    local_user_found = user is not None
    magic_link_requested = False
    magic_link_ok = False
    local_magic_link_enqueued = False

    if supabase_mode:
        magic_link_requested = True
        redirect_to = f"{_public_base_url()}/auth/verify?type=email"
        magic_link_ok = await send_supabase_magic_link(email=email, redirect_to=redirect_to)
        if not magic_link_ok:
            logger.warning("Supabase magic-link request failed")
    elif user is not None:
        try:
            code, _ = await _issue_verification_token(
                db,
                user_id=user.id,
                purpose=VERIFICATION_PURPOSE_MAGIC_LOGIN,
            )
            enqueued_job_id = await enqueue_template_email(_magic_link_template_payload(email=user.email, code=code))
            if enqueued_job_id is None:
                raise RuntimeError("email_enqueue_returned_none")
            local_magic_link_enqueued = True
        except Exception as exc:
            logger.warning("Failed to queue magic-link email for user %s: %s", user.id, type(exc).__name__)

    logger.info(
        "Auth magic-link requested",
        extra={
            "mode": "supabase" if supabase_mode else "legacy",
            "local_user_found": local_user_found,
            "magic_link_requested": magic_link_requested,
            "magic_link_ok": magic_link_ok,
            "local_magic_link_enqueued": local_magic_link_enqueued,
        },
    )
    return MessageResponse(message="If eligible, a sign-in link has been sent")


@public_router.post("/magic-link/confirm", response_model=AuthResponse, dependencies=[Depends(auth_rate_limit)])
async def magic_link_confirm(
    payload: MagicLinkConfirmRequest,
    request: Request,
    background_tasks: BackgroundTasks,
    db: AsyncSession = Depends(get_db),
) -> Response:
    if settings.auth_mode == "supabase" and settings.supabase_auth_enabled:
        raise HTTPException(status_code=status.HTTP_400_BAD_REQUEST, detail="Use Supabase auth links for sign-in")

    email = payload.email.lower()
    user = (await db.execute(select(User).options(joinedload(User.role)).where(User.email == email))).scalar_one_or_none()
    if user is None:
        raise HTTPException(status_code=status.HTTP_400_BAD_REQUEST, detail="Invalid or expired sign-in link")

    token_hash = hash_token(payload.code)
    now = utc_now()
    token = (
        await db.execute(
            select(VerificationToken)
            .where(VerificationToken.user_id == user.id)
            .where(VerificationToken.purpose == VERIFICATION_PURPOSE_MAGIC_LOGIN)
            .where(VerificationToken.token_hash == token_hash)
            .where(VerificationToken.used_at.is_(None))
            .where(VerificationToken.expires_at > now)
        )
    ).scalar_one_or_none()
    if token is None:
        raise HTTPException(status_code=status.HTTP_400_BAD_REQUEST, detail="Invalid or expired sign-in link")

    token.used_at = now
    session_token, session = _new_user_session(user, request, remember_me=False)
    db.add(session)
    await db.commit()
    background_tasks.add_task(_refresh_train_reservations_after_sign_in_background, user_id=user.id)

    response = JSONResponse(content=AuthResponse(user=user_to_out(user)).model_dump(mode="json"))
    if background_tasks.tasks:
        response.background = background_tasks
    set_session_cookie(response, session_token, False)
    passkey_setup_context_token = await _issue_passkey_setup_context(db, user_id=user.id, source="magiclink")
    _set_passkey_setup_context_cookie(response, passkey_setup_context_token)
    return response


@public_router.post("/request-password-reset", response_model=MessageResponse)
async def request_password_reset(
    payload: PasswordResetRequest | None = None,
    db: AsyncSession = Depends(get_db),
) -> MessageResponse:
    if payload is None:
        return MessageResponse(message="If eligible, a password reset email has been sent")

    email = payload.email.lower()
    supabase_mode = settings.auth_mode == "supabase" and settings.supabase_auth_enabled
    user = (await db.execute(select(User).where(User.email == email))).scalar_one_or_none()
    local_user_found = user is not None
    supabase_recovery_requested = False
    supabase_recovery_ok = False
    local_reset_enqueued = False
    local_reset_direct_fallback = False
    if supabase_mode:
        supabase_recovery_requested = True
        redirect_to = f"{_public_base_url()}/auth/verify?type=recovery"
        supabase_recovery_ok = await send_supabase_password_recovery(email=email, redirect_to=redirect_to)
        if not supabase_recovery_ok:
            logger.warning("Supabase password recovery request failed")
    elif user is not None:
        template_payload: EmailTemplateJobPayload | None = None
        try:
            code, _ = await _issue_password_reset_token(db, user_id=user.id)
            template_payload = _password_reset_template_payload(email=user.email, code=code)
            enqueued_job_id = await enqueue_template_email(template_payload)
            if enqueued_job_id is None:
                raise RuntimeError("email_enqueue_returned_none")
            local_reset_enqueued = True
        except Exception as exc:
            logger.warning("Failed to queue password reset email for user %s: %s", user.id, type(exc).__name__)
            if template_payload is not None:
                try:
                    await deliver_email_job({}, template_payload.model_dump(mode="json"))
                    local_reset_direct_fallback = True
                except Exception as fallback_exc:  # noqa: BLE001
                    logger.warning(
                        "Failed direct password reset delivery fallback for user %s: %s",
                        user.id,
                        type(fallback_exc).__name__,
                    )

    logger.info(
        "Auth password reset requested",
        extra={
            "mode": "supabase" if supabase_mode else "legacy",
            "local_user_found": local_user_found,
            "supabase_recovery_requested": supabase_recovery_requested,
            "supabase_recovery_ok": supabase_recovery_ok,
            "local_reset_enqueued": local_reset_enqueued,
            "local_reset_direct_fallback": local_reset_direct_fallback,
        },
    )

    return MessageResponse(message="If eligible, a password reset email has been sent")


@public_router.post("/request-signin-otp", response_model=MessageResponse)
async def request_signin_otp(payload: SignInOtpRequest | None = None) -> MessageResponse:
    if payload is None:
        return MessageResponse(message="If eligible, a sign-in code has been sent")

    email = payload.email.lower()
    otp_requested = False
    otp_ok = False
    if _supabase_signin_otp_enabled():
        otp_requested = True
        otp_ok = await send_supabase_signin_otp(email=email)
        if not otp_ok:
            logger.warning("Supabase sign-in OTP request failed")

    logger.info(
        "Auth sign-in OTP requested",
        extra={
            "mode": "supabase" if settings.auth_mode == "supabase" else "legacy",
            "otp_requested": otp_requested,
            "otp_ok": otp_ok,
        },
    )
    return MessageResponse(message="If eligible, a sign-in code has been sent")


@public_router.post("/verify-signin-otp", response_model=AuthResponse, dependencies=[Depends(auth_rate_limit)])
async def verify_signin_otp(
    payload: SignInOtpVerifyRequest,
    request: Request,
    background_tasks: BackgroundTasks,
    db: AsyncSession = Depends(get_db),
) -> Response:
    if not _supabase_signin_otp_enabled():
        raise HTTPException(status_code=status.HTTP_404_NOT_FOUND, detail="Sign-in OTP is not enabled")

    email = payload.email.lower()
    identity = await verify_supabase_signin_otp(email=email, code=payload.code)
    if identity is None:
        raise HTTPException(status_code=status.HTTP_400_BAD_REQUEST, detail="Invalid or expired sign-in code")

    user = await get_or_create_local_user_from_supabase_claims(
        db,
        claims={"sub": identity.user_id, "email": identity.email},
    )
    if user.supabase_user_id != identity.user_id:
        user.supabase_user_id = identity.user_id

    session_token, session = _new_user_session(user, request, remember_me=payload.remember_me)
    db.add(session)
    await db.commit()
    background_tasks.add_task(_refresh_train_reservations_after_sign_in_background, user_id=user.id)

    response = JSONResponse(content=AuthResponse(user=user_to_out(user)).model_dump(mode="json"))
    if background_tasks.tasks:
        response.background = background_tasks
    set_session_cookie(response, session_token, payload.remember_me)
    return response


@public_router.post(
    "/supabase/confirm",
    response_model=SupabaseConfirmResponse,
    dependencies=[Depends(auth_rate_limit)],
)
async def supabase_confirm(
    payload: SupabaseConfirmRequest,
    request: Request,
    background_tasks: BackgroundTasks,
    db: AsyncSession = Depends(get_db),
) -> Response:
    confirm_correlation_id = request.headers.get("x-request-id") or secrets.token_hex(8)
    callback_result = await exchange_supabase_token_hash_detailed(token_hash=payload.token_hash, token_type=payload.type)
    callback_session = callback_result.session
    if callback_session is None:
        failure = callback_result.failure
        logger.warning(
            "Supabase confirm failed",
            extra={
                "confirm_correlation_id": confirm_correlation_id,
                "confirm_type": payload.type,
                "failure_category": failure.category if failure else "invalid",
                "failure_status_code": failure.status_code if failure else None,
                "failure_error_code": failure.error_code if failure else None,
            },
        )
        raise HTTPException(
            status_code=status.HTTP_400_BAD_REQUEST,
            detail="Invalid or expired authentication link. Request a fresh link.",
        )

    callback_user_id = callback_session.user_id
    callback_email = callback_session.email
    callback_access_token = callback_session.access_token
    callback_refresh_token = getattr(callback_session, "refresh_token", None)

    if payload.type == "recovery":
        logger.info("Supabase confirm completed", extra={"mode": "recovery"})
        recovery_context_token = await _issue_supabase_recovery_context(
            access_token=callback_access_token,
            refresh_token=callback_refresh_token,
        )
        if not recovery_context_token:
            raise HTTPException(
                status_code=status.HTTP_503_SERVICE_UNAVAILABLE,
                detail="Could not prepare recovery session. Request a fresh link.",
            )
        response_payload = SupabaseConfirmResponse(
            mode="recovery",
            redirect_to=f"{_public_base_url()}/reset-password",
        )
        response = JSONResponse(content=response_payload.model_dump(mode="json"))
        _set_supabase_recovery_context_cookie(response, token=recovery_context_token)
        return response

    user = await get_or_create_local_user_from_supabase_claims(
        db,
        claims={"sub": callback_user_id, "email": callback_email},
    )
    if user.supabase_user_id != callback_user_id:
        user.supabase_user_id = callback_user_id

    session_token, session = _new_user_session(user, request, remember_me=False)
    db.add(session)
    await db.commit()
    background_tasks.add_task(_refresh_train_reservations_after_sign_in_background, user_id=user.id)

    response_payload = SupabaseConfirmResponse(
        mode="magiclink",
        redirect_to="/auth/passkey-setup?source=magiclink&next=/modules/train",
        access_token=callback_access_token,
    )
    response = JSONResponse(content=response_payload.model_dump(mode="json"))
    _clear_supabase_recovery_context_cookie(response)
    if background_tasks.tasks:
        response.background = background_tasks
    set_session_cookie(response, session_token, False)
    passkey_setup_context_token = await _issue_passkey_setup_context(db, user_id=user.id, source="magiclink")
    _set_passkey_setup_context_cookie(response, passkey_setup_context_token)
    logger.info("Supabase confirm completed", extra={"mode": "magiclink", "user_id": str(user.id)})
    return response


@public_router.post(
    "/reset-password/supabase",
    response_model=MessageResponse,
    dependencies=[Depends(auth_rate_limit)],
)
async def reset_password_supabase(
    payload: SupabasePasswordResetConfirmRequest,
    request: Request,
    background_tasks: BackgroundTasks,
    db: AsyncSession = Depends(get_db),
    supabase_recovery_context_cookie: Annotated[str | None, Cookie(alias=SUPABASE_RECOVERY_CONTEXT_COOKIE)] = None,
) -> Response:
    recovery_context_token = str(supabase_recovery_context_cookie or "").strip()
    header_access_token = _extract_bearer_token(request.headers.get("authorization"))
    access_token = str(header_access_token or "").strip()
    refresh_token: str | None = None
    if not access_token:
        context = await _load_supabase_recovery_context(token=recovery_context_token)
        if context is not None:
            access_token, refresh_token = context

    if not access_token:
        return _supabase_recovery_error_response(
            status_code=status.HTTP_401_UNAUTHORIZED,
            detail="Recovery token required",
        )

    password_update_result = await update_supabase_password_detailed(
        access_token=access_token,
        new_password=payload.new_password,
        refresh_token=refresh_token,
    )
    supabase_identity = password_update_result.identity
    if supabase_identity is None:
        if _is_supabase_password_policy_failure(
            status_code=password_update_result.status_code,
            error_message=password_update_result.error_message,
        ):
            detail = password_update_result.error_message or "Password does not meet policy requirements"
            return JSONResponse(status_code=status.HTTP_400_BAD_REQUEST, content={"detail": detail})
        if recovery_context_token:
            await _delete_supabase_recovery_context(token=recovery_context_token)
        return _supabase_recovery_error_response(
            status_code=status.HTTP_400_BAD_REQUEST,
            detail="Invalid or expired recovery link",
        )

    user = await get_or_create_local_user_from_supabase_claims(
        db,
        claims={"sub": supabase_identity.user_id, "email": supabase_identity.email},
    )
    user.password_hash = await async_hash_password(payload.new_password)
    user.supabase_user_id = supabase_identity.user_id

    now = utc_now()
    active_sessions = (
        await db.execute(
            select(Session)
            .where(Session.user_id == user.id)
            .where(Session.revoked_at.is_(None))
            .where(Session.expires_at > now)
        )
    ).scalars().all()
    for auth_session in active_sessions:
        auth_session.revoked_at = now

    session_token, session = _new_user_session(user, request, remember_me=False)
    db.add(session)
    await db.commit()
    background_tasks.add_task(_refresh_train_reservations_after_sign_in_background, user_id=user.id)
    logger.info("Supabase password reset complete", extra={"user_id": str(user.id), "revoked_sessions": len(active_sessions)})
    response = JSONResponse(content=MessageResponse(message="Password reset complete").model_dump(mode="json"))
    if background_tasks.tasks:
        response.background = background_tasks
    set_session_cookie(response, session_token, False)
    if recovery_context_token:
        await _delete_supabase_recovery_context(token=recovery_context_token)
    _clear_supabase_recovery_context_cookie(response)
    passkey_setup_context_token = await _issue_passkey_setup_context(db, user_id=user.id, source="reset")
    _set_passkey_setup_context_cookie(response, passkey_setup_context_token)
    return response


@public_router.post("/reset-password", response_model=MessageResponse)
async def reset_password(
    payload: PasswordResetConfirmRequest,
    request: Request,
    background_tasks: BackgroundTasks,
    db: AsyncSession = Depends(get_db),
) -> Response:
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
    user.password_hash = await async_hash_password(payload.new_password)
    active_sessions = (
        await db.execute(
            select(Session)
            .where(Session.user_id == user.id)
            .where(Session.revoked_at.is_(None))
            .where(Session.expires_at > now)
        )
    ).scalars().all()
    for auth_session in active_sessions:
        auth_session.revoked_at = now
    session_token, session = _new_user_session(user, request, remember_me=False)
    db.add(session)
    await db.commit()
    background_tasks.add_task(_refresh_train_reservations_after_sign_in_background, user_id=user.id)
    response = JSONResponse(content=MessageResponse(message="Password reset complete").model_dump(mode="json"))
    if background_tasks.tasks:
        response.background = background_tasks
    set_session_cookie(response, session_token, False)
    passkey_setup_context_token = await _issue_passkey_setup_context(db, user_id=user.id, source="reset")
    _set_passkey_setup_context_cookie(response, passkey_setup_context_token)
    return response


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
    background_tasks: BackgroundTasks,
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
    background_tasks.add_task(_refresh_train_reservations_after_sign_in_background, user_id=user.id)

    response = JSONResponse(content=AuthResponse(user=user_to_out(user)).model_dump(mode="json"))
    if background_tasks.tasks:
        response.background = background_tasks
    set_session_cookie(response, session_token, payload.remember_me)
    return response


@public_router.post("/passkeys/auth/dev-demo", response_model=AuthResponse, dependencies=[Depends(auth_rate_limit)])
async def passkey_auth_dev_demo(
    payload: DevDemoPasskeySignInRequest,
    request: Request,
    background_tasks: BackgroundTasks,
    db: AsyncSession = Depends(get_db),
) -> Response:
    if not _is_dev_demo_mode_enabled():
        raise HTTPException(status_code=status.HTTP_404_NOT_FOUND, detail="Passkey authentication is not available")
    if not _is_dev_demo_email(payload.email.lower()):
        raise HTTPException(status_code=status.HTTP_401_UNAUTHORIZED, detail="Passkey authentication failed")

    user = await _resolve_dev_demo_user(db=db)
    session_token, session = _new_user_session(user, request, remember_me=payload.remember_me)
    db.add(session)
    await db.commit()
    background_tasks.add_task(_refresh_train_reservations_after_sign_in_background, user_id=user.id)

    response = JSONResponse(content=AuthResponse(user=user_to_out(user)).model_dump(mode="json"))
    if background_tasks.tasks:
        response.background = background_tasks
    set_session_cookie(response, session_token, payload.remember_me)
    return response
