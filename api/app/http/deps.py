import hmac
from datetime import datetime, timezone

from fastapi import Cookie, Depends, Header, HTTPException, Request, status
from sqlalchemy import select
from sqlalchemy.ext.asyncio import AsyncSession
from sqlalchemy.orm import joinedload

from app.core.config import get_settings
from app.core.rate_limit import rate_limiter
from app.core.internal_identity import InternalIdentityError, verify_internal_service_token
from app.core.security import hash_token
from app.core.supabase_jwt import SupabaseJWTError, verify_supabase_jwt
from app.db.models import Session, User
from app.db.session import get_db
from app.services.auth import request_ip, should_update_session_activity
from app.services.identity import get_or_create_local_user_from_supabase_claims

settings = get_settings()
ACCESS_REVIEW_PENDING_DETAIL = "Application is under review"
PAYMENT_DISABLED_DETAIL = "Payment features are currently disabled"


def _unauthorized() -> HTTPException:
    return HTTPException(status_code=status.HTTP_401_UNAUTHORIZED, detail="Authentication required")


def _forbidden(detail: str = "Insufficient permissions") -> HTTPException:
    return HTTPException(status_code=status.HTTP_403_FORBIDDEN, detail=detail)


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


async def auth_rate_limit(request: Request) -> None:
    client_ip = request_ip(
        request.client.host if request.client else None,
        request.headers.get("x-forwarded-for"),
        request.headers.get("cf-connecting-ip"),
    ) or "unknown"
    key = f"auth:{client_ip}:{request.url.path}"
    await rate_limiter.check(
        key=key,
        limit=settings.rate_limit_max_requests,
        window_seconds=settings.rate_limit_window_seconds,
    )


async def get_current_session(
    session_token: str | None = Cookie(default=None, alias=settings.session_cookie_name),
    db: AsyncSession = Depends(get_db),
) -> Session:
    auth_session = await _resolve_session_from_cookie(session_token=session_token, db=db)
    if auth_session is None:
        raise _unauthorized()
    return auth_session


async def _resolve_session_from_cookie(*, session_token: str | None, db: AsyncSession) -> Session | None:
    if not session_token:
        return None

    now = datetime.now(timezone.utc)
    token_hash = hash_token(session_token)

    stmt = (
        select(Session)
        .options(joinedload(Session.user).joinedload(User.role))
        .where(Session.token_hash == token_hash)
        .where(Session.revoked_at.is_(None))
        .where(Session.expires_at > now)
    )
    result = await db.execute(stmt)
    auth_session = result.scalar_one_or_none()
    if auth_session is None:
        return None

    if should_update_session_activity(
        last_seen_at=auth_session.last_seen_at,
        now=now,
        debounce_seconds=settings.session_activity_debounce_seconds,
    ):
        auth_session.last_seen_at = now
        await db.commit()

    return auth_session


async def _resolve_user_from_supabase_bearer(*, bearer_token: str, db: AsyncSession) -> User:
    try:
        claims = verify_supabase_jwt(bearer_token)
    except SupabaseJWTError as exc:
        raise _unauthorized() from exc

    try:
        return await get_or_create_local_user_from_supabase_claims(db, claims=claims)
    except ValueError as exc:
        raise _unauthorized() from exc


async def get_current_user(
    request: Request,
    session_token: str | None = Cookie(default=None, alias=settings.session_cookie_name),
    db: AsyncSession = Depends(get_db),
) -> User:
    auth_mode = settings.auth_mode
    bearer_token = _extract_bearer_token(request.headers.get("authorization"))

    if auth_mode == "supabase":
        if bearer_token:
            return await _resolve_user_from_supabase_bearer(bearer_token=bearer_token, db=db)

        # Compatibility fallback for current web session-cookie auth flow.
        # Bearer remains primary when provided (including invalid bearer -> 401).
        auth_session = await _resolve_session_from_cookie(session_token=session_token, db=db)
        if auth_session is None:
            raise _unauthorized()
        return auth_session.user

    auth_session = await _resolve_session_from_cookie(session_token=session_token, db=db)
    if auth_session is None:
        raise _unauthorized()
    return auth_session.user


def _is_access_approved(user: User) -> bool:
    if not settings.access_approval_required:
        return True
    return str(user.access_status).lower() == "approved"


async def get_current_approved_user(user: User = Depends(get_current_user)) -> User:
    if not _is_access_approved(user):
        raise HTTPException(status_code=status.HTTP_403_FORBIDDEN, detail=ACCESS_REVIEW_PENDING_DETAIL)
    return user


async def get_current_admin(user: User = Depends(get_current_approved_user)) -> User:
    if user.role.name != "admin":
        raise _forbidden()
    return user


async def require_internal_access(
    internal_api_key: str | None = Header(default=None, alias="X-Internal-Api-Key"),
    internal_service_token: str | None = Header(default=None, alias="X-Internal-Service-Token"),
) -> None:
    api_key = internal_api_key if isinstance(internal_api_key, str) else None
    service_token = internal_service_token if isinstance(internal_service_token, str) else None

    if service_token:
        try:
            verify_internal_service_token(service_token, expected_audience="internal-api")
        except InternalIdentityError as exc:
            raise _forbidden("Internal API access denied") from exc
        return

    configured_key = settings.internal_api_key
    if configured_key:
        if not api_key or not hmac.compare_digest(api_key, configured_key):
            raise _forbidden("Internal API access denied")
        return

    raise HTTPException(
        status_code=status.HTTP_503_SERVICE_UNAVAILABLE,
        detail="Internal API access is not configured",
    )


def require_role(role_name: str):
    async def dependency(user: User = Depends(get_current_user)) -> User:
        if user.role.name != role_name:
            raise _forbidden()
        return user

    return dependency


async def require_payment_enabled() -> None:
    if not settings.payment_enabled:
        raise HTTPException(status_code=status.HTTP_403_FORBIDDEN, detail=PAYMENT_DISABLED_DETAIL)
