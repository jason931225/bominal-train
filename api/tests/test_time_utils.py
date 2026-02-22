from __future__ import annotations

from datetime import datetime, timezone

from app.core import time as time_utils


def test_utc_now_and_kst_now_are_timezone_aware() -> None:
    utc_value = time_utils.utc_now()
    kst_value = time_utils.kst_now()

    assert utc_value.tzinfo == timezone.utc
    assert kst_value.tzinfo == time_utils.KST


def test_to_kst_and_to_utc_for_naive_and_aware_values() -> None:
    naive_utc = datetime(2026, 2, 22, 0, 0, 0)
    aware_utc = datetime(2026, 2, 22, 0, 0, 0, tzinfo=timezone.utc)
    naive_kst = datetime(2026, 2, 22, 9, 0, 0)

    converted_from_naive = time_utils.to_kst(naive_utc)
    converted_from_aware = time_utils.to_kst(aware_utc)
    converted_to_utc = time_utils.to_utc(naive_kst)

    assert converted_from_naive.tzinfo == time_utils.KST
    assert converted_from_aware.tzinfo == time_utils.KST
    assert converted_to_utc.tzinfo == timezone.utc
    assert converted_to_utc.hour == 0


def test_parse_kst_datetime_handles_valid_and_invalid_inputs() -> None:
    parsed = time_utils.parse_kst_datetime("20260222", "1430")
    assert parsed is not None
    assert parsed.tzinfo == time_utils.KST
    assert parsed.hour == 14
    assert parsed.minute == 30

    assert time_utils.parse_kst_datetime("", "1430") is None
    assert time_utils.parse_kst_datetime("20260222", "") is None
    assert time_utils.parse_kst_datetime("not-date", "not-time") is None
