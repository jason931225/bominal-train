from __future__ import annotations

from datetime import datetime, timezone
from types import SimpleNamespace
from uuid import uuid4

import pytest

from app import admin_cli


class _Result:
    def __init__(self, *, scalar_value=None, first_value=None, all_value=None):  # noqa: ANN001
        self._scalar = scalar_value
        self._first = first_value
        self._all = all_value or []

    def scalar(self):  # noqa: ANN201
        return self._scalar

    def first(self):  # noqa: ANN201
        return self._first

    def all(self):  # noqa: ANN201
        return self._all


class _DB:
    def __init__(self, results: list[_Result]):
        self.results = results
        self.commits = 0

    async def execute(self, _query):  # noqa: ANN001
        return self.results.pop(0)

    async def commit(self):
        self.commits += 1


class _SessionCtx:
    def __init__(self, db: _DB):
        self.db = db

    async def __aenter__(self):
        return self.db

    async def __aexit__(self, *_args):
        return None


class _SessionFactory:
    def __init__(self, db: _DB):
        self.db = db

    def __call__(self):
        return _SessionCtx(self.db)


def _patch_session(monkeypatch, db: _DB) -> None:
    monkeypatch.setattr(admin_cli, "SessionLocal", _SessionFactory(db))


def _user(*, role_id: int = 2):
    now = datetime.now(timezone.utc)
    return SimpleNamespace(
        id=uuid4(),
        email="user@example.com",
        display_name="User",
        phone_number="01000000000",
        role_id=role_id,
        created_at=now,
        updated_at=now,
        email_verified_at=now,
    )


@pytest.mark.asyncio
async def test_helpers_and_print_table(capsys):
    assert admin_cli.color("x", admin_cli.Colors.BOLD).endswith(admin_cli.Colors.RESET)

    admin_cli.print_table(["A"], [])
    output = capsys.readouterr().out
    assert "No data found" in output

    admin_cli.print_table(["A", "B"], [["1", "2"]])
    output = capsys.readouterr().out
    assert "A" in output and "B" in output


@pytest.mark.asyncio
async def test_user_list_and_user_info(monkeypatch, capsys):
    role = SimpleNamespace(name="user")
    user = _user()
    db = _DB(
        [
            _Result(all_value=[(user, role)]),  # user_list
            _Result(first_value=(user, role)),  # user_info row
            _Result(scalar_value=2),  # active sessions
            _Result(scalar_value=5),  # tasks
            _Result(scalar_value=1),  # secrets
            _Result(first_value=None),  # user_info not found
        ]
    )
    _patch_session(monkeypatch, db)

    await admin_cli.user_list()
    assert "Total: 1 users" in capsys.readouterr().out

    await admin_cli.user_info(user.email)
    out = capsys.readouterr().out
    assert "User Details" in out
    assert "Active Sessions" in out

    await admin_cli.user_info("missing@example.com")
    assert "User not found" in capsys.readouterr().out


@pytest.mark.asyncio
async def test_user_promote_and_demote_branches(monkeypatch, capsys):
    admin_role = SimpleNamespace(id=1, name="admin")
    user_role = SimpleNamespace(id=2, name="user")
    user = _user(role_id=2)
    admin_user = _user(role_id=1)
    regular_user = _user(role_id=2)

    db_promote = _DB(
        [
            _Result(scalar_value=None),  # no admin role
            _Result(scalar_value=admin_role),  # role exists
            _Result(scalar_value=None),  # user missing
            _Result(scalar_value=admin_role),  # role exists
            _Result(scalar_value=admin_user),  # already admin
            _Result(scalar_value=admin_role),  # role exists
            _Result(scalar_value=user),  # promote success
        ]
    )
    _patch_session(monkeypatch, db_promote)

    await admin_cli.user_promote("x@example.com")
    await admin_cli.user_promote("x@example.com")
    await admin_cli.user_promote("x@example.com")
    await admin_cli.user_promote("x@example.com")
    output = capsys.readouterr().out
    assert "Admin role not found" in output
    assert "User not found" in output
    assert "already an admin" in output
    assert "promoted to admin" in output
    assert db_promote.commits == 1

    db_demote = _DB(
        [
            _Result(scalar_value=None),  # no user role
            _Result(scalar_value=user_role),  # role exists
                _Result(scalar_value=None),  # user missing
                _Result(scalar_value=user_role),  # role exists
                _Result(scalar_value=regular_user),  # already user
                _Result(scalar_value=user_role),  # role exists
                _Result(scalar_value=admin_user),  # demote success
            ]
    )
    _patch_session(monkeypatch, db_demote)
    await admin_cli.user_demote("x@example.com")
    await admin_cli.user_demote("x@example.com")
    await admin_cli.user_demote("x@example.com")
    await admin_cli.user_demote("x@example.com")
    output = capsys.readouterr().out
    assert "User role not found" in output
    assert "already a regular user" in output
    assert "demoted to regular user" in output
    assert db_demote.commits == 1


@pytest.mark.asyncio
async def test_task_cancel_and_unhide_branches(monkeypatch, capsys):
    cancelled_task = SimpleNamespace(id=uuid4(), state="cancelled", cancelled_at=None, hidden_at=None)
    active_task = SimpleNamespace(id=uuid4(), state="running", cancelled_at=None, hidden_at=None)
    hidden_task = SimpleNamespace(id=uuid4(), state="running", hidden_at=datetime.now(timezone.utc))
    visible_task = SimpleNamespace(id=uuid4(), state="running", hidden_at=None)

    db = _DB(
        [
            _Result(scalar_value=None),  # task_cancel invalid prefix missing
            _Result(scalar_value=cancelled_task),  # terminal
            _Result(scalar_value=active_task),  # cancel success
            _Result(scalar_value=None),  # unhide missing
            _Result(scalar_value=visible_task),  # not hidden
            _Result(scalar_value=hidden_task),  # unhide success
        ]
    )
    _patch_session(monkeypatch, db)

    await admin_cli.task_cancel("no-such")
    await admin_cli.task_cancel(str(cancelled_task.id))
    await admin_cli.task_cancel(str(active_task.id))
    out = capsys.readouterr().out
    assert "Task not found" in out
    assert "already cancelled" in out
    assert "cancelled" in out

    await admin_cli.task_unhide("no-such")
    await admin_cli.task_unhide(str(visible_task.id))
    await admin_cli.task_unhide(str(hidden_task.id))
    out = capsys.readouterr().out
    assert "is not hidden" in out
    assert "unhidden" in out
    assert db.commits == 2


@pytest.mark.asyncio
async def test_db_stats_and_secret_commands(monkeypatch, capsys):
    db = _DB(
        [
            _Result(scalar_value=10),
            _Result(scalar_value=20),
            _Result(scalar_value=30),
            _Result(scalar_value=40),
            _Result(scalar_value=50),
            _Result(scalar_value=1024 * 1024),  # size 1MB
            _Result(scalar_value=7),  # connections
            _Result(all_value=[]),  # secret_check no versions
            _Result(all_value=[(1, 5), (2, 4)]),  # secret_check with rotation needed
            _Result(all_value=[(2, 9)]),  # all current
        ]
    )
    _patch_session(monkeypatch, db)
    monkeypatch.setattr(admin_cli.settings, "kek_version", 2)

    await admin_cli.db_stats()
    out = capsys.readouterr().out
    assert "Database Size" in out
    assert "Active Connections" in out

    await admin_cli.secret_check()
    assert "No encrypted secrets found" in capsys.readouterr().out

    await admin_cli.secret_check()
    assert "need rotation" in capsys.readouterr().out

    await admin_cli.secret_check()
    assert "All secrets using current KEK version" in capsys.readouterr().out

    await admin_cli.secret_purge_payment(yes=False)
    assert "Refusing to purge payment data" in capsys.readouterr().out

    async def _fake_purge(_db):  # noqa: ANN001
        return {
            "db_payment_card_secrets_deleted": 1,
            "redis_cvv_keys_deleted_current": 2,
            "redis_cvv_keys_deleted_legacy": 3,
            "redis_cvv_keys_deleted_total": 5,
        }

    monkeypatch.setattr(admin_cli, "purge_all_saved_payment_data", _fake_purge)
    await admin_cli.secret_purge_payment(yes=True)
    out = capsys.readouterr().out
    assert "Payment Data Purge" in out
    assert "total" in out.lower()


def test_db_vacuum(monkeypatch, capsys):
    calls = {"executed": False}

    class _Conn:
        def execution_options(self, **_kwargs):  # noqa: ANN003
            return self

        def execute(self, _stmt):  # noqa: ANN001
            calls["executed"] = True
            return None

    class _Ctx:
        def __enter__(self):
            return _Conn()

        def __exit__(self, *_args):
            return None

    class _Engine:
        def connect(self):
            return _Ctx()

    monkeypatch.setattr("sqlalchemy.create_engine", lambda _url: _Engine())
    monkeypatch.setattr(admin_cli.settings, "sync_database_url", "sqlite:///tmp/test.db")
    asyncio_run_calls = []

    # Direct function call.
    pytest.importorskip("sqlalchemy")
    import asyncio

    asyncio.run(admin_cli.db_vacuum())
    assert calls["executed"] is True
    assert "VACUUM ANALYZE completed" in capsys.readouterr().out
    assert asyncio_run_calls == []


def test_main_dispatch(monkeypatch):
    called: list[str] = []

    async def _mark(name: str):
        called.append(name)

    monkeypatch.setattr(admin_cli, "user_list", lambda: _mark("user_list"))
    monkeypatch.setattr(admin_cli, "user_info", lambda _email: _mark("user_info"))  # noqa: ANN001
    monkeypatch.setattr(admin_cli, "task_list", lambda show_all=False: _mark(f"task_list:{show_all}"))  # noqa: ARG005
    monkeypatch.setattr(admin_cli, "db_stats", lambda: _mark("db_stats"))
    monkeypatch.setattr(admin_cli, "secret_check", lambda: _mark("secret_check"))

    def _fake_run(coro):
        try:
            coro.send(None)
        except StopIteration:
            pass

    monkeypatch.setattr(admin_cli.asyncio, "run", _fake_run)

    monkeypatch.setattr(admin_cli.sys, "argv", ["admin_cli"])
    admin_cli.main()

    monkeypatch.setattr(admin_cli.sys, "argv", ["admin_cli", "user", "list"])
    admin_cli.main()
    monkeypatch.setattr(admin_cli.sys, "argv", ["admin_cli", "user", "info", "u@example.com"])
    admin_cli.main()
    monkeypatch.setattr(admin_cli.sys, "argv", ["admin_cli", "task", "list", "--all"])
    admin_cli.main()
    monkeypatch.setattr(admin_cli.sys, "argv", ["admin_cli", "db", "stats"])
    admin_cli.main()
    monkeypatch.setattr(admin_cli.sys, "argv", ["admin_cli", "secret", "check"])
    admin_cli.main()

    assert any("task_list" in item for item in called)
    assert any("db_stats" in item for item in called)
