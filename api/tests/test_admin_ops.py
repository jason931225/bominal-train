from __future__ import annotations

from datetime import datetime, timedelta, timezone
from uuid import uuid4

import fakeredis.aioredis
import pytest
from sqlalchemy import select

from app.db.models import Artifact, Role, Secret, Session, Task, TaskAttempt, User
from tests.conftest import make_fake_get_redis_client


async def _register_and_login(client, *, email: str, display_name: str | None = None) -> str:
    resolved_display_name = display_name or f"Admin Ops {email.split('@', 1)[0][:24]}"
    register = await client.post(
        "/api/auth/register",
        json={"email": email, "password": "SuperSecret123", "display_name": resolved_display_name},
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


async def _promote_admin(db_session, *, email: str) -> User:
    user = (await db_session.execute(select(User).where(User.email == email))).scalar_one()
    admin_role = (await db_session.execute(select(Role).where(Role.name == "admin"))).scalar_one()
    user.role_id = admin_role.id
    user.access_status = "approved"
    user.access_reviewed_at = datetime.now(timezone.utc)
    await db_session.commit()
    return user


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


@pytest.mark.asyncio
async def test_admin_stats_user_list_and_user_detail(client, db_session):
    admin_email = f"stats-admin-{uuid4().hex[:8]}@example.com"
    target_email = f"stats-target-{uuid4().hex[:8]}@example.com"

    admin_cookie = await _register_and_login(client, email=admin_email)
    target_cookie = await _register_and_login(client, email=target_email)
    assert target_cookie

    admin_user = await _promote_admin(db_session, email=admin_email)
    target_user = (await db_session.execute(select(User).where(User.email == target_email))).scalar_one()
    target_user.access_status = "approved"
    target_user.access_reviewed_at = datetime.now(timezone.utc)
    await db_session.commit()

    now = datetime.now(timezone.utc)
    task = Task(
        user_id=target_user.id,
        module="train",
        state="COMPLETED",
        deadline_at=now + timedelta(hours=2),
        spec_json={"provider": "SRT"},
        idempotency_key=f"stats-{uuid4().hex}",
        created_at=now - timedelta(hours=3),
        updated_at=now - timedelta(hours=2),
        completed_at=now - timedelta(minutes=30),
    )
    db_session.add(task)
    db_session.add(
        Secret(
            user_id=target_user.id,
            kind="payment.card",
            ciphertext="cipher",
            nonce="nonce",
            wrapped_dek="wrapped",
            dek_nonce="dek",
            aad="aad",
            kek_version=1,
        )
    )
    await db_session.commit()

    stats = await client.get("/api/admin/stats", cookies={"bominal_session": admin_cookie})
    assert stats.status_code == 200
    stats_payload = stats.json()
    assert stats_payload["total_users"] >= 2
    assert stats_payload["total_tasks"] >= 1
    assert stats_payload["tasks_by_state"]["COMPLETED"] >= 1
    assert stats_payload["tasks_completed_24h"] >= 1

    users = await client.get(
        "/api/admin/users?page=1&page_size=10&search=stats-target&access_status=approved",
        cookies={"bominal_session": admin_cookie},
    )
    assert users.status_code == 200
    users_payload = users.json()
    assert users_payload["total"] >= 1
    assert any(row["email"] == target_email for row in users_payload["users"])

    detail = await client.get(f"/api/admin/users/{target_user.id}", cookies={"bominal_session": admin_cookie})
    assert detail.status_code == 200
    detail_payload = detail.json()
    assert detail_payload["id"] == str(target_user.id)
    assert detail_payload["secret_count"] >= 1
    assert detail_payload["task_count"] >= 1
    assert detail_payload["session_count"] >= 1
    assert detail_payload["active_session_count"] >= 1
    assert detail_payload["access_status"] == "approved"
    assert detail_payload["role"] in {"admin", "user"}
    assert detail_payload["id"] != str(admin_user.id)

    missing = await client.get(f"/api/admin/users/{uuid4()}", cookies={"bominal_session": admin_cookie})
    assert missing.status_code == 404


@pytest.mark.asyncio
async def test_admin_user_role_access_revoke_and_delete_paths(client, db_session):
    admin_email = f"admin-role-{uuid4().hex[:8]}@example.com"
    target_email = f"target-role-{uuid4().hex[:8]}@example.com"

    admin_cookie = await _register_and_login(client, email=admin_email)
    await _register_and_login(client, email=target_email)
    admin_user = await _promote_admin(db_session, email=admin_email)
    target_user = (await db_session.execute(select(User).where(User.email == target_email))).scalar_one()

    update_role_ok = await client.patch(
        f"/api/admin/users/{target_user.id}/role",
        json={"role": "admin"},
        cookies={"bominal_session": admin_cookie},
    )
    assert update_role_ok.status_code == 200
    assert "updated to admin" in update_role_ok.json()["message"]

    update_role_missing_user = await client.patch(
        f"/api/admin/users/{uuid4()}/role",
        json={"role": "admin"},
        cookies={"bominal_session": admin_cookie},
    )
    assert update_role_missing_user.status_code == 404
    assert "User not found" in update_role_missing_user.json()["detail"]

    self_reject = await client.patch(
        f"/api/admin/users/{admin_user.id}/access",
        json={"access_status": "rejected"},
        cookies={"bominal_session": admin_cookie},
    )
    assert self_reject.status_code == 400
    assert "Cannot remove your own approved access status" in self_reject.json()["detail"]

    target_user.access_status = "approved"
    target_user.access_reviewed_at = datetime.now(timezone.utc)
    db_session.add(
        Session(
            user_id=target_user.id,
            token_hash=f"token-{uuid4().hex}",
            expires_at=datetime.now(timezone.utc) + timedelta(days=3),
            last_seen_at=datetime.now(timezone.utc),
        )
    )
    await db_session.commit()

    reject_target = await client.patch(
        f"/api/admin/users/{target_user.id}/access",
        json={"access_status": "rejected"},
        cookies={"bominal_session": admin_cookie},
    )
    assert reject_target.status_code == 200
    assert "revoked" in reject_target.json()["message"]
    await db_session.refresh(target_user)
    assert target_user.access_status == "rejected"
    assert target_user.access_reviewed_at is not None

    pending_target = await client.patch(
        f"/api/admin/users/{target_user.id}/access",
        json={"access_status": "pending"},
        cookies={"bominal_session": admin_cookie},
    )
    assert pending_target.status_code == 200
    await db_session.refresh(target_user)
    assert target_user.access_status == "pending"
    assert target_user.access_reviewed_at is None

    revoke_missing = await client.post(
        f"/api/admin/users/{uuid4()}/revoke-sessions",
        cookies={"bominal_session": admin_cookie},
    )
    assert revoke_missing.status_code == 404

    db_session.add(
        Session(
            user_id=target_user.id,
            token_hash=f"token-{uuid4().hex}",
            expires_at=datetime.now(timezone.utc) + timedelta(days=1),
            last_seen_at=datetime.now(timezone.utc),
        )
    )
    await db_session.commit()
    revoke_ok = await client.post(
        f"/api/admin/users/{target_user.id}/revoke-sessions",
        cookies={"bominal_session": admin_cookie},
    )
    assert revoke_ok.status_code == 200
    assert "Revoked" in revoke_ok.json()["message"]

    task = Task(
        user_id=target_user.id,
        module="train",
        state="FAILED",
        deadline_at=datetime.now(timezone.utc) + timedelta(hours=1),
        spec_json={"provider": "SRT"},
        idempotency_key=f"delete-{uuid4().hex}",
    )
    db_session.add(task)
    await db_session.flush()
    db_session.add(
        TaskAttempt(
            task_id=task.id,
            action="search",
            provider="SRT",
            ok=False,
            retryable=True,
            error_code="provider_timeout",
            error_message_safe="timeout",
            duration_ms=120,
            meta_json_safe={"a": 1},
        )
    )
    db_session.add(
        Artifact(
            task_id=task.id,
            module="train",
            kind="reservation",
            data_json_safe={"reservation_id": "r1"},
        )
    )
    db_session.add(
        Secret(
            user_id=target_user.id,
            kind="payment.card",
            ciphertext="cipher",
            nonce="nonce",
            wrapped_dek="wrapped",
            dek_nonce="dek",
            aad="aad",
            kek_version=1,
        )
    )
    await db_session.commit()

    delete_missing = await client.delete(f"/api/admin/users/{uuid4()}", cookies={"bominal_session": admin_cookie})
    assert delete_missing.status_code == 404

    delete_ok = await client.delete(
        f"/api/admin/users/{target_user.id}",
        cookies={"bominal_session": admin_cookie},
    )
    assert delete_ok.status_code == 200
    assert target_email in delete_ok.json()["message"]

    assert (await db_session.execute(select(User).where(User.id == target_user.id))).scalar_one_or_none() is None
    assert (await db_session.execute(select(Task).where(Task.user_id == target_user.id))).scalars().all() == []
    assert (
        await db_session.execute(
            select(TaskAttempt).join(Task, Task.id == TaskAttempt.task_id).where(Task.user_id == target_user.id)
        )
    ).scalars().all() == []
    assert (
        await db_session.execute(
            select(Artifact).join(Task, Task.id == Artifact.task_id).where(Task.user_id == target_user.id)
        )
    ).scalars().all() == []
    assert (await db_session.execute(select(Secret).where(Secret.user_id == target_user.id))).scalars().all() == []


@pytest.mark.asyncio
async def test_admin_train_ops_recent_failures_recover_and_requeue_edge_cases(client, db_session, monkeypatch):
    admin_email = f"admin-ops2-{uuid4().hex[:8]}@example.com"
    admin_cookie = await _register_and_login(client, email=admin_email)
    admin_user = await _promote_admin(db_session, email=admin_email)

    task_with_failure = Task(
        user_id=admin_user.id,
        module="train",
        state="RUNNING",
        deadline_at=datetime.now(timezone.utc) + timedelta(hours=2),
        spec_json={"provider": "SRT"},
        idempotency_key=f"failure-{uuid4().hex}",
    )
    db_session.add(task_with_failure)
    await db_session.flush()
    db_session.add(
        TaskAttempt(
            task_id=task_with_failure.id,
            action="reserve",
            provider="SRT",
            ok=False,
            retryable=True,
            error_code="provider_timeout",
            error_message_safe="timeout",
            duration_ms=250,
            meta_json_safe={"provider": "SRT"},
            started_at=datetime.now(timezone.utc) - timedelta(minutes=5),
            finished_at=datetime.now(timezone.utc) - timedelta(minutes=4),
        )
    )
    await db_session.commit()

    recent = await client.get(
        "/api/admin/ops/train/recent-failures?hours=12&limit=20",
        cookies={"bominal_session": admin_cookie},
    )
    assert recent.status_code == 200
    recent_payload = recent.json()["failures"]
    assert any(row["task_id"] == str(task_with_failure.id) for row in recent_payload)

    async def _fake_recover(_db):  # noqa: ANN001
        return 7

    monkeypatch.setattr("app.http.routes.admin.enqueue_recoverable_tasks", _fake_recover)
    recover = await client.post("/api/admin/ops/train/recover", cookies={"bominal_session": admin_cookie})
    assert recover.status_code == 200
    assert recover.json()["enqueued_count"] == 7

    not_found = await client.post(
        f"/api/admin/ops/train/tasks/{uuid4()}/requeue",
        cookies={"bominal_session": admin_cookie},
    )
    assert not_found.status_code == 404

    wrong_module = Task(
        user_id=admin_user.id,
        module="restaurant",
        state="RUNNING",
        deadline_at=datetime.now(timezone.utc) + timedelta(hours=1),
        spec_json={},
        idempotency_key=f"module-{uuid4().hex}",
    )
    hidden_task = Task(
        user_id=admin_user.id,
        module="train",
        state="RUNNING",
        deadline_at=datetime.now(timezone.utc) + timedelta(hours=1),
        spec_json={},
        idempotency_key=f"hidden-{uuid4().hex}",
        hidden_at=datetime.now(timezone.utc),
    )
    cancelled_task = Task(
        user_id=admin_user.id,
        module="train",
        state="CANCELLED",
        deadline_at=datetime.now(timezone.utc) + timedelta(hours=1),
        spec_json={},
        idempotency_key=f"cancelled-{uuid4().hex}",
        cancelled_at=datetime.now(timezone.utc),
    )
    paused_task = Task(
        user_id=admin_user.id,
        module="train",
        state="PAUSED",
        deadline_at=datetime.now(timezone.utc) + timedelta(hours=1),
        spec_json={},
        idempotency_key=f"paused-{uuid4().hex}",
        paused_at=datetime.now(timezone.utc),
    )
    terminal_task = Task(
        user_id=admin_user.id,
        module="train",
        state="FAILED",
        deadline_at=datetime.now(timezone.utc) + timedelta(hours=1),
        spec_json={},
        idempotency_key=f"terminal-{uuid4().hex}",
    )
    db_session.add_all([wrong_module, hidden_task, cancelled_task, paused_task, terminal_task])
    await db_session.commit()

    wrong_module_res = await client.post(
        f"/api/admin/ops/train/tasks/{wrong_module.id}/requeue",
        cookies={"bominal_session": admin_cookie},
    )
    assert wrong_module_res.status_code == 404

    hidden_res = await client.post(
        f"/api/admin/ops/train/tasks/{hidden_task.id}/requeue",
        cookies={"bominal_session": admin_cookie},
    )
    assert hidden_res.status_code == 404

    cancelled_res = await client.post(
        f"/api/admin/ops/train/tasks/{cancelled_task.id}/requeue",
        cookies={"bominal_session": admin_cookie},
    )
    assert cancelled_res.status_code == 409
    assert "cancelled" in cancelled_res.json()["detail"].lower()

    paused_res = await client.post(
        f"/api/admin/ops/train/tasks/{paused_task.id}/requeue",
        cookies={"bominal_session": admin_cookie},
    )
    assert paused_res.status_code == 409
    assert "paused" in paused_res.json()["detail"].lower()

    terminal_res = await client.post(
        f"/api/admin/ops/train/tasks/{terminal_task.id}/requeue",
        cookies={"bominal_session": admin_cookie},
    )
    assert terminal_res.status_code == 409
    assert "terminal" in terminal_res.json()["detail"].lower()
