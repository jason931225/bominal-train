from __future__ import annotations

import asyncio

import pytest
from sqlalchemy import select


@pytest.mark.asyncio
async def test_on_shutdown_recovers_when_heartbeat_cancelled(monkeypatch):
    from app import worker as worker_mod

    recovered = {"called": False}

    async def _fake_recover() -> int:
        recovered["called"] = True
        return 1

    monkeypatch.setattr(worker_mod, "_recover_in_flight_tasks", _fake_recover)
    worker_mod._in_flight_tasks.clear()

    heartbeat_task = asyncio.create_task(asyncio.sleep(60))
    await worker_mod.on_shutdown({"heartbeat_task": heartbeat_task})

    assert recovered["called"] is True


@pytest.mark.asyncio
async def test_recover_in_flight_skips_hidden_cancelled_and_paused(db_session, monkeypatch):
    from datetime import datetime, timedelta, timezone
    from uuid import uuid4

    from app import worker as worker_mod
    from app.db.models import Task
    from app.modules.train.constants import TASK_MODULE

    enqueued: list[str] = []

    async def _fake_enqueue(task_id: str, defer_seconds: float = 0.0) -> None:
        enqueued.append(task_id)

    monkeypatch.setattr(worker_mod, "enqueue_train_task", _fake_enqueue)

    class _SessionCtx:
        def __init__(self, session):
            self._session = session

        async def __aenter__(self):
            return self._session

        async def __aexit__(self, exc_type, exc, tb):
            return False

    monkeypatch.setattr(worker_mod, "SessionLocal", lambda: _SessionCtx(db_session))

    now = datetime.now(timezone.utc)

    hidden_id = uuid4()
    cancelled_id = uuid4()
    paused_id = uuid4()
    ok_id = uuid4()

    db_session.add_all(
        [
            Task(
                id=hidden_id,
                user_id=uuid4(),
                module=TASK_MODULE,
                state="RUNNING",
                deadline_at=now + timedelta(hours=1),
                spec_json={"provider": "SRT"},
                idempotency_key="hidden",
                hidden_at=now,
            ),
            Task(
                id=cancelled_id,
                user_id=uuid4(),
                module=TASK_MODULE,
                state="RUNNING",
                deadline_at=now + timedelta(hours=1),
                spec_json={"provider": "SRT"},
                idempotency_key="cancelled",
                cancelled_at=now,
            ),
            Task(
                id=paused_id,
                user_id=uuid4(),
                module=TASK_MODULE,
                state="PAUSED",
                deadline_at=now + timedelta(hours=1),
                spec_json={"provider": "SRT"},
                idempotency_key="paused",
                paused_at=now,
            ),
            Task(
                id=ok_id,
                user_id=uuid4(),
                module=TASK_MODULE,
                state="RUNNING",
                deadline_at=now + timedelta(hours=1),
                spec_json={"provider": "SRT"},
                idempotency_key="ok",
            ),
        ]
    )
    await db_session.commit()

    worker_mod._in_flight_tasks.clear()
    worker_mod._in_flight_tasks.update(
        [str(hidden_id), str(cancelled_id), str(paused_id), str(ok_id)]
    )

    recovered = await worker_mod._recover_in_flight_tasks()

    assert recovered == 1
    assert enqueued == [str(ok_id)]

    refreshed_ok = (
        await db_session.execute(select(Task).where(Task.id == ok_id))
    ).scalar_one()
    assert refreshed_ok.state == "QUEUED"
