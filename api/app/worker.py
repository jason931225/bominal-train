from __future__ import annotations

import logging

from arq.connections import RedisSettings

from app.core.config import get_settings
from app.db.session import SessionLocal
from app.modules.train.worker import enqueue_recoverable_tasks, run_train_task
from app.services.email_worker import deliver_email_job

settings = get_settings()
logger = logging.getLogger(__name__)


async def on_startup(ctx: dict) -> None:
    ctx["db_factory"] = SessionLocal
    async with SessionLocal() as db:
        recovered = await enqueue_recoverable_tasks(db)
    logger.info("Recovered %s active train tasks into queue", recovered)


class WorkerSettings:
    functions = [run_train_task, deliver_email_job]
    redis_settings = RedisSettings.from_dsn(settings.redis_url)
    on_startup = on_startup
    max_jobs = 20
    job_timeout = 300
