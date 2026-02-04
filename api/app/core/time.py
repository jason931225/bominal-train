"""
Time utilities for consistent timestamp handling.

Provides:
- UTC time utilities for logs and system timestamps
- KST (Korea Standard Time) utilities for Korean services (train, restaurant reservations)
- Conversion utilities between timezones

Train/rail services (SRT, KTX) and Korean platforms (CatchTable) use KST.
International services (Resy, OpenTable) use restaurant-local timezones.
"""

from datetime import datetime, timedelta, timezone

__all__ = [
    "utc_now",
    "KST",
    "kst_now",
    "to_kst",
    "to_utc",
    "parse_kst_datetime",
]

# Korea Standard Time (UTC+9)
KST = timezone(timedelta(hours=9), name="KST")


def utc_now() -> datetime:
    """Return the current UTC time as a timezone-aware datetime."""
    return datetime.now(timezone.utc)


def kst_now() -> datetime:
    """Return the current time in Korea Standard Time (UTC+9)."""
    return datetime.now(KST)


def to_kst(dt: datetime) -> datetime:
    """Convert a datetime to Korea Standard Time.
    
    If the input is naive (no timezone), it's assumed to be UTC.
    """
    if dt.tzinfo is None:
        dt = dt.replace(tzinfo=timezone.utc)
    return dt.astimezone(KST)


def to_utc(dt: datetime) -> datetime:
    """Convert a datetime to UTC.
    
    If the input is naive (no timezone), it's assumed to be in KST.
    """
    if dt.tzinfo is None:
        dt = dt.replace(tzinfo=KST)
    return dt.astimezone(timezone.utc)


def parse_kst_datetime(date_str: str, time_str: str, fmt: str = "%Y%m%d%H%M%S") -> datetime | None:
    """Parse a date and time string as KST datetime.
    
    Args:
        date_str: Date portion (e.g., "20260204")
        time_str: Time portion (e.g., "143000" or "1430")
        fmt: Combined datetime format (default: "%Y%m%d%H%M%S")
    
    Returns:
        KST-aware datetime, or None if parsing fails.
    """
    if not date_str or not time_str:
        return None
    # Pad time to 6 digits if needed
    time_str = str(time_str)[:6].ljust(6, '0')
    try:
        dt = datetime.strptime(f"{date_str}{time_str}", fmt)
        return dt.replace(tzinfo=KST)
    except ValueError:
        return None
