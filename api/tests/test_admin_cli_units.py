from __future__ import annotations

import base64
from datetime import datetime, timezone
from types import SimpleNamespace
from uuid import uuid4

import pytest
import runpy
import sys

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

    def scalars(self):  # noqa: ANN201
        return SimpleNamespace(all=lambda: list(self._all))


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
async def test_task_list_and_uuid_not_found_paths(monkeypatch, capsys):
    now = datetime.now(timezone.utc)
    hidden_task = SimpleNamespace(
        id=uuid4(),
        module="train",
        state="running",
        hidden_at=now,
        updated_at=now,
    )
    visible_task = SimpleNamespace(
        id=uuid4(),
        module="restaurant",
        state="queued",
        hidden_at=None,
        updated_at=now,
    )
    rows = [
        (hidden_task, SimpleNamespace(email="hidden-user@example.com")),
        (visible_task, SimpleNamespace(email="visible-user@example.com")),
    ]
    missing_uuid = str(uuid4())

    db = _DB(
        [
            _Result(all_value=rows),  # task_list(show_all=False)
            _Result(all_value=rows),  # task_list(show_all=True)
            _Result(scalar_value=None),  # task_cancel uuid not found
            _Result(scalar_value=None),  # task_unhide uuid not found
        ]
    )
    _patch_session(monkeypatch, db)

    await admin_cli.task_list(show_all=False)
    out = capsys.readouterr().out
    assert "Visible Tasks" in out
    assert "(H)" in out

    await admin_cli.task_list(show_all=True)
    out = capsys.readouterr().out
    assert "All Tasks (including hidden)" in out

    await admin_cli.task_cancel(missing_uuid)
    assert "Task not found" in capsys.readouterr().out

    await admin_cli.task_unhide(missing_uuid)
    assert "Task not found" in capsys.readouterr().out


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

    await admin_cli.secret_purge_payment_cvv(yes=False)
    assert "Refusing to purge cached CVV data" in capsys.readouterr().out

    async def _fake_purge_cvv() -> dict[str, int]:
        return {
            "redis_cvv_keys_deleted_current": 4,
            "redis_cvv_keys_deleted_legacy": 1,
            "redis_cvv_keys_deleted_total": 5,
        }

    monkeypatch.setattr(admin_cli, "purge_cached_payment_cvv_data", _fake_purge_cvv)
    await admin_cli.secret_purge_payment_cvv(yes=True)
    out = capsys.readouterr().out
    assert "Payment CVV Cache Purge" in out
    assert "total" in out.lower()


@pytest.mark.asyncio
async def test_secret_rotate_kek_branches(monkeypatch, capsys):
    secret_ok = SimpleNamespace(
        id=uuid4(),
        user_id=uuid4(),
        kind="train_credentials_srt",
        ciphertext="c",
        nonce="n",
        wrapped_dek="w",
        dek_nonce="d",
        aad="a",
        kek_version=1,
        updated_at=datetime.now(timezone.utc),
    )
    secret_fail = SimpleNamespace(
        id=uuid4(),
        user_id=uuid4(),
        kind="train_credentials_ktx",
        ciphertext="c2",
        nonce="n2",
        wrapped_dek="w2",
        dek_nonce="d2",
        aad="a2",
        kek_version=1,
        updated_at=datetime.now(timezone.utc),
    )

    db = _DB([_Result(all_value=[secret_ok, secret_fail]), _Result(all_value=[secret_ok])])
    _patch_session(monkeypatch, db)
    monkeypatch.setattr(admin_cli.settings, "kek_version", 2)

    class _Crypto:
        def decrypt_payload(self, **kwargs):  # noqa: ANN003
            if kwargs["ciphertext"] == "c2":
                raise RuntimeError("boom")
            return {"username": "u", "password": "p"}

        def encrypt_payload(self, payload, aad_text):  # noqa: ANN001
            _ = payload, aad_text
            return SimpleNamespace(
                ciphertext="new-c",
                nonce="new-n",
                wrapped_dek="new-w",
                dek_nonce="new-d",
                aad="new-a",
                kek_version=2,
            )

    monkeypatch.setattr(admin_cli, "get_envelope_crypto", lambda: _Crypto())

    await admin_cli.secret_rotate_kek(yes=False, dry_run=False, limit=None)
    assert "Refusing to rotate secrets" in capsys.readouterr().out

    await admin_cli.secret_rotate_kek(yes=False, dry_run=True, limit=None)
    out = capsys.readouterr().out
    assert "DRY RUN" in out
    assert "Failed" in out
    assert db.commits == 0

    await admin_cli.secret_rotate_kek(yes=True, dry_run=False, limit=1)
    out = capsys.readouterr().out
    assert "EXECUTE" in out
    assert "Rotated:" in out
    assert db.commits == 1
    assert secret_ok.kek_version == 2


@pytest.mark.asyncio
async def test_secret_rotate_kek_prints_failure_overflow(monkeypatch, capsys):
    secrets = [
        SimpleNamespace(
            id=uuid4(),
            user_id=uuid4(),
            kind="train_credentials_srt",
            ciphertext=f"c-{idx}",
            nonce="n",
            wrapped_dek="w",
            dek_nonce="d",
            aad="a",
            kek_version=1,
            updated_at=datetime.now(timezone.utc),
        )
        for idx in range(11)
    ]
    db = _DB([_Result(all_value=secrets)])
    _patch_session(monkeypatch, db)
    monkeypatch.setattr(admin_cli.settings, "kek_version", 2)

    class _Crypto:
        def decrypt_payload(self, **_kwargs):  # noqa: ANN003
            raise RuntimeError("boom")

        def encrypt_payload(self, payload, aad_text):  # noqa: ANN001
            _ = payload, aad_text
            raise AssertionError("encrypt should not be called when decrypt fails")

    monkeypatch.setattr(admin_cli, "get_envelope_crypto", lambda: _Crypto())

    await admin_cli.secret_rotate_kek(yes=False, dry_run=True, limit=None)
    out = capsys.readouterr().out
    assert "... and 1 more" in out


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


def test_prepare_rotation_and_timestamp_helpers(monkeypatch, capsys):
    generated = admin_cli._generate_master_key_b64()  # noqa: SLF001
    assert len(base64.b64decode(generated.encode("utf-8"))) == 32

    monkeypatch.setattr(admin_cli.settings, "kek_version", 2)
    monkeypatch.setattr(admin_cli.settings, "master_key", "active-master-key")
    monkeypatch.setattr(admin_cli.settings, "master_keys_by_version", {1: "old-master-key"})
    monkeypatch.setattr(admin_cli, "_generate_master_key_b64", lambda: "new-master-key")

    admin_cli.secret_prepare_kek_rotation(new_version=0)
    assert "must be >= 1" in capsys.readouterr().out

    admin_cli.secret_prepare_kek_rotation(new_version=1)
    assert "already exists in keyring" in capsys.readouterr().out

    admin_cli.secret_prepare_kek_rotation(new_version=3)
    output = capsys.readouterr().out
    assert "KEK_VERSION=3" in output
    assert "MASTER_KEY=new-master-key" in output
    assert '"1":"old-master-key"' in output
    assert '"2":"active-master-key"' in output
    assert '"3":"new-master-key"' in output

    assert admin_cli._parse_rotation_completed_at("2026-02-22T12:00:00Z").tzinfo is not None  # noqa: SLF001
    assert admin_cli._parse_rotation_completed_at("2026-02-22T12:00:00").tzinfo is not None  # noqa: SLF001


@pytest.mark.asyncio
async def test_rotate_kek_background_and_retire_kek(monkeypatch, capsys):
    monkeypatch.setattr(admin_cli.settings, "kek_version", 2)
    monkeypatch.setattr(admin_cli.settings, "master_key", "active-master-key")
    monkeypatch.setattr(admin_cli.settings, "master_keys_by_version", {1: "old-master-key", 2: "active-master-key"})
    monkeypatch.setattr(admin_cli.settings, "kek_retirement_window_days", 30)

    calls: list[tuple[bool, bool, int | None]] = []

    async def _fake_rotate(*, yes: bool, dry_run: bool, limit: int | None):  # noqa: ANN001
        calls.append((yes, dry_run, limit))

    monkeypatch.setattr(admin_cli, "secret_rotate_kek", _fake_rotate)

    await admin_cli.secret_rotate_kek_background(yes=False, batch_size=50, sleep_seconds=0.0)
    assert "without --yes" in capsys.readouterr().out

    await admin_cli.secret_rotate_kek_background(yes=True, batch_size=0, sleep_seconds=0.0)
    assert "Batch size must be >= 1" in capsys.readouterr().out

    await admin_cli.secret_rotate_kek_background(yes=True, batch_size=50, sleep_seconds=-1.0)
    assert "Sleep seconds must be >= 0" in capsys.readouterr().out

    # Complete path: first count check returns 0.
    _patch_session(monkeypatch, _DB([_Result(scalar_value=0)]))
    await admin_cli.secret_rotate_kek_background(yes=True, batch_size=50, sleep_seconds=0.0)
    output = capsys.readouterr().out
    assert "Background KEK rotation completed" in output
    assert "Old KEK retirement not before" in output
    assert calls[-1] == (True, False, 50)

    # Stalled path: one remaining, final pass still remaining.
    _patch_session(monkeypatch, _DB([_Result(scalar_value=1), _Result(scalar_value=1)]))
    await admin_cli.secret_rotate_kek_background(yes=True, batch_size=50, sleep_seconds=0.0)
    assert "Rotation stalled" in capsys.readouterr().out

    # Final-pass success path: first remaining <= batch and second remaining resolves to zero.
    _patch_session(monkeypatch, _DB([_Result(scalar_value=1), _Result(scalar_value=0)]))
    await admin_cli.secret_rotate_kek_background(yes=True, batch_size=50, sleep_seconds=0.0)
    assert "Background KEK rotation completed" in capsys.readouterr().out

    sleep_calls: list[float] = []

    async def _fake_sleep(seconds: float) -> None:
        sleep_calls.append(seconds)

    # Sleep branch path: first loop remains above batch, then second loop completes.
    _patch_session(monkeypatch, _DB([_Result(scalar_value=100), _Result(scalar_value=0)]))
    monkeypatch.setattr(admin_cli.asyncio, "sleep", _fake_sleep)
    await admin_cli.secret_rotate_kek_background(yes=True, batch_size=50, sleep_seconds=0.25)
    assert sleep_calls == [0.25]

    # No-progress stall branch when remaining stays >= previous_remaining above batch size.
    _patch_session(monkeypatch, _DB([_Result(scalar_value=120), _Result(scalar_value=120)]))
    await admin_cli.secret_rotate_kek_background(yes=True, batch_size=50, sleep_seconds=0.0)
    assert "without progress" in capsys.readouterr().out

    await admin_cli.secret_retire_kek(version=1, yes=False, rotation_completed_at="2020-01-01T00:00:00Z")
    assert "without --yes" in capsys.readouterr().out

    await admin_cli.secret_retire_kek(version=0, yes=True, rotation_completed_at="2020-01-01T00:00:00Z")
    assert "must be >= 1" in capsys.readouterr().out

    await admin_cli.secret_retire_kek(version=2, yes=True, rotation_completed_at="2020-01-01T00:00:00Z")
    assert "Cannot retire the active KEK_VERSION" in capsys.readouterr().out

    await admin_cli.secret_retire_kek(version=9, yes=True, rotation_completed_at="2020-01-01T00:00:00Z")
    assert "not present in current keyring" in capsys.readouterr().out

    await admin_cli.secret_retire_kek(version=1, yes=True, rotation_completed_at="not-a-time")
    assert "Invalid --rotation-completed-at" in capsys.readouterr().out

    now_utc = datetime.now(timezone.utc).isoformat()
    await admin_cli.secret_retire_kek(version=1, yes=True, rotation_completed_at=now_utc)
    assert "Retention window has not elapsed" in capsys.readouterr().out

    _patch_session(monkeypatch, _DB([_Result(scalar_value=3)]))
    await admin_cli.secret_retire_kek(version=1, yes=True, rotation_completed_at="2020-01-01T00:00:00Z")
    assert "secrets still reference this version" in capsys.readouterr().out

    _patch_session(monkeypatch, _DB([_Result(scalar_value=0)]))
    await admin_cli.secret_retire_kek(version=1, yes=True, rotation_completed_at="2020-01-01T00:00:00Z")
    output = capsys.readouterr().out
    assert "KEK Retirement Ready" in output
    assert "KEK_VERSION=2" in output
    assert '"2":"active-master-key"' in output
    assert '"1":"old-master-key"' not in output


def test_main_dispatch(monkeypatch):
    called: list[str] = []

    async def _mark(name: str):
        called.append(name)

    monkeypatch.setattr(admin_cli, "user_list", lambda: _mark("user_list"))
    monkeypatch.setattr(admin_cli, "user_info", lambda _email: _mark("user_info"))  # noqa: ANN001
    monkeypatch.setattr(admin_cli, "task_list", lambda show_all=False: _mark(f"task_list:{show_all}"))  # noqa: ARG005
    monkeypatch.setattr(admin_cli, "task_cancel", lambda _id: _mark("task_cancel"))  # noqa: ANN001
    monkeypatch.setattr(admin_cli, "task_unhide", lambda _id: _mark("task_unhide"))  # noqa: ANN001
    monkeypatch.setattr(admin_cli, "db_stats", lambda: _mark("db_stats"))
    monkeypatch.setattr(admin_cli, "db_vacuum", lambda: _mark("db_vacuum"))
    monkeypatch.setattr(admin_cli, "secret_check", lambda: _mark("secret_check"))
    monkeypatch.setattr(admin_cli, "secret_prepare_kek_rotation", lambda new_version: called.append(f"prepare:{new_version}"))  # noqa: ANN001
    monkeypatch.setattr(admin_cli, "secret_rotate_kek", lambda yes=False, dry_run=False, limit=None: _mark(f"rotate:{yes}:{dry_run}:{limit}"))  # noqa: ARG005
    monkeypatch.setattr(admin_cli, "secret_rotate_kek_background", lambda yes=False, batch_size=200, sleep_seconds=0.5: _mark(f"rotate-bg:{yes}:{batch_size}:{sleep_seconds}"))  # noqa: ARG005
    monkeypatch.setattr(admin_cli, "secret_retire_kek", lambda version=1, yes=False, rotation_completed_at="": _mark(f"retire:{version}:{yes}:{rotation_completed_at}"))  # noqa: ARG005
    monkeypatch.setattr(admin_cli, "secret_purge_payment", lambda yes=False: _mark(f"purge:{yes}"))  # noqa: ARG005
    monkeypatch.setattr(admin_cli, "user_promote", lambda _email: _mark("user_promote"))  # noqa: ANN001
    monkeypatch.setattr(admin_cli, "user_demote", lambda _email: _mark("user_demote"))  # noqa: ANN001

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
    monkeypatch.setattr(admin_cli.sys, "argv", ["admin_cli", "user", "promote", "u@example.com"])
    admin_cli.main()
    monkeypatch.setattr(admin_cli.sys, "argv", ["admin_cli", "user", "demote", "u@example.com"])
    admin_cli.main()
    monkeypatch.setattr(admin_cli.sys, "argv", ["admin_cli", "user"])
    admin_cli.main()
    monkeypatch.setattr(admin_cli.sys, "argv", ["admin_cli", "task", "list", "--all"])
    admin_cli.main()
    monkeypatch.setattr(admin_cli.sys, "argv", ["admin_cli", "task", "cancel", "abc"])
    admin_cli.main()
    monkeypatch.setattr(admin_cli.sys, "argv", ["admin_cli", "task", "unhide", "abc"])
    admin_cli.main()
    monkeypatch.setattr(admin_cli.sys, "argv", ["admin_cli", "task"])
    admin_cli.main()
    monkeypatch.setattr(admin_cli.sys, "argv", ["admin_cli", "db", "stats"])
    admin_cli.main()
    monkeypatch.setattr(admin_cli.sys, "argv", ["admin_cli", "db", "vacuum"])
    admin_cli.main()
    monkeypatch.setattr(admin_cli.sys, "argv", ["admin_cli", "db"])
    admin_cli.main()
    monkeypatch.setattr(admin_cli.sys, "argv", ["admin_cli", "secret", "check"])
    admin_cli.main()
    monkeypatch.setattr(admin_cli.sys, "argv", ["admin_cli", "secret", "prepare-kek-rotation", "--new-version", "3"])
    admin_cli.main()
    monkeypatch.setattr(admin_cli.sys, "argv", ["admin_cli", "secret", "rotate-kek", "--dry-run", "--limit", "5"])
    admin_cli.main()
    monkeypatch.setattr(
        admin_cli.sys,
        "argv",
        ["admin_cli", "secret", "rotate-kek-background", "--yes", "--batch-size", "50", "--sleep-seconds", "0.0"],
    )
    admin_cli.main()
    monkeypatch.setattr(
        admin_cli.sys,
        "argv",
        [
            "admin_cli",
            "secret",
            "retire-kek",
            "--version",
            "1",
            "--rotation-completed-at",
            "2020-01-01T00:00:00Z",
            "--yes",
        ],
    )
    admin_cli.main()
    monkeypatch.setattr(admin_cli.sys, "argv", ["admin_cli", "secret", "purge-payment", "--yes"])
    admin_cli.main()
    monkeypatch.setattr(admin_cli.sys, "argv", ["admin_cli", "secret"])
    admin_cli.main()

    assert any("task_list" in item for item in called)
    assert any("db_stats" in item for item in called)
    assert "user_promote" in called
    assert "user_demote" in called
    assert "task_cancel" in called
    assert "task_unhide" in called
    assert "db_vacuum" in called
    assert any(item.startswith("prepare:") for item in called)
    assert any(item.startswith("rotate:") for item in called)
    assert any(item.startswith("rotate-bg:") for item in called)
    assert any(item.startswith("retire:") for item in called)
    assert any(item.startswith("purge:") for item in called)


def test_main_module_entrypoint(monkeypatch):
    monkeypatch.setattr(sys, "argv", ["admin_cli"])
    sys.modules.pop("app.admin_cli", None)
    module_globals = runpy.run_module("app.admin_cli", run_name="__main__")
    assert module_globals.get("__name__") == "__main__"
    assert callable(module_globals.get("main"))
