from __future__ import annotations

from datetime import datetime, timedelta, timezone
from uuid import uuid4

import pytest
from fastapi import HTTPException
from sqlalchemy import delete, select

from app.db.models import Artifact, Role, Secret, Session, Task, TaskAttempt, User
from app.http.routes import admin as admin_routes


async def _role_id(db_session, role_name: str) -> int:
    return (await db_session.execute(select(Role.id).where(Role.name == role_name))).scalar_one()


async def _make_user(
    db_session,
    *,
    email: str,
    role_name: str = "user",
    access_status: str = "approved",
) -> User:
    user = User(
        email=email,
        password_hash=f"hash-{uuid4().hex}",
        display_name=f"user-{uuid4().hex[:12]}",
        role_id=await _role_id(db_session, role_name),
        access_status=access_status,
        access_reviewed_at=datetime.now(timezone.utc) if access_status != "pending" else None,
    )
    db_session.add(user)
    await db_session.commit()
    await db_session.refresh(user)
    return user


@pytest.mark.asyncio
async def test_admin_only_returns_expected_message():
    payload = await admin_routes.admin_only()
    assert payload.message == "Admin access granted"


@pytest.mark.asyncio
async def test_admin_payment_settings_controls(db_session):
    admin_user = await _make_user(
        db_session,
        email=f"payment-admin-{uuid4().hex[:8]}@example.com",
        role_name="admin",
        access_status="approved",
    )

    initial = await admin_routes.get_payment_settings(db=db_session)
    assert isinstance(initial.payment_enabled, bool)
    assert initial.wallet_only is True

    disabled = await admin_routes.set_payment_settings_enabled(
        body=admin_routes.AdminPaymentEnabledRequest(enabled=False),
        db=db_session,
        admin_user=admin_user,
    )
    assert disabled.payment_enabled is False
    assert disabled.payment_enabled_override is False

    enabled = await admin_routes.set_payment_settings_enabled(
        body=admin_routes.AdminPaymentEnabledRequest(enabled=True),
        db=db_session,
        admin_user=admin_user,
    )
    assert enabled.payment_enabled_override is True

    with pytest.raises(HTTPException) as put_exc:
        await admin_routes.set_payment_settings_card(
            db=db_session,
            admin_user=admin_user,
        )
    assert put_exc.value.status_code == 410

    with pytest.raises(HTTPException) as delete_exc:
        await admin_routes.delete_payment_settings_card(
            db=db_session,
            admin_user=admin_user,
        )
    assert delete_exc.value.status_code == 410


@pytest.mark.asyncio
async def test_get_system_stats_aggregates_counts(db_session):
    now = datetime.now(timezone.utc)
    user = await _make_user(db_session, email=f"stats-{uuid4().hex[:8]}@example.com")

    db_session.add(
        Session(
            user_id=user.id,
            token_hash=f"active-{uuid4().hex}",
            expires_at=now + timedelta(days=1),
            last_seen_at=now - timedelta(hours=1),
        )
    )
    db_session.add(
        Session(
            user_id=user.id,
            token_hash=f"expired-{uuid4().hex}",
            expires_at=now - timedelta(minutes=5),
            last_seen_at=now - timedelta(days=2),
            revoked_at=now - timedelta(days=1),
        )
    )
    db_session.add(
        Task(
            user_id=user.id,
            module="train",
            state="COMPLETED",
            deadline_at=now + timedelta(hours=2),
            spec_json={"provider": "SRT"},
            idempotency_key=f"done-{uuid4().hex}",
            completed_at=now - timedelta(hours=1),
        )
    )
    db_session.add(
        Task(
            user_id=user.id,
            module="train",
            state="RUNNING",
            deadline_at=now + timedelta(hours=3),
            spec_json={"provider": "KTX"},
            idempotency_key=f"running-{uuid4().hex}",
        )
    )
    await db_session.commit()

    stats = await admin_routes.get_system_stats(db_session)
    assert stats.total_users >= 1
    assert stats.active_users_24h >= 1
    assert stats.total_sessions >= 2
    assert stats.active_sessions >= 1
    assert stats.total_tasks >= 2
    assert stats.tasks_by_state["COMPLETED"] >= 1
    assert stats.tasks_completed_24h >= 1


@pytest.mark.asyncio
async def test_get_ops_status_handles_redis_success_and_failure(db_session, monkeypatch):
    now = datetime.now(timezone.utc)
    user = await _make_user(db_session, email=f"ops-{uuid4().hex[:8]}@example.com")
    stale_task = Task(
        user_id=user.id,
        module="train",
        state="RUNNING",
        deadline_at=now + timedelta(hours=2),
        spec_json={"provider": "SRT"},
        idempotency_key=f"stale-{uuid4().hex}",
        updated_at=now - admin_routes.STALE_TASK_WINDOW - timedelta(minutes=2),
    )
    db_session.add(stale_task)
    await db_session.commit()

    class _RedisOk:
        async def ping(self) -> None:
            return None

        async def zcard(self, key: bytes) -> int:
            if key == b"arq:queue":
                return 2
            return 1

        async def get(self, _key: bytes):
            # Invalid timestamp path should leave last_heartbeat_at as None.
            return b"not-a-datetime"

    async def _get_ok():
        return _RedisOk()

    monkeypatch.setattr(admin_routes, "get_redis_client", _get_ok)
    ok_payload = await admin_routes.get_ops_status(db_session)
    assert ok_payload.redis.ok is True
    assert ok_payload.arq.queued == 2
    assert ok_payload.arq.in_progress == 1
    assert ok_payload.worker.online is True
    assert ok_payload.worker.last_heartbeat_at is None
    assert ok_payload.train.stale_task_count >= 1

    class _RedisNaiveHeartbeat:
        async def ping(self) -> None:
            return None

        async def zcard(self, _key: bytes) -> int:
            return 0

        async def get(self, _key: bytes):
            # Naive timestamp should be normalized to UTC.
            return b"2026-01-01T00:00:00"

    async def _get_naive():
        return _RedisNaiveHeartbeat()

    monkeypatch.setattr(admin_routes, "get_redis_client", _get_naive)
    naive_payload = await admin_routes.get_ops_status(db_session)
    assert naive_payload.worker.online is True
    assert naive_payload.worker.last_heartbeat_at is not None
    assert naive_payload.worker.last_heartbeat_at.tzinfo is not None

    class _RedisFail:
        async def ping(self) -> None:
            raise RuntimeError("boom")

    async def _get_fail():
        return _RedisFail()

    monkeypatch.setattr(admin_routes, "get_redis_client", _get_fail)
    fail_payload = await admin_routes.get_ops_status(db_session)
    assert fail_payload.redis.ok is False
    assert fail_payload.redis.detail == "RuntimeError"


@pytest.mark.asyncio
async def test_list_stale_and_recent_failure_queries(db_session):
    now = datetime.now(timezone.utc)
    user = await _make_user(db_session, email=f"stale-{uuid4().hex[:8]}@example.com")
    task = Task(
        user_id=user.id,
        module="train",
        state="RUNNING",
        deadline_at=now + timedelta(hours=2),
        spec_json={"provider": "SRT"},
        idempotency_key=f"task-{uuid4().hex}",
        created_at=now - timedelta(hours=1),
        updated_at=now - admin_routes.STALE_TASK_WINDOW - timedelta(minutes=1),
    )
    db_session.add(task)
    await db_session.flush()
    attempt = TaskAttempt(
        task_id=task.id,
        action="search",
        provider="SRT",
        ok=False,
        retryable=True,
        error_code="provider_timeout",
        error_message_safe="timeout",
        duration_ms=123,
        started_at=now - timedelta(minutes=5),
        finished_at=now - timedelta(minutes=4),
    )
    db_session.add(attempt)
    await db_session.commit()

    stale = await admin_routes.list_stale_train_tasks(limit=20, db=db_session)
    assert len(stale.tasks) == 1
    assert stale.tasks[0].last_error_code == "provider_timeout"

    failures = await admin_routes.list_recent_train_failures(hours=24, limit=20, db=db_session)
    assert len(failures.failures) == 1
    assert failures.failures[0].task_id == task.id


@pytest.mark.asyncio
async def test_recover_and_requeue_direct_paths(db_session, monkeypatch):
    now = datetime.now(timezone.utc)
    user = await _make_user(db_session, email=f"requeue-{uuid4().hex[:8]}@example.com")
    task = Task(
        user_id=user.id,
        module="train",
        state="RUNNING",
        deadline_at=now + timedelta(hours=2),
        spec_json={admin_routes.SPEC_KEY_NEXT_RUN_AT: now.isoformat()},
        idempotency_key=f"requeue-{uuid4().hex}",
    )
    db_session.add(task)
    await db_session.commit()

    async def _recover(_db):  # noqa: ANN001
        return 9

    enqueue_calls: list[str] = []

    async def _enqueue(task_id: str) -> None:
        enqueue_calls.append(task_id)

    monkeypatch.setattr(admin_routes, "enqueue_recoverable_tasks", _recover)
    monkeypatch.setattr(admin_routes, "enqueue_train_task", _enqueue)

    recovered = await admin_routes.recover_train_tasks(db=db_session)
    assert recovered.enqueued_count == 9

    response = await admin_routes.requeue_train_task(task_id=task.id, db=db_session)
    await db_session.refresh(task)
    assert response.message == "Task requeued"
    assert task.state == "QUEUED"
    assert admin_routes.SPEC_KEY_NEXT_RUN_AT not in (task.spec_json or {})
    assert enqueue_calls == [str(task.id)]


@pytest.mark.asyncio
async def test_requeue_guard_conditions_raise_expected_errors(db_session):
    now = datetime.now(timezone.utc)
    user = await _make_user(db_session, email=f"guards-{uuid4().hex[:8]}@example.com")

    missing_id = uuid4()
    with pytest.raises(HTTPException) as missing:
        await admin_routes.requeue_train_task(task_id=missing_id, db=db_session)
    assert missing.value.status_code == 404

    cases = [
        ("restaurant", "RUNNING", None, None, None, 404, "Task not found"),
        ("train", "RUNNING", now, None, None, 404, "Task not found"),
        ("train", "CANCELLED", None, now, None, 409, "Task is cancelled"),
        ("train", "PAUSED", None, None, now, 409, "Task is paused"),
        ("train", "FAILED", None, None, None, 409, "Task is terminal"),
    ]
    for module, state, hidden_at, cancelled_at, paused_at, status_code, detail in cases:
        task = Task(
            user_id=user.id,
            module=module,
            state=state,
            deadline_at=now + timedelta(hours=1),
            spec_json={},
            idempotency_key=f"guard-{uuid4().hex}",
            hidden_at=hidden_at,
            cancelled_at=cancelled_at,
            paused_at=paused_at,
        )
        db_session.add(task)
        await db_session.commit()
        with pytest.raises(HTTPException) as exc:
            await admin_routes.requeue_train_task(task_id=task.id, db=db_session)
        assert exc.value.status_code == status_code
        assert exc.value.detail == detail


@pytest.mark.asyncio
async def test_user_admin_routes_cover_role_access_revoke_and_delete(db_session):
    now = datetime.now(timezone.utc)
    admin_user = await _make_user(
        db_session,
        email=f"admin-{uuid4().hex[:8]}@example.com",
        role_name="admin",
        access_status="approved",
    )
    target_user = await _make_user(
        db_session,
        email=f"target-{uuid4().hex[:8]}@example.com",
        role_name="user",
        access_status="approved",
    )

    users_page = await admin_routes.list_users(page=1, page_size=10, search="target-", access_status="approved", db=db_session)
    assert users_page.total >= 1
    assert any(row.id == target_user.id for row in users_page.users)

    detail = await admin_routes.get_user_detail(user_id=target_user.id, db=db_session)
    assert detail.id == target_user.id

    with pytest.raises(HTTPException) as missing_user:
        await admin_routes.get_user_detail(user_id=uuid4(), db=db_session)
    assert missing_user.value.status_code == 404

    # Force invalid-role branch by deleting target role row first.
    await db_session.execute(delete(Role).where(Role.name == "admin"))
    await db_session.commit()
    with pytest.raises(HTTPException) as invalid_role:
        await admin_routes.update_user_role(
            user_id=target_user.id,
            body=admin_routes.UpdateUserRole(role="admin"),
            db=db_session,
        )
    assert invalid_role.value.status_code == 400

    with pytest.raises(HTTPException) as missing_for_role:
        await admin_routes.update_user_role(
            user_id=uuid4(),
            body=admin_routes.UpdateUserRole(role="user"),
            db=db_session,
        )
    assert missing_for_role.value.status_code == 404

    # Recreate admin role for remaining flows.
    db_session.add(Role(name="admin"))
    await db_session.commit()

    updated_role = await admin_routes.update_user_role(
        user_id=target_user.id,
        body=admin_routes.UpdateUserRole(role="admin"),
        db=db_session,
    )
    assert "updated to admin" in updated_role.message

    with pytest.raises(HTTPException) as self_downgrade:
        await admin_routes.update_user_access(
            user_id=admin_user.id,
            body=admin_routes.UpdateUserAccess(access_status="rejected"),
            db=db_session,
            admin_user=admin_user,
        )
    assert self_downgrade.value.status_code == 400

    db_session.add(
        Session(
            user_id=target_user.id,
            token_hash=f"session-{uuid4().hex}",
            expires_at=now + timedelta(days=1),
            last_seen_at=now,
        )
    )
    await db_session.commit()

    rejected = await admin_routes.update_user_access(
        user_id=target_user.id,
        body=admin_routes.UpdateUserAccess(access_status="rejected"),
        db=db_session,
        admin_user=admin_user,
    )
    assert "revoked" in rejected.message

    pending = await admin_routes.update_user_access(
        user_id=target_user.id,
        body=admin_routes.UpdateUserAccess(access_status="pending"),
        db=db_session,
        admin_user=admin_user,
    )
    assert "pending" in pending.message

    with pytest.raises(HTTPException) as missing_access_user:
        await admin_routes.update_user_access(
            user_id=uuid4(),
            body=admin_routes.UpdateUserAccess(access_status="approved"),
            db=db_session,
            admin_user=admin_user,
        )
    assert missing_access_user.value.status_code == 404

    with pytest.raises(HTTPException) as missing_revoke_user:
        await admin_routes.revoke_user_sessions(user_id=uuid4(), db=db_session)
    assert missing_revoke_user.value.status_code == 404

    revoke_ok = await admin_routes.revoke_user_sessions(user_id=target_user.id, db=db_session)
    assert "Revoked" in revoke_ok.message

    task = Task(
        user_id=target_user.id,
        module="train",
        state="FAILED",
        deadline_at=now + timedelta(hours=2),
        spec_json={},
        idempotency_key=f"delete-{uuid4().hex}",
    )
    db_session.add(task)
    await db_session.flush()
    db_session.add(
        TaskAttempt(
            task_id=task.id,
            action="reserve",
            provider="SRT",
            ok=False,
            retryable=True,
            error_code="provider_error",
            error_message_safe="safe",
            duration_ms=42,
        )
    )
    db_session.add(
        Artifact(
            task_id=task.id,
            module="train",
            kind="reservation",
            data_json_safe={"reservation_id": "r-1"},
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

    with pytest.raises(HTTPException) as missing_delete_user:
        await admin_routes.delete_user(user_id=uuid4(), db=db_session)
    assert missing_delete_user.value.status_code == 404

    deleted = await admin_routes.delete_user(user_id=target_user.id, db=db_session)
    assert target_user.email in deleted.message
    assert (await db_session.execute(select(User).where(User.id == target_user.id))).scalar_one_or_none() is None
