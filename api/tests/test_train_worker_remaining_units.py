from __future__ import annotations

from contextlib import asynccontextmanager
from datetime import datetime, timedelta, timezone
from types import SimpleNamespace
from uuid import uuid4

import pytest

from app.db.models import Artifact, Secret, Task
from app.modules.train import worker as train_worker
from app.modules.train.providers.base import ProviderOutcome, ProviderSchedule


class _Limiter:
    async def acquire_provider_call(self, **_kwargs):  # noqa: ANN003
        return SimpleNamespace(waited_ms=3, rounds=1)


class _CountResult:
    def __init__(self, count: int) -> None:
        self._count = count

    def scalar_one(self) -> int:
        return self._count


class _TasksResult:
    def __init__(self, tasks: list[Task]) -> None:
        self._tasks = tasks

    def scalars(self):  # noqa: ANN201
        return SimpleNamespace(all=lambda: list(self._tasks))


class _RunDB:
    def __init__(self, task: Task | None, *, search_attempt_count: int = 0) -> None:
        self.task = task
        self.search_attempt_count = search_attempt_count
        self.commits = 0
        self.added: list[object] = []

    async def get(self, _model, _id):  # noqa: ANN001
        return self.task

    async def execute(self, _stmt):  # noqa: ANN001
        return _CountResult(self.search_attempt_count)

    async def commit(self) -> None:
        self.commits += 1

    def add(self, obj: object) -> None:
        self.added.append(obj)


class _RecoverDB:
    def __init__(self, tasks: list[Task]) -> None:
        self._tasks = tasks
        self.commits = 0

    async def execute(self, _stmt):  # noqa: ANN001
        return _TasksResult(self._tasks)

    async def commit(self) -> None:
        self.commits += 1


def _db_factory(db):
    @asynccontextmanager
    async def _factory():
        yield db

    return _factory


def _schedule(*, provider: str = "SRT", schedule_id: str = "SRT-1", rank: int = 1) -> train_worker.ReservationCandidate:
    schedule = ProviderSchedule(
        schedule_id=schedule_id,
        provider=provider,
        dep="수서",
        arr="부산",
        departure_at=datetime(2026, 2, 23, 9, 0, tzinfo=timezone.utc),
        arrival_at=datetime(2026, 2, 23, 11, 0, tzinfo=timezone.utc),
        train_no="101",
        availability={"general": True, "special": False},
        metadata={},
    )
    return train_worker.ReservationCandidate(
        provider=provider,
        rank=rank,
        schedule=schedule,
        seat_class_reserved="general",
        reservation_id=f"{provider}-PNR-{rank}",
        reserve_data={"journey_no": "001", "http_trace": {"authorization": "Bearer secret"}},
        client=SimpleNamespace(),
    )


def _task(*, state: str = "QUEUED", auto_pay: bool = True, deadline_hours: int = 2) -> Task:
    now = datetime(2026, 2, 22, 12, 0, tzinfo=timezone.utc)
    task = Task(
        user_id=uuid4(),
        module="train",
        state=state,
        deadline_at=now + timedelta(hours=deadline_hours),
        spec_json={
            "dep": "수서",
            "arr": "부산",
            "date": "2026-02-23",
            "seat_class": "general",
            "passengers": {"adults": 1, "children": 0},
            "auto_pay": auto_pay,
        },
        idempotency_key=f"task-{uuid4().hex}",
    )
    task.id = uuid4()
    task.created_at = now
    task.updated_at = now
    task.paused_at = None
    task.cancelled_at = None
    task.hidden_at = None
    task.completed_at = None
    task.failed_at = None
    task.artifacts = []
    task.attempts = []
    return task


def _ranked(provider: str = "SRT") -> list[dict[str, str | int]]:
    return [
        {
            "provider": provider,
            "rank": 1,
            "schedule_id": f"{provider}-1",
            "departure_at": datetime(2026, 2, 23, 9, 0, tzinfo=timezone.utc).isoformat(),
        }
    ]


def _open_ticket_artifact(*, provider: str = "SRT", reservation_id: str = "PNR-1") -> Artifact:
    return Artifact(
        task_id=uuid4(),
        module="train",
        kind="ticket",
        data_json_safe={
            "provider": provider,
            "reservation_id": reservation_id,
            "status": "reserved",
            "paid": False,
        },
    )


def test_worker_helper_branches_cover_remaining_non_async_paths(monkeypatch):
    assert train_worker._seat_preference_order("special") == ("special",)  # noqa: SLF001
    unknown_provider_schedule = ProviderSchedule(
        schedule_id="x",
        provider="MOCK",
        dep="수서",
        arr="부산",
        departure_at=datetime(2026, 2, 23, 9, 0, tzinfo=timezone.utc),
        arrival_at=datetime(2026, 2, 23, 11, 0, tzinfo=timezone.utc),
        train_no="x",
        availability={"general": False, "special": False},
        metadata={},
    )
    assert train_worker._wait_reserve_supported(unknown_provider_schedule) is False  # noqa: SLF001
    assert train_worker._is_provider_auth_required_error(ProviderOutcome(ok=True)) is False  # noqa: SLF001
    assert train_worker._is_non_payment_expiry_reserve_error(ProviderOutcome(ok=True)) is False  # noqa: SLF001

    fixed_now = datetime(2026, 2, 22, 12, 0, tzinfo=timezone.utc)
    monkeypatch.setattr(train_worker, "_utc_now_aware", lambda: fixed_now)
    assert train_worker._seconds_until_next_departure([{"departure_at": ""}, {"departure_at": "not-iso"}]) is None  # noqa: SLF001
    past_rows = [{"departure_at": (fixed_now - timedelta(minutes=1)).isoformat()}]
    assert train_worker._seconds_until_next_departure(past_rows) == 0.0  # noqa: SLF001

    with pytest.raises(ValueError):
        train_worker.fit_stretched_exp_params(1.0, 1.25)

    monkeypatch.setattr(train_worker, "POLL_CURVE_T48_SECONDS", train_worker.POLL_CURVE_T0_SECONDS)
    with pytest.raises(ValueError):
        train_worker.fit_stretched_exp_params(6.0, 1.25)

    monkeypatch.setattr(train_worker, "POLL_CURVE_T48_SECONDS", 48 * 60 * 60)
    monkeypatch.setattr(train_worker, "POLL_CURVE_ANCHOR_72H_MEAN_SECONDS", train_worker.POLL_CURVE_ANCHOR_48H_MEAN_SECONDS)
    with pytest.raises(ValueError):
        train_worker.fit_stretched_exp_params(6.0, 1.25)

    monkeypatch.setattr(train_worker, "POLL_CURVE_ANCHOR_72H_MEAN_SECONDS", 2.0)
    assert train_worker._mean_poll_delay_seconds(3600, 1.0) == 1.0  # noqa: SLF001

    real_log = train_worker.math.log
    log_calls = {"n": 0}

    def _log_with_invalid_transform(value: float) -> float:
        log_calls["n"] += 1
        if log_calls["n"] <= 2:
            return 0.0
        return real_log(value)

    monkeypatch.setattr(train_worker.math, "log", _log_with_invalid_transform)
    with pytest.raises(ValueError):
        train_worker.fit_stretched_exp_params(6.0, 1.25)

    normalized = train_worker._normalize_ranked_selection(  # noqa: SLF001
        {
            "provider": "SRT",
            "selected_trains_ranked": [
                {"provider": "BAD", "rank": "1", "schedule_id": "x", "departure_at": "2026-02-23T09:00:00+00:00"},
                {"provider": "SRT", "rank": "NaN", "schedule_id": "x", "departure_at": "2026-02-23T09:00:00+00:00"},
                {"provider": "SRT", "rank": "2", "schedule_id": "", "departure_at": "2026-02-23T09:00:00+00:00"},
                {"provider": "SRT", "rank": "3", "schedule_id": "ok", "departure_at": ""},
            ],
        }
    )
    assert normalized == []

    shutdown_event = SimpleNamespace(is_set=lambda: True)
    assert train_worker._is_shutdown_requested({"shutdown_event": shutdown_event}) is True  # noqa: SLF001
    assert train_worker._is_shutdown_requested({}) is False  # noqa: SLF001


@pytest.mark.asyncio
async def test_worker_notification_and_credentials_loader_branch_matrix(monkeypatch):
    task = _task()
    task.spec_json = {"notify": True, "notify_email_sent_at": "2026-02-22T00:00:00+00:00"}

    class _NotifyDB:
        async def get(self, _model, _id):  # noqa: ANN001
            return SimpleNamespace(email="user@example.com")

        async def commit(self) -> None:
            raise AssertionError("should not commit when notify_email_sent_at already set")

    await train_worker._enqueue_terminal_notification(_NotifyDB(), task=task, final_state="COMPLETED")  # noqa: SLF001

    task.spec_json = {"notify": True}

    class _NotifyDBNoEmail:
        async def get(self, _model, _id):  # noqa: ANN001
            return SimpleNamespace(email=None)

        async def commit(self) -> None:
            raise AssertionError("should not commit when user email is missing")

    await train_worker._enqueue_terminal_notification(_NotifyDBNoEmail(), task=task, final_state="FAILED")  # noqa: SLF001

    secret = Secret(
        user_id=uuid4(),
        kind="train_credentials_srt",
        ciphertext="c",
        nonce="n",
        wrapped_dek="w",
        dek_nonce="d",
        aad="a",
        kek_version=1,
    )

    class _SecretResult:
        def __init__(self, row):  # noqa: ANN001
            self._row = row

        def scalar_one_or_none(self):  # noqa: ANN201
            return self._row

    class _SecretDB:
        def __init__(self, row):  # noqa: ANN001
            self._row = row

        async def execute(self, _stmt):  # noqa: ANN001
            return _SecretResult(self._row)

    assert await train_worker._load_provider_credentials(_SecretDB(None), user_id=uuid4(), provider="SRT") is None  # noqa: SLF001

    monkeypatch.setattr(train_worker, "decrypt_secret", lambda _secret: (_ for _ in ()).throw(RuntimeError("boom")))
    assert await train_worker._load_provider_credentials(_SecretDB(secret), user_id=uuid4(), provider="SRT") is None  # noqa: SLF001

    monkeypatch.setattr(train_worker, "decrypt_secret", lambda _secret: {"username": " ", "password": ""})
    assert await train_worker._load_provider_credentials(_SecretDB(secret), user_id=uuid4(), provider="SRT") is None  # noqa: SLF001

    monkeypatch.setattr(train_worker, "decrypt_secret", lambda _secret: {"username": "user", "password": "pw"})
    loaded = await train_worker._load_provider_credentials(_SecretDB(secret), user_id=uuid4(), provider="SRT")  # noqa: SLF001
    assert loaded == {"username": "user", "password": "pw"}


@pytest.mark.asyncio
async def test_provider_search_and_reserve_remaining_relogin_and_missing_id_branches(monkeypatch):
    spec = {
        "dep": "수서",
        "arr": "부산",
        "date": "2026-02-23",
        "seat_class": "general",
        "passengers": {"adults": 1, "children": 0},
        "auto_pay": True,
    }

    class _LoginFailClient:
        async def login(self, **_kwargs):  # noqa: ANN003
            return ProviderOutcome(ok=False, retryable=False, error_code="bad_login", error_message_safe="bad")

    monkeypatch.setattr(train_worker, "get_provider_client", lambda _provider: _LoginFailClient())
    login_failed = await train_worker._provider_search_and_reserve(  # noqa: SLF001
        provider="SRT",
        ranked_for_provider=_ranked(),
        spec=spec,
        task_user_id=uuid4(),
        credentials={"username": "u", "password": "p"},
        limiter=_Limiter(),
    )
    assert login_failed.attempts[-1].error_code == "bad_login"

    class _SearchRaisesClient:
        async def login(self, **_kwargs):  # noqa: ANN003
            return ProviderOutcome(ok=True)

        async def search(self, **_kwargs):  # noqa: ANN003
            raise RuntimeError("transport")

    monkeypatch.setattr(train_worker, "get_provider_client", lambda _provider: _SearchRaisesClient())
    search_raises = await train_worker._provider_search_and_reserve(  # noqa: SLF001
        provider="SRT",
        ranked_for_provider=_ranked(),
        spec=spec,
        task_user_id=uuid4(),
        credentials={"username": "u", "password": "p"},
        limiter=_Limiter(),
    )
    assert search_raises.attempts[-1].error_code == "provider_transport_error"

    schedule = ProviderSchedule(
        schedule_id="SRT-1",
        provider="SRT",
        dep="수서",
        arr="부산",
        departure_at=datetime(2026, 2, 23, 9, 0, tzinfo=timezone.utc),
        arrival_at=datetime(2026, 2, 23, 11, 0, tzinfo=timezone.utc),
        train_no="101",
        availability={"general": True, "special": False},
        metadata={},
    )

    class _ReserveAuthThenReloginRaises:
        def __init__(self) -> None:
            self.reserve_calls = 0

        async def login(self, **_kwargs):  # noqa: ANN003
            if self.reserve_calls == 0:
                return ProviderOutcome(ok=True)
            raise RuntimeError("relogin transport")

        async def search(self, **_kwargs):  # noqa: ANN003
            return ProviderOutcome(ok=True, data={"schedules": [schedule]})

        async def reserve(self, **_kwargs):  # noqa: ANN003
            self.reserve_calls += 1
            return ProviderOutcome(ok=False, retryable=False, error_code="login_required", error_message_safe="login needed")

    monkeypatch.setattr(train_worker, "get_provider_client", lambda _provider: _ReserveAuthThenReloginRaises())
    relogin_transport = await train_worker._provider_search_and_reserve(  # noqa: SLF001
        provider="SRT",
        ranked_for_provider=_ranked(),
        spec=spec,
        task_user_id=uuid4(),
        credentials={"username": "u", "password": "p"},
        limiter=_Limiter(),
    )
    assert relogin_transport.attempts[-1].error_code == "provider_relogin_transport_error"

    class _ReserveAuthThenReloginFail:
        def __init__(self) -> None:
            self.login_calls = 0

        async def login(self, **_kwargs):  # noqa: ANN003
            self.login_calls += 1
            if self.login_calls == 1:
                return ProviderOutcome(ok=True)
            return ProviderOutcome(ok=False, error_code="relogin_bad", error_message_safe="relogin bad")

        async def search(self, **_kwargs):  # noqa: ANN003
            return ProviderOutcome(ok=True, data={"schedules": [schedule]})

        async def reserve(self, **_kwargs):  # noqa: ANN003
            return ProviderOutcome(ok=False, retryable=False, error_code="login_required", error_message_safe="login needed")

    monkeypatch.setattr(train_worker, "get_provider_client", lambda _provider: _ReserveAuthThenReloginFail())
    relogin_failed = await train_worker._provider_search_and_reserve(  # noqa: SLF001
        provider="SRT",
        ranked_for_provider=_ranked(),
        spec=spec,
        task_user_id=uuid4(),
        credentials={"username": "u", "password": "p"},
        limiter=_Limiter(),
    )
    assert relogin_failed.attempts[-1].error_code == "relogin_bad"

    class _ReserveAuthThenRetryRaises:
        def __init__(self) -> None:
            self.login_calls = 0
            self.reserve_calls = 0

        async def login(self, **_kwargs):  # noqa: ANN003
            self.login_calls += 1
            return ProviderOutcome(ok=True)

        async def search(self, **_kwargs):  # noqa: ANN003
            return ProviderOutcome(ok=True, data={"schedules": [schedule]})

        async def reserve(self, **_kwargs):  # noqa: ANN003
            self.reserve_calls += 1
            if self.reserve_calls == 1:
                return ProviderOutcome(ok=False, retryable=False, error_code="login_required", error_message_safe="login needed")
            raise RuntimeError("retry transport")

    monkeypatch.setattr(train_worker, "get_provider_client", lambda _provider: _ReserveAuthThenRetryRaises())
    retry_raises = await train_worker._provider_search_and_reserve(  # noqa: SLF001
        provider="SRT",
        ranked_for_provider=_ranked(),
        spec=spec,
        task_user_id=uuid4(),
        credentials={"username": "u", "password": "p"},
        limiter=_Limiter(),
    )
    assert retry_raises.attempts[-1].error_code == "provider_transport_error"

    class _ReserveMissingId:
        async def login(self, **_kwargs):  # noqa: ANN003
            return ProviderOutcome(ok=True)

        async def search(self, **_kwargs):  # noqa: ANN003
            return ProviderOutcome(ok=True, data={"schedules": [schedule]})

        async def reserve(self, **_kwargs):  # noqa: ANN003
            return ProviderOutcome(ok=True, data={})

    monkeypatch.setattr(train_worker, "get_provider_client", lambda _provider: _ReserveMissingId())
    missing_id = await train_worker._provider_search_and_reserve(  # noqa: SLF001
        provider="SRT",
        ranked_for_provider=_ranked(),
        spec=spec,
        task_user_id=uuid4(),
        credentials={"username": "u", "password": "p"},
        limiter=_Limiter(),
    )
    assert missing_id.attempts[-1].error_code == "reservation_id_missing"


@pytest.mark.asyncio
async def test_attempt_pay_and_login_for_worker_remaining_relogin_branches(monkeypatch):
    class _PayClientReloginRaises:
        async def pay(self, **_kwargs):  # noqa: ANN003
            return ProviderOutcome(ok=False, retryable=False, error_code="login_required", error_message_safe="login needed")

        async def login(self, **_kwargs):  # noqa: ANN003
            raise RuntimeError("relogin transport")

    attempt_raise, outcome_raise = await train_worker._attempt_pay_reservation(  # noqa: SLF001
        provider="SRT",
        client=_PayClientReloginRaises(),
        reservation_id="PNR-1",
        task_user_id=uuid4(),
        limiter=_Limiter(),
        payment_card={"token": "tok"},
        credentials={"username": "u", "password": "p"},
    )
    assert attempt_raise.meta_json_safe["auth_relogin_retry"] is True
    assert outcome_raise.error_code == "provider_relogin_transport_error"

    class _PayClientReloginFail:
        async def pay(self, **_kwargs):  # noqa: ANN003
            return ProviderOutcome(ok=False, retryable=False, error_code="login_required", error_message_safe="login needed")

        async def login(self, **_kwargs):  # noqa: ANN003
            return ProviderOutcome(ok=False, error_code="relogin_failed", error_message_safe="bad login")

    _attempt, outcome_failed = await train_worker._attempt_pay_reservation(  # noqa: SLF001
        provider="SRT",
        client=_PayClientReloginFail(),
        reservation_id="PNR-1",
        task_user_id=uuid4(),
        limiter=_Limiter(),
        payment_card={"token": "tok"},
        credentials={"username": "u", "password": "p"},
    )
    assert outcome_failed.error_code == "relogin_failed"

    async def _no_credentials(*_args, **_kwargs):  # noqa: ANN002, ANN003
        return None

    monkeypatch.setattr(train_worker, "_load_provider_credentials", _no_credentials)
    missing_client, missing_attempt, missing_retryable = await train_worker._login_provider_client_for_worker(  # noqa: SLF001
        SimpleNamespace(),
        task=_task(),
        provider="SRT",
    )
    assert missing_client is None
    assert missing_attempt is not None
    assert missing_attempt.error_code == "credentials_missing"
    assert missing_retryable is False


@pytest.mark.asyncio
async def test_login_provider_for_worker_exception_and_login_failed_branches(monkeypatch):
    task = _task()

    async def _load_creds(*_args, **_kwargs):  # noqa: ANN002, ANN003
        return {"username": "u", "password": "p"}

    monkeypatch.setattr(train_worker, "_load_provider_credentials", _load_creds)

    class _LoginRaises:
        async def login(self, **_kwargs):  # noqa: ANN003
            raise RuntimeError("transport")

    monkeypatch.setattr(train_worker, "get_provider_client", lambda _provider: _LoginRaises())
    client_none, attempt_transport, retryable_transport = await train_worker._login_provider_client_for_worker(  # noqa: SLF001
        SimpleNamespace(),
        task=task,
        provider="SRT",
    )
    assert client_none is None
    assert attempt_transport is not None
    assert retryable_transport is True

    class _LoginFail:
        async def login(self, **_kwargs):  # noqa: ANN003
            return ProviderOutcome(ok=False, retryable=False, error_code="bad_login", error_message_safe="bad")

    monkeypatch.setattr(train_worker, "get_provider_client", lambda _provider: _LoginFail())
    client_none2, attempt_bad, retryable_bad = await train_worker._login_provider_client_for_worker(  # noqa: SLF001
        SimpleNamespace(),
        task=task,
        provider="SRT",
    )
    assert client_none2 is None
    assert attempt_bad is not None
    assert attempt_bad.error_code == "bad_login"
    assert retryable_bad is False


def test_build_ticket_data_includes_optional_reservation_fields(monkeypatch):
    candidate = _schedule()
    candidate.reserve_data.update({"journey_cnt": "01", "rsv_chg_no": "00000", "wct_no": "abc"})
    monkeypatch.setattr(train_worker, "validate_safe_metadata", lambda payload: payload)

    payload = train_worker._build_ticket_data(  # noqa: SLF001
        candidate=candidate,
        spec={"seat_class": "general"},
        paid=False,
    )
    assert payload["journey_no"] == "001"
    assert payload["journey_cnt"] == "01"
    assert payload["rsv_chg_no"] == "00000"
    assert payload["wct_no"] == "abc"


@pytest.mark.asyncio
async def test_run_train_task_register_unregister_hooks_are_invoked(monkeypatch):
    calls: list[tuple[str, str]] = []

    def _register(task_id: str) -> None:
        calls.append(("register", task_id))

    def _unregister(task_id: str) -> None:
        calls.append(("unregister", task_id))

    async def _inner(*_args, **_kwargs):  # noqa: ANN002, ANN003
        return None

    monkeypatch.setattr(train_worker, "_run_train_task_inner", _inner)
    task_id = str(uuid4())
    await train_worker.run_train_task(  # noqa: SLF001
        {"db_factory": object(), "redis": object(), "register_in_flight": _register, "unregister_in_flight": _unregister},
        task_id,
    )
    assert calls == [("register", task_id), ("unregister", task_id)]


@pytest.mark.asyncio
async def test_run_train_task_inner_early_exit_branch_matrix(monkeypatch):
    fixed_now = datetime(2026, 2, 22, 12, 0, tzinfo=timezone.utc)
    monkeypatch.setattr(train_worker, "_utc_now_aware", lambda: fixed_now)

    async def _no_mark(*_args, **_kwargs):  # noqa: ANN002, ANN003
        return None

    monkeypatch.setattr(train_worker, "_mark_failed", _no_mark)
    monkeypatch.setattr(train_worker, "_mark_expired", _no_mark)
    monkeypatch.setattr(train_worker, "_mark_completed", _no_mark)

    # task missing / module mismatch
    none_db = _RunDB(None)
    await train_worker._run_train_task_inner({}, str(uuid4()), _db_factory(none_db), object(), _Limiter())  # noqa: SLF001

    wrong_module = _task()
    wrong_module.module = "restaurant"
    wrong_db = _RunDB(wrong_module)
    await train_worker._run_train_task_inner({}, str(wrong_module.id), _db_factory(wrong_db), object(), _Limiter())  # noqa: SLF001

    # paused / hidden / paused_at mismatch / cancelled
    paused = _task(state="PAUSED")
    await train_worker._run_train_task_inner({}, str(paused.id), _db_factory(_RunDB(paused)), object(), _Limiter())  # noqa: SLF001

    hidden = _task()
    hidden.hidden_at = datetime(2026, 2, 22, 13, 0, tzinfo=timezone.utc)
    await train_worker._run_train_task_inner({}, str(hidden.id), _db_factory(_RunDB(hidden)), object(), _Limiter())  # noqa: SLF001

    paused_mismatch = _task(state="RUNNING")
    paused_mismatch.paused_at = datetime(2026, 2, 22, 13, 0, tzinfo=timezone.utc)
    paused_mismatch_db = _RunDB(paused_mismatch)
    await train_worker._run_train_task_inner({}, str(paused_mismatch.id), _db_factory(paused_mismatch_db), object(), _Limiter())  # noqa: SLF001
    assert paused_mismatch.state == "PAUSED"
    assert paused_mismatch_db.commits >= 1

    cancelled = _task(state="RUNNING")
    cancelled.cancelled_at = datetime(2026, 2, 22, 13, 0, tzinfo=timezone.utc)
    cancelled_db = _RunDB(cancelled)
    await train_worker._run_train_task_inner({}, str(cancelled.id), _db_factory(cancelled_db), object(), _Limiter())  # noqa: SLF001
    assert cancelled.state == "CANCELLED"

    # no ranked + no providers
    failed_calls: list[str] = []

    async def _mark_failed(db, task):  # noqa: ANN001
        failed_calls.append(str(task.id))

    monkeypatch.setattr(train_worker, "_mark_failed", _mark_failed)
    monkeypatch.setattr(train_worker, "_normalize_ranked_selection", lambda _spec: [])
    no_ranked = _task()
    await train_worker._run_train_task_inner({}, str(no_ranked.id), _db_factory(_RunDB(no_ranked)), object(), _Limiter())  # noqa: SLF001
    assert failed_calls

    monkeypatch.setattr(train_worker, "_normalize_ranked_selection", lambda _spec: [{"provider": "SRT", "rank": 1, "schedule_id": "x"}])
    monkeypatch.setattr("builtins.sorted", lambda _iterable, **_kwargs: [])
    no_provider = _task()
    await train_worker._run_train_task_inner({}, str(no_provider.id), _db_factory(_RunDB(no_provider)), object(), _Limiter())  # noqa: SLF001
    assert len(failed_calls) >= 2


@pytest.mark.asyncio
async def test_run_train_task_inner_open_ticket_branch_matrix(monkeypatch):
    fixed_now = datetime(2026, 2, 22, 12, 0, tzinfo=timezone.utc)
    monkeypatch.setattr(train_worker, "_utc_now_aware", lambda: fixed_now)
    monkeypatch.setattr(train_worker, "utc_now", lambda: fixed_now)
    monkeypatch.setattr(train_worker, "_normalize_ranked_selection", lambda _spec: _ranked())
    monkeypatch.setattr(train_worker, "_seconds_until_next_departure", lambda _rows: 3600.0)

    async def _persist_noop(*_args, **_kwargs):  # noqa: ANN002, ANN003
        return None

    monkeypatch.setattr(train_worker, "_persist_attempts", _persist_noop)
    monkeypatch.setattr(train_worker, "_poll_delay_seconds", lambda *_args, **_kwargs: 1.0)
    monkeypatch.setattr(train_worker, "validate_safe_metadata", lambda payload: payload)
    monkeypatch.setattr(train_worker, "redact_sensitive", lambda payload: payload)
    async def _payment_card(*_args, **_kwargs):  # noqa: ANN002, ANN003
        return {"token": "tok"}

    monkeypatch.setattr(train_worker, "get_payment_card_for_execution", _payment_card)

    completed_calls: list[str] = []
    failed_calls: list[str] = []
    retry_calls: list[str] = []

    async def _mark_completed(_db, task):  # noqa: ANN001
        completed_calls.append(str(task.id))

    async def _mark_failed(_db, task):  # noqa: ANN001
        failed_calls.append(str(task.id))

    async def _schedule_retry(_db, task, _delay):  # noqa: ANN001
        retry_calls.append(str(task.id))

    monkeypatch.setattr(train_worker, "_mark_completed", _mark_completed)
    monkeypatch.setattr(train_worker, "_mark_failed", _mark_failed)
    monkeypatch.setattr(train_worker, "_schedule_retry", _schedule_retry)

    # auto_pay disabled -> completed
    auto_pay_off = _task(auto_pay=False)
    async def _load_open_ticket(*_args, **_kwargs):  # noqa: ANN002, ANN003
        return [_open_ticket_artifact()]

    monkeypatch.setattr(train_worker, "_load_ticket_artifacts", _load_open_ticket)
    await train_worker._run_train_task_inner({}, str(auto_pay_off.id), _db_factory(_RunDB(auto_pay_off)), object(), _Limiter())  # noqa: SLF001
    assert str(auto_pay_off.id) in completed_calls

    # invalid provider/reservation -> failed
    invalid_ticket_task = _task(auto_pay=True)
    async def _load_invalid_open_ticket(*_args, **_kwargs):  # noqa: ANN002, ANN003
        return [_open_ticket_artifact(provider="OTHER")]

    monkeypatch.setattr(train_worker, "_load_ticket_artifacts", _load_invalid_open_ticket)
    await train_worker._run_train_task_inner(  # noqa: SLF001
        {},
        str(invalid_ticket_task.id),
        _db_factory(_RunDB(invalid_ticket_task)),
        object(),
        _Limiter(),
    )
    assert str(invalid_ticket_task.id) in failed_calls

    # login attempt branch: retry + fail
    retry_login_task = _task(auto_pay=True)
    login_attempt = train_worker.PendingAttempt(
        action="search",
        provider="SRT",
        ok=False,
        retryable=True,
        error_code="provider_login_transport_error",
        error_message_safe="boom",
        duration_ms=1,
        meta_json_safe={},
        started_at=fixed_now,
    )
    monkeypatch.setattr(train_worker, "_load_ticket_artifacts", _load_open_ticket)

    async def _login_with_retry(*_args, **_kwargs):  # noqa: ANN002, ANN003
        return None, login_attempt, True

    monkeypatch.setattr(train_worker, "_login_provider_client_for_worker", _login_with_retry)
    await train_worker._run_train_task_inner({}, str(retry_login_task.id), _db_factory(_RunDB(retry_login_task)), object(), _Limiter())  # noqa: SLF001
    assert str(retry_login_task.id) in retry_calls

    fail_login_task = _task(auto_pay=True)
    async def _login_without_retry(*_args, **_kwargs):  # noqa: ANN002, ANN003
        return None, login_attempt, False

    monkeypatch.setattr(train_worker, "_login_provider_client_for_worker", _login_without_retry)
    await train_worker._run_train_task_inner({}, str(fail_login_task.id), _db_factory(_RunDB(fail_login_task)), object(), _Limiter())  # noqa: SLF001
    assert str(fail_login_task.id) in failed_calls

    # pay failure branch
    pay_fail_task = _task(auto_pay=True)
    async def _login_success(*_args, **_kwargs):  # noqa: ANN002, ANN003
        return SimpleNamespace(), None, False

    async def _provider_creds(*_args, **_kwargs):  # noqa: ANN002, ANN003
        return {"username": "u", "password": "p"}

    monkeypatch.setattr(train_worker, "_login_provider_client_for_worker", _login_success)
    monkeypatch.setattr(train_worker, "_load_provider_credentials", _provider_creds)
    pay_attempt = train_worker.PendingAttempt(
        action="pay",
        provider="SRT",
        ok=False,
        retryable=False,
        error_code="pay_failed",
        error_message_safe="pay failed",
        duration_ms=1,
        meta_json_safe={},
        started_at=fixed_now,
    )
    async def _attempt_pay_fail(*_args, **_kwargs):  # noqa: ANN002, ANN003
        return pay_attempt, ProviderOutcome(ok=False, retryable=False, error_code="pay_failed")

    monkeypatch.setattr(train_worker, "_attempt_pay_reservation", _attempt_pay_fail)
    await train_worker._run_train_task_inner({}, str(pay_fail_task.id), _db_factory(_RunDB(pay_fail_task)), object(), _Limiter())  # noqa: SLF001
    assert str(pay_fail_task.id) in failed_calls

    # pay failure retryable branch schedules retry
    pay_retry_task = _task(auto_pay=True)
    pay_retry_attempt = train_worker.PendingAttempt(
        action="pay",
        provider="SRT",
        ok=False,
        retryable=True,
        error_code="pay_retry",
        error_message_safe="retry",
        duration_ms=1,
        meta_json_safe={},
        started_at=fixed_now,
    )

    async def _attempt_pay_retry(*_args, **_kwargs):  # noqa: ANN002, ANN003
        return pay_retry_attempt, ProviderOutcome(ok=False, retryable=True, error_code="pay_retry")

    monkeypatch.setattr(train_worker, "_attempt_pay_reservation", _attempt_pay_retry)
    await train_worker._run_train_task_inner({}, str(pay_retry_task.id), _db_factory(_RunDB(pay_retry_task)), object(), _Limiter())  # noqa: SLF001
    assert str(pay_retry_task.id) in retry_calls

    # post-pay sync fallback exception branch (fetch sync throws)
    pay_ok_task = _task(auto_pay=True)
    open_artifact = _open_ticket_artifact()
    async def _load_pay_ok_open_ticket(*_args, **_kwargs):  # noqa: ANN002, ANN003
        return [open_artifact]

    monkeypatch.setattr(train_worker, "_load_ticket_artifacts", _load_pay_ok_open_ticket)
    pay_ok_attempt = train_worker.PendingAttempt(
        action="pay",
        provider="SRT",
        ok=True,
        retryable=False,
        error_code=None,
        error_message_safe=None,
        duration_ms=1,
        meta_json_safe={},
        started_at=fixed_now,
    )
    async def _attempt_pay_success(*_args, **_kwargs):  # noqa: ANN002, ANN003
        return pay_ok_attempt, ProviderOutcome(ok=True, data={"payment_id": "pay-1", "ticket_no": "T1", "http_trace": {"x": 1}})

    monkeypatch.setattr(train_worker, "_attempt_pay_reservation", _attempt_pay_success)

    async def _sync_boom(*_args, **_kwargs):  # noqa: ANN002, ANN003
        raise RuntimeError("sync error")

    monkeypatch.setattr(train_worker, "fetch_ticket_sync_snapshot", _sync_boom)
    await train_worker._run_train_task_inner({}, str(pay_ok_task.id), _db_factory(_RunDB(pay_ok_task)), object(), _Limiter())  # noqa: SLF001
    assert str(pay_ok_task.id) in completed_calls


@pytest.mark.asyncio
async def test_run_train_task_inner_provider_loop_and_post_reserve_branches(monkeypatch):
    fixed_now = datetime(2026, 2, 22, 12, 0, tzinfo=timezone.utc)
    monkeypatch.setattr(train_worker, "_poll_delay_seconds", lambda *_args, **_kwargs: 1.0)
    async def _persist_noop(*_args, **_kwargs):  # noqa: ANN002, ANN003
        return None

    monkeypatch.setattr(train_worker, "_persist_attempts", _persist_noop)
    monkeypatch.setattr(train_worker, "validate_safe_metadata", lambda payload: payload)
    async def _payment_card(*_args, **_kwargs):  # noqa: ANN002, ANN003
        return {"token": "tok"}

    monkeypatch.setattr(train_worker, "get_payment_card_for_execution", _payment_card)
    async def _load_no_tickets(*_args, **_kwargs):  # noqa: ANN002, ANN003
        return []

    monkeypatch.setattr(train_worker, "_load_ticket_artifacts", _load_no_tickets)
    monkeypatch.setattr(train_worker, "_normalize_ranked_selection", lambda _spec: _ranked("SRT"))
    monkeypatch.setattr(train_worker, "_seconds_until_next_departure", lambda _rows: 3600.0)

    completed_calls: list[str] = []
    failed_calls: list[str] = []
    expired_calls: list[str] = []
    retry_calls: list[str] = []

    async def _mark_completed(_db, task):  # noqa: ANN001
        completed_calls.append(str(task.id))

    async def _mark_failed(_db, task):  # noqa: ANN001
        failed_calls.append(str(task.id))

    async def _mark_expired(_db, task):  # noqa: ANN001
        expired_calls.append(str(task.id))

    async def _schedule_retry(_db, task, _delay):  # noqa: ANN001
        retry_calls.append(str(task.id))

    monkeypatch.setattr(train_worker, "_mark_completed", _mark_completed)
    monkeypatch.setattr(train_worker, "_mark_failed", _mark_failed)
    monkeypatch.setattr(train_worker, "_mark_expired", _mark_expired)
    monkeypatch.setattr(train_worker, "_schedule_retry", _schedule_retry)

    # missing credentials -> no candidate -> non-retryable failure path
    task_missing_creds = _task(auto_pay=True)
    monkeypatch.setattr(train_worker, "_utc_now_aware", lambda: fixed_now)
    async def _no_provider_credentials(*_args, **_kwargs):  # noqa: ANN002, ANN003
        return None

    monkeypatch.setattr(train_worker, "_load_provider_credentials", _no_provider_credentials)
    await train_worker._run_train_task_inner(  # noqa: SLF001
        {},
        str(task_missing_creds.id),
        _db_factory(_RunDB(task_missing_creds)),
        object(),
        _Limiter(),
    )
    assert str(task_missing_creds.id) in failed_calls

    # deadline reached after provider attempts branch
    task_deadline_after_attempts = _task(auto_pay=True)
    time_calls = {"n": 0}

    def _now_switch() -> datetime:
        time_calls["n"] += 1
        if time_calls["n"] == 1:
            return fixed_now
        return fixed_now + timedelta(hours=3)

    monkeypatch.setattr(train_worker, "_utc_now_aware", _now_switch)
    async def _provider_credentials(*_args, **_kwargs):  # noqa: ANN002, ANN003
        return {"username": "u", "password": "p"}

    async def _provider_retryable_none(*_args, **_kwargs):  # noqa: ANN002, ANN003
        return train_worker.ProviderExecutionResult(provider="SRT", attempts=[], candidate=None, retryable=True)

    monkeypatch.setattr(train_worker, "_load_provider_credentials", _provider_credentials)
    monkeypatch.setattr(train_worker, "_provider_search_and_reserve", _provider_retryable_none)
    await train_worker._run_train_task_inner(  # noqa: SLF001
        {},
        str(task_deadline_after_attempts.id),
        _db_factory(_RunDB(task_deadline_after_attempts)),
        object(),
        _Limiter(),
    )
    assert str(task_deadline_after_attempts.id) in expired_calls

    # winner sync-exception path + auto_pay disabled
    task_sync_exception = _task(auto_pay=False)
    monkeypatch.setattr(train_worker, "_utc_now_aware", lambda: fixed_now)
    winner = _schedule(provider="SRT", schedule_id="SRT-1", rank=1)
    async def _provider_winner(*_args, **_kwargs):  # noqa: ANN002, ANN003
        return train_worker.ProviderExecutionResult("SRT", [], winner, False)

    monkeypatch.setattr(train_worker, "_provider_search_and_reserve", _provider_winner)

    async def _sync_exception(*_args, **_kwargs):  # noqa: ANN002, ANN003
        raise RuntimeError("sync")

    monkeypatch.setattr(train_worker, "fetch_ticket_sync_snapshot", _sync_exception)
    monkeypatch.setattr(train_worker, "_load_provider_credentials", _provider_credentials)
    await train_worker._run_train_task_inner(  # noqa: SLF001
        {},
        str(task_sync_exception.id),
        _db_factory(_RunDB(task_sync_exception)),
        object(),
        _Limiter(),
    )
    assert str(task_sync_exception.id) in completed_calls

    # loser cancel failure branch
    task_cancel_loser_fail = _task(auto_pay=False)
    winner = _schedule(provider="SRT", schedule_id="SRT-1", rank=1)
    loser = _schedule(provider="KTX", schedule_id="KTX-1", rank=2)
    monkeypatch.setattr(train_worker, "_normalize_ranked_selection", lambda _spec: _ranked("SRT") + _ranked("KTX"))

    async def _provider_result(*_args, **kwargs):  # noqa: ANN002, ANN003
        if kwargs["provider"] == "SRT":
            return train_worker.ProviderExecutionResult("SRT", [], winner, False)
        return train_worker.ProviderExecutionResult("KTX", [], loser, False)

    monkeypatch.setattr(train_worker, "_provider_search_and_reserve", _provider_result)
    monkeypatch.setattr(train_worker, "_load_provider_credentials", _provider_credentials)
    async def _attempt_cancel_failed(*_args, **_kwargs):  # noqa: ANN002, ANN003
        return (
            train_worker.PendingAttempt("cancel", "KTX", False, False, "cancel_failed", "x", 1, {}, fixed_now),
            ProviderOutcome(ok=False, retryable=False, error_code="cancel_failed"),
        )

    monkeypatch.setattr(train_worker, "_attempt_cancel_candidate", _attempt_cancel_failed)
    await train_worker._run_train_task_inner(  # noqa: SLF001
        {},
        str(task_cancel_loser_fail.id),
        _db_factory(_RunDB(task_cancel_loser_fail)),
        object(),
        _Limiter(),
    )
    assert str(task_cancel_loser_fail.id) in failed_calls

    # loser cancel not_supported should continue without failure
    task_cancel_not_supported = _task(auto_pay=False)

    async def _attempt_cancel_not_supported(*_args, **_kwargs):  # noqa: ANN002, ANN003
        return (
            train_worker.PendingAttempt("cancel", "KTX", False, False, "not_supported", "n/a", 1, {}, fixed_now),
            ProviderOutcome(ok=False, retryable=False, error_code="not_supported"),
        )

    monkeypatch.setattr(train_worker, "_attempt_cancel_candidate", _attempt_cancel_not_supported)
    await train_worker._run_train_task_inner(  # noqa: SLF001
        {},
        str(task_cancel_not_supported.id),
        _db_factory(_RunDB(task_cancel_not_supported)),
        object(),
        _Limiter(),
    )
    assert str(task_cancel_not_supported.id) in completed_calls

    # loser cancel retryable should schedule retry
    task_cancel_retryable = _task(auto_pay=False)

    async def _attempt_cancel_retryable(*_args, **_kwargs):  # noqa: ANN002, ANN003
        return (
            train_worker.PendingAttempt("cancel", "KTX", False, True, "cancel_retry", "retry", 1, {}, fixed_now),
            ProviderOutcome(ok=False, retryable=True, error_code="cancel_retry"),
        )

    monkeypatch.setattr(train_worker, "_attempt_cancel_candidate", _attempt_cancel_retryable)
    await train_worker._run_train_task_inner(  # noqa: SLF001
        {},
        str(task_cancel_retryable.id),
        _db_factory(_RunDB(task_cancel_retryable)),
        object(),
        _Limiter(),
    )
    assert str(task_cancel_retryable.id) in retry_calls

    # pay fail branch after winner
    task_pay_fail = _task(auto_pay=True)
    monkeypatch.setattr(train_worker, "_normalize_ranked_selection", lambda _spec: _ranked("SRT"))
    monkeypatch.setattr(train_worker, "_provider_search_and_reserve", _provider_winner)
    async def _no_loser_cancel(*_args, **_kwargs):  # noqa: ANN002, ANN003
        raise AssertionError("no losers")

    async def _attempt_pay_failed(*_args, **_kwargs):  # noqa: ANN002, ANN003
        return (
            train_worker.PendingAttempt("pay", "SRT", False, False, "pay_failed", "x", 1, {}, fixed_now),
            ProviderOutcome(ok=False, retryable=False, error_code="pay_failed"),
        )

    monkeypatch.setattr(train_worker, "_attempt_cancel_candidate", _no_loser_cancel)
    monkeypatch.setattr(train_worker, "_attempt_pay_reservation", _attempt_pay_failed)
    await train_worker._run_train_task_inner(
        {},
        str(task_pay_fail.id),
        _db_factory(_RunDB(task_pay_fail)),
        object(),
        _Limiter(),
    )
    assert str(task_pay_fail.id) in failed_calls

    # pay retry branch schedules retry
    task_pay_retry = _task(auto_pay=True)

    async def _attempt_pay_retry(*_args, **_kwargs):  # noqa: ANN002, ANN003
        return (
            train_worker.PendingAttempt("pay", "SRT", False, True, "pay_retry", "retry", 1, {}, fixed_now),
            ProviderOutcome(ok=False, retryable=True, error_code="pay_retry"),
        )

    monkeypatch.setattr(train_worker, "_attempt_pay_reservation", _attempt_pay_retry)
    await train_worker._run_train_task_inner(
        {},
        str(task_pay_retry.id),
        _db_factory(_RunDB(task_pay_retry)),
        object(),
        _Limiter(),
    )
    assert str(task_pay_retry.id) in retry_calls

    # post-pay sync exception branch
    task_post_pay_sync_exception = _task(auto_pay=True)
    sync_calls = {"n": 0}

    async def _sync_then_raise(*_args, **_kwargs):  # noqa: ANN002, ANN003
        sync_calls["n"] += 1
        if sync_calls["n"] == 1:
            return {}
        raise RuntimeError("post-pay sync failed")

    monkeypatch.setattr(train_worker, "fetch_ticket_sync_snapshot", _sync_then_raise)
    async def _attempt_pay_ok(*_args, **_kwargs):  # noqa: ANN002, ANN003
        return (
            train_worker.PendingAttempt("pay", "SRT", True, False, None, None, 1, {}, fixed_now),
            ProviderOutcome(ok=True, data={"payment_id": "pay-1", "ticket_no": "T-1"}),
        )

    monkeypatch.setattr(train_worker, "_attempt_pay_reservation", _attempt_pay_ok)
    await train_worker._run_train_task_inner(
        {},
        str(task_post_pay_sync_exception.id),
        _db_factory(_RunDB(task_post_pay_sync_exception)),
        object(),
        _Limiter(),
    )
    assert str(task_post_pay_sync_exception.id) in completed_calls


@pytest.mark.asyncio
async def test_enqueue_recoverable_tasks_handles_paused_stale_and_active_paths(monkeypatch):
    now = datetime(2026, 2, 22, 12, 0, tzinfo=timezone.utc)
    monkeypatch.setattr(train_worker, "_utc_now_aware", lambda: now)

    paused_task = _task(state="PAUSED")
    paused_task.paused_at = now - timedelta(minutes=2)

    stale_task = _task(state="RUNNING")
    stale_task.updated_at = now - timedelta(seconds=train_worker.STALE_TASK_THRESHOLD_SECONDS + 30)

    fresh_task = _task(state="QUEUED")
    fresh_task.updated_at = now - timedelta(seconds=10)

    db = _RecoverDB([paused_task, stale_task, fresh_task])
    enqueued: list[str] = []

    async def _enqueue(task_id: str, defer_seconds: float = 0.0):  # noqa: ANN001
        _ = defer_seconds
        enqueued.append(task_id)

    monkeypatch.setattr(train_worker, "enqueue_train_task", _enqueue)
    recovered = await train_worker.enqueue_recoverable_tasks(db)  # noqa: SLF001
    assert recovered == 2
    assert str(stale_task.id) in enqueued
    assert str(fresh_task.id) in enqueued
    assert stale_task.state == "QUEUED"
    assert db.commits >= 1
