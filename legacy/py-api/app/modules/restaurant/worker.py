from __future__ import annotations

import logging
from uuid import UUID

from app.core.config import get_settings
from app.core.crypto.safe_metadata import validate_safe_metadata
from app.core.redis import get_redis_client
from app.core.time import utc_now
from app.db.models import Task, TaskAttempt
from app.db.session import SessionLocal
from app.modules.restaurant.lease import acquire_payment_lease, release_payment_lease
from app.modules.restaurant.policy import (
    build_payment_lease_key,
    is_non_committing_phase,
    resolve_auth_fallback_step,
)
from app.modules.restaurant.types import RestaurantAuthStep

logger = logging.getLogger(__name__)
settings = get_settings()


async def _append_attempt(
    *,
    db,
    task: Task,
    action: str,
    provider: str,
    ok: bool,
    retryable: bool,
    error_code: str | None,
    error_message_safe: str | None,
    meta_json_safe: dict | None,
) -> None:
    now = utc_now()
    db.add(
        TaskAttempt(
            task_id=task.id,
            action=action[:16],
            provider=provider[:8],
            ok=ok,
            retryable=retryable,
            error_code=error_code,
            error_message_safe=error_message_safe,
            duration_ms=0,
            meta_json_safe=validate_safe_metadata(meta_json_safe) if meta_json_safe else None,
            started_at=now,
            finished_at=now,
        )
    )
    task.updated_at = now


async def run_restaurant_task(ctx: dict, task_id: str) -> None:
    """Restaurant worker policy scaffold.

    Stage 4 enforces auth fallback and payment lease policies with safe metadata.
    Business/provider execution is still intentionally minimal at this stage.
    """
    try:
        task_uuid = UUID(task_id)
    except ValueError:
        logger.warning("Restaurant worker received invalid task id: %s", task_id)
        return

    async with SessionLocal() as db:
        task = await db.get(Task, task_uuid)
        if task is None:
            logger.warning("Restaurant task %s not found", task_id)
            return
        if task.module != "restaurant":
            logger.warning("Task %s is not a restaurant task", task_id)
            return
        if task.hidden_at is not None or task.cancelled_at is not None:
            logger.info("Skipping hidden/cancelled restaurant task %s", task_id)
            return

        spec = dict(task.spec_json or {})
        provider = str(spec.get("provider") or "RESY")
        phase = str(spec.get("phase") or "search")
        auth_ok = bool(spec.get("auth_ok") or False)

        if not auth_ok:
            refresh_attempts = int(spec.get("auth_refresh_attempts") or 0)
            bootstrap_attempted = bool(spec.get("auth_bootstrap_attempted") or False)
            auth_step = resolve_auth_fallback_step(
                refresh_attempts=refresh_attempts,
                bootstrap_attempted=bootstrap_attempted,
                max_refresh_retries=settings.restaurant_auth_refresh_retries,
            )

            if auth_step == RestaurantAuthStep.REFRESH:
                spec["auth_refresh_attempts"] = refresh_attempts + 1
                task.spec_json = spec
                task.state = "POLLING"
                await _append_attempt(
                    db=db,
                    task=task,
                    action="AUTH",
                    provider=provider,
                    ok=False,
                    retryable=True,
                    error_code="auth_refresh_retry",
                    error_message_safe="Restaurant auth refresh retry scheduled.",
                    meta_json_safe={"auth_step": auth_step.value, "phase": phase},
                )
                await db.commit()
                return

            if auth_step == RestaurantAuthStep.BOOTSTRAP:
                spec["auth_bootstrap_attempted"] = True
                task.spec_json = spec
                task.state = "POLLING"
                await _append_attempt(
                    db=db,
                    task=task,
                    action="AUTH",
                    provider=provider,
                    ok=False,
                    retryable=True,
                    error_code="auth_bootstrap_required",
                    error_message_safe="Restaurant auth bootstrap required.",
                    meta_json_safe={
                        "auth_step": auth_step.value,
                        "phase": phase,
                        "bootstrap_timeout_seconds": settings.restaurant_bootstrap_timeout_seconds,
                    },
                )
                await db.commit()
                return

            task.state = "FAILED"
            task.failed_at = utc_now()
            await _append_attempt(
                db=db,
                task=task,
                action="AUTH",
                provider=provider,
                ok=False,
                retryable=False,
                error_code="auth_failed",
                error_message_safe="Restaurant authentication failed after fallback attempts.",
                meta_json_safe={"auth_step": auth_step.value, "phase": phase},
            )
            await db.commit()
            return

        if is_non_committing_phase(phase):
            task.state = "COMPLETED"
            task.completed_at = utc_now()
            await _append_attempt(
                db=db,
                task=task,
                action=phase.upper(),
                provider=provider,
                ok=True,
                retryable=False,
                error_code=None,
                error_message_safe=None,
                meta_json_safe={"phase": phase, "phase_type": "non_committing"},
            )
            await db.commit()
            return

        account_ref = str(spec.get("account_ref") or task.user_id)
        restaurant_id = str(spec.get("restaurant_id") or "unknown")
        lease_key = build_payment_lease_key(
            provider=provider,
            account_ref=account_ref,
            restaurant_id=restaurant_id,
        )
        holder_token = f"task:{task.id}"

        redis = await get_redis_client()
        acquired = await acquire_payment_lease(
            redis,
            lease_key=lease_key,
            holder_token=holder_token,
            ttl_seconds=settings.restaurant_payment_lease_ttl_seconds,
        )
        if not acquired:
            task.state = "POLLING"
            await _append_attempt(
                db=db,
                task=task,
                action="PAY",
                provider=provider,
                ok=False,
                retryable=True,
                error_code="payment_lease_busy",
                error_message_safe="Payment step is currently leased by another task.",
                meta_json_safe={"lease_key": lease_key, "phase": phase},
            )
            await db.commit()
            return

        try:
            task.state = "COMPLETED"
            task.completed_at = utc_now()
            await _append_attempt(
                db=db,
                task=task,
                action="PAY",
                provider=provider,
                ok=True,
                retryable=False,
                error_code=None,
                error_message_safe=None,
                meta_json_safe={"lease_key": lease_key, "phase": phase},
            )
            await db.commit()
        finally:
            await release_payment_lease(redis, lease_key=lease_key, holder_token=holder_token)
