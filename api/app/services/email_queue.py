from __future__ import annotations

from arq import create_pool
from arq.connections import ArqRedis, RedisSettings

from app.core.config import get_settings
from app.core.queue_domains import TRAIN_QUEUE_NAME
from app.schemas.notification import EmailJobPayload, EmailTemplateJobPayload

settings = get_settings()

_pool: ArqRedis | None = None


def _redis_settings() -> RedisSettings:
    return RedisSettings.from_dsn(settings.redis_url)


async def get_email_queue_pool() -> ArqRedis:
    global _pool
    if _pool is None:
        _pool = await create_pool(_redis_settings(), default_queue_name=TRAIN_QUEUE_NAME)
    return _pool


async def enqueue_email(payload: EmailJobPayload, *, defer_seconds: float = 0.0) -> str | None:
    pool = await get_email_queue_pool()
    if defer_seconds > 0:
        job = await pool.enqueue_job(
            "deliver_email_job",
            payload.model_dump(mode="json"),
            _defer_by=defer_seconds,
        )
    else:
        job = await pool.enqueue_job("deliver_email_job", payload.model_dump(mode="json"))

    if job is None:
        return None
    return job.job_id


async def enqueue_template_email(payload: EmailTemplateJobPayload, *, defer_seconds: float = 0.0) -> str | None:
    pool = await get_email_queue_pool()
    if defer_seconds > 0:
        job = await pool.enqueue_job(
            "deliver_email_job",
            payload.model_dump(mode="json"),
            _defer_by=defer_seconds,
        )
    else:
        job = await pool.enqueue_job("deliver_email_job", payload.model_dump(mode="json"))

    if job is None:
        return None
    return job.job_id
