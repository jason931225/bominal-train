from __future__ import annotations

import asyncio
import logging
import math
import random
import time
from dataclasses import dataclass, field
from datetime import date, datetime, timedelta, timezone
from typing import Any
from uuid import UUID

from sqlalchemy import and_, delete, func, or_, select
from sqlalchemy.ext.asyncio import AsyncSession

from app.core.config import get_settings
from app.core.crypto import redact_sensitive, validate_safe_metadata
from app.core.crypto.secrets_store import decrypt_secret
from app.core.time import to_kst, utc_now
from app.db.models import Artifact, Secret, Task, TaskAttempt, User
from app.modules.train.constants import (
    ACTIVE_TASK_STATES,
    ATTEMPT_ACTION_CANCEL,
    ATTEMPT_ACTION_PAY,
    ATTEMPT_ACTION_RESERVE,
    ATTEMPT_ACTION_SEARCH,
    ATTEMPT_ACTION_SYNC,
    SECRET_KIND_KTX_CREDENTIALS,
    SECRET_KIND_SRT_CREDENTIALS,
    TASK_MODULE,
    TERMINAL_TASK_STATES,
    credential_kind,
)
from app.modules.train.providers import get_provider_client
from app.modules.train.providers.base import ProviderOutcome, ProviderSchedule
from app.modules.train.events import publish_task_state_event, publish_task_ticket_status_event
from app.modules.train.queue import enqueue_train_task
from app.modules.train.rate_limiter import RedisTokenBucketLimiter
from app.modules.train.ticket_sync import fetch_ticket_sync_snapshot
from app.schemas.notification import EmailTemplateBlock, EmailTemplateJobPayload
from app.services.email_queue import enqueue_template_email
from app.services.email_template import format_completion_summary
from app.services.system_payment import is_payment_runtime_enabled
from app.services.wallet import get_payment_card_for_execution

settings = get_settings()
logger = logging.getLogger(__name__)
# Poll-delay model constants.
#
# We model provider-search retry delay with:
#   1) a deterministic stretched-exponential mean curve mu(t)
#   2) multiplicative mean-preserving gamma jitter G, where E[G] = 1
#
# Definitions:
#   t  = seconds until departure/expiry
#   t0 = 24h boundary (seconds)
#   M  = TRAIN_POLL_MAX_SECONDS
#   B  = mean target at t0 (24h), default 1.25s
#
# Mean curve:
#   x = max(t - t0, 0)
#   mu(t) = M - (M - B) * exp(-(x / tau)^p)
#
# Anchors used to solve p,tau in closed form:
#   mu(48h)=1.5, mu(72h)=2.0
#   (see fit_stretched_exp_params docstring for derivation)
#
# Final delay:
#   raw = mu(t) * G, with G ~ Gamma(k, theta=1/k), k=4 -> E[G]=1
#   delay = clamp(raw, Dmin, M)
POLL_CURVE_T0_SECONDS = 24 * 60 * 60
POLL_CURVE_T48_SECONDS = 48 * 60 * 60
POLL_CURVE_T72_SECONDS = 72 * 60 * 60
POLL_CURVE_BASELINE_MEAN_SECONDS = 1.25
POLL_CURVE_ANCHOR_48H_MEAN_SECONDS = 1.5
POLL_CURVE_ANCHOR_72H_MEAN_SECONDS = 2.0
POLL_GAMMA_SHAPE = 4.0
POLL_DELAY_MIN_SECONDS = 0.1
WAITING_STATUS_POLL_SECONDS = 5 * 60
SYNC_PROVIDER_META_STABLE_KEYS = frozenset(
    {
        "provider",
        "reservation_id",
        "reservations_ok",
        "reservations_error",
        "ticket_info_ok",
        "ticket_info_error",
        "error",
        "pay_sync_error",
    }
)


@dataclass(slots=True)
class PendingAttempt:
    action: str
    provider: str
    ok: bool
    retryable: bool
    error_code: str | None
    error_message_safe: str | None
    duration_ms: int
    meta_json_safe: dict[str, Any] | None
    started_at: datetime


@dataclass(slots=True)
class ReservationCandidate:
    provider: str
    rank: int
    schedule: ProviderSchedule
    seat_class_reserved: str
    reservation_id: str
    reserve_data: dict[str, Any]
    client: Any


@dataclass(slots=True)
class ProviderExecutionResult:
    provider: str
    attempts: list[PendingAttempt]
    candidate: ReservationCandidate | None
    retryable: bool
    schedule_backfill: list[dict[str, Any]] = field(default_factory=list)


def _as_aware_utc(value: datetime) -> datetime:
    if value.tzinfo is None:
        return value.replace(tzinfo=timezone.utc)
    return value.astimezone(timezone.utc)


def _utc_now_aware() -> datetime:
    return _as_aware_utc(utc_now())


def _seat_preference_order(seat_class: str) -> tuple[str, ...]:
    if seat_class == "special":
        return ("special",)
    if seat_class == "general_preferred":
        return ("general", "special")
    if seat_class == "special_preferred":
        return ("special", "general")
    return ("general",)


def _to_int(value: Any, default: int = 0) -> int:
    try:
        return int(str(value))
    except (TypeError, ValueError):
        return default


def _wait_reserve_supported(schedule: ProviderSchedule) -> bool:
    metadata = schedule.metadata or {}
    if schedule.provider == "KTX":
        return _to_int(metadata.get("wait_reserve_flag"), default=-1) >= 0
    if schedule.provider == "SRT":
        return _to_int(metadata.get("reserve_wait_code"), default=-1) >= 0
    return False


def _standby_seat_class(seat_class: str) -> str:
    return "special" if seat_class in {"special", "special_preferred"} else "general"


def _pick_reservable_seat_class(schedule: ProviderSchedule, seat_class: str) -> str | None:
    availability = schedule.availability or {}
    for candidate in _seat_preference_order(seat_class):
        if bool(availability.get(candidate)):
            return candidate
    if _wait_reserve_supported(schedule):
        return _standby_seat_class(seat_class)
    return None


def _is_provider_auth_required_error(outcome: ProviderOutcome) -> bool:
    if outcome.ok:
        return False

    error_code = str(outcome.error_code or "").lower()
    error_message = str(outcome.error_message_safe or "")
    error_message_lower = error_message.lower()

    auth_code_markers = (
        "not_logged_in",
        "login_required",
        "session_expired",
        "invalid_session",
        "auth_required",
    )
    if any(marker in error_code for marker in auth_code_markers):
        return True

    auth_message_markers = (
        "로그인 후 사용",
        "로그인이 필요",
        "로그인 후 이용",
        "not logged in",
        "login is required",
        "please login",
    )
    return any(marker in error_message for marker in auth_message_markers) or any(
        marker in error_message_lower for marker in auth_message_markers
    )


def _is_non_payment_expiry_reserve_error(outcome: ProviderOutcome) -> bool:
    if outcome.ok:
        return False

    error_code = str(outcome.error_code or "").lower()
    error_message = str(outcome.error_message_safe or "")
    error_message_lower = error_message.lower()

    payment_markers = (
        "payment",
        "unpaid",
        "non_payment",
        "non-payment",
        "결제",
        "미결제",
    )
    expiry_markers = (
        "expire",
        "expired",
        "expiration",
        "deadline",
        "timeout",
        "window",
        "만료",
        "기한",
    )
    reservation_status_markers = (
        "ticket not found",
        "reservation status",
        "reservation_not_found",
        "조회자료가 없습니다",
        "rowcnt: 0",
    )

    message_is_non_payment_expiry = any(marker in error_message_lower for marker in payment_markers) and any(
        marker in error_message_lower for marker in expiry_markers
    )
    code_is_non_payment_expiry = any(marker in error_code for marker in payment_markers) and any(
        marker in error_code for marker in expiry_markers
    )
    message_is_korean_payment_expiry = ("결제" in error_message) and any(
        marker in error_message for marker in ("만료", "기한")
    )
    reservation_status_mismatch = (
        any(marker in error_message_lower for marker in reservation_status_markers)
        or any(marker in error_code for marker in reservation_status_markers)
    )
    return bool(
        message_is_non_payment_expiry
        or code_is_non_payment_expiry
        or message_is_korean_payment_expiry
        or reservation_status_mismatch
    )


def _is_transient_sold_out_reserve_error(outcome: ProviderOutcome) -> bool:
    if outcome.ok:
        return False
    return str(outcome.error_code or "").lower() == "sold_out"


def _seconds_until_next_departure(ranked: list[dict[str, Any]]) -> float | None:
    now = _utc_now_aware()
    deltas: list[float] = []
    for row in ranked:
        departure_at = str(row.get("departure_at") or "")
        if not departure_at:
            continue
        try:
            departure = _as_aware_utc(datetime.fromisoformat(departure_at))
        except ValueError:
            continue
        deltas.append((departure - now).total_seconds())

    if not deltas:
        return None

    future = [delta for delta in deltas if delta > 0]
    if future:
        return min(future)
    return 0.0


def fit_stretched_exp_params(max_interval: float, baseline_mean: float) -> tuple[float, float]:
    """
    Fit stretched-exponential parameters from fixed anchors.

    Curve:
        mu(t) = M - (M - B) * exp(-(x/tau)^p), x = max(t - t0, 0)

    with:
        mu(48h)=1.5, mu(72h)=2.0

    Closed-form derivation used in code:
        r_i = (M - mu_i)/(M - B)
        y_i = -ln(r_i) = (x_i/tau)^p
        p = ln(y_2 / y_1) / ln(x_2 / x_1)
        tau = x_1 / y_1^(1/p)

    Notes:
    - No regression/optimization step is used; anchors determine p,tau exactly.
    - This keeps behavior explainable, stable, and easy to validate in tests.
    - We intentionally validate ordering/bounds before solving to fail fast if
      constants or env overrides become inconsistent.
    """
    denominator = max_interval - baseline_mean
    if denominator <= 0:
        raise ValueError("max_interval must be greater than baseline_mean")

    x1 = float(POLL_CURVE_T48_SECONDS - POLL_CURVE_T0_SECONDS)
    x2 = float(POLL_CURVE_T72_SECONDS - POLL_CURVE_T0_SECONDS)
    if x1 <= 0 or x2 <= x1:
        raise ValueError("invalid anchor times for stretched exponential fitting")

    r1 = (max_interval - POLL_CURVE_ANCHOR_48H_MEAN_SECONDS) / denominator
    r2 = (max_interval - POLL_CURVE_ANCHOR_72H_MEAN_SECONDS) / denominator
    if not (0.0 < r2 < r1 < 1.0):
        raise ValueError("anchor means must satisfy baseline < mu(48h) < mu(72h) < max_interval")

    y1 = -math.log(r1)
    y2 = -math.log(r2)
    if y1 <= 0 or y2 <= y1:
        raise ValueError("invalid transformed anchor values for stretched exponential fitting")

    p = math.log(y2 / y1) / math.log(x2 / x1)
    tau = x1 / (y1 ** (1.0 / p))
    return p, tau


def _mean_poll_delay_seconds(seconds_until_departure: float, max_interval: float) -> float:
    """
    Deterministic mean delay mu(t) before stochastic jitter.

    Behavior summary:
    - For t <= 24h, x=0 so mu(t)=B (baseline target around 1.25s).
    - As t grows, mu(t) increases smoothly toward M with stretched-exponential
      curvature controlled by (p, tau).
    - The result is bounded to [POLL_DELAY_MIN_SECONDS, M] for safety.
    """
    baseline = min(POLL_CURVE_BASELINE_MEAN_SECONDS, max_interval)
    if max_interval <= baseline:
        return max_interval

    p, tau = fit_stretched_exp_params(max_interval, baseline)
    x = max(seconds_until_departure - POLL_CURVE_T0_SECONDS, 0.0)
    mean = max_interval - (max_interval - baseline) * math.exp(-((x / tau) ** p))
    return max(POLL_DELAY_MIN_SECONDS, min(max_interval, mean))


def _poll_delay_seconds(search_attempt_count: int, *, seconds_until_departure: float | None) -> float:
    """
    Compute provider-search retry delay.

    This function intentionally combines:
    1) deterministic mean curve: mu(t)
    2) mean-preserving multiplicative gamma jitter:
         raw = mu(t) * G,  G ~ Gamma(k, theta=1/k), k=4
       so:
         E[raw] = mu(t) * E[G] = mu(t)

    Why multiplicative jitter:
    - Preserves expected delay equal to mu(t) across all t.
    - Maintains relative variability instead of fixed absolute jitter.
    - Avoids piecewise scheduling while still providing stochastic spacing.
    """
    # search_attempt_count is intentionally retained for signature compatibility.
    _ = search_attempt_count
    if settings.train_poll_force_max_rate:
        # Maximum poll rate means shortest configured delay.
        max_interval = max(settings.train_poll_max_seconds, POLL_DELAY_MIN_SECONDS)
        min_interval = max(settings.train_poll_min_seconds, POLL_DELAY_MIN_SECONDS)
        return max(POLL_DELAY_MIN_SECONDS, min(max_interval, float(min_interval)))

    max_interval = max(settings.train_poll_max_seconds, POLL_DELAY_MIN_SECONDS)
    t = float(seconds_until_departure if seconds_until_departure is not None else POLL_CURVE_T0_SECONDS)

    # Mean curve + mean-preserving gamma scaling:
    # raw = mu(t) * G, where E[G]=1, so unclamped expectation tracks mu(t).
    mean_delay = _mean_poll_delay_seconds(t, max_interval)
    gamma_scale = 1.0 / POLL_GAMMA_SHAPE
    gamma_multiplier = random.gammavariate(POLL_GAMMA_SHAPE, gamma_scale)
    raw_delay = mean_delay * gamma_multiplier
    return max(POLL_DELAY_MIN_SECONDS, min(max_interval, raw_delay))


def _normalize_ranked_selection(spec_json: dict[str, Any]) -> list[dict[str, Any]]:
    ranked_raw = sorted(spec_json.get("selected_trains_ranked", []), key=lambda row: row.get("rank", 999))
    default_provider = spec_json.get("provider")
    normalized: list[dict[str, Any]] = []

    for row in ranked_raw:
        provider = row.get("provider") or default_provider
        if provider not in {"SRT", "KTX"}:
            continue
        try:
            rank = int(row.get("rank"))
        except (TypeError, ValueError):
            continue
        schedule_id = str(row.get("schedule_id") or "")
        departure_at = str(row.get("departure_at") or "")
        if not schedule_id or not departure_at:
            continue

        normalized.append(
            {
                "provider": provider,
                "rank": rank,
                "schedule_id": schedule_id,
                "departure_at": departure_at,
            }
        )

    return normalized


def _apply_ranked_schedule_backfill(
    spec_json: dict[str, Any],
    provider_results: list[ProviderExecutionResult],
) -> tuple[dict[str, Any], bool]:
    ranked_raw = spec_json.get("selected_trains_ranked")
    if not isinstance(ranked_raw, list):
        return spec_json, False

    by_schedule_and_provider: dict[tuple[str, str], dict[str, str]] = {}
    by_schedule: dict[str, dict[str, str]] = {}
    for result in provider_results:
        for row in result.schedule_backfill:
            schedule_id = str(row.get("schedule_id") or "").strip()
            provider = str(row.get("provider") or "").strip()
            departure_at = str(row.get("departure_at") or "").strip()
            arrival_at = str(row.get("arrival_at") or "").strip()
            if not schedule_id or not departure_at or not arrival_at:
                continue
            payload = {
                "departure_at": departure_at,
                "arrival_at": arrival_at,
            }
            by_schedule_and_provider[(schedule_id, provider)] = payload
            if schedule_id not in by_schedule:
                by_schedule[schedule_id] = payload

    if not by_schedule:
        return spec_json, False

    default_provider = str(spec_json.get("provider") or "").strip()
    changed = False
    updated_ranked: list[Any] = []
    for row in ranked_raw:
        if not isinstance(row, dict):
            updated_ranked.append(row)
            continue

        schedule_id = str(row.get("schedule_id") or "").strip()
        if not schedule_id:
            updated_ranked.append(row)
            continue

        provider = str(row.get("provider") or default_provider).strip()
        resolved = by_schedule_and_provider.get((schedule_id, provider)) or by_schedule.get(schedule_id)
        if not resolved:
            updated_ranked.append(row)
            continue

        departure_at = str(row.get("departure_at") or "").strip()
        arrival_at = str(row.get("arrival_at") or "").strip()
        if departure_at == resolved["departure_at"] and arrival_at == resolved["arrival_at"]:
            updated_ranked.append(row)
            continue

        next_row = dict(row)
        next_row["departure_at"] = resolved["departure_at"]
        next_row["arrival_at"] = resolved["arrival_at"]
        updated_ranked.append(next_row)
        changed = True

    if not changed:
        return spec_json, False

    next_spec = dict(spec_json)
    next_spec["selected_trains_ranked"] = updated_ranked
    return next_spec, True


def _ticket_artifact_schedule_backfill_rows(artifact: Artifact) -> list[dict[str, str]]:
    data = dict(artifact.data_json_safe or {})
    default_provider = str(data.get("provider") or "").strip()

    rows: list[dict[str, str]] = []
    seen: set[tuple[str, str]] = set()

    def _push(row: dict[str, Any], *, provider_fallback: str) -> None:
        schedule_id = str(row.get("schedule_id") or "").strip()
        departure_at = str(row.get("departure_at") or "").strip()
        arrival_at = str(row.get("arrival_at") or "").strip()
        provider = str(row.get("provider") or provider_fallback).strip()
        if not schedule_id or not departure_at or not arrival_at:
            return
        dedupe_key = (schedule_id, provider)
        if dedupe_key in seen:
            return
        seen.add(dedupe_key)
        rows.append(
            {
                "provider": provider,
                "schedule_id": schedule_id,
                "departure_at": departure_at,
                "arrival_at": arrival_at,
            }
        )

    _push(data, provider_fallback=default_provider)

    reservation_snapshot = data.get("reservation_snapshot")
    if isinstance(reservation_snapshot, dict):
        _push(reservation_snapshot, provider_fallback=default_provider)

    return rows


def _apply_ticket_artifact_schedule_backfill(
    spec_json: dict[str, Any],
    artifact: Artifact,
) -> tuple[dict[str, Any], bool]:
    schedule_backfill = _ticket_artifact_schedule_backfill_rows(artifact)
    if not schedule_backfill:
        return spec_json, False
    return _apply_ranked_schedule_backfill(
        spec_json,
        [
            ProviderExecutionResult(
                provider=str(schedule_backfill[0].get("provider") or ""),
                attempts=[],
                candidate=None,
                retryable=True,
                schedule_backfill=schedule_backfill,
            )
        ],
    )


async def _save_attempt(
    db: AsyncSession,
    *,
    task: Task,
    action: str,
    provider: str,
    ok: bool,
    retryable: bool,
    error_code: str | None,
    error_message_safe: str | None,
    duration_ms: int,
    meta_json_safe: dict[str, Any] | None,
    started_at: datetime,
) -> TaskAttempt:
    attempt = TaskAttempt(
        task_id=task.id,
        action=action,
        provider=provider,
        ok=ok,
        retryable=retryable,
        error_code=error_code,
        error_message_safe=error_message_safe,
        duration_ms=duration_ms,
        meta_json_safe=validate_safe_metadata(meta_json_safe) if meta_json_safe else None,
        started_at=started_at,
        finished_at=utc_now(),
    )
    db.add(attempt)
    await db.flush()
    return attempt


async def _persist_attempts(db: AsyncSession, *, task: Task, attempts: list[PendingAttempt]) -> None:
    if not attempts:
        return

    def _attempt_signature(*, ok: bool, retryable: bool, error_code: str | None, error_message_safe: str | None) -> tuple[bool, bool, str, str]:
        return (
            bool(ok),
            bool(retryable),
            str(error_code or ""),
            str(error_message_safe or ""),
        )

    def _assign_attempt_to_row(existing: TaskAttempt, current: PendingAttempt) -> None:
        existing.ok = current.ok
        existing.retryable = current.retryable
        existing.error_code = current.error_code
        existing.error_message_safe = current.error_message_safe
        existing.duration_ms = current.duration_ms
        existing.meta_json_safe = validate_safe_metadata(current.meta_json_safe) if current.meta_json_safe else None
        existing.started_at = current.started_at
        existing.finished_at = utc_now()

    def _should_persist_attempt(current: PendingAttempt, previous: TaskAttempt | None) -> bool:
        if settings.train_persist_all_attempts:
            return True
        # Payment/cancel actions are intentional user-visible transitions and
        # should always remain fully auditable.
        if current.action in {ATTEMPT_ACTION_PAY, ATTEMPT_ACTION_CANCEL}:
            return True
        if previous is None:
            return True
        if current.action == ATTEMPT_ACTION_SYNC:
            return False
        return _attempt_signature(
            ok=current.ok,
            retryable=current.retryable,
            error_code=current.error_code,
            error_message_safe=current.error_message_safe,
        ) != _attempt_signature(
            ok=previous.ok,
            retryable=previous.retryable,
            error_code=previous.error_code,
            error_message_safe=previous.error_message_safe,
        )

    # Persist only state/error transitions for polling-heavy actions.
    latest_by_key: dict[tuple[str, str], TaskAttempt | None] = {}
    for action, provider in {(row.action, row.provider) for row in attempts}:
        stmt = (
            select(TaskAttempt)
            .where(TaskAttempt.task_id == task.id)
            .where(TaskAttempt.action == action)
            .where(TaskAttempt.provider == provider)
            .order_by(TaskAttempt.finished_at.desc(), TaskAttempt.id.desc())
            .limit(1)
        )
        latest_by_key[(action, provider)] = (await db.execute(stmt)).scalar_one_or_none()

    for attempt in sorted(attempts, key=lambda row: row.started_at):
        key = (attempt.action, attempt.provider)
        previous = latest_by_key.get(key)
        if attempt.action == ATTEMPT_ACTION_SYNC and previous is not None:
            _assign_attempt_to_row(previous, attempt)
            latest_by_key[key] = previous
            continue
        if not _should_persist_attempt(attempt, previous):
            continue
        persisted = await _save_attempt(
            db,
            task=task,
            action=attempt.action,
            provider=attempt.provider,
            ok=attempt.ok,
            retryable=attempt.retryable,
            error_code=attempt.error_code,
            error_message_safe=attempt.error_message_safe,
            duration_ms=attempt.duration_ms,
            meta_json_safe=attempt.meta_json_safe,
            started_at=attempt.started_at,
        )
        latest_by_key[key] = persisted


def _attempt_transition_signature(attempt: TaskAttempt) -> tuple[bool, bool, str, str]:
    return (
        bool(attempt.ok),
        bool(attempt.retryable),
        str(attempt.error_code or ""),
        str(attempt.error_message_safe or ""),
    )


async def compact_and_prune_task_attempts(db: AsyncSession) -> dict[str, int]:
    """Compact high-churn attempt history for Supabase-free-tier efficiency."""
    stats = {
        "deleted_sync_rows": 0,
        "deleted_repetitive_rows": 0,
        "deleted_retention_rows": 0,
    }
    dirty = False

    if settings.train_sync_keep_latest_only:
        sync_rows = (
            await db.execute(
                select(TaskAttempt.id, TaskAttempt.task_id, TaskAttempt.provider)
                .where(TaskAttempt.action == ATTEMPT_ACTION_SYNC)
                .order_by(
                    TaskAttempt.task_id.asc(),
                    TaskAttempt.provider.asc(),
                    TaskAttempt.finished_at.desc(),
                    TaskAttempt.id.desc(),
                )
            )
        ).all()
        seen_sync_keys: set[tuple[UUID, str]] = set()
        stale_sync_ids: list[UUID] = []
        for row in sync_rows:
            key = (row.task_id, str(row.provider))
            if key in seen_sync_keys:
                stale_sync_ids.append(row.id)
                continue
            seen_sync_keys.add(key)
        if stale_sync_ids:
            deleted = await db.execute(delete(TaskAttempt).where(TaskAttempt.id.in_(stale_sync_ids)))
            stats["deleted_sync_rows"] = int(deleted.rowcount or 0)
            dirty = dirty or stats["deleted_sync_rows"] > 0

    if settings.train_compact_repetitive_attempts:
        repetitive_rows = (
            await db.execute(
                select(TaskAttempt)
                .where(TaskAttempt.action.in_((ATTEMPT_ACTION_SEARCH, ATTEMPT_ACTION_RESERVE)))
                .order_by(
                    TaskAttempt.task_id.asc(),
                    TaskAttempt.action.asc(),
                    TaskAttempt.provider.asc(),
                    TaskAttempt.started_at.asc(),
                    TaskAttempt.id.asc(),
                )
            )
        ).scalars().all()

        stale_repetitive_ids: list[UUID] = []
        grouped_rows: list[TaskAttempt] = []
        grouped_key: tuple[UUID, str, str] | None = None

        def _flush_group(rows: list[TaskAttempt]) -> None:
            if len(rows) <= 2:
                return
            keep_ids: set[UUID] = {rows[0].id, rows[-1].id}
            previous_signature = _attempt_transition_signature(rows[0])
            for row in rows[1:-1]:
                signature = _attempt_transition_signature(row)
                if signature != previous_signature:
                    keep_ids.add(row.id)
                previous_signature = signature
            for row in rows:
                if row.id not in keep_ids:
                    stale_repetitive_ids.append(row.id)

        for row in repetitive_rows:
            key = (row.task_id, row.action, row.provider)
            if grouped_key is None:
                grouped_key = key
                grouped_rows = [row]
                continue
            if key != grouped_key:
                _flush_group(grouped_rows)
                grouped_key = key
                grouped_rows = [row]
                continue
            grouped_rows.append(row)
        if grouped_rows:
            _flush_group(grouped_rows)

        if stale_repetitive_ids:
            deleted = await db.execute(delete(TaskAttempt).where(TaskAttempt.id.in_(stale_repetitive_ids)))
            stats["deleted_repetitive_rows"] = int(deleted.rowcount or 0)
            dirty = dirty or stats["deleted_repetitive_rows"] > 0

    retention_days = int(settings.train_attempt_retention_days)
    if retention_days > 0:
        cutoff = _utc_now_aware() - timedelta(days=retention_days)
        terminal_cutoff = func.coalesce(Task.completed_at, Task.failed_at, Task.cancelled_at, Task.updated_at)
        stale_task_ids = select(Task.id).where(
            or_(
                and_(Task.hidden_at.is_not(None), Task.hidden_at < cutoff),
                and_(Task.state.in_(TERMINAL_TASK_STATES), terminal_cutoff < cutoff),
            )
        )
        deleted = await db.execute(delete(TaskAttempt).where(TaskAttempt.task_id.in_(stale_task_ids)))
        stats["deleted_retention_rows"] = int(deleted.rowcount or 0)
        dirty = dirty or stats["deleted_retention_rows"] > 0

    if dirty:
        await db.commit()

    return stats


async def _enqueue_terminal_notification(
    db: AsyncSession,
    *,
    task: Task,
    final_state: str,
) -> None:
    spec = task.spec_json if isinstance(task.spec_json, dict) else {}
    if not bool(spec.get("notify")):
        return
    if spec.get("notify_email_sent_at"):
        return

    user = await db.get(User, task.user_id)
    if user is None or not user.email:
        return

    dep = str(spec.get("dep") or "-")
    arr = str(spec.get("arr") or "-")
    people = int(spec.get("people_count") or 1)
    item_code = str(spec.get("item_code") or "N/A")
    target_date = str(spec.get("item_date") or "-")
    completed_at_kst = to_kst(_utc_now_aware())
    completion_date = completed_at_kst.strftime("%Y-%m-%d")
    completion_time = completed_at_kst.strftime("%H:%M")

    status_text = "Successfully completed" if final_state == "COMPLETED" else f"Task ended ({final_state})"
    summary = format_completion_summary(
        status=status_text,
        task="reservation",
        module="train",
        completion_date=completion_date,
        completion_time=completion_time,
        item=item_code,
        target_date=target_date,
        people=people,
    )

    subject = f"bominal Train Task {final_state}: {dep} -> {arr}"
    template_payload = EmailTemplateJobPayload(
        to_email=user.email,
        subject=subject,
        preheader=f"Train task update: {final_state}",
        theme="spring",
        blocks=[
            EmailTemplateBlock(
                type="hero",
                data={"title": f"Train task {final_state}", "subtitle": "Your task reached a terminal state."},
            ),
            EmailTemplateBlock(
                type="kv",
                data={
                    "rows": [
                        {"k": "Task ID", "v": {"$ref": "task.id"}},
                        {"k": "State", "v": {"$ref": "task.state"}},
                        {"k": "Route", "v": {"$ref": "task.route"}},
                        {"k": "Completed (KST)", "v": {"$ref": "task.completed_kst"}},
                    ]
                },
            ),
            EmailTemplateBlock(type="mono", data={"text": {"$ref": "task.summary"}}),
        ],
        context={
            "task": {
                "id": str(task.id),
                "state": final_state,
                "route": f"{dep} -> {arr}",
                "completed_kst": f"{completion_date} {completion_time}",
                "summary": summary,
            }
        },
        tags=["train", "task", final_state.lower()],
        metadata={
            "module": "train",
            "task_id": str(task.id),
            "state": final_state,
            "user_id": str(task.user_id),
        },
    )
    job_id: str | None = None

    try:
        job_id = await enqueue_template_email(template_payload)
    except Exception as exc:
        logger.warning(
            "Failed to enqueue terminal notification email for task %s: %s",
            task.id,
            type(exc).__name__,
        )
        return

    next_spec = dict(spec)
    next_spec["notify_email_sent_at"] = utc_now().isoformat()
    next_spec["notify_email_state"] = final_state
    if job_id:
        next_spec["notify_email_job_id"] = job_id
    task.spec_json = next_spec
    task.updated_at = utc_now()
    await db.commit()


async def _mark_expired(db: AsyncSession, task: Task) -> None:
    task.state = "EXPIRED"
    task.updated_at = utc_now()
    await db.commit()
    await publish_task_state_event(user_id=task.user_id, task_id=task.id, state=task.state, updated_at=task.updated_at)
    await _enqueue_terminal_notification(db, task=task, final_state="EXPIRED")


async def _mark_failed(db: AsyncSession, task: Task) -> None:
    task.state = "FAILED"
    task.failed_at = utc_now()
    task.updated_at = utc_now()
    await db.commit()
    await publish_task_state_event(user_id=task.user_id, task_id=task.id, state=task.state, updated_at=task.updated_at)
    await _enqueue_terminal_notification(db, task=task, final_state="FAILED")


async def _mark_completed(db: AsyncSession, task: Task) -> None:
    task.state = "COMPLETED"
    task.completed_at = utc_now()
    task.updated_at = utc_now()
    await db.commit()
    await publish_task_state_event(user_id=task.user_id, task_id=task.id, state=task.state, updated_at=task.updated_at)
    await _enqueue_terminal_notification(db, task=task, final_state="COMPLETED")


async def _schedule_retry(db: AsyncSession, task: Task, delay_seconds: float) -> None:
    # State-change-only persistence: keep retry scheduling mostly stateless
    # (ARQ defer queue), and only touch DB when lifecycle state changes.
    if task.state != "POLLING":
        next_spec = dict(task.spec_json or {})
        next_spec["next_run_at"] = (utc_now() + timedelta(seconds=delay_seconds)).isoformat()
        task.spec_json = next_spec
        task.state = "POLLING"
        task.updated_at = utc_now()
        await db.commit()
        await publish_task_state_event(user_id=task.user_id, task_id=task.id, state=task.state, updated_at=task.updated_at)
    await enqueue_train_task(str(task.id), defer_seconds=delay_seconds)


async def _load_provider_credentials(
    db: AsyncSession,
    *,
    user_id: UUID,
    provider: str,
) -> dict[str, str] | None:
    kind = credential_kind(provider)
    stmt = (
        select(Secret)
        .where(Secret.user_id == user_id)
        .where(Secret.kind == kind)
        .order_by(Secret.updated_at.desc())
        .limit(1)
    )
    secret = (await db.execute(stmt)).scalar_one_or_none()
    if secret is None:
        return None
    try:
        payload = decrypt_secret(secret)
    except Exception:
        return None

    username = str(payload.get("username") or "").strip()
    password = str(payload.get("password") or "")
    if not username or not password:
        return None
    return {"username": username, "password": password}


async def _load_ticket_artifacts(db: AsyncSession, *, task_id: UUID) -> list[Artifact]:
    stmt = (
        select(Artifact)
        .where(Artifact.task_id == task_id)
        .where(Artifact.kind == "ticket")
        .order_by(Artifact.created_at.asc())
    )
    return list((await db.execute(stmt)).scalars().all())


def _find_paid_ticket_artifact(artifacts: list[Artifact]) -> Artifact | None:
    for artifact in artifacts:
        if bool(artifact.data_json_safe.get("paid")) or artifact.data_json_safe.get("payment_id"):
            return artifact
    return None


def _find_open_ticket_artifact(artifacts: list[Artifact]) -> Artifact | None:
    for artifact in artifacts:
        if artifact.data_json_safe.get("reservation_id") and not bool(artifact.data_json_safe.get("paid")):
            return artifact
    return None


async def _provider_search_and_reserve(
    *,
    provider: str,
    ranked_for_provider: list[dict[str, Any]],
    spec: dict[str, Any],
    auto_pay_enabled: bool,
    task_user_id: UUID,
    credentials: dict[str, str],
    limiter: RedisTokenBucketLimiter,
) -> ProviderExecutionResult:
    attempts: list[PendingAttempt] = []
    client = get_provider_client(provider)

    login_started = utc_now()
    login_timer = time.perf_counter()
    try:
        login_outcome = await client.login(
            user_id=str(task_user_id),
            credentials=credentials,
        )
    except Exception as exc:
        login_duration = int((time.perf_counter() - login_timer) * 1000)
        attempts.append(
            PendingAttempt(
                action=ATTEMPT_ACTION_SEARCH,
                provider=provider,
                ok=False,
                retryable=True,
                error_code="provider_login_transport_error",
                error_message_safe=f"{provider} login transport error: {type(exc).__name__}",
                duration_ms=login_duration,
                meta_json_safe={"stage": "login"},
                started_at=login_started,
            )
        )
        return ProviderExecutionResult(provider=provider, attempts=attempts, candidate=None, retryable=True)

    login_duration = int((time.perf_counter() - login_timer) * 1000)
    if not login_outcome.ok:
        attempts.append(
            PendingAttempt(
                action=ATTEMPT_ACTION_SEARCH,
                provider=provider,
                ok=False,
                retryable=bool(login_outcome.retryable),
                error_code=login_outcome.error_code or "login_failed",
                error_message_safe=login_outcome.error_message_safe or f"{provider} login failed",
                duration_ms=login_duration,
                meta_json_safe={"stage": "login"},
                started_at=login_started,
            )
        )
        return ProviderExecutionResult(
            provider=provider,
            attempts=attempts,
            candidate=None,
            retryable=bool(login_outcome.retryable),
        )

    search_started = utc_now()
    search_timer = time.perf_counter()
    try:
        limit_result = await limiter.acquire_provider_call(
            provider=provider,
            user_bucket_key=str(task_user_id),
            host_bucket_key="default-host",
        )
        search_outcome = await client.search(
            dep=spec["dep"],
            arr=spec["arr"],
            date_value=date.fromisoformat(spec["date"]),
            time_window_start="00:00",
            time_window_end="23:59",
            user_id=str(task_user_id),
        )
    except Exception as exc:
        search_duration = int((time.perf_counter() - search_timer) * 1000)
        attempts.append(
            PendingAttempt(
                action=ATTEMPT_ACTION_SEARCH,
                provider=provider,
                ok=False,
                retryable=True,
                error_code="provider_transport_error",
                error_message_safe=f"{provider} search transport error: {type(exc).__name__}",
                duration_ms=search_duration,
                meta_json_safe={"stage": "search"},
                started_at=search_started,
            )
        )
        return ProviderExecutionResult(provider=provider, attempts=attempts, candidate=None, retryable=True)

    search_duration = int((time.perf_counter() - search_timer) * 1000)

    if not search_outcome.ok:
        attempts.append(
            PendingAttempt(
                action=ATTEMPT_ACTION_SEARCH,
                provider=provider,
                ok=False,
                retryable=bool(search_outcome.retryable),
                error_code=search_outcome.error_code,
                error_message_safe=search_outcome.error_message_safe,
                duration_ms=search_duration,
                meta_json_safe={
                    "rate_limit_wait_ms": limit_result.waited_ms,
                    "rate_limit_rounds": limit_result.rounds,
                    "requested_seat_class": spec["seat_class"],
                },
                started_at=search_started,
            )
        )
        return ProviderExecutionResult(
            provider=provider,
            attempts=attempts,
            candidate=None,
            retryable=bool(search_outcome.retryable),
        )

    schedule_map = {
        row.schedule_id: row for row in search_outcome.data.get("schedules", []) if isinstance(row, ProviderSchedule)
    }
    schedule_backfill = [
        {
            "provider": provider,
            "schedule_id": candidate.schedule_id,
            "departure_at": candidate.departure_at.isoformat(),
            "arrival_at": candidate.arrival_at.isoformat(),
        }
        for row in ranked_for_provider
        for candidate in [schedule_map.get(str(row.get("schedule_id") or ""))]
        if candidate is not None
    ]

    selected_schedule: ProviderSchedule | None = None
    selected_rank: int | None = None
    selected_seat_class: str | None = None

    for row in ranked_for_provider:
        candidate = schedule_map.get(row["schedule_id"])
        chosen_seat = _pick_reservable_seat_class(candidate, spec["seat_class"]) if candidate else None
        if candidate and chosen_seat:
            selected_schedule = candidate
            selected_rank = int(row["rank"])
            selected_seat_class = chosen_seat
            break

    if selected_schedule is None or selected_rank is None or selected_seat_class is None:
        attempts.append(
            PendingAttempt(
                action=ATTEMPT_ACTION_SEARCH,
                provider=provider,
                ok=False,
                retryable=True,
                error_code="seat_unavailable",
                error_message_safe="No reservable seats are available for this schedule.",
                duration_ms=search_duration,
                meta_json_safe={
                    "rate_limit_wait_ms": limit_result.waited_ms,
                    "rate_limit_rounds": limit_result.rounds,
                    "requested_seat_class": spec["seat_class"],
                },
                started_at=search_started,
            )
        )
        return ProviderExecutionResult(
            provider=provider,
            attempts=attempts,
            candidate=None,
            retryable=True,
            schedule_backfill=schedule_backfill,
        )

    attempts.append(
        PendingAttempt(
            action=ATTEMPT_ACTION_SEARCH,
            provider=provider,
            ok=True,
            retryable=False,
            error_code=None,
            error_message_safe=None,
            duration_ms=search_duration,
            meta_json_safe={
                "rate_limit_wait_ms": limit_result.waited_ms,
                "rate_limit_rounds": limit_result.rounds,
                "requested_seat_class": spec["seat_class"],
                "selected_seat_class": selected_seat_class,
                "selected_schedule_id": selected_schedule.schedule_id,
                "selected_rank": selected_rank,
            },
            started_at=search_started,
        )
    )

    reserve_started = utc_now()
    reserve_limit_wait_ms = 0
    reserve_limit_rounds = 0
    reserve_duration = 0
    relogin_retry_attempted = False
    relogin_duration_ms = 0
    initial_reserve_error_code: str | None = None
    initial_reserve_error_message: str | None = None

    async def _reserve_once() -> tuple[ProviderOutcome, int, int, int]:
        timer = time.perf_counter()
        limit = await limiter.acquire_provider_call(
            provider=provider,
            user_bucket_key=str(task_user_id),
            host_bucket_key="default-host",
        )
        outcome = await client.reserve(
            schedule_id=selected_schedule.schedule_id,
            seat_class=selected_seat_class,
            passengers=spec["passengers"],
            user_id=str(task_user_id),
        )
        duration_ms = int((time.perf_counter() - timer) * 1000)
        return outcome, duration_ms, int(limit.waited_ms), int(limit.rounds)

    first_reserve_timer = time.perf_counter()
    try:
        reserve_outcome, first_duration_ms, first_wait_ms, first_rounds = await _reserve_once()
    except Exception as exc:
        reserve_duration = int((time.perf_counter() - first_reserve_timer) * 1000)
        attempts.append(
            PendingAttempt(
                action=ATTEMPT_ACTION_RESERVE,
                provider=provider,
                ok=False,
                retryable=True,
                error_code="provider_transport_error",
                error_message_safe=f"{provider} reserve transport error: {type(exc).__name__}",
                duration_ms=reserve_duration,
                meta_json_safe={
                    "rate_limit_wait_ms": 0,
                    "selected_schedule_id": selected_schedule.schedule_id,
                },
                started_at=reserve_started,
            )
        )
        return ProviderExecutionResult(provider=provider, attempts=attempts, candidate=None, retryable=True)

    reserve_duration += first_duration_ms
    reserve_limit_wait_ms = first_wait_ms
    reserve_limit_rounds = first_rounds
    if not reserve_outcome.ok and _is_provider_auth_required_error(reserve_outcome):
        relogin_retry_attempted = True
        initial_reserve_error_code = reserve_outcome.error_code
        initial_reserve_error_message = reserve_outcome.error_message_safe

        relogin_timer = time.perf_counter()
        try:
            relogin_outcome = await client.login(
                user_id=str(task_user_id),
                credentials=credentials,
            )
        except Exception as exc:
            relogin_duration_ms = int((time.perf_counter() - relogin_timer) * 1000)
            reserve_duration += relogin_duration_ms
            reserve_outcome = ProviderOutcome(
                ok=False,
                retryable=True,
                error_code="provider_relogin_transport_error",
                error_message_safe=f"{provider} relogin transport error: {type(exc).__name__}",
            )
        else:
            relogin_duration_ms = int((time.perf_counter() - relogin_timer) * 1000)
            reserve_duration += relogin_duration_ms
            if not relogin_outcome.ok:
                reserve_outcome = ProviderOutcome(
                    ok=False,
                    retryable=True,
                    error_code=relogin_outcome.error_code or "relogin_failed",
                    error_message_safe=relogin_outcome.error_message_safe or f"{provider} relogin failed",
                )
            else:
                try:
                    reserve_outcome, second_duration_ms, second_wait_ms, second_rounds = await _reserve_once()
                except Exception as exc:
                    reserve_outcome = ProviderOutcome(
                        ok=False,
                        retryable=True,
                        error_code="provider_transport_error",
                        error_message_safe=f"{provider} reserve retry transport error: {type(exc).__name__}",
                    )
                else:
                    reserve_duration += second_duration_ms
                    reserve_limit_wait_ms = second_wait_ms
                    reserve_limit_rounds = second_rounds

    reservation_id = str(reserve_outcome.data.get("reservation_id") or "")
    reserve_ok = bool(reserve_outcome.ok and reservation_id)
    reserve_error_code = reserve_outcome.error_code
    reserve_error_message = reserve_outcome.error_message_safe
    reserve_retryable = bool(reserve_outcome.retryable)

    if reserve_outcome.ok and not reservation_id:
        reserve_ok = False
        reserve_retryable = True
        reserve_error_code = "reservation_id_missing"
        reserve_error_message = f"{provider} reserve succeeded but reservation id was missing"
    elif relogin_retry_attempted and not reserve_ok:
        reserve_retryable = True

    non_payment_expiry_retry = (not auto_pay_enabled and not reserve_ok and _is_non_payment_expiry_reserve_error(reserve_outcome))
    if non_payment_expiry_retry:
        reserve_retryable = True
    sold_out_retry = not reserve_ok and _is_transient_sold_out_reserve_error(reserve_outcome)
    if sold_out_retry:
        reserve_retryable = True

    attempts.append(
        PendingAttempt(
            action=ATTEMPT_ACTION_RESERVE,
            provider=provider,
            ok=reserve_ok,
            retryable=reserve_retryable,
            error_code=reserve_error_code,
            error_message_safe=reserve_error_message,
            duration_ms=reserve_duration,
            meta_json_safe={
                "rate_limit_wait_ms": reserve_limit_wait_ms,
                "rate_limit_rounds": reserve_limit_rounds,
                "requested_seat_class": spec["seat_class"],
                "reserved_seat_class": selected_seat_class,
                "reservation_id": reservation_id or None,
                "schedule_id": selected_schedule.schedule_id,
                "auth_relogin_retry": relogin_retry_attempted,
                "auth_relogin_duration_ms": relogin_duration_ms if relogin_retry_attempted else None,
                "initial_error_code": initial_reserve_error_code,
                "initial_error_message": initial_reserve_error_message,
                "non_payment_expiry_retry": non_payment_expiry_retry,
                "sold_out_retry": sold_out_retry,
            },
            started_at=reserve_started,
        )
    )

    if not reserve_ok:
        return ProviderExecutionResult(
            provider=provider,
            attempts=attempts,
            candidate=None,
            retryable=reserve_retryable,
            schedule_backfill=schedule_backfill,
        )

    return ProviderExecutionResult(
        provider=provider,
        attempts=attempts,
        candidate=ReservationCandidate(
            provider=provider,
            rank=selected_rank,
            schedule=selected_schedule,
            seat_class_reserved=selected_seat_class,
            reservation_id=reservation_id,
            reserve_data=reserve_outcome.data,
            client=client,
        ),
        retryable=False,
        schedule_backfill=schedule_backfill,
    )


async def _attempt_cancel_candidate(
    *,
    candidate: ReservationCandidate,
    task_user_id: UUID,
    limiter: RedisTokenBucketLimiter,
) -> tuple[PendingAttempt, ProviderOutcome]:
    started_at = utc_now()
    timer = time.perf_counter()

    limit = await limiter.acquire_provider_call(
        provider=candidate.provider,
        user_bucket_key=str(task_user_id),
        host_bucket_key="default-host",
    )
    outcome = await candidate.client.cancel(
        artifact_data={
            "reservation_id": candidate.reservation_id,
            **candidate.reserve_data,
        },
        user_id=str(task_user_id),
    )
    duration = int((time.perf_counter() - timer) * 1000)
    attempt = PendingAttempt(
        action=ATTEMPT_ACTION_CANCEL,
        provider=candidate.provider,
        ok=outcome.ok,
        retryable=bool(outcome.retryable),
        error_code=outcome.error_code,
        error_message_safe=outcome.error_message_safe,
        duration_ms=duration,
        meta_json_safe={
            "rate_limit_wait_ms": limit.waited_ms,
            "reservation_id": candidate.reservation_id,
            "schedule_id": candidate.schedule.schedule_id,
        },
        started_at=started_at,
    )
    return attempt, outcome


async def _attempt_pay_reservation(
    *,
    provider: str,
    client: Any,
    reservation_id: str,
    task_user_id: UUID,
    limiter: RedisTokenBucketLimiter,
    payment_card: dict[str, Any] | None = None,
    credentials: dict[str, str] | None = None,
) -> tuple[PendingAttempt, ProviderOutcome]:
    started_at = utc_now()
    duration = 0
    rate_limit_wait_ms = 0
    rate_limit_rounds = 0
    relogin_retry_attempted = False
    relogin_duration_ms = 0
    initial_error_code: str | None = None
    initial_error_message: str | None = None

    async def _pay_once() -> tuple[ProviderOutcome, int, int, int]:
        timer = time.perf_counter()
        limit = await limiter.acquire_provider_call(
            provider=provider,
            user_bucket_key=str(task_user_id),
            host_bucket_key="default-host",
        )
        outcome = await client.pay(
            reservation_id=reservation_id,
            user_id=str(task_user_id),
            payment_card=payment_card,
        )
        call_duration = int((time.perf_counter() - timer) * 1000)
        return outcome, call_duration, int(limit.waited_ms), int(limit.rounds)

    outcome, pay_duration, wait_ms, rounds = await _pay_once()
    duration += pay_duration
    rate_limit_wait_ms = wait_ms
    rate_limit_rounds = rounds

    if not outcome.ok and credentials and _is_provider_auth_required_error(outcome):
        relogin_retry_attempted = True
        initial_error_code = outcome.error_code
        initial_error_message = outcome.error_message_safe
        relogin_timer = time.perf_counter()
        try:
            relogin_outcome = await client.login(
                user_id=str(task_user_id),
                credentials=credentials,
            )
        except Exception as exc:
            relogin_duration_ms = int((time.perf_counter() - relogin_timer) * 1000)
            duration += relogin_duration_ms
            outcome = ProviderOutcome(
                ok=False,
                retryable=True,
                error_code="provider_relogin_transport_error",
                error_message_safe=f"{provider} relogin transport error: {type(exc).__name__}",
            )
        else:
            relogin_duration_ms = int((time.perf_counter() - relogin_timer) * 1000)
            duration += relogin_duration_ms
            if not relogin_outcome.ok:
                outcome = ProviderOutcome(
                    ok=False,
                    retryable=True,
                    error_code=relogin_outcome.error_code or "relogin_failed",
                    error_message_safe=relogin_outcome.error_message_safe or f"{provider} relogin failed",
                )
            else:
                outcome, pay_duration, wait_ms, rounds = await _pay_once()
                duration += pay_duration
                rate_limit_wait_ms = wait_ms
                rate_limit_rounds = rounds

    attempt = PendingAttempt(
        action=ATTEMPT_ACTION_PAY,
        provider=provider,
        ok=outcome.ok,
        retryable=bool(outcome.retryable),
        error_code=outcome.error_code,
        error_message_safe=outcome.error_message_safe,
        duration_ms=duration,
        meta_json_safe={
            "rate_limit_wait_ms": rate_limit_wait_ms,
            "rate_limit_rounds": rate_limit_rounds,
            "reservation_id": reservation_id,
            "payment_id": outcome.data.get("payment_id"),
            "payment_card_configured": bool(payment_card),
            "auth_relogin_retry": relogin_retry_attempted,
            "auth_relogin_duration_ms": relogin_duration_ms if relogin_retry_attempted else None,
            "initial_error_code": initial_error_code,
            "initial_error_message": initial_error_message,
        },
        started_at=started_at,
    )
    return attempt, outcome


async def _login_provider_client_for_worker(
    db: AsyncSession,
    *,
    task: Task,
    provider: str,
) -> tuple[Any | None, PendingAttempt | None, bool]:
    credentials = await _load_provider_credentials(db, user_id=task.user_id, provider=provider)
    if credentials is None:
        return (
            None,
            PendingAttempt(
                action=ATTEMPT_ACTION_SEARCH,
                provider=provider,
                ok=False,
                retryable=False,
                error_code="credentials_missing",
                error_message_safe=f"{provider} credentials are missing",
                duration_ms=0,
                meta_json_safe={"stage": "login"},
                started_at=utc_now(),
            ),
            False,
        )

    client = get_provider_client(provider)
    started_at = utc_now()
    timer = time.perf_counter()
    try:
        login_outcome = await client.login(
            user_id=str(task.user_id),
            credentials=credentials,
        )
    except Exception as exc:
        duration = int((time.perf_counter() - timer) * 1000)
        return (
            None,
            PendingAttempt(
                action=ATTEMPT_ACTION_SEARCH,
                provider=provider,
                ok=False,
                retryable=True,
                error_code="provider_login_transport_error",
                error_message_safe=f"{provider} login transport error: {type(exc).__name__}",
                duration_ms=duration,
                meta_json_safe={"stage": "login"},
                started_at=started_at,
            ),
            True,
        )

    duration = int((time.perf_counter() - timer) * 1000)
    if not login_outcome.ok:
        return (
            None,
            PendingAttempt(
                action=ATTEMPT_ACTION_SEARCH,
                provider=provider,
                ok=False,
                retryable=bool(login_outcome.retryable),
                error_code=login_outcome.error_code or "login_failed",
                error_message_safe=login_outcome.error_message_safe or f"{provider} login failed",
                duration_ms=duration,
                meta_json_safe={"stage": "login"},
                started_at=started_at,
            ),
            bool(login_outcome.retryable),
        )

    return client, None, False


def _build_ticket_data(
    *,
    candidate: ReservationCandidate,
    spec: dict[str, Any],
    paid: bool,
    payment_id: str | None = None,
    ticket_no: str | None = None,
    sync_snapshot: dict[str, Any] | None = None,
    extra_provider_http: dict[str, Any] | None = None,
) -> dict[str, Any]:
    payload: dict[str, Any] = {
        "provider": candidate.provider,
        "reservation_id": candidate.reservation_id,
        "schedule_id": candidate.schedule.schedule_id,
        "departure_at": candidate.schedule.departure_at.isoformat(),
        "arrival_at": candidate.schedule.arrival_at.isoformat(),
        "train_no": candidate.schedule.train_no,
        "dep": candidate.schedule.dep,
        "arr": candidate.schedule.arr,
        "seat_class_requested": spec["seat_class"],
        "seat_class_reserved": candidate.seat_class_reserved,
        "paid": paid,
        "status": "paid" if paid else "reserved",
        "selected_rank": candidate.rank,
    }
    for key in ("journey_no", "journey_cnt", "rsv_chg_no", "wct_no"):
        value = candidate.reserve_data.get(key)
        if value:
            payload[key] = value
    if payment_id:
        payload["payment_id"] = payment_id
    if ticket_no:
        payload["ticket_no"] = ticket_no

    provider_http: dict[str, Any] = {}
    reserve_trace = candidate.reserve_data.get("http_trace")
    if reserve_trace:
        provider_http["reserve"] = redact_sensitive(reserve_trace)
    if extra_provider_http:
        provider_http.update(redact_sensitive(extra_provider_http))
    if provider_http:
        payload["provider_http"] = provider_http

    if sync_snapshot:
        compact_sync_meta = _compact_provider_sync_payload(sync_snapshot.get("provider_sync"))
        for key in (
            "status",
            "paid",
            "waiting",
            "expired",
            "payment_deadline_at",
            "tickets",
            "seat_count",
            "reservation_snapshot",
        ):
            value = sync_snapshot.get(key)
            if value is not None:
                payload[key] = value
        if compact_sync_meta is not None:
            payload["provider_sync"] = compact_sync_meta
        sync_http = sync_snapshot.get("provider_http")
        if isinstance(sync_http, dict):
            merged_http = _merge_provider_http_once(
                current_payload=dict(payload.get("provider_http") or {}),
                incoming_payload=redact_sensitive(sync_http),
            )
            if merged_http is not None:
                payload["provider_http"] = merged_http
    return validate_safe_metadata(payload)


def _is_waiting_snapshot(snapshot: dict[str, Any] | None) -> bool:
    if not snapshot:
        return False
    if bool(snapshot.get("paid")):
        return False
    if str(snapshot.get("status") or "").lower() == "waiting":
        return True
    return bool(snapshot.get("waiting"))


def _is_paid_snapshot(snapshot: dict[str, Any] | None) -> bool:
    if not snapshot:
        return False
    if bool(snapshot.get("paid")):
        return True
    return str(snapshot.get("status") or "").lower() == "paid"


def _is_reservation_not_found_snapshot(snapshot: dict[str, Any] | None) -> bool:
    if not snapshot:
        return False
    if snapshot.get("reservation_found") is False:
        return True
    return str(snapshot.get("status") or "").lower() == "reservation_not_found"


def _normalize_ticket_status(value: Any) -> str | None:
    normalized = str(value or "").strip().lower()
    return normalized or None


def _compact_provider_sync_payload(value: Any) -> dict[str, Any] | None:
    if not isinstance(value, dict):
        return None
    compact: dict[str, Any] = {}
    for key in SYNC_PROVIDER_META_STABLE_KEYS:
        if key in value:
            compact[key] = value[key]
    return compact or None


def _merge_provider_http_once(*, current_payload: dict[str, Any] | None, incoming_payload: Any) -> dict[str, Any] | None:
    existing = dict(current_payload or {})
    if not isinstance(incoming_payload, dict):
        return existing or None
    for key, value in incoming_payload.items():
        key_str = str(key)
        if key_str not in existing and value is not None:
            existing[key_str] = value
    return existing or None


def _merge_ticket_sync_snapshot(
    artifact: Artifact, *, sync_snapshot: dict[str, Any]
) -> tuple[str | None, str | None, bool]:
    merged = dict(artifact.data_json_safe or {})
    previous_status = _normalize_ticket_status(merged.get("status"))
    compact_sync_meta = _compact_provider_sync_payload(sync_snapshot.get("provider_sync"))
    for key in (
        "status",
        "paid",
        "waiting",
        "expired",
        "payment_deadline_at",
        "tickets",
        "seat_count",
        "reservation_snapshot",
    ):
        value = sync_snapshot.get(key)
        if value is not None:
            merged[key] = value
    if compact_sync_meta is not None:
        merged["provider_sync"] = compact_sync_meta

    sync_http = sync_snapshot.get("provider_http")
    if isinstance(sync_http, dict):
        merged_http = _merge_provider_http_once(
            current_payload=dict(merged.get("provider_http") or {}),
            incoming_payload=redact_sensitive(sync_http),
        )
        if merged_http is not None:
            merged["provider_http"] = merged_http

    changed = merged != dict(artifact.data_json_safe or {})
    if changed:
        artifact.data_json_safe = validate_safe_metadata(merged)
    current_status = _normalize_ticket_status(merged.get("status"))
    return previous_status, current_status, changed


def _is_shutdown_requested(ctx: dict) -> bool:
    """Check if worker shutdown has been requested."""
    shutdown_event = ctx.get("shutdown_event")
    if shutdown_event and shutdown_event.is_set():
        return True
    return False


async def run_train_task(ctx: dict, task_id: str) -> None:
    db_factory = ctx["db_factory"]
    redis = ctx["redis"]
    limiter = RedisTokenBucketLimiter(redis)
    
    # Register task as in-flight for graceful shutdown tracking
    register_fn = ctx.get("register_in_flight")
    unregister_fn = ctx.get("unregister_in_flight")
    if register_fn:
        register_fn(task_id)
    
    try:
        await _run_train_task_inner(ctx, task_id, db_factory, redis, limiter)
    finally:
        # Unregister from in-flight tracking
        if unregister_fn:
            unregister_fn(task_id)


async def _run_train_task_inner(
    ctx: dict,
    task_id: str,
    db_factory,
    redis,
    limiter: RedisTokenBucketLimiter,
) -> None:
    """Inner implementation of train task processing."""
    async with db_factory() as db:
        task = await db.get(Task, UUID(task_id))
        if task is None or task.module != TASK_MODULE:
            return

        now = _utc_now_aware()
        
        # Skip tasks that are in terminal states, paused, or hidden (deleted)
        if task.state in TERMINAL_TASK_STATES or task.state == "PAUSED":
            return
        if task.hidden_at is not None:
            # Task was deleted by user - don't process
            logger.debug("Skipping hidden/deleted task %s", task_id)
            return
        if task.paused_at is not None and task.state != "PAUSED":
            # Task was paused but state not updated - fix it
            task.state = "PAUSED"
            task.updated_at = now
            await db.commit()
            await publish_task_state_event(user_id=task.user_id, task_id=task.id, state=task.state, updated_at=task.updated_at)
            return
        if task.cancelled_at is not None:
            task.state = "CANCELLED"
            task.updated_at = now
            await db.commit()
            await publish_task_state_event(user_id=task.user_id, task_id=task.id, state=task.state, updated_at=task.updated_at)
            return
        if now >= _as_aware_utc(task.deadline_at):
            await _mark_expired(db, task)
            return

        spec = dict(task.spec_json or {})
        auto_pay_enabled = bool((await is_payment_runtime_enabled(db)) and spec.get("auto_pay", True))
        ranked = _normalize_ranked_selection(spec)
        if not ranked:
            await _mark_failed(db, task)
            return
        seconds_until_departure = _seconds_until_next_departure(ranked)

        providers = sorted({row["provider"] for row in ranked})
        if not providers:
            await _mark_failed(db, task)
            return

        search_attempt_count = (
            await db.execute(
                select(func.count(TaskAttempt.id))
                .where(TaskAttempt.task_id == task.id)
                .where(TaskAttempt.action == ATTEMPT_ACTION_SEARCH)
            )
        ).scalar_one()

        existing_ticket_artifacts = await _load_ticket_artifacts(db, task_id=task.id)
        existing_paid_artifact = _find_paid_ticket_artifact(existing_ticket_artifacts)
        if existing_paid_artifact is not None:
            await _mark_completed(db, task)
            return

        open_ticket_artifact = _find_open_ticket_artifact(existing_ticket_artifacts)
        if open_ticket_artifact is not None:
            spec, backfilled = _apply_ticket_artifact_schedule_backfill(spec, open_ticket_artifact)
            if backfilled:
                task.spec_json = spec
                ranked = _normalize_ranked_selection(spec)
                seconds_until_departure = _seconds_until_next_departure(ranked)

            if not auto_pay_enabled:
                await _mark_completed(db, task)
                return

            provider = str(open_ticket_artifact.data_json_safe.get("provider") or "")
            reservation_id = str(open_ticket_artifact.data_json_safe.get("reservation_id") or "")
            if provider not in {"SRT", "KTX"} or not reservation_id:
                await _mark_failed(db, task)
                return

            client, login_attempt, login_retryable = await _login_provider_client_for_worker(
                db,
                task=task,
                provider=provider,
            )
            if login_attempt is not None:
                await _persist_attempts(db, task=task, attempts=[login_attempt])
                if login_retryable and _utc_now_aware() < _as_aware_utc(task.deadline_at):
                    await _schedule_retry(
                        db,
                        task,
                        _poll_delay_seconds(search_attempt_count + 1, seconds_until_departure=seconds_until_departure),
                    )
                else:
                    await _mark_failed(db, task)
                return

            open_sync_started_at = utc_now()
            open_sync_started_monotonic = time.perf_counter()
            open_sync_error_message_safe: str | None = None
            try:
                open_sync_snapshot = await fetch_ticket_sync_snapshot(
                    client=client,
                    provider=provider,
                    reservation_id=reservation_id,
                    user_id=task.user_id,
                    limiter=limiter,
                )
            except Exception as exc:
                open_sync_snapshot = {}
                open_sync_error_message_safe = f"provider_sync_error:{type(exc).__name__}"

            open_sync_status = str(open_sync_snapshot.get("status") or "").strip().lower()
            open_sync_meta: dict[str, Any] = {"stage": "ticket_sync_poll"}
            if open_sync_status:
                open_sync_meta["ticket_status"] = open_sync_status
            if "waiting" in open_sync_snapshot:
                open_sync_meta["waiting"] = bool(open_sync_snapshot.get("waiting"))
            if "paid" in open_sync_snapshot:
                open_sync_meta["paid"] = bool(open_sync_snapshot.get("paid"))

            await _persist_attempts(
                db,
                task=task,
                attempts=[
                    PendingAttempt(
                        action=ATTEMPT_ACTION_SYNC,
                        provider=provider,
                        ok=open_sync_error_message_safe is None,
                        retryable=open_sync_error_message_safe is not None or not bool(open_sync_snapshot.get("paid")),
                        error_code="provider_sync_error" if open_sync_error_message_safe is not None else None,
                        error_message_safe=open_sync_error_message_safe,
                        duration_ms=max(0, int((time.perf_counter() - open_sync_started_monotonic) * 1000)),
                        meta_json_safe=open_sync_meta,
                        started_at=open_sync_started_at,
                    )
                ],
            )

            if open_sync_snapshot:
                previous_ticket_status, current_ticket_status, ticket_snapshot_changed = _merge_ticket_sync_snapshot(
                    open_ticket_artifact, sync_snapshot=open_sync_snapshot
                )
                if ticket_snapshot_changed:
                    task.updated_at = utc_now()
                    await db.commit()
                if previous_ticket_status != current_ticket_status:
                    await publish_task_ticket_status_event(
                        user_id=task.user_id,
                        task_id=task.id,
                        state=task.state,
                        previous_ticket_status=previous_ticket_status,
                        ticket_status=current_ticket_status,
                        updated_at=task.updated_at,
                    )

                if bool(open_sync_snapshot.get("paid")):
                    await _mark_completed(db, task)
                    return
                if _is_waiting_snapshot(open_sync_snapshot):
                    if _utc_now_aware() < _as_aware_utc(task.deadline_at):
                        await _schedule_retry(db, task, WAITING_STATUS_POLL_SECONDS)
                    else:
                        await _mark_failed(db, task)
                    return

            task.state = "PAYING"
            task.updated_at = utc_now()
            await db.commit()
            await publish_task_state_event(user_id=task.user_id, task_id=task.id, state=task.state, updated_at=task.updated_at)

            provider_credentials = await _load_provider_credentials(
                db,
                user_id=task.user_id,
                provider=provider,
            )
            pay_attempt, pay_outcome = await _attempt_pay_reservation(
                provider=provider,
                client=client,
                reservation_id=reservation_id,
                task_user_id=task.user_id,
                limiter=limiter,
                payment_card=await get_payment_card_for_execution(db, user_id=task.user_id),
                credentials=provider_credentials,
            )
            await _persist_attempts(db, task=task, attempts=[pay_attempt])

            if not pay_outcome.ok:
                try:
                    failed_pay_sync_snapshot = await fetch_ticket_sync_snapshot(
                        client=client,
                        provider=provider,
                        reservation_id=reservation_id,
                        user_id=task.user_id,
                        limiter=limiter,
                    )
                except Exception:
                    failed_pay_sync_snapshot = {}

                if failed_pay_sync_snapshot:
                    previous_ticket_status, current_ticket_status, ticket_snapshot_changed = _merge_ticket_sync_snapshot(
                        open_ticket_artifact, sync_snapshot=failed_pay_sync_snapshot
                    )
                    if ticket_snapshot_changed:
                        task.updated_at = utc_now()
                        await db.commit()
                    if previous_ticket_status != current_ticket_status:
                        await publish_task_ticket_status_event(
                            user_id=task.user_id,
                            task_id=task.id,
                            state=task.state,
                            previous_ticket_status=previous_ticket_status,
                            ticket_status=current_ticket_status,
                            updated_at=task.updated_at,
                        )

                    if _is_paid_snapshot(failed_pay_sync_snapshot):
                        await _mark_completed(db, task)
                        return
                    if _is_waiting_snapshot(failed_pay_sync_snapshot) and _utc_now_aware() < _as_aware_utc(task.deadline_at):
                        await _schedule_retry(db, task, WAITING_STATUS_POLL_SECONDS)
                        return
                    if _is_reservation_not_found_snapshot(failed_pay_sync_snapshot) and _utc_now_aware() < _as_aware_utc(
                        task.deadline_at
                    ):
                        await _schedule_retry(db, task, WAITING_STATUS_POLL_SECONDS)
                        return

                if pay_outcome.retryable and _utc_now_aware() < _as_aware_utc(task.deadline_at):
                    await _schedule_retry(
                        db,
                        task,
                        _poll_delay_seconds(search_attempt_count + 1, seconds_until_departure=seconds_until_departure),
                    )
                else:
                    await _mark_failed(db, task)
                return

            try:
                sync_snapshot = await fetch_ticket_sync_snapshot(
                    client=client,
                    provider=provider,
                    reservation_id=reservation_id,
                    user_id=task.user_id,
                    limiter=limiter,
                )
            except Exception:
                sync_snapshot = {}
            provider_http = dict(open_ticket_artifact.data_json_safe.get("provider_http") or {})
            pay_trace = pay_outcome.data.get("http_trace")
            if pay_trace:
                provider_http["pay"] = redact_sensitive(pay_trace)

            open_ticket_artifact.data_json_safe = validate_safe_metadata(
                {
                    **open_ticket_artifact.data_json_safe,
                    "paid": True,
                    "status": "paid",
                    "payment_id": pay_outcome.data.get("payment_id"),
                    "ticket_no": pay_outcome.data.get("ticket_no"),
                    "provider_http": provider_http,
                    "provider_sync": sync_snapshot.get("provider_sync"),
                    "reservation_snapshot": sync_snapshot.get("reservation_snapshot"),
                    "tickets": sync_snapshot.get("tickets", open_ticket_artifact.data_json_safe.get("tickets", [])),
                    "seat_count": sync_snapshot.get("seat_count"),
                    "payment_deadline_at": sync_snapshot.get("payment_deadline_at"),
                }
            )
            task.updated_at = utc_now()
            await db.commit()
            await _mark_completed(db, task)
            return

        task.state = "RUNNING"
        task.updated_at = now
        await db.commit()
        await publish_task_state_event(user_id=task.user_id, task_id=task.id, state=task.state, updated_at=task.updated_at)

        provider_results: list[ProviderExecutionResult] = []
        provider_credentials_map: dict[str, dict[str, str]] = {}
        jobs: list[asyncio.Future] = []

        for provider in providers:
            credentials = await _load_provider_credentials(db, user_id=task.user_id, provider=provider)
            if credentials is None:
                provider_results.append(
                    ProviderExecutionResult(
                        provider=provider,
                        attempts=[
                            PendingAttempt(
                                action=ATTEMPT_ACTION_SEARCH,
                                provider=provider,
                                ok=False,
                                retryable=False,
                                error_code="credentials_missing",
                                error_message_safe=f"{provider} credentials are missing",
                                duration_ms=0,
                                meta_json_safe={"stage": "login"},
                                started_at=utc_now(),
                            )
                        ],
                        candidate=None,
                        retryable=False,
                    )
                )
                continue

            provider_credentials_map[provider] = credentials
            ranked_for_provider = [row for row in ranked if row["provider"] == provider]
            jobs.append(
                asyncio.create_task(
                    _provider_search_and_reserve(
                        provider=provider,
                        ranked_for_provider=ranked_for_provider,
                        spec=spec,
                        auto_pay_enabled=auto_pay_enabled,
                        task_user_id=task.user_id,
                        credentials=credentials,
                        limiter=limiter,
                    )
                )
            )

        if jobs:
            provider_results.extend(await asyncio.gather(*jobs))

        spec, backfilled = _apply_ranked_schedule_backfill(spec, provider_results)
        if backfilled:
            task.spec_json = spec
            ranked = _normalize_ranked_selection(spec)
            seconds_until_departure = _seconds_until_next_departure(ranked)

        for result in provider_results:
            await _persist_attempts(db, task=task, attempts=result.attempts)

        if _utc_now_aware() >= _as_aware_utc(task.deadline_at):
            await _mark_expired(db, task)
            return

        candidates = [result.candidate for result in provider_results if result.candidate is not None]
        if not candidates:
            retryable_any = any(result.retryable for result in provider_results)
            if retryable_any and _utc_now_aware() < _as_aware_utc(task.deadline_at):
                await _schedule_retry(
                    db,
                    task,
                    _poll_delay_seconds(search_attempt_count + 1, seconds_until_departure=seconds_until_departure),
                )
            else:
                await _mark_failed(db, task)
            return

        winner = min(candidates, key=lambda row: row.rank)
        losers = [candidate for candidate in candidates if candidate is not winner]

        task.state = "RESERVING"
        task.updated_at = utc_now()

        try:
            reservation_sync_snapshot = await fetch_ticket_sync_snapshot(
                client=winner.client,
                provider=winner.provider,
                reservation_id=winner.reservation_id,
                user_id=task.user_id,
                limiter=limiter,
            )
        except Exception:
            reservation_sync_snapshot = {}

        ticket_artifact = Artifact(
            task_id=task.id,
            module=TASK_MODULE,
            kind="ticket",
            data_json_safe=_build_ticket_data(
                candidate=winner,
                spec=spec,
                paid=False,
                sync_snapshot=reservation_sync_snapshot,
            ),
        )
        db.add(ticket_artifact)
        await db.commit()
        await publish_task_state_event(user_id=task.user_id, task_id=task.id, state=task.state, updated_at=task.updated_at)

        for loser in losers:
            cancel_attempt, cancel_outcome = await _attempt_cancel_candidate(
                candidate=loser,
                task_user_id=task.user_id,
                limiter=limiter,
            )
            await _persist_attempts(db, task=task, attempts=[cancel_attempt])

            if cancel_outcome.ok:
                continue
            if cancel_outcome.error_code == "not_supported":
                continue
            if cancel_outcome.retryable and _utc_now_aware() < _as_aware_utc(task.deadline_at):
                await _schedule_retry(
                    db,
                    task,
                    _poll_delay_seconds(search_attempt_count + 1, seconds_until_departure=seconds_until_departure),
                )
            else:
                await _mark_failed(db, task)
            return

        if not auto_pay_enabled:
            await _mark_completed(db, task)
            return

        task.state = "PAYING"
        task.updated_at = utc_now()
        await db.commit()
        await publish_task_state_event(user_id=task.user_id, task_id=task.id, state=task.state, updated_at=task.updated_at)

        pay_attempt, pay_outcome = await _attempt_pay_reservation(
            provider=winner.provider,
            client=winner.client,
            reservation_id=winner.reservation_id,
            task_user_id=task.user_id,
            limiter=limiter,
            payment_card=await get_payment_card_for_execution(db, user_id=task.user_id),
            credentials=provider_credentials_map.get(winner.provider),
        )
        await _persist_attempts(db, task=task, attempts=[pay_attempt])

        if not pay_outcome.ok:
            try:
                failed_pay_sync_snapshot = await fetch_ticket_sync_snapshot(
                    client=winner.client,
                    provider=winner.provider,
                    reservation_id=winner.reservation_id,
                    user_id=task.user_id,
                    limiter=limiter,
                )
            except Exception:
                failed_pay_sync_snapshot = {}

            if failed_pay_sync_snapshot:
                previous_ticket_status, current_ticket_status, ticket_snapshot_changed = _merge_ticket_sync_snapshot(
                    ticket_artifact, sync_snapshot=failed_pay_sync_snapshot
                )
                if ticket_snapshot_changed:
                    task.updated_at = utc_now()
                    await db.commit()
                if previous_ticket_status != current_ticket_status:
                    await publish_task_ticket_status_event(
                        user_id=task.user_id,
                        task_id=task.id,
                        state=task.state,
                        previous_ticket_status=previous_ticket_status,
                        ticket_status=current_ticket_status,
                        updated_at=task.updated_at,
                    )

                if _is_paid_snapshot(failed_pay_sync_snapshot):
                    await _mark_completed(db, task)
                    return
                if _is_waiting_snapshot(failed_pay_sync_snapshot) and _utc_now_aware() < _as_aware_utc(task.deadline_at):
                    await _schedule_retry(db, task, WAITING_STATUS_POLL_SECONDS)
                    return
                if _is_reservation_not_found_snapshot(failed_pay_sync_snapshot) and _utc_now_aware() < _as_aware_utc(
                    task.deadline_at
                ):
                    await _schedule_retry(db, task, WAITING_STATUS_POLL_SECONDS)
                    return

            if pay_outcome.retryable and _utc_now_aware() < _as_aware_utc(task.deadline_at):
                await _schedule_retry(
                    db,
                    task,
                    _poll_delay_seconds(search_attempt_count + 1, seconds_until_departure=seconds_until_departure),
                )
            else:
                await _mark_failed(db, task)
            return

        try:
            post_pay_sync_snapshot = await fetch_ticket_sync_snapshot(
                client=winner.client,
                provider=winner.provider,
                reservation_id=winner.reservation_id,
                user_id=task.user_id,
                limiter=limiter,
            )
        except Exception:
            post_pay_sync_snapshot = {}

        ticket_artifact.data_json_safe = _build_ticket_data(
            candidate=winner,
            spec=spec,
            paid=True,
            payment_id=pay_outcome.data.get("payment_id"),
            ticket_no=pay_outcome.data.get("ticket_no"),
            sync_snapshot=post_pay_sync_snapshot,
            extra_provider_http={"pay": pay_outcome.data.get("http_trace")},
        )
        task.updated_at = utc_now()
        await db.commit()

        await _mark_completed(db, task)


# Stale task threshold: tasks stuck in a processing state longer than this
# are considered interrupted and will be recovered
STALE_TASK_THRESHOLD_SECONDS = 600  # 10 minutes


async def enqueue_recoverable_tasks(db: AsyncSession) -> int:
    """
    Re-enqueue active tasks on worker startup.
    
    This handles:
    1. Tasks that were queued but not yet processed
    2. Tasks that were in-progress when worker crashed/restarted
    3. Tasks stuck in processing states (stale tasks)
    
    Skipped tasks:
    - PAUSED: require explicit resume action
    - hidden_at set: user deleted the task
    - cancelled_at set: user cancelled the task
    """
    now = _utc_now_aware()
    
    stmt = (
        select(Task)
        .where(Task.module == TASK_MODULE)
        .where(Task.state.in_(ACTIVE_TASK_STATES))
        .where(Task.hidden_at.is_(None))  # Exclude deleted tasks
        .where(Task.cancelled_at.is_(None))  # Exclude cancelled tasks
    )
    tasks = (await db.execute(stmt)).scalars().all()
    
    recovered_count = 0
    stale_count = 0
    skipped_paused = 0
    
    for task in tasks:
        # Skip paused tasks - they need explicit resume
        if task.state == "PAUSED" or task.paused_at is not None:
            skipped_paused += 1
            continue
            
        # Check for stale tasks (stuck in processing states)
        is_processing = task.state in ("RUNNING", "RESERVING", "PAYING", "POLLING")
        if is_processing and task.updated_at:
            updated_at = _as_aware_utc(task.updated_at)
            age_seconds = (now - updated_at).total_seconds()
            
            if age_seconds > STALE_TASK_THRESHOLD_SECONDS:
                # Reset stale task to QUEUED for clean re-processing
                logger.warning(
                    "Recovering stale task %s (state=%s, age=%.0fs)",
                    task.id, task.state, age_seconds
                )
                task.state = "QUEUED"
                task.updated_at = now
                await db.commit()
                stale_count += 1
        
        await enqueue_train_task(str(task.id))
        recovered_count += 1
    
    if stale_count:
        logger.info("Reset %d stale tasks to QUEUED", stale_count)
    if skipped_paused:
        logger.debug("Skipped %d paused tasks (require explicit resume)", skipped_paused)
    
    return recovered_count
