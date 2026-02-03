from __future__ import annotations

from arq import create_pool
from arq.connections import ArqRedis, RedisSettings

from app.core.config import get_settings

settings = get_settings()

_pool: ArqRedis | None = None


def _redis_settings() -> RedisSettings:
    return RedisSettings.from_dsn(settings.redis_url)


async def get_queue_pool() -> ArqRedis:
    global _pool
    if _pool is None:
        _pool = await create_pool(_redis_settings())
    return _pool


async def enqueue_train_task(task_id: str, defer_seconds: float = 0.0) -> None:
    pool = await get_queue_pool()
    if defer_seconds > 0:
        await pool.enqueue_job("run_train_task", task_id, _defer_by=defer_seconds)
    else:
        await pool.enqueue_job("run_train_task", task_id)
