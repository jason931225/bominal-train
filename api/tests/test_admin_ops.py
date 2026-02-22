from __future__ import annotations

from datetime import datetime, timedelta, timezone

import fakeredis.aioredis
import pytest
from sqlalchemy import select

from app.db.models import Role, Task, User
from tests.conftest import make_fake_get_redis_client


async def _register_and_login(client, *, email: str) -> str:
    register = await client.post(
        "/api/auth/register",
        json={"email": email, "password": "SuperSecret123", "display_name": "Admin Ops"},
    )
    assert register.status_code in (200, 201)
    login = await client.post(
        "/api/auth/login",
        json={"email": email, "password": "SuperSecret123", "remember_me": False},
    )
    assert login.status_code == 200
    cookie = login.cookies.get("bominal_session")
    assert cookie
    return cookie


@pytest.mark.asyncio
async def test_admin_ops_requires_admin_role(client):
    cookie = await _register_and_login(client, email="ops-user@example.com")
    forbidden = await client.get("/api/admin/ops/status", cookies={"bominal_session": cookie})
    assert forbidden.status_code == 403


@pytest.mark.asyncio
async def test_admin_ops_status_and_requeue(client, db_session, monkeypatch):
    fake_redis = fakeredis.aioredis.FakeRedis()
    monkeypatch.setattr("app.http.routes.admin.get_redis_client", make_fake_get_redis_client(fake_redis))

    cookie = await _register_and_login(client, email="ops-admin@example.com")

    user = (await db_session.execute(select(User).where(User.email == "ops-admin@example.com"))).scalar_one()
    admin_role = (await db_session.execute(select(Role).where(Role.name == "admin"))).scalar_one()
    user.role_id = admin_role.id
    user.access_status = "approved"
    await db_session.commit()

    # Seed arq keys and heartbeat.
    await fake_redis.zadd(b"arq:queue", {b"job1": 1})
    await fake_redis.zadd(b"arq:in-progress", {b"job2": 1, b"job3": 2})
    await fake_redis.set(b"bominal:worker:heartbeat", b"2026-01-01T00:00:00+00:00", ex=30)

    now = datetime.now(timezone.utc)
    stale_task = Task(
        user_id=user.id,
        module="train",
        state="RUNNING",
        deadline_at=now + timedelta(hours=1),
        spec_json={},
        idempotency_key="x" * 64,
        created_at=now - timedelta(hours=1),
        updated_at=now - timedelta(minutes=15),
    )
    db_session.add(stale_task)
    await db_session.commit()

    status_res = await client.get("/api/admin/ops/status", cookies={"bominal_session": cookie})
    assert status_res.status_code == 200
    payload = status_res.json()
    assert payload["redis"]["ok"] is True
    assert payload["arq"]["queued"] == 1
    assert payload["arq"]["in_progress"] == 2
    assert payload["worker"]["online"] is True
    assert payload["train"]["stale_task_count"] >= 1

    stale_res = await client.get("/api/admin/ops/train/stale-tasks?limit=10", cookies={"bominal_session": cookie})
    assert stale_res.status_code == 200
    stale_payload = stale_res.json()
    assert any(row["task_id"] == str(stale_task.id) for row in stale_payload["tasks"])

    # Requeue resets state to QUEUED.
    requeue_res = await client.post(
        f"/api/admin/ops/train/tasks/{stale_task.id}/requeue",
        cookies={"bominal_session": cookie},
    )
    assert requeue_res.status_code == 200
    await db_session.refresh(stale_task)
    assert stale_task.state == "QUEUED"
