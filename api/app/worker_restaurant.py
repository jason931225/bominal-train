from __future__ import annotations

from dataclasses import replace

from arq.connections import RedisSettings

from app.core.config import get_settings
from app.core.queue_domains import RESTAURANT_QUEUE_NAME
from app.modules.restaurant.worker import run_restaurant_task

settings = get_settings()
WORKER_REDIS_CONN_TIMEOUT_SECONDS = 5
WORKER_REDIS_CONN_RETRIES = 8
WORKER_REDIS_CONN_RETRY_DELAY_SECONDS = 1
WORKER_REDIS_MAX_CONNECTIONS = 100


def _worker_redis_settings() -> RedisSettings:
    base = RedisSettings.from_dsn(settings.resolved_redis_url_non_cde)
    return replace(
        base,
        conn_timeout=WORKER_REDIS_CONN_TIMEOUT_SECONDS,
        conn_retries=WORKER_REDIS_CONN_RETRIES,
        conn_retry_delay=WORKER_REDIS_CONN_RETRY_DELAY_SECONDS,
        max_connections=WORKER_REDIS_MAX_CONNECTIONS,
        retry_on_timeout=True,
    )


class WorkerRestaurantSettings:
    functions = [run_restaurant_task]
    redis_settings = _worker_redis_settings()
    queue_name = RESTAURANT_QUEUE_NAME
    max_jobs = 10
    job_timeout = 300
    health_check_interval = 10
    max_tries = 1
