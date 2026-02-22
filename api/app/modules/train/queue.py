from __future__ import annotations

from uuid import UUID

from arq import create_pool
from arq.connections import ArqRedis, RedisSettings

from app.core.config import get_settings
from app.core.queue_domains import TRAIN_QUEUE_NAME

settings = get_settings()

_pool: ArqRedis | None = None


def _redis_settings() -> RedisSettings:
    return RedisSettings.from_dsn(settings.resolved_redis_url_non_cde)


async def get_queue_pool() -> ArqRedis:
    global _pool
    if _pool is None:
        _pool = await create_pool(_redis_settings(), default_queue_name=TRAIN_QUEUE_NAME)
    return _pool


async def enqueue_train_task(task_id: str, defer_seconds: float = 0.0) -> None:
    # Queue safety contract: task queues only carry task identifiers.
    try:
        UUID(task_id)
    except Exception as exc:
        raise ValueError("train queue payload must contain a valid task UUID only") from exc

    pool = await get_queue_pool()
    job_id = f"train:{task_id}"
    if defer_seconds > 0:
        await pool.enqueue_job("run_train_task", task_id, _job_id=job_id, _defer_by=defer_seconds)
    else:
        await pool.enqueue_job("run_train_task", task_id, _job_id=job_id)
