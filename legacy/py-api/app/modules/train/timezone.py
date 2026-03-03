"""
Train module timezone utilities.

Re-exports KST and kst_now from app.core.time for convenience.
The train module uses KST for all train schedules and departure times.
"""

from __future__ import annotations

from datetime import datetime, timedelta, timezone

# Korea Standard Time (UTC+9) - Korea does not observe DST
KST = timezone(timedelta(hours=9), name="KST")


def now_kst() -> datetime:
    """Return the current time in Korea Standard Time."""
    return datetime.now(KST)
