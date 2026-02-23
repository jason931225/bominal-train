from __future__ import annotations

from datetime import timedelta
from uuid import uuid4

import fakeredis.aioredis
import pytest
from sqlalchemy import select

from app.core.time import utc_now
from app.db.models import Task, TaskAttempt, User
from app.modules.restaurant.lease import acquire_payment_lease
from app.modules.restaurant.policy import build_payment_lease_key
from app.modules.restaurant.worker import run_restaurant_task


async def _create_user(db_session) -> User:
    user = User(
        email=f"restaurant-user-{uuid4().hex[:8]}@example.com",
        password_hash="hash",
        display_name=f"Restaurant User {uuid4().hex[:6]}",
        role_id=2,
    )
    db_session.add(user)
    await db_session.commit()
    await db_session.refresh(user)
    return user


async def _create_restaurant_task(db_session, *, user_id, spec_json: dict) -> Task:
    task = Task(
        user_id=user_id,
        module="restaurant",
        state="QUEUED",
        deadline_at=utc_now() + timedelta(minutes=30),
        spec_json=spec_json,
        idempotency_key=f"restaurant-{uuid4().hex}",
    )
    db_session.add(task)
    await db_session.commit()
    await db_session.refresh(task)
    return task


@pytest.mark.asyncio
async def test_worker_uses_auth_fallback_sequence_before_failure(db_session, db_session_factory, monkeypatch):
    user = await _create_user(db_session)
    task = await _create_restaurant_task(
        db_session,
        user_id=user.id,
        spec_json={"provider": "RESY", "phase": "search"},
    )

    monkeypatch.setattr("app.modules.restaurant.worker.SessionLocal", db_session_factory)

    await run_restaurant_task({}, str(task.id))
    await db_session.refresh(task)
    attempts = (await db_session.execute(select(TaskAttempt).where(TaskAttempt.task_id == task.id))).scalars().all()
    assert task.state == "POLLING"
    assert attempts[-1].error_code == "auth_refresh_retry"
    assert attempts[-1].retryable is True

    await run_restaurant_task({}, str(task.id))
    await db_session.refresh(task)
    attempts = (await db_session.execute(select(TaskAttempt).where(TaskAttempt.task_id == task.id))).scalars().all()
    assert task.state == "POLLING"
    assert attempts[-1].error_code == "auth_refresh_retry"
    assert attempts[-1].retryable is True

    await run_restaurant_task({}, str(task.id))
    await db_session.refresh(task)
    attempts = (await db_session.execute(select(TaskAttempt).where(TaskAttempt.task_id == task.id))).scalars().all()
    assert task.state == "POLLING"
    assert attempts[-1].error_code == "auth_bootstrap_required"
    assert attempts[-1].retryable is True

    await run_restaurant_task({}, str(task.id))
    await db_session.refresh(task)
    attempts = (await db_session.execute(select(TaskAttempt).where(TaskAttempt.task_id == task.id))).scalars().all()
    assert task.state == "FAILED"
    assert task.failed_at is not None
    assert attempts[-1].error_code == "auth_failed"
    assert attempts[-1].retryable is False


@pytest.mark.asyncio
async def test_worker_blocks_parallel_payment_with_same_lease_key(db_session, db_session_factory, monkeypatch):
    user = await _create_user(db_session)
    task = await _create_restaurant_task(
        db_session,
        user_id=user.id,
        spec_json={
            "provider": "RESY",
            "phase": "payment",
            "auth_ok": True,
            "account_ref": "acct-1",
            "restaurant_id": "rest-1",
        },
    )

    fake_redis = fakeredis.aioredis.FakeRedis()
    lease_key = build_payment_lease_key(provider="RESY", account_ref="acct-1", restaurant_id="rest-1")
    acquired = await acquire_payment_lease(fake_redis, lease_key=lease_key, holder_token="other-holder", ttl_seconds=30)
    assert acquired is True

    monkeypatch.setattr("app.modules.restaurant.worker.SessionLocal", db_session_factory)

    async def _fake_get_redis():
        return fake_redis

    monkeypatch.setattr("app.modules.restaurant.worker.get_redis_client", _fake_get_redis)

    await run_restaurant_task({}, str(task.id))
    await db_session.refresh(task)

    attempt = (
        await db_session.execute(
            select(TaskAttempt).where(TaskAttempt.task_id == task.id).order_by(TaskAttempt.started_at.desc())
        )
    ).scalars().first()
    assert attempt is not None
    assert task.state == "POLLING"
    assert attempt.error_code == "payment_lease_busy"
    assert attempt.retryable is True


@pytest.mark.asyncio
async def test_non_committing_phase_can_run_without_payment_lease(db_session, db_session_factory, monkeypatch):
    user = await _create_user(db_session)
    task = await _create_restaurant_task(
        db_session,
        user_id=user.id,
        spec_json={
            "provider": "RESY",
            "phase": "search",
            "auth_ok": True,
            "account_ref": "acct-1",
            "restaurant_id": "rest-1",
        },
    )

    monkeypatch.setattr("app.modules.restaurant.worker.SessionLocal", db_session_factory)

    async def _should_not_call_redis():
        raise AssertionError("get_redis_client should not be called for non-committing phase")

    monkeypatch.setattr("app.modules.restaurant.worker.get_redis_client", _should_not_call_redis)

    await run_restaurant_task({}, str(task.id))
    await db_session.refresh(task)

    attempt = (
        await db_session.execute(
            select(TaskAttempt).where(TaskAttempt.task_id == task.id).order_by(TaskAttempt.started_at.desc())
        )
    ).scalars().first()
    assert attempt is not None
    assert task.state == "COMPLETED"
    assert task.completed_at is not None
    assert attempt.ok is True
    assert attempt.retryable is False


@pytest.mark.asyncio
async def test_worker_skips_invalid_or_ineligible_tasks(db_session, db_session_factory, monkeypatch):
    warnings: list[str] = []
    infos: list[str] = []
    monkeypatch.setattr("app.modules.restaurant.worker.logger.warning", lambda msg, *_args: warnings.append(msg))
    monkeypatch.setattr("app.modules.restaurant.worker.logger.info", lambda msg, *_args: infos.append(msg))

    monkeypatch.setattr("app.modules.restaurant.worker.SessionLocal", db_session_factory)

    await run_restaurant_task({}, "not-a-uuid")
    assert any("invalid task id" in line.lower() for line in warnings)

    await run_restaurant_task({}, str(uuid4()))
    assert any("not found" in line.lower() for line in warnings)

    user = await _create_user(db_session)
    non_restaurant = Task(
        user_id=user.id,
        module="train",
        state="QUEUED",
        deadline_at=utc_now() + timedelta(minutes=30),
        spec_json={"provider": "RESY"},
        idempotency_key=f"not-restaurant-{uuid4().hex}",
    )
    db_session.add(non_restaurant)
    await db_session.commit()

    await run_restaurant_task({}, str(non_restaurant.id))
    assert any("not a restaurant task" in line.lower() for line in warnings)

    hidden_task = await _create_restaurant_task(
        db_session,
        user_id=user.id,
        spec_json={"provider": "RESY", "phase": "search", "auth_ok": True},
    )
    hidden_task.hidden_at = utc_now()
    await db_session.commit()

    await run_restaurant_task({}, str(hidden_task.id))
    assert any("hidden/cancelled restaurant task" in line.lower() for line in infos)


@pytest.mark.asyncio
async def test_payment_phase_completes_and_releases_lease(db_session, db_session_factory, monkeypatch):
    user = await _create_user(db_session)
    task = await _create_restaurant_task(
        db_session,
        user_id=user.id,
        spec_json={
            "provider": "RESY",
            "phase": "payment",
            "auth_ok": True,
            "account_ref": "acct-2",
            "restaurant_id": "rest-2",
        },
    )

    fake_redis = fakeredis.aioredis.FakeRedis()
    releases: list[tuple[str, str]] = []

    monkeypatch.setattr("app.modules.restaurant.worker.SessionLocal", db_session_factory)
    async def _fake_get_redis():
        return fake_redis

    monkeypatch.setattr("app.modules.restaurant.worker.get_redis_client", _fake_get_redis)

    async def _release(redis_client, *, lease_key: str, holder_token: str):  # noqa: ANN001
        assert redis_client is fake_redis
        releases.append((lease_key, holder_token))

    monkeypatch.setattr("app.modules.restaurant.worker.release_payment_lease", _release)

    await run_restaurant_task({}, str(task.id))
    await db_session.refresh(task)

    assert task.state == "COMPLETED"
    assert task.completed_at is not None
    assert len(releases) == 1
    assert releases[0][0] == build_payment_lease_key(provider="RESY", account_ref="acct-2", restaurant_id="rest-2")

    attempts = (
        await db_session.execute(
            select(TaskAttempt).where(TaskAttempt.task_id == task.id).order_by(TaskAttempt.started_at.desc())
        )
    ).scalars().all()
    assert attempts
    assert attempts[0].action == "PAY"
    assert attempts[0].ok is True


@pytest.mark.asyncio
async def test_payment_phase_busy_lease_returns_without_release(db_session, db_session_factory, monkeypatch):
    user = await _create_user(db_session)
    task = await _create_restaurant_task(
        db_session,
        user_id=user.id,
        spec_json={
            "provider": "RESY",
            "phase": "payment",
            "auth_ok": True,
            "account_ref": "acct-busy",
            "restaurant_id": "rest-busy",
        },
    )

    monkeypatch.setattr("app.modules.restaurant.worker.SessionLocal", db_session_factory)
    async def _fake_get_redis():
        return fakeredis.aioredis.FakeRedis()

    monkeypatch.setattr("app.modules.restaurant.worker.get_redis_client", _fake_get_redis)

    async def _acquire_false(*_args, **_kwargs):  # noqa: ANN001, ANN003
        return False

    release_calls: list[tuple[str, str]] = []

    async def _release(redis_client, *, lease_key: str, holder_token: str):  # noqa: ANN001
        _ = redis_client
        release_calls.append((lease_key, holder_token))

    monkeypatch.setattr("app.modules.restaurant.worker.acquire_payment_lease", _acquire_false)
    monkeypatch.setattr("app.modules.restaurant.worker.release_payment_lease", _release)

    await run_restaurant_task({}, str(task.id))
    await db_session.refresh(task)
    assert task.state == "POLLING"
    assert release_calls == []


@pytest.mark.asyncio
async def test_payment_phase_success_path_always_releases_lease(db_session, db_session_factory, monkeypatch):
    user = await _create_user(db_session)
    task = await _create_restaurant_task(
        db_session,
        user_id=user.id,
        spec_json={
            "provider": "RESY",
            "phase": "payment",
            "auth_ok": True,
            "account_ref": "acct-release",
            "restaurant_id": "rest-release",
        },
    )

    monkeypatch.setattr("app.modules.restaurant.worker.SessionLocal", db_session_factory)
    fake_redis = fakeredis.aioredis.FakeRedis()
    async def _fake_get_redis():
        return fake_redis

    monkeypatch.setattr("app.modules.restaurant.worker.get_redis_client", _fake_get_redis)

    async def _acquire_true(*_args, **_kwargs):  # noqa: ANN001, ANN003
        return True

    release_calls: list[tuple[str, str]] = []

    async def _release(redis_client, *, lease_key: str, holder_token: str):  # noqa: ANN001
        assert redis_client is fake_redis
        release_calls.append((lease_key, holder_token))

    monkeypatch.setattr("app.modules.restaurant.worker.acquire_payment_lease", _acquire_true)
    monkeypatch.setattr("app.modules.restaurant.worker.release_payment_lease", _release)

    await run_restaurant_task({}, str(task.id))
    await db_session.refresh(task)
    assert task.state == "COMPLETED"
    assert len(release_calls) == 1
