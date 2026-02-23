import os
from contextlib import asynccontextmanager
from pathlib import Path
from typing import Any
from typing import Mapping

import fakeredis.aioredis
import pytest_asyncio
from httpx import ASGITransport, AsyncClient
from sqlalchemy.ext.asyncio import async_sessionmaker, create_async_engine

os.environ["TRAIN_PROVIDER_MODE"] = "mock"
os.environ["RATE_LIMIT_MAX_REQUESTS"] = "1000000"

from app.core.config import get_settings
get_settings.cache_clear()

from app.db.models import Base, Role
from app.db.session import get_db
from app.main import app


def _format_cookie_header(cookies: Any) -> str:
    if isinstance(cookies, Mapping):
        items = cookies.items()
    else:
        try:
            items = dict(cookies).items()
        except Exception:
            items = []
    return "; ".join(f"{key}={value}" for key, value in items)


class CompatAsyncClient(AsyncClient):
    """httpx client shim for tests.

    httpx is deprecating per-request `cookies=`. Tests still use that call style heavily,
    so this shim converts request cookies into a `cookie` header without mutating client state.
    """

    async def request(  # noqa: D401
        self,
        method: str,
        url: str,
        *,
        cookies: Any | None = None,
        headers: Mapping[str, str] | None = None,
        **kwargs: Any,
    ):
        resolved_headers = dict(headers or {})
        if cookies is not None and not any(key.lower() == "cookie" for key in resolved_headers):
            cookie_header = _format_cookie_header(cookies)
            if cookie_header:
                resolved_headers["cookie"] = cookie_header
        return await super().request(method, url, headers=resolved_headers, **kwargs)


@pytest_asyncio.fixture
async def db_session_factory(tmp_path: Path):
    db_path = tmp_path / "test.db"
    engine = create_async_engine(f"sqlite+aiosqlite:///{db_path}", future=True)
    session_factory = async_sessionmaker(engine, expire_on_commit=False)

    async with engine.begin() as conn:
        await conn.run_sync(Base.metadata.create_all)

    async with session_factory() as session:
        session.add_all([Role(id=1, name="admin"), Role(id=2, name="user")])
        await session.commit()

    yield session_factory

    async with engine.begin() as conn:
        await conn.run_sync(Base.metadata.drop_all)

    await engine.dispose()
    if os.path.exists(db_path):
        os.remove(db_path)


@pytest_asyncio.fixture
async def db_session(db_session_factory):
    async with db_session_factory() as session:
        yield session


@pytest_asyncio.fixture
async def client(db_session_factory):

    async def override_get_db():
        async with db_session_factory() as session:
            yield session

    app.dependency_overrides[get_db] = override_get_db

    async with CompatAsyncClient(transport=ASGITransport(app=app), base_url="http://testserver") as async_client:
        yield async_client

    app.dependency_overrides.clear()


# --- Shared Redis test helpers ---


class MockRedisContextManager:
    """Async context manager wrapper for fakeredis in tests.
    
    Used to mock get_cde_redis_pool()/get_redis_pool() context manager helpers.
    
    Usage:
        fake_redis = fakeredis.aioredis.FakeRedis()
        monkeypatch.setattr("app.services.wallet.get_cde_redis_pool",
                            lambda: MockRedisContextManager(fake_redis))
    """
    def __init__(self, redis):
        self._redis = redis

    async def __aenter__(self):
        return self._redis

    async def __aexit__(self, *_):
        pass


def make_fake_get_redis_client(fake_redis):
    """Create a fake get_redis_client that returns the provided fake redis instance.
    
    Used to mock get_redis_client() which returns a Redis client directly.
    
    Usage:
        fake_redis = fakeredis.aioredis.FakeRedis()
        monkeypatch.setattr("app.modules.train.service.get_redis_client", 
                            make_fake_get_redis_client(fake_redis))
    """
    async def _get_fake_redis():
        return fake_redis
    return _get_fake_redis


@asynccontextmanager
async def fake_redis_pool():
    """Fixture-style async context manager for fake redis.
    
    Usage:
        fake_redis = fakeredis.aioredis.FakeRedis()
        monkeypatch.setattr("app.services.wallet.get_cde_redis_pool",
                            lambda: fake_redis_pool_wrapper(fake_redis))
    """
    redis = fakeredis.aioredis.FakeRedis()
    yield redis
