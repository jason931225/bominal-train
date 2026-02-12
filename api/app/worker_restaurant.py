from __future__ import annotations

from arq.connections import RedisSettings

from app.core.config import get_settings
from app.modules.restaurant.worker import run_restaurant_task

settings = get_settings()


class WorkerRestaurantSettings:
    functions = [run_restaurant_task]
    redis_settings = RedisSettings.from_dsn(settings.redis_url)
    queue_name = "restaurant:queue"
    max_jobs = 10
    job_timeout = 300
    health_check_interval = 10
    max_tries = 1

