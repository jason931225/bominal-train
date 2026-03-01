from types import SimpleNamespace

from app.db import session as session_mod


def test_build_async_engine_options_applies_pool_and_timeout_settings() -> None:
    settings = SimpleNamespace(
        db_pool_pre_ping=False,
        db_pool_size=7,
        db_max_overflow=3,
        db_pool_timeout_seconds=11.5,
        db_pool_recycle_seconds=900,
        db_pool_use_lifo=True,
        db_connect_timeout_seconds=4.5,
        db_command_timeout_seconds=9.0,
        db_statement_timeout_ms=14000,
    )

    options = session_mod.build_async_engine_options(settings)

    assert options["pool_pre_ping"] is False
    assert options["pool_size"] == 7
    assert options["max_overflow"] == 3
    assert options["pool_timeout"] == 11.5
    assert options["pool_recycle"] == 900
    assert options["pool_use_lifo"] is True
    assert options["connect_args"]["timeout"] == 4.5
    assert options["connect_args"]["command_timeout"] == 9.0
    assert options["connect_args"]["server_settings"]["statement_timeout"] == "14000"


def test_build_async_engine_options_omits_statement_timeout_when_disabled() -> None:
    settings = SimpleNamespace(
        db_pool_pre_ping=True,
        db_pool_size=5,
        db_max_overflow=5,
        db_pool_timeout_seconds=10.0,
        db_pool_recycle_seconds=600,
        db_pool_use_lifo=True,
        db_connect_timeout_seconds=5.0,
        db_command_timeout_seconds=8.0,
        db_statement_timeout_ms=0,
    )

    options = session_mod.build_async_engine_options(settings)

    assert options["pool_pre_ping"] is True
    assert "server_settings" not in options["connect_args"]
