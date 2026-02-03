from datetime import datetime, timezone

from fastapi import Cookie, Depends, HTTPException, Request, status
from sqlalchemy import select
from sqlalchemy.ext.asyncio import AsyncSession
from sqlalchemy.orm import joinedload

from app.core.config import get_settings
from app.core.rate_limit import rate_limiter
from app.core.security import hash_token
from app.db.models import Session, User
from app.db.session import get_db

settings = get_settings()


def _unauthorized() -> HTTPException:
    return HTTPException(status_code=status.HTTP_401_UNAUTHORIZED, detail="Authentication required")


async def auth_rate_limit(request: Request) -> None:
    client_ip = request.client.host if request.client else "unknown"
    key = f"auth:{client_ip}:{request.url.path}"
    rate_limiter.check(
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

    auth_session.last_seen_at = now
    await db.commit()

    return auth_session


async def get_current_user(auth_session: Session = Depends(get_current_session)) -> User:
    return auth_session.user


def require_role(role_name: str):
    async def dependency(user: User = Depends(get_current_user)) -> User:
        if user.role.name != role_name:
            raise HTTPException(status_code=status.HTTP_403_FORBIDDEN, detail="Insufficient permissions")
        return user

    return dependency
