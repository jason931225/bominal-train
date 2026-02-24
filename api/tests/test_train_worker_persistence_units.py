from __future__ import annotations

from datetime import datetime, timedelta, timezone
from uuid import uuid4

import pytest

from app.db.models import Task, TaskAttempt
from app.modules.train.constants import ATTEMPT_ACTION_CANCEL, ATTEMPT_ACTION_PAY, ATTEMPT_ACTION_SEARCH
from app.modules.train import worker as train_worker
from app.modules.train.worker import PendingAttempt, _persist_attempts


@pytest.fixture(autouse=True)
def _default_attempt_persistence_override(monkeypatch):
    # Keep unit expectations deterministic regardless of env/test runtime flags.
    monkeypatch.setattr(train_worker.settings, "train_persist_all_attempts", False)


class _Result:
    def __init__(self, scalar_value):  # noqa: ANN001
        self._scalar_value = scalar_value

    def scalar_one_or_none(self):  # noqa: ANN201
        return self._scalar_value


class _DB:
    def __init__(self, previous_attempt):  # noqa: ANN001
        self._previous_attempt = previous_attempt

    async def execute(self, _stmt):  # noqa: ANN001
        return _Result(self._previous_attempt)


def _task() -> Task:
    return Task(
        user_id=uuid4(),
        module="train",
        state="POLLING",
        deadline_at=datetime.now(timezone.utc) + timedelta(hours=1),
        spec_json={},
        idempotency_key=f"task-{uuid4().hex}",
    )


def _attempt(*, action: str, ok: bool, retryable: bool, error_code: str | None, error_message_safe: str | None) -> PendingAttempt:
    return PendingAttempt(
        action=action,
        provider="SRT",
        ok=ok,
        retryable=retryable,
        error_code=error_code,
        error_message_safe=error_message_safe,
        duration_ms=10,
        meta_json_safe=None,
        started_at=datetime.now(timezone.utc),
    )


def _persisted_attempt(*, action: str, ok: bool, retryable: bool, error_code: str | None, error_message_safe: str | None) -> TaskAttempt:
    return TaskAttempt(
        task_id=uuid4(),
        action=action,
        provider="SRT",
        ok=ok,
        retryable=retryable,
        error_code=error_code,
        error_message_safe=error_message_safe,
        duration_ms=9,
        meta_json_safe=None,
        started_at=datetime.now(timezone.utc),
        finished_at=datetime.now(timezone.utc),
    )


@pytest.mark.asyncio
async def test_persist_attempts_skips_duplicate_retry_noise(monkeypatch):
    previous = _persisted_attempt(
        action=ATTEMPT_ACTION_SEARCH,
        ok=False,
        retryable=True,
        error_code="seat_unavailable",
        error_message_safe="No reservable seats are available for this schedule.",
    )
    db = _DB(previous_attempt=previous)
    task = _task()
    saved: list[PendingAttempt] = []

    async def _fake_save_attempt(_db, *, task, action, provider, ok, retryable, error_code, error_message_safe, duration_ms, meta_json_safe, started_at):  # noqa: ANN001, ANN003
        saved.append(
            _attempt(
                action=action,
                ok=ok,
                retryable=retryable,
                error_code=error_code,
                error_message_safe=error_message_safe,
            )
        )
        return _persisted_attempt(
            action=action,
            ok=ok,
            retryable=retryable,
            error_code=error_code,
            error_message_safe=error_message_safe,
        )

    monkeypatch.setattr("app.modules.train.worker._save_attempt", _fake_save_attempt)

    await _persist_attempts(
        db,
        task=task,
        attempts=[
            _attempt(
                action=ATTEMPT_ACTION_SEARCH,
                ok=False,
                retryable=True,
                error_code="seat_unavailable",
                error_message_safe="No reservable seats are available for this schedule.",
            )
        ],
    )

    assert saved == []


@pytest.mark.asyncio
async def test_persist_attempts_returns_early_when_attempt_list_is_empty(monkeypatch):
    db = _DB(previous_attempt=None)
    task = _task()

    async def _should_not_save(*_args, **_kwargs):  # noqa: ANN002, ANN003
        raise AssertionError("_save_attempt should not be called for empty attempts")

    monkeypatch.setattr("app.modules.train.worker._save_attempt", _should_not_save)

    await _persist_attempts(
        db,
        task=task,
        attempts=[],
    )


@pytest.mark.asyncio
async def test_persist_attempts_stores_transition_when_retry_signature_changes(monkeypatch):
    previous = _persisted_attempt(
        action=ATTEMPT_ACTION_SEARCH,
        ok=False,
        retryable=True,
        error_code="seat_unavailable",
        error_message_safe="No reservable seats are available for this schedule.",
    )
    db = _DB(previous_attempt=previous)
    task = _task()
    persisted: list[tuple[str, bool, bool, str | None]] = []

    async def _fake_save_attempt(_db, *, task, action, provider, ok, retryable, error_code, error_message_safe, duration_ms, meta_json_safe, started_at):  # noqa: ANN001, ANN003
        persisted.append((action, ok, retryable, error_code))
        return _persisted_attempt(
            action=action,
            ok=ok,
            retryable=retryable,
            error_code=error_code,
            error_message_safe=error_message_safe,
        )

    monkeypatch.setattr("app.modules.train.worker._save_attempt", _fake_save_attempt)

    await _persist_attempts(
        db,
        task=task,
        attempts=[
            _attempt(
                action=ATTEMPT_ACTION_SEARCH,
                ok=False,
                retryable=False,
                error_code="provider_unreachable",
                error_message_safe="provider outage",
            )
        ],
    )

    assert persisted == [(ATTEMPT_ACTION_SEARCH, False, False, "provider_unreachable")]


@pytest.mark.asyncio
async def test_persist_attempts_stores_duplicate_retry_noise_when_override_enabled(monkeypatch):
    previous = _persisted_attempt(
        action=ATTEMPT_ACTION_SEARCH,
        ok=False,
        retryable=True,
        error_code="seat_unavailable",
        error_message_safe="No reservable seats are available for this schedule.",
    )
    db = _DB(previous_attempt=previous)
    task = _task()
    persisted: list[tuple[str, bool, bool, str | None]] = []

    async def _fake_save_attempt(_db, *, task, action, provider, ok, retryable, error_code, error_message_safe, duration_ms, meta_json_safe, started_at):  # noqa: ANN001, ANN003
        persisted.append((action, ok, retryable, error_code))
        return _persisted_attempt(
            action=action,
            ok=ok,
            retryable=retryable,
            error_code=error_code,
            error_message_safe=error_message_safe,
        )

    monkeypatch.setattr(train_worker.settings, "train_persist_all_attempts", True)
    monkeypatch.setattr("app.modules.train.worker._save_attempt", _fake_save_attempt)

    await _persist_attempts(
        db,
        task=task,
        attempts=[
            _attempt(
                action=ATTEMPT_ACTION_SEARCH,
                ok=False,
                retryable=True,
                error_code="seat_unavailable",
                error_message_safe="No reservable seats are available for this schedule.",
            )
        ],
    )

    assert persisted == [(ATTEMPT_ACTION_SEARCH, False, True, "seat_unavailable")]


@pytest.mark.asyncio
async def test_persist_attempts_always_stores_pay_and_cancel_actions(monkeypatch):
    previous_pay = _persisted_attempt(
        action=ATTEMPT_ACTION_PAY,
        ok=False,
        retryable=True,
        error_code="provider_transport_error",
        error_message_safe="transport",
    )
    db = _DB(previous_attempt=previous_pay)
    task = _task()
    persisted_actions: list[str] = []

    async def _fake_save_attempt(_db, *, task, action, provider, ok, retryable, error_code, error_message_safe, duration_ms, meta_json_safe, started_at):  # noqa: ANN001, ANN003
        persisted_actions.append(action)
        return _persisted_attempt(
            action=action,
            ok=ok,
            retryable=retryable,
            error_code=error_code,
            error_message_safe=error_message_safe,
        )

    monkeypatch.setattr("app.modules.train.worker._save_attempt", _fake_save_attempt)

    await _persist_attempts(
        db,
        task=task,
        attempts=[
            _attempt(
                action=ATTEMPT_ACTION_PAY,
                ok=False,
                retryable=True,
                error_code="provider_transport_error",
                error_message_safe="transport",
            ),
            _attempt(
                action=ATTEMPT_ACTION_CANCEL,
                ok=True,
                retryable=False,
                error_code=None,
                error_message_safe=None,
            ),
        ],
    )

    assert persisted_actions == [ATTEMPT_ACTION_PAY, ATTEMPT_ACTION_CANCEL]
