from __future__ import annotations

from arq.connections import RedisSettings

from app.core.config import get_settings
from app.core.queue_domains import RESTAURANT_QUEUE_NAME
from app.modules.restaurant.worker import run_restaurant_task

settings = get_settings()


class WorkerRestaurantSettings:
    functions = [run_restaurant_task]
    redis_settings = RedisSettings.from_dsn(settings.resolved_redis_url_non_cde)
    queue_name = RESTAURANT_QUEUE_NAME
    max_jobs = 10
    job_timeout = 300
    health_check_interval = 10
    max_tries = 1
