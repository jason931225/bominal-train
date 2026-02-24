from __future__ import annotations

import asyncio
import json
import logging
from datetime import datetime
from uuid import UUID

from app.core.redis import get_redis_client
from app.core.time import utc_now

logger = logging.getLogger(__name__)
TRAIN_TASK_EVENTS_CHANNEL_PREFIX = "train:task-events:user"
EVENT_PUBLISH_TIMEOUT_SECONDS = 0.25
SUPPRESSED_TASK_EVENT_STATES = frozenset({"POLLING", "RUNNING", "RESERVING", "PAYING"})


def task_events_channel(user_id: UUID | str) -> str:
    return f"{TRAIN_TASK_EVENTS_CHANNEL_PREFIX}:{user_id}"


def build_task_state_event_payload(
    *,
    user_id: UUID | str,
    task_id: UUID | str,
    state: str,
    updated_at: datetime | None = None,
) -> dict[str, str]:
    event_time = updated_at or utc_now()
    return {
        "type": "task_state_changed",
        "user_id": str(user_id),
        "task_id": str(task_id),
        "state": state,
        "updated_at": event_time.isoformat(),
    }


def should_publish_task_state_event(*, state: str) -> bool:
    # Suppress high-frequency internal worker states so SSE updates represent
    # user-visible transitions instead of retry-loop churn.
    return state not in SUPPRESSED_TASK_EVENT_STATES


async def publish_task_state_event(
    *,
    user_id: UUID | str,
    task_id: UUID | str,
    state: str,
    updated_at: datetime | None = None,
) -> None:
    if not should_publish_task_state_event(state=state):
        return

    payload = build_task_state_event_payload(
        user_id=user_id,
        task_id=task_id,
        state=state,
        updated_at=updated_at,
    )
    channel = task_events_channel(user_id)
    try:
        redis = await asyncio.wait_for(get_redis_client(), timeout=EVENT_PUBLISH_TIMEOUT_SECONDS)
        await asyncio.wait_for(
            redis.publish(channel, json.dumps(payload, separators=(",", ":"))),
            timeout=EVENT_PUBLISH_TIMEOUT_SECONDS,
        )
    except Exception:
        logger.warning(
            "Failed to publish train task state event",
            extra={"task_id": str(task_id), "state": state},
        )
