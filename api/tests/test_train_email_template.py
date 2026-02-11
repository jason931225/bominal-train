from __future__ import annotations

from datetime import datetime, timedelta, timezone

import pytest

from app.core.security import hash_password
from app.db.models import Task, User
from app.modules.train import worker as train_worker


def _find_summary_from_blocks(blocks) -> object:
    for block in blocks:
        if block.type == "mono":
            return block.data.get("text")
    raise AssertionError("summary mono block not found")


@pytest.mark.asyncio
async def test_terminal_notification_uses_template_payload_with_completion_summary(db_session, monkeypatch):
    captured: dict[str, object] = {}

    async def _fake_enqueue(payload, defer_seconds: float = 0.0):
        captured["payload"] = payload
        return "job-train-1"

    monkeypatch.setattr(train_worker, "enqueue_template_email", _fake_enqueue, raising=False)

    user = User(
        email="train-template@example.com",
        password_hash=hash_password("SuperSecret123"),
        display_name="Train Template",
        ui_locale="en",
        role_id=2,
    )
    db_session.add(user)
    await db_session.flush()

    task = Task(
        user_id=user.id,
        module="train",
        state="POLLING",
        deadline_at=datetime.now(timezone.utc) + timedelta(hours=1),
        spec_json={
            "notify": True,
            "dep": "수서",
            "arr": "부산",
            "people_count": 2,
            "item_code": "SRT345",
            "item_date": "2026-02-11",
        },
        idempotency_key="train-template-1",
    )
    db_session.add(task)
    await db_session.commit()
    await db_session.refresh(task)

    await train_worker._enqueue_terminal_notification(db_session, task=task, final_state="COMPLETED")

    template_payload = captured["payload"]
    assert template_payload.to_email == "train-template@example.com"
    summary = _find_summary_from_blocks(template_payload.blocks)
    assert summary == {"$ref": "task.summary"}
    assert template_payload.context["task"]["summary"].startswith("Successfully completed reservation for train on ")
    assert "SRT345 2026-02-11 for 2 people." in template_payload.context["task"]["summary"]
