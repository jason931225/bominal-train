from collections.abc import AsyncGenerator

from sqlalchemy.ext.asyncio import AsyncSession, async_sessionmaker, create_async_engine

from app.core.config import get_settings

settings = get_settings()


def build_async_engine_options(cfg) -> dict[str, object]:
    connect_args: dict[str, object] = {
        "timeout": cfg.db_connect_timeout_seconds,
        "command_timeout": cfg.db_command_timeout_seconds,
    }
    if cfg.db_statement_timeout_ms > 0:
        connect_args["server_settings"] = {"statement_timeout": str(cfg.db_statement_timeout_ms)}

    return {
        "future": True,
        "pool_pre_ping": cfg.db_pool_pre_ping,
        "pool_size": cfg.db_pool_size,
        "max_overflow": cfg.db_max_overflow,
        "pool_timeout": cfg.db_pool_timeout_seconds,
        "pool_recycle": cfg.db_pool_recycle_seconds,
        "pool_use_lifo": cfg.db_pool_use_lifo,
        "connect_args": connect_args,
    }


engine = create_async_engine(settings.resolved_database_url_async_active, **build_async_engine_options(settings))
SessionLocal = async_sessionmaker(engine, class_=AsyncSession, expire_on_commit=False)


async def get_db() -> AsyncGenerator[AsyncSession, None]:
    async with SessionLocal() as session:
        yield session
