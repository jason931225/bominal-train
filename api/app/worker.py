from __future__ import annotations

import asyncio
import logging
import signal
from contextlib import suppress
from datetime import datetime, timezone
from typing import Any
from uuid import UUID

from arq.connections import RedisSettings
from sqlalchemy import select

from app.core.config import get_settings
from app.db.models import Task
from app.db.session import SessionLocal
from app.modules.train.constants import TASK_MODULE, TERMINAL_TASK_STATES
from app.modules.train.queue import enqueue_train_task
from app.modules.train.worker import enqueue_recoverable_tasks, run_train_task
from app.services.email_worker import deliver_email_job

settings = get_settings()
logger = logging.getLogger(__name__)

# Track in-flight task IDs for graceful shutdown recovery
_in_flight_tasks: set[str] = set()
_shutdown_event: asyncio.Event | None = None


def _register_in_flight(task_id: str) -> None:
    """Register a task as currently being processed."""
    _in_flight_tasks.add(task_id)


def _unregister_in_flight(task_id: str) -> None:
    """Remove a task from in-flight tracking."""
    _in_flight_tasks.discard(task_id)


async def _recover_in_flight_tasks() -> int:
    """Re-enqueue tasks that were in-flight when worker shut down."""
    if not _in_flight_tasks:
        return 0
    
    count = 0
    async with SessionLocal() as db:
        for task_id in list(_in_flight_tasks):
            try:
                task = await db.get(Task, UUID(task_id))
                if task and task.module == TASK_MODULE and task.state not in TERMINAL_TASK_STATES:
                    # Skip deleted, cancelled, or paused tasks
                    if task.hidden_at is not None:
                        logger.debug("Skipping hidden/deleted in-flight task %s", task_id)
                        continue
                    if task.cancelled_at is not None:
                        logger.debug("Skipping cancelled in-flight task %s", task_id)
                        continue
                    if task.paused_at is not None or task.state == "PAUSED":
                        logger.debug("Skipping paused in-flight task %s", task_id)
                        continue
                    
                    # Reset to QUEUED for clean re-processing
                    if task.state not in ("PAUSED", "QUEUED"):
                        task.state = "QUEUED"
                        task.updated_at = datetime.now(timezone.utc)
                        await db.commit()
                    await enqueue_train_task(task_id, defer_seconds=2.0)
                    count += 1
                    logger.info("Re-queued in-flight task %s after shutdown", task_id)
            except Exception as e:
                logger.warning("Failed to recover in-flight task %s: %s", task_id, e)
            finally:
                _in_flight_tasks.discard(task_id)
    return count


async def on_startup(ctx: dict) -> None:
    """Initialize worker context and recover any pending tasks."""
    global _shutdown_event
    _shutdown_event = asyncio.Event()
    
    ctx["db_factory"] = SessionLocal
    ctx["shutdown_event"] = _shutdown_event
    ctx["register_in_flight"] = _register_in_flight
    ctx["unregister_in_flight"] = _unregister_in_flight
    
    # Recover tasks from previous worker run
    async with SessionLocal() as db:
        recovered = await enqueue_recoverable_tasks(db)
    logger.info("Recovered %s active train tasks into queue", recovered)
    
    # Set up graceful shutdown signal handlers
    loop = asyncio.get_running_loop()
    for sig in (signal.SIGTERM, signal.SIGINT):
        loop.add_signal_handler(sig, _handle_shutdown_signal, sig)
    
    logger.info("Worker started with graceful shutdown handlers")


def _handle_shutdown_signal(sig: signal.Signals) -> None:
    """Handle shutdown signals gracefully."""
    logger.info("Received signal %s, initiating graceful shutdown...", sig.name)
    if _shutdown_event:
        _shutdown_event.set()


async def on_shutdown(ctx: dict) -> None:
    """Clean up on worker shutdown - ensure in-flight tasks are recoverable."""
    logger.info("Worker shutdown initiated, %d tasks in-flight", len(_in_flight_tasks))
    
    # Give in-flight tasks a brief moment to complete naturally
    if _in_flight_tasks:
        await asyncio.sleep(0.5)
    
    # Re-queue any remaining in-flight tasks for recovery on next startup
    recovered = await _recover_in_flight_tasks()
    if recovered:
        logger.info("Re-queued %d in-flight tasks for recovery", recovered)
    
    logger.info("Worker shutdown complete")


async def on_job_start(ctx: dict, job: Any = None) -> None:
    """Called when a job starts - track it for graceful shutdown."""
    # Extract task_id from job arguments if it's a train task
    if hasattr(job, 'args') and job.args and job.function == 'run_train_task':
        task_id = job.args[0] if job.args else None
        if task_id:
            _register_in_flight(task_id)


async def on_job_end(ctx: dict, job: Any, result: Any) -> None:
    """Called when a job ends - remove from tracking."""
    if hasattr(job, 'args') and job.args and job.function == 'run_train_task':
        task_id = job.args[0] if job.args else None
        if task_id:
            _unregister_in_flight(task_id)


class WorkerSettings:
    functions = [run_train_task, deliver_email_job]
    redis_settings = RedisSettings.from_dsn(settings.redis_url)
    on_startup = on_startup
    on_shutdown = on_shutdown
    on_job_start = on_job_start
    on_job_end = on_job_end
    max_jobs = 20
    job_timeout = 300
    # Allow time for graceful shutdown
    health_check_interval = 10
    # Don't retry failed jobs automatically - we handle retry logic ourselves
    max_tries = 1
