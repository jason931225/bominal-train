from __future__ import annotations

from arq import create_pool
from arq.connections import ArqRedis, RedisSettings

from app.core.config import get_settings
from app.core.queue_domains import TRAIN_QUEUE_NAME

settings = get_settings()

_pool: ArqRedis | None = None


def _redis_settings() -> RedisSettings:
    return RedisSettings.from_dsn(settings.redis_url)


async def get_queue_pool() -> ArqRedis:
    global _pool
    if _pool is None:
        _pool = await create_pool(_redis_settings(), default_queue_name=TRAIN_QUEUE_NAME)
    return _pool


def _result_key(job_id: str) -> str:
    return f"arq:result:{job_id}"


async def enqueue_train_task(task_id: str, defer_seconds: float = 0.0) -> bool:
    pool = await get_queue_pool()
    if defer_seconds > 0:
        # Deferred polling retries must use non-deterministic ids so the current
        # running job can schedule the next attempt without self-dedup blocking.
        job = await pool.enqueue_job("run_train_task", task_id, _defer_by=defer_seconds)
        return job is not None

    job_id = f"train:{task_id}"
    await pool.delete(_result_key(job_id))
    job = await pool.enqueue_job("run_train_task", task_id, _job_id=job_id)
    return job is not None
