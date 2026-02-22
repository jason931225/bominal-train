from app.main import _redis_appendonly_is_disabled, _redis_save_is_disabled


def test_redis_save_guard() -> None:
    assert _redis_save_is_disabled({"save": ""}) is True
    assert _redis_save_is_disabled({"save": "900 1"}) is False


def test_redis_appendonly_guard() -> None:
    assert _redis_appendonly_is_disabled({"appendonly": "no"}) is True
    assert _redis_appendonly_is_disabled({"appendonly": "yes"}) is False
