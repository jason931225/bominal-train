import hmac
from datetime import datetime, timezone

from fastapi import Cookie, Depends, Header, HTTPException, Request, status
from sqlalchemy import select
from sqlalchemy.ext.asyncio import AsyncSession
from sqlalchemy.orm import joinedload

from app.core.config import get_settings
from app.core.rate_limit import rate_limiter
from app.core.security import hash_token
from app.db.models import Session, User
from app.db.session import get_db
from app.services.auth import request_ip, should_update_session_activity

settings = get_settings()


def _unauthorized() -> HTTPException:
    return HTTPException(status_code=status.HTTP_401_UNAUTHORIZED, detail="Authentication required")


def _forbidden(detail: str = "Insufficient permissions") -> HTTPException:
    return HTTPException(status_code=status.HTTP_403_FORBIDDEN, detail=detail)


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
    if not session_token:
        raise _unauthorized()

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

    if not auth_session:
        raise _unauthorized()

    if should_update_session_activity(
        last_seen_at=auth_session.last_seen_at,
        now=now,
        debounce_seconds=settings.session_activity_debounce_seconds,
    ):
        auth_session.last_seen_at = now
        await db.commit()

    return auth_session


async def get_current_user(auth_session: Session = Depends(get_current_session)) -> User:
    return auth_session.user


async def get_current_admin(user: User = Depends(get_current_user)) -> User:
    if user.role.name != "admin":
        raise _forbidden()
    return user


async def require_internal_access(
    internal_api_key: str | None = Header(default=None, alias="X-Internal-Api-Key"),
) -> None:
    configured_key = settings.internal_api_key
    if not configured_key:
        raise HTTPException(
            status_code=status.HTTP_503_SERVICE_UNAVAILABLE,
            detail="Internal API access is not configured",
        )

    if not internal_api_key or not hmac.compare_digest(internal_api_key, configured_key):
        raise _forbidden("Internal API access denied")


def require_role(role_name: str):
    async def dependency(user: User = Depends(get_current_user)) -> User:
        if user.role.name != role_name:
            raise _forbidden()
        return user

    return dependency
