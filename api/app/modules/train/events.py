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


def build_task_ticket_status_event_payload(
    *,
    user_id: UUID | str,
    task_id: UUID | str,
    state: str,
    ticket_status: str | None,
    previous_ticket_status: str | None,
    updated_at: datetime | None = None,
) -> dict[str, str]:
    event_time = updated_at or utc_now()
    return {
        "type": "task_ticket_status_changed",
        "user_id": str(user_id),
        "task_id": str(task_id),
        "state": state,
        "ticket_status": str(ticket_status or ""),
        "previous_ticket_status": str(previous_ticket_status or ""),
        "updated_at": event_time.isoformat(),
    }


def should_publish_task_state_event(*, state: str) -> bool:
    # Suppress high-frequency internal worker states to keep SSE traffic and
    # downstream dashboard refresh load bounded in production.
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


async def publish_task_ticket_status_event(
    *,
    user_id: UUID | str,
    task_id: UUID | str,
    state: str,
    ticket_status: str | None,
    previous_ticket_status: str | None,
    updated_at: datetime | None = None,
) -> None:
    payload = build_task_ticket_status_event_payload(
        user_id=user_id,
        task_id=task_id,
        state=state,
        ticket_status=ticket_status,
        previous_ticket_status=previous_ticket_status,
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
            "Failed to publish train task ticket-status event",
            extra={"task_id": str(task_id), "state": state},
        )
