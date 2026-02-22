"use client";

import Link from "next/link";
import { FormEvent, useEffect, useMemo, useRef, useState } from "react";

import { useLocale } from "@/components/locale-provider";
import { clientApiBaseUrl } from "@/lib/api-base";
import { formatDateTimeKst, kstDateInputValue } from "@/lib/kst";
import { ROUTES } from "@/lib/routes";
import { UI_BUTTON_OUTLINE_SM, UI_BUTTON_PRIMARY, UI_BUTTON_DANGER_SM, UI_FIELD } from "@/lib/ui";
import {
  clearStoredDummyTaskCards,
  storeDummyTaskCards,
  TRAIN_DUMMY_TASKS_ENABLED,
} from "@/lib/train/dummy-task-cards";
import { formatStationLabel } from "@/lib/train/stations-i18n";
import type {
  TrainArtifact,
  TrainCredentialStatusResponse,
  TrainSchedule,
  TrainSeatClass,
  TrainStation,
  TrainTaskState,
  TrainTaskSummary,
  WalletPaymentCardStatus,
} from "@/lib/types";

type SearchFormState = {
  dep: string;
  arr: string;
  date: string;
  start: string;
  end: string;
  providers: { SRT: boolean; KTX: boolean };
};

type CreateTaskState = {
  seatClass: TrainSeatClass;
  adults: number;
  children: number;
  autoPay: boolean;
  notify: boolean;
};

type CredentialFormState = {
  username: string;
  password: string;
};

type CredentialProvider = "KTX" | "SRT";
type TaskListStatus = "active" | "completed";
type ScheduleSortOrder = "asc" | "desc";

const POLL_MS_DEFAULT = 60000;
const POLL_MS_AWAITING_PAYMENT = 10000;
const CREDENTIAL_STATUS_TIMEOUT_MS = 10000;
const DEFAULT_DEP_STATION = "수서";
const DEFAULT_ARR_STATION = "마산";
const TASK_LIST_ERROR_MESSAGE = "task_list_error";
const SESSION_EXPIRED_MESSAGE = "session_expired";
const ACTIVE_TASK_FETCH_LIMIT = 60;
const COMPLETED_TASK_FETCH_LIMIT = 80;
const COMPLETED_TASK_REFRESH_INTERVAL_TICKS = 3;
const ACTIVE_TASK_STATES_FOR_LIST = new Set<TrainTaskState>([
  "QUEUED",
  "RUNNING",
  "POLLING",
  "RESERVING",
  "PAYING",
  "PAUSED",
]);
const TRAIN_AUTO_PAY_FEATURE_ENABLED = ["1", "true", "yes", "on"].includes(
  (process.env.NEXT_PUBLIC_TRAIN_AUTO_PAY_ENABLED ?? "false").trim().toLowerCase(),
);
const TRAIN_TYPE_NAME_BY_CODE: Record<string, string> = {
  "00": "KTX",
  "02": "무궁화",
  "03": "통근열차",
  "04": "누리로",
  "05": "전체",
  "07": "KTX-산천",
  "08": "ITX-새마을",
  "09": "ITX-청춘",
  "10": "KTX-산천",
  "17": "SRT",
  "18": "ITX-마음",
};

/**
 * Normalize Korean phone numbers to 11-digit format (e.g., 01012345678).
 * Handles: 010-1234-5678, 010 1234 5678, +82-10-1234-5678, +8210-1234-5678, etc.
 * Returns original input if not a recognizable phone pattern.
 */
function normalizePhoneNumber(input: string): string {
  const trimmed = input.trim();
  // Remove all non-digit characters except leading +
  let digits = trimmed.replace(/[^\d+]/g, "");
  
  // Handle Korean international format (+82)
  if (digits.startsWith("+82")) {
    digits = "0" + digits.slice(3);
  } else if (digits.startsWith("82") && digits.length >= 11) {
    digits = "0" + digits.slice(2);
  }
  
  // Remove any remaining + signs
  digits = digits.replace(/\+/g, "");
  
  // Check if it looks like a Korean mobile number (starts with 01)
  if (/^01[0-9]/.test(digits) && digits.length >= 10 && digits.length <= 11) {
    return digits;
  }
  
  // Not a phone number pattern, return original trimmed input
  return trimmed;
}

const FIELD_BASE_CLASS = `mt-1 ${UI_FIELD}`;
const PRIMARY_BUTTON_CLASS = UI_BUTTON_PRIMARY;
const SMALL_BUTTON_CLASS = UI_BUTTON_OUTLINE_SM;
const SMALL_DANGER_BUTTON_CLASS = UI_BUTTON_DANGER_SM;
const SMALL_SUCCESS_BUTTON_CLASS =
  "inline-flex h-8 items-center justify-center rounded-full border border-emerald-200 bg-emerald-50 px-2.5 text-xs font-medium text-emerald-700 shadow-sm transition hover:bg-emerald-100 focus:outline-none focus:ring-2 focus:ring-emerald-100 disabled:cursor-not-allowed disabled:opacity-60";
const SMALL_DISABLED_BUTTON_CLASS =
  "inline-flex h-8 items-center justify-center rounded-full border border-slate-200 bg-slate-100 px-2.5 text-xs font-medium text-slate-500 shadow-sm transition focus:outline-none focus:ring-2 focus:ring-slate-100";
const EMPTY_CREDENTIAL_STATUS: TrainCredentialStatusResponse = {
  ktx: { configured: false, verified: false, username: null, verified_at: null, detail: null },
  srt: { configured: false, verified: false, username: null, verified_at: null, detail: null },
};

function isUnpaidAwaitingPaymentTicket(task: Pick<TrainTaskSummary, "ticket_status" | "ticket_paid">): boolean {
  return task.ticket_status === "awaiting_payment" && task.ticket_paid !== true;
}

function isActiveTaskForList(task: TrainTaskSummary): boolean {
  if (ACTIVE_TASK_STATES_FOR_LIST.has(task.state)) {
    return true;
  }
  return task.state === "COMPLETED" && isUnpaidAwaitingPaymentTicket(task);
}

function buildDummyTaskCards(now: Date = new Date()): { active: TrainTaskSummary[]; completed: TrainTaskSummary[] } {
  const isoAtOffsetMinutes = (minutesFromNow: number): string =>
    new Date(now.getTime() + minutesFromNow * 60_000).toISOString();

  const makeSpec = (provider: "SRT" | "KTX", departureAtIso: string): Record<string, unknown> => ({
    dep: "수서",
    arr: "부산",
    date: departureAtIso.slice(0, 10),
    passengers: { adults: 1, children: 0 },
    seat_class: "general_preferred",
    selected_trains_ranked: [
      {
        rank: 1,
        departure_at: departureAtIso,
        provider,
        schedule_id: `dummy-${provider.toLowerCase()}-${departureAtIso}`,
      },
    ],
  });

  const makeTask = (
    id: string,
    state: TrainTaskState,
    options: {
      provider?: "SRT" | "KTX";
      departureOffsetMinutes?: number;
      deadlineOffsetMinutes?: number;
      lastAttemptOffsetMinutes?: number | null;
      nextRunOffsetMinutes?: number | null;
      lastAttemptOk?: boolean | null;
      lastAttemptErrorCode?: string | null;
      lastAttemptErrorMessageSafe?: string | null;
      paused?: boolean;
      completed?: boolean;
      cancelled?: boolean;
      failed?: boolean;
      retryNowAllowed?: boolean;
      retryNowReason?: string | null;
      ticketStatus?: string | null;
      ticketPaid?: boolean | null;
      ticketPaymentDeadlineOffsetMinutes?: number | null;
      ticketReservationId?: string | null;
    } = {},
  ): TrainTaskSummary => {
    const provider = options.provider ?? "SRT";
    const departureAt = isoAtOffsetMinutes(options.departureOffsetMinutes ?? 120);
    const createdAt = isoAtOffsetMinutes(-45);
    const updatedAt = isoAtOffsetMinutes(-2);
    const deadlineAt = isoAtOffsetMinutes(options.deadlineOffsetMinutes ?? 240);
    const lastAttemptAt =
      options.lastAttemptOffsetMinutes === null
        ? null
        : isoAtOffsetMinutes(options.lastAttemptOffsetMinutes ?? -1);
    const nextRunAt =
      options.nextRunOffsetMinutes == null ? null : isoAtOffsetMinutes(options.nextRunOffsetMinutes);
    return {
      id: `dummy-${id}`,
      module: "train",
      state,
      deadline_at: deadlineAt,
      created_at: createdAt,
      updated_at: updatedAt,
      paused_at: options.paused ? isoAtOffsetMinutes(-20) : null,
      cancelled_at: options.cancelled ? isoAtOffsetMinutes(-10) : null,
      completed_at: options.completed ? isoAtOffsetMinutes(-6) : null,
      failed_at: options.failed ? isoAtOffsetMinutes(-6) : null,
      hidden_at: null,
      last_attempt_at: lastAttemptAt,
      last_attempt_action: lastAttemptAt ? "SEARCH" : null,
      last_attempt_ok: options.lastAttemptOk ?? null,
      last_attempt_error_code: options.lastAttemptErrorCode ?? null,
      last_attempt_error_message_safe: options.lastAttemptErrorMessageSafe ?? null,
      last_attempt_finished_at: lastAttemptAt,
      next_run_at: nextRunAt,
      retry_now_allowed: options.retryNowAllowed ?? true,
      retry_now_reason: options.retryNowReason ?? null,
      retry_now_available_at: null,
      spec_json: makeSpec(provider, departureAt),
      ticket_status: options.ticketStatus ?? null,
      ticket_paid: options.ticketPaid ?? null,
      ticket_payment_deadline_at:
        options.ticketPaymentDeadlineOffsetMinutes == null
          ? null
          : isoAtOffsetMinutes(options.ticketPaymentDeadlineOffsetMinutes),
      ticket_reservation_id: options.ticketReservationId ?? null,
    };
  };

  const allSampleTasks: TrainTaskSummary[] = [
    makeTask("queued", "QUEUED", {
      provider: "SRT",
      departureOffsetMinutes: 35,
      deadlineOffsetMinutes: 90,
      lastAttemptOffsetMinutes: null,
      nextRunOffsetMinutes: 1,
      retryNowAllowed: false,
      retryNowReason: "task_running",
    }),
    makeTask("polling-seat", "POLLING", {
      provider: "SRT",
      departureOffsetMinutes: 12 * 60 + 15,
      deadlineOffsetMinutes: 18 * 60,
      lastAttemptOk: false,
      lastAttemptErrorCode: "seat_unavailable",
      lastAttemptErrorMessageSafe: "No available seats in selected trains right now",
      nextRunOffsetMinutes: 1,
    }),
    makeTask("polling-error", "POLLING", {
      provider: "KTX",
      departureOffsetMinutes: 48 * 60 + 20,
      deadlineOffsetMinutes: 72 * 60,
      lastAttemptOk: false,
      lastAttemptErrorCode: "provider_transport_error",
      lastAttemptErrorMessageSafe: "KTX search transport error: TimeoutError",
      nextRunOffsetMinutes: 2,
    }),
    makeTask("paused", "PAUSED", {
      provider: "KTX",
      departureOffsetMinutes: 6 * 24 * 60 + 45,
      deadlineOffsetMinutes: 7 * 24 * 60,
      paused: true,
      lastAttemptOk: true,
      nextRunOffsetMinutes: null,
    }),
    makeTask("completed-awaiting-payment", "COMPLETED", {
      provider: "KTX",
      departureOffsetMinutes: 4 * 60 + 10,
      deadlineOffsetMinutes: 9 * 60,
      completed: true,
      lastAttemptOk: true,
      ticketStatus: "awaiting_payment",
      ticketPaid: false,
      ticketPaymentDeadlineOffsetMinutes: 30,
      ticketReservationId: "DUMMY-RSV-002",
    }),
    makeTask("completed-confirmed", "COMPLETED", {
      provider: "SRT",
      departureOffsetMinutes: -6 * 60,
      deadlineOffsetMinutes: -2 * 60,
      completed: true,
      lastAttemptOk: true,
      ticketStatus: "ticket_issued",
      ticketPaid: true,
      ticketReservationId: "DUMMY-RSV-001",
    }),
    makeTask("expired", "EXPIRED", {
      provider: "SRT",
      departureOffsetMinutes: -20 * 60,
      deadlineOffsetMinutes: -30,
      completed: true,
      lastAttemptOk: false,
      lastAttemptErrorCode: "deadline_passed",
      lastAttemptErrorMessageSafe: "Task deadline has passed.",
    }),
    makeTask("cancelled", "CANCELLED", {
      provider: "KTX",
      departureOffsetMinutes: -2 * 24 * 60 - 25,
      deadlineOffsetMinutes: -2 * 24 * 60 + 60,
      completed: true,
      cancelled: true,
      lastAttemptOk: true,
      ticketStatus: "cancelled",
      ticketPaid: false,
    }),
    makeTask("failed", "FAILED", {
      provider: "SRT",
      departureOffsetMinutes: -26 * 60,
      deadlineOffsetMinutes: -12 * 60,
      completed: true,
      failed: true,
      lastAttemptOk: false,
      lastAttemptErrorCode: "provider_transport_error",
      lastAttemptErrorMessageSafe: "Provider transport failed after retries.",
    }),
  ];

  return {
    active: allSampleTasks.filter((task) => isActiveTaskForList(task)),
    completed: allSampleTasks.filter((task) => !isActiveTaskForList(task)),
  };
}

async function fetchTasksByStatus(
  status: TaskListStatus,
  options?: { refreshCompleted?: boolean; limit?: number },
) {
  const query = new URLSearchParams({ status });
  if (status === "completed" && options?.refreshCompleted) {
    query.set("refresh_completed", "true");
  }
  query.set("limit", String(options?.limit ?? (status === "active" ? ACTIVE_TASK_FETCH_LIMIT : COMPLETED_TASK_FETCH_LIMIT)));

  const response = await fetch(`${clientApiBaseUrl}/api/train/tasks?${query.toString()}`, {
    credentials: "include",
    cache: "no-store",
  });

  if (!response.ok) {
    if (response.status === 401) {
      throw new Error(SESSION_EXPIRED_MESSAGE);
    }
    throw new Error(TASK_LIST_ERROR_MESSAGE);
  }

  const payload = (await response.json()) as { tasks: TrainTaskSummary[] };
  return payload.tasks;
}

async function parseApiErrorMessage(response: Response, fallback: string): Promise<string> {
  const contentType = response.headers.get("content-type") ?? "";
  if (contentType.includes("application/json")) {
    const payload = (await response.json().catch(() => null)) as { detail?: string } | null;
    if (payload?.detail) {
      return payload.detail;
    }
    return fallback;
  }

  const text = await response.text().catch(() => "");
  const trimmed = text.trim();
  if (!trimmed) {
    return fallback;
  }
  return trimmed.length > 160 ? `${trimmed.slice(0, 160)}...` : trimmed;
}

function formatTransitDuration(departureAt: string, arrivalAt: string): string {
  const departure = new Date(departureAt);
  const arrival = new Date(arrivalAt);
  if (Number.isNaN(departure.getTime()) || Number.isNaN(arrival.getTime())) {
    return "-";
  }

  const totalMinutes = Math.max(0, Math.round((arrival.getTime() - departure.getTime()) / 60000));
  const hours = Math.floor(totalMinutes / 60);
  const minutes = totalMinutes % 60;

  if (hours === 0) return `${minutes}m`;
  if (minutes === 0) return `${hours}h`;
  return `${hours}h ${minutes}m`;
}

function formatTimeKst(value: string, locale: string): string {
  const date = new Date(value);
  if (Number.isNaN(date.getTime())) {
    return "-";
  }
  const time = new Intl.DateTimeFormat(locale === "ko" ? "ko-KR" : "en-US", {
    timeZone: "Asia/Seoul",
    hour: "2-digit",
    minute: "2-digit",
    hour12: false,
  }).format(date);
  return time;
}

function formatTicketStatus(value: string | null | undefined): string | null {
  if (!value) return null;
  const words = value.split("_").filter(Boolean);
  if (words.length === 0) return null;
  return words.map((word) => word[0].toUpperCase() + word.slice(1)).join(" ");
}

function getTaskTicketBadge(task: TrainTaskSummary): { label: string; className: string } | null {
  const status = task.ticket_status ?? null;
  const paid = task.ticket_paid === true;

  if (!status && !paid) return null;
  if (status === "cancelled") {
    return { label: "train.badge.cancelled", className: "bg-slate-100 text-slate-700" };
  }
  if (status === "reservation_not_found") {
    return { label: "train.badge.reservationNotFound", className: "bg-rose-100 text-rose-700" };
  }
  if (status === "awaiting_payment" && !paid) {
    return { label: "train.badge.awaitingPayment", className: "bg-amber-100 text-amber-700" };
  }
  if (paid) {
    return { label: "train.badge.confirmed", className: "bg-emerald-100 text-emerald-700" };
  }
  return {
    label: formatTicketStatus(status) ?? status ?? "Unknown",
    className: "bg-slate-100 text-slate-700",
  };
}

function isAwaitingPaymentTask(task: TrainTaskSummary): boolean {
  return isUnpaidAwaitingPaymentTicket(task);
}

function shouldShowCompletedCancel(task: TrainTaskSummary): boolean {
  if (task.state !== "COMPLETED") return false;
  if (task.ticket_status === "cancelled") return false;
  if (task.ticket_status === "reservation_not_found" && task.ticket_paid !== true) return false;
  return true;
}

function retryNowDisabledTitle(task: TrainTaskSummary): string {
  const reason = task.retry_now_reason ?? null;
  if (!reason) return "Retry is not available.";
  if (reason === "cooldown_active" && task.retry_now_available_at) {
    return `Retry available at ${formatDateTimeKst(task.retry_now_available_at)}.`;
  }
  if (reason === "deadline_passed") return "Task deadline has passed.";
  if (reason === "paused_use_resume") return "Task is paused. Use Resume instead.";
  if (reason === "task_running") return "Task is currently running.";
  if (reason === "terminal_state") return "Task is already finished.";
  if (reason === "not_eligible_state") return "Task is not eligible for retry.";
  return "Retry is not available.";
}

function formatScheduleTitleDate(value: string): string {
  if (!value) return "MM/DD/YYYY";
  const [year, month, day] = value.split("-");
  if (!year || !month || !day) return "MM/DD/YYYY";
  return `${month.padStart(2, "0")}/${day.padStart(2, "0")}/${year}`;
}

function isRecord(value: unknown): value is Record<string, unknown> {
  return typeof value === "object" && value !== null && !Array.isArray(value);
}

function readInteger(value: unknown): number | null {
  return typeof value === "number" && Number.isFinite(value) ? Math.trunc(value) : null;
}

function clampPassengerCount(value: number, min: number, max: number): number {
  const normalized = Number.isFinite(value) ? Math.trunc(value) : min;
  return Math.min(max, Math.max(min, normalized));
}

function parsePassengerInputValue(value: string): number {
  const trimmed = value.trim();
  if (trimmed.length === 0) return 0;
  const parsed = Number.parseInt(trimmed, 10);
  return Number.isNaN(parsed) ? 0 : parsed;
}

function isMobileViewport(): boolean {
  if (typeof window === "undefined") return false;
  if (typeof window.matchMedia !== "function") {
    return window.innerWidth < 768;
  }
  return window.matchMedia("(max-width: 767px)").matches;
}

function scrollElementToViewportCenter(element: HTMLElement | null): void {
  if (!element || typeof window === "undefined") return;
  window.requestAnimationFrame(() => {
    window.requestAnimationFrame(() => {
      const rect = element.getBoundingClientRect();
      const absoluteTop = rect.top + window.scrollY;
      const centeredTop = Math.max(0, absoluteTop - window.innerHeight / 2 + rect.height / 2);
      window.scrollTo({ top: centeredTop, behavior: "smooth" });
    });
  });
}

function scrollElementToViewportTop(element: HTMLElement | null): void {
  if (!element || typeof window === "undefined") return;
  window.requestAnimationFrame(() => {
    window.requestAnimationFrame(() => {
      const rect = element.getBoundingClientRect();
      const absoluteTop = Math.max(0, rect.top + window.scrollY);
      window.scrollTo({ top: absoluteTop, behavior: "smooth" });
    });
  });
}

function metadataString(
  metadata: TrainSchedule["metadata"] | undefined,
  key: string,
): string | null {
  if (!metadata || !(key in metadata)) return null;
  const value = metadata[key];
  if (typeof value === "string") {
    const trimmed = value.trim();
    return trimmed.length > 0 ? trimmed : null;
  }
  if (typeof value === "number" && Number.isFinite(value)) {
    return String(Math.trunc(value));
  }
  return null;
}

function scheduleTrainLabel(schedule: TrainSchedule): string {
  const explicitTypeName = metadataString(schedule.metadata, "train_type_name");
  if (explicitTypeName) {
    return `${explicitTypeName} ${schedule.train_no}`;
  }

  const typeCode = metadataString(schedule.metadata, "train_type_code") ?? metadataString(schedule.metadata, "train_code");
  if (typeCode && TRAIN_TYPE_NAME_BY_CODE[typeCode]) {
    return `${TRAIN_TYPE_NAME_BY_CODE[typeCode]} ${schedule.train_no}`;
  }

  return `${schedule.provider} ${schedule.train_no}`;
}

function taskInfoFromSpec(task: TrainTaskSummary): {
  scheduleLabel: string;
  dep: string;
  arr: string;
  passengerLabel: string;
} {
  const fallback = {
    scheduleLabel: "-",
    dep: "-",
    arr: "-",
    passengerLabel: "-",
  };

  if (!isRecord(task.spec_json)) {
    return fallback;
  }

  const dep = typeof task.spec_json.dep === "string" && task.spec_json.dep.length > 0 ? task.spec_json.dep : "-";
  const arr = typeof task.spec_json.arr === "string" && task.spec_json.arr.length > 0 ? task.spec_json.arr : "-";
  const dateString = typeof task.spec_json.date === "string" ? task.spec_json.date : "";
  const dateLabel = formatScheduleTitleDate(dateString);

  const rankedRaw = Array.isArray(task.spec_json.selected_trains_ranked) ? task.spec_json.selected_trains_ranked : [];
  const ranked = rankedRaw
    .filter(isRecord)
    .map((row) => ({
      rank: readInteger(row.rank) ?? 999,
      departureAt: typeof row.departure_at === "string" ? row.departure_at : "",
    }))
    .filter((row) => row.departureAt.length > 0)
    .sort((a, b) => a.rank - b.rank);

  let scheduleLabel = "-";
  if (ranked.length > 0) {
    const firstTime = formatTimeKst(ranked[0].departureAt, "ko");
    scheduleLabel = `${dateLabel} ${firstTime}`;
    if (ranked.length > 1) {
      scheduleLabel += ` (+${ranked.length - 1})`;
    }
  } else if (dateString) {
    scheduleLabel = dateLabel;
  }

  const passengersRaw = isRecord(task.spec_json.passengers) ? task.spec_json.passengers : {};
  const adults = Math.max(0, readInteger(passengersRaw.adults) ?? 0);
  const children = Math.max(0, readInteger(passengersRaw.children) ?? 0);
  const adultLabel = `${adults} adult${adults === 1 ? "" : "s"}`;
  const childLabel = `${children} child${children === 1 ? "" : "ren"}`;

  return {
    scheduleLabel,
    dep,
    arr,
    passengerLabel: `${adultLabel}, ${childLabel}`,
  };
}

function taskSummaryRenderKey(task: TrainTaskSummary): string {
  return [
    task.id,
    task.state,
    task.updated_at,
    task.last_attempt_at ?? "",
    task.last_attempt_action ?? "",
    task.last_attempt_ok == null ? "" : String(task.last_attempt_ok),
    task.last_attempt_error_code ?? "",
    task.next_run_at ?? "",
    task.retry_now_allowed ? "1" : "0",
    task.retry_now_reason ?? "",
    task.retry_now_available_at ?? "",
    task.ticket_status ?? "",
    task.ticket_paid == null ? "" : String(task.ticket_paid),
    task.ticket_payment_deadline_at ?? "",
    task.ticket_reservation_id ?? "",
  ].join("|");
}

function taskListRenderKey(tasks: TrainTaskSummary[]): string {
  return tasks.map((task) => taskSummaryRenderKey(task)).join(";");
}

function taskPrimaryDepartureAtMs(task: TrainTaskSummary): number | null {
  if (!isRecord(task.spec_json)) {
    return null;
  }

  const rankedRaw = Array.isArray(task.spec_json.selected_trains_ranked) ? task.spec_json.selected_trains_ranked : [];
  const ranked = rankedRaw
    .filter(isRecord)
    .map((row) => ({
      rank: readInteger(row.rank) ?? 999,
      departureAt: typeof row.departure_at === "string" ? row.departure_at : "",
    }))
    .filter((row) => row.departureAt.length > 0)
    .sort((a, b) => a.rank - b.rank);

  if (ranked.length === 0) {
    return null;
  }

  const parsed = new Date(ranked[0].departureAt).getTime();
  return Number.isNaN(parsed) ? null : parsed;
}

function sortTasksByScheduleProximity(tasks: TrainTaskSummary[], order: ScheduleSortOrder): TrainTaskSummary[] {
  const direction = order === "asc" ? 1 : -1;
  return [...tasks].sort((a, b) => {
    // Always pin unpaid awaiting-payment tasks to the top, regardless of sort direction.
    const aAwaitingPayment = isUnpaidAwaitingPaymentTicket(a);
    const bAwaitingPayment = isUnpaidAwaitingPaymentTicket(b);
    if (aAwaitingPayment !== bAwaitingPayment) {
      return aAwaitingPayment ? -1 : 1;
    }

    const aSchedule = taskPrimaryDepartureAtMs(a);
    const bSchedule = taskPrimaryDepartureAtMs(b);

    if (aSchedule !== null && bSchedule !== null && aSchedule !== bSchedule) {
      return direction * (aSchedule - bSchedule);
    }
    if (aSchedule !== null && bSchedule === null) return -1;
    if (aSchedule === null && bSchedule !== null) return 1;

    const aCreated = new Date(a.created_at).getTime();
    const bCreated = new Date(b.created_at).getTime();
    const aFallback = Number.isNaN(aCreated) ? 0 : aCreated;
    const bFallback = Number.isNaN(bCreated) ? 0 : bCreated;
    return direction * (aFallback - bFallback);
  });
}

export function TrainDashboard() {
  const { locale, t } = useLocale();
  const seatClassLabels = useMemo<Record<TrainSeatClass, string>>(
    () => ({
      general_preferred: t("train.seatClassLabels.general_preferred"),
      general: t("train.seatClassLabels.general"),
      special_preferred: t("train.seatClassLabels.special_preferred"),
      special: t("train.seatClassLabels.special"),
    }),
    [t],
  );
  const [searchForm, setSearchForm] = useState<SearchFormState>({
    dep: DEFAULT_DEP_STATION,
    arr: DEFAULT_ARR_STATION,
    date: kstDateInputValue(new Date()),
    start: "06:00",
    end: "23:59",
    providers: { SRT: true, KTX: true },
  });
  const [createForm, setCreateForm] = useState<CreateTaskState>({
    seatClass: "general_preferred",
    adults: 1,
    children: 0,
    autoPay: false,
    notify: false,
  });
  const [searching, setSearching] = useState(false);
  const [hasSearched, setHasSearched] = useState(false);
  const [mobileSearchCollapsed, setMobileSearchCollapsed] = useState(false);
  const [lastSearchResultDate, setLastSearchResultDate] = useState<string | null>(null);
  const [schedules, setSchedules] = useState<TrainSchedule[]>([]);
  const [selectedScheduleIds, setSelectedScheduleIds] = useState<string[]>([]);
  const [creatingTask, setCreatingTask] = useState(false);
  const [cancellingTaskId, setCancellingTaskId] = useState<string | null>(null);
  const [payingTaskId, setPayingTaskId] = useState<string | null>(null);
  const [signingOutProvider, setSigningOutProvider] = useState<CredentialProvider | null>(null);
  const [activeTasks, setActiveTasks] = useState<TrainTaskSummary[]>([]);
  const [completedTasks, setCompletedTasks] = useState<TrainTaskSummary[]>([]);
  const [stations, setStations] = useState<TrainStation[]>([]);
  const [stationsLoading, setStationsLoading] = useState(false);
  const [credentialStatus, setCredentialStatus] = useState<TrainCredentialStatusResponse | null>(null);
  const [paymentCardStatus, setPaymentCardStatus] = useState<WalletPaymentCardStatus | null>(null);
  const [credentialLoading, setCredentialLoading] = useState(false);
  const [credentialSubmitting, setCredentialSubmitting] = useState(false);
  const [credentialProvider, setCredentialProvider] = useState<CredentialProvider | null>(null);
  const [credentialPanelOpen, setCredentialPanelOpen] = useState(false);
  const [omittedProviders, setOmittedProviders] = useState<Set<CredentialProvider>>(new Set());
  const [credentialForm, setCredentialForm] = useState<CredentialFormState>({ username: "", password: "" });
  const [errorMessage, setErrorMessage] = useState<string | null>(null);
  const [notice, setNotice] = useState<string | null>(null);
  const [dummyTaskCardsMode, setDummyTaskCardsMode] = useState(false);
  const [scheduleSortOrder, setScheduleSortOrder] = useState<ScheduleSortOrder>("asc");
  const tasksLoadInFlight = useRef(false);
  const tasksPollTick = useRef(0);
  const dummyTaskCardsModeRef = useRef(false);
  const searchPanelRef = useRef<HTMLDivElement | null>(null);
  const schedulePanelRef = useRef<HTMLDivElement | null>(null);

  const scheduleById = useMemo(() => {
    const map = new Map<string, TrainSchedule>();
    for (const schedule of schedules) {
      map.set(schedule.schedule_id, schedule);
    }
    return map;
  }, [schedules]);

  const selectedSchedules = useMemo(
    () => selectedScheduleIds.map((id) => scheduleById.get(id)).filter(Boolean) as TrainSchedule[],
    [selectedScheduleIds, scheduleById]
  );
  const selectedSearchProviders = useMemo(
    () => (["SRT", "KTX"] as const).filter((provider) => searchForm.providers[provider]),
    [searchForm.providers],
  );
  const selectedDateLabel = useMemo(
    () => formatScheduleTitleDate(lastSearchResultDate ?? searchForm.date),
    [lastSearchResultDate, searchForm.date],
  );
  const searchSummaryRoute = `${formatStationLabel(searchForm.dep, locale, { compact: true })} -> ${formatStationLabel(
    searchForm.arr,
    locale,
    { compact: true },
  )}`;
  const searchSummaryDateTime = `${formatScheduleTitleDate(searchForm.date)} · ${searchForm.start} - ${searchForm.end}`;
  const searchSummaryProvider = selectedSearchProviders.length > 0 ? selectedSearchProviders.join(" + ") : "-";
  const searchSummaryPassengers = `${createForm.adults} ${t("train.adult")} / ${createForm.children} ${t("train.child")}`;
  const showMobileSearchSummary = hasSearched && mobileSearchCollapsed;

  const ktxVerified = Boolean(credentialStatus?.ktx.verified);
  const srtVerified = Boolean(credentialStatus?.srt.verified);
  const selectedProviderCount = Number(searchForm.providers.SRT) + Number(searchForm.providers.KTX);
  const hasSearchResults = schedules.length > 0;
  const showRanking = hasSearched && hasSearchResults;
  const selectedProviders = new Set(selectedSchedules.map((schedule) => schedule.provider));
  const selectedProviderList = Array.from(selectedProviders).sort();
  const suggestedCredentialProvider =
    credentialStatus == null
      ? "KTX"
      : !credentialStatus.ktx.verified && !omittedProviders.has("KTX")
        ? "KTX"
        : !credentialStatus.srt.verified && !omittedProviders.has("SRT")
          ? "SRT"
          : null;
  const activeCredentialProvider = credentialProvider ?? suggestedCredentialProvider;
  const hasAnyConnectedProvider = ktxVerified || srtVerified;
  const searchUnlocked = credentialStatus != null && hasAnyConnectedProvider;
  const searchDisabled = searching || selectedProviderCount === 0 || !searchUnlocked;
  const totalPassengers = createForm.adults + createForm.children;
  const createDisabled = !showRanking || selectedSchedules.length === 0 || creatingTask || totalPassengers < 1;
  const hasAwaitingPaymentActiveTask = useMemo(
    () => activeTasks.some((task) => isAwaitingPaymentTask(task)),
    [activeTasks],
  );
  const sortedActiveTasks = useMemo(
    () => sortTasksByScheduleProximity(activeTasks, scheduleSortOrder),
    [activeTasks, scheduleSortOrder],
  );
  const sortedCompletedTasks = useMemo(
    () => sortTasksByScheduleProximity(completedTasks, scheduleSortOrder),
    [completedTasks, scheduleSortOrder],
  );
  const taskListPollMs = hasAwaitingPaymentActiveTask ? POLL_MS_AWAITING_PAYMENT : POLL_MS_DEFAULT;
  const autoPayAvailable = TRAIN_AUTO_PAY_FEATURE_ENABLED && Boolean(paymentCardStatus?.configured);
  const isProviderVerified = (provider: CredentialProvider): boolean =>
    provider === "SRT" ? srtVerified : ktxVerified;
  const isProviderSelected = (provider: CredentialProvider): boolean =>
    provider === "SRT" ? searchForm.providers.SRT : searchForm.providers.KTX;
  const isProviderSelectable = (provider: CredentialProvider): boolean => searchUnlocked && isProviderVerified(provider);
  const setProviderSelected = (provider: CredentialProvider, selected: boolean) => {
    setSearchForm((cur) => ({
      ...cur,
      providers: {
        ...cur.providers,
        [provider]: selected,
      },
    }));
  };
  const toggleProviderSelection = (provider: CredentialProvider) => {
    if (!isProviderSelectable(provider)) return;
    setProviderSelected(provider, !isProviderSelected(provider));
  };
  const setAdults = (nextValue: number) => {
    setCreateForm((cur) => {
      const adults = clampPassengerCount(nextValue, 0, 9);
      if (adults + cur.children < 1) {
        return cur;
      }
      return {
        ...cur,
        adults,
      };
    });
  };
  const setChildren = (nextValue: number) => {
    setCreateForm((cur) => {
      const children = clampPassengerCount(nextValue, 0, 9);
      if (cur.adults + children < 1) {
        return cur;
      }
      return {
        ...cur,
        children,
      };
    });
  };

  const reloadTasks = async (options?: { refreshCompleted?: boolean; forceCompleted?: boolean }) => {
    if (dummyTaskCardsModeRef.current) return;
    if (tasksLoadInFlight.current) return;
    tasksLoadInFlight.current = true;
    try {
      const shouldLoadCompleted =
        options?.forceCompleted === true ||
        options?.refreshCompleted === true ||
        tasksPollTick.current % COMPLETED_TASK_REFRESH_INTERVAL_TICKS === 0;
      const [active, completed] = await Promise.all([
        fetchTasksByStatus("active", { limit: ACTIVE_TASK_FETCH_LIMIT }),
        shouldLoadCompleted
          ? fetchTasksByStatus("completed", {
              refreshCompleted: options?.refreshCompleted,
              limit: COMPLETED_TASK_FETCH_LIMIT,
            })
          : Promise.resolve(null),
      ]);
      const nextActiveKey = taskListRenderKey(active);
      setActiveTasks((current) => (taskListRenderKey(current) === nextActiveKey ? current : active));
      if (completed !== null) {
        const nextCompletedKey = taskListRenderKey(completed);
        setCompletedTasks((current) => (taskListRenderKey(current) === nextCompletedKey ? current : completed));
      }
      setErrorMessage((current) => {
        if (current === TASK_LIST_ERROR_MESSAGE || current === SESSION_EXPIRED_MESSAGE) {
          return null;
        }
        return current;
      });
    } catch (error) {
      const message = error instanceof Error ? error.message : TASK_LIST_ERROR_MESSAGE;
      setErrorMessage((current) => {
        if (current && current !== TASK_LIST_ERROR_MESSAGE && current !== SESSION_EXPIRED_MESSAGE) {
          return current;
        }
        return message;
      });
    } finally {
      tasksLoadInFlight.current = false;
    }
  };

  const renderError = (value: string): string => {
    if (value === TASK_LIST_ERROR_MESSAGE) return t("train.taskListError");
    if (value === SESSION_EXPIRED_MESSAGE) return t("train.sessionExpired");
    return value;
  };

  const renderMaybeKey = (value: string): string => {
    // Some helpers return translation keys for known statuses.
    if (value.startsWith("train.")) return t(value);
    return value;
  };

  const seedDummyTaskCards = () => {
    if (!TRAIN_DUMMY_TASKS_ENABLED) return;
    const dummy = buildDummyTaskCards();
    storeDummyTaskCards([...dummy.active, ...dummy.completed]);
    dummyTaskCardsModeRef.current = true;
    setDummyTaskCardsMode(true);
    setActiveTasks(dummy.active);
    setCompletedTasks(dummy.completed);
    setErrorMessage(null);
    setNotice("Loaded dummy task cards for local UI testing.");
  };

  const clearDummyTaskCards = async () => {
    if (!TRAIN_DUMMY_TASKS_ENABLED) return;
    clearStoredDummyTaskCards();
    dummyTaskCardsModeRef.current = false;
    setDummyTaskCardsMode(false);
    setNotice("Restored live task cards.");
    await reloadTasks({ forceCompleted: true, refreshCompleted: true });
  };

  const loadCredentialStatus = async () => {
    setCredentialLoading(true);
    const abortController = new AbortController();
    const timeoutHandle = window.setTimeout(() => abortController.abort(), CREDENTIAL_STATUS_TIMEOUT_MS);
    try {
      const response = await fetch(`${clientApiBaseUrl}/api/train/credentials/status`, {
        credentials: "include",
        cache: "no-store",
        signal: abortController.signal,
      });
      if (!response.ok) {
        throw new Error("failed");
      }

      const payload = (await response.json()) as TrainCredentialStatusResponse;
      setCredentialStatus(payload);
      setOmittedProviders((current) => {
        const next = new Set(current);
        if (payload.ktx.verified) next.delete("KTX");
        if (payload.srt.verified) next.delete("SRT");
        return next;
      });

      setCredentialProvider((currentProvider) => {
        if (!currentProvider) return null;
        const currentStatus = currentProvider === "KTX" ? payload.ktx : payload.srt;
        return currentStatus.verified ? null : currentProvider;
      });
      if (payload.ktx.verified && payload.srt.verified) {
        setCredentialPanelOpen(false);
      }
    } catch (error) {
      setCredentialStatus((current) => current ?? EMPTY_CREDENTIAL_STATUS);
      if (error instanceof DOMException && error.name === "AbortError") {
        setErrorMessage(t("train.credential.timeout"));
      } else {
        setErrorMessage(t("train.error.credentialStatusLoad"));
      }
    } finally {
      window.clearTimeout(timeoutHandle);
      setCredentialStatus((current) => current ?? EMPTY_CREDENTIAL_STATUS);
      setCredentialLoading(false);
    }
  };

  const loadPaymentCardStatus = async () => {
    try {
      const response = await fetch(`${clientApiBaseUrl}/api/wallet/payment-card`, {
        credentials: "include",
        cache: "no-store",
      });
      if (!response.ok) {
        throw new Error("failed");
      }
      const payload = (await response.json()) as WalletPaymentCardStatus;
      setPaymentCardStatus(payload);
    } catch {
      setErrorMessage((current) => current ?? t("train.error.walletStatusLoad"));
    }
  };

  useEffect(() => {
    void loadCredentialStatus();
  }, []);

  useEffect(() => {
    if (!TRAIN_AUTO_PAY_FEATURE_ENABLED) {
      setPaymentCardStatus(null);
      return;
    }
    void loadPaymentCardStatus();
  }, []);

  useEffect(() => {
    if (!searchUnlocked) {
      setMobileSearchCollapsed(false);
    }
  }, [searchUnlocked]);

  useEffect(() => {
    if (!mobileSearchCollapsed || !showRanking) return;
    if (!isMobileViewport()) return;
    scrollElementToViewportCenter(schedulePanelRef.current);
  }, [mobileSearchCollapsed, showRanking, schedules.length]);

  useEffect(() => {
    setCreateForm((current) => {
      if (!autoPayAvailable) {
        return current.autoPay ? { ...current, autoPay: false } : current;
      }
      return current.autoPay ? current : { ...current, autoPay: true };
    });
  }, [autoPayAvailable]);

  useEffect(() => {
    if (!activeCredentialProvider || !credentialStatus) return;
    const statusInfo = activeCredentialProvider === "KTX" ? credentialStatus.ktx : credentialStatus.srt;
    setCredentialForm({
      username: statusInfo.username || "",
      password: "",
    });
  }, [activeCredentialProvider, credentialStatus]);

  useEffect(() => {
    if (!credentialStatus) return;

    setSearchForm((current) => ({
      ...current,
      providers: {
        SRT: srtVerified,
        KTX: ktxVerified,
      },
    }));
  }, [credentialStatus, ktxVerified, srtVerified]);

  useEffect(() => {
    if (!hasAnyConnectedProvider) {
      setCredentialPanelOpen(true);
      return;
    }
    if (ktxVerified && srtVerified) {
      setCredentialPanelOpen(false);
    }
  }, [hasAnyConnectedProvider, ktxVerified, srtVerified]);

  useEffect(() => {
    const tick = async (options?: { refreshCompleted?: boolean; forceCompleted?: boolean }) => {
      if (document.visibilityState === "hidden") {
        return;
      }
      tasksPollTick.current += 1;
      await reloadTasks(options);
    };

    void reloadTasks({ refreshCompleted: true, forceCompleted: true });
    const interval = window.setInterval(() => {
      void tick();
    }, taskListPollMs);

    const onVisibilityChange = () => {
      if (document.visibilityState === "visible") {
        void tick({ forceCompleted: true });
      }
    };

    document.addEventListener("visibilitychange", onVisibilityChange);
    return () => {
      window.clearInterval(interval);
      document.removeEventListener("visibilitychange", onVisibilityChange);
    };
  }, [taskListPollMs]);

  useEffect(() => {
    let alive = true;
    const loadStations = async () => {
      setStationsLoading(true);
      try {
        const response = await fetch(`${clientApiBaseUrl}/api/train/stations`, {
          credentials: "include",
          cache: "no-store",
        });
        if (!response.ok) {
          return;
        }
        const payload = (await response.json()) as { stations: TrainStation[] };
        if (!alive) {
          return;
        }
        setStations(payload.stations);
        if (payload.stations.length > 0) {
          const names = new Set(payload.stations.map((station) => station.name));
          setSearchForm((current) => ({
            ...current,
            dep: names.has(current.dep)
              ? current.dep
              : names.has(DEFAULT_DEP_STATION)
                ? DEFAULT_DEP_STATION
                : payload.stations[0].name,
            arr: names.has(current.arr)
              ? current.arr
              : names.has(DEFAULT_ARR_STATION)
                ? DEFAULT_ARR_STATION
                : payload.stations[Math.min(1, payload.stations.length - 1)].name,
          }));
        }
      } finally {
        if (alive) {
          setStationsLoading(false);
        }
      }
    };
    void loadStations();
    return () => {
      alive = false;
    };
  }, []);

  const onSearch = async (event: FormEvent<HTMLFormElement>) => {
    event.preventDefault();
    setSearching(true);
    setErrorMessage(null);
    setNotice(null);

    if (!searchUnlocked) {
      setErrorMessage(t("train.unlockSearch"));
      setSearching(false);
      return;
    }

    const providers: Array<"SRT" | "KTX"> = [];
    if (searchForm.providers.SRT) providers.push("SRT");
    if (searchForm.providers.KTX) providers.push("KTX");

    if (providers.length === 0) {
      setErrorMessage(t("train.error.selectProvider"));
      setSearching(false);
      return;
    }

    try {
      const response = await fetch(`${clientApiBaseUrl}/api/train/search`, {
        method: "POST",
        credentials: "include",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({
          providers,
          dep: searchForm.dep,
          arr: searchForm.arr,
          date: searchForm.date,
          time_window: {
            start: searchForm.start,
            end: searchForm.end,
          },
        }),
      });

      if (!response.ok) {
        const payload = (await response.json().catch(() => null)) as { detail?: string } | null;
        setErrorMessage(payload?.detail ?? t("train.error.searchFailed"));
        setSchedules([]);
        return;
      }

      const payload = (await response.json()) as { schedules: TrainSchedule[] };
      setHasSearched(true);
      setLastSearchResultDate(searchForm.date);
      setMobileSearchCollapsed(true);
      setSchedules(payload.schedules);
      setSelectedScheduleIds([]);
      if (payload.schedules.length === 0) {
        setNotice(t("train.notice.noSchedulesInWindow"));
      }
    } catch {
      setErrorMessage(t("train.error.searchUnreachable"));
    } finally {
      setSearching(false);
    }
  };

  const onSubmitCredentials = async (event: FormEvent<HTMLFormElement>) => {
    event.preventDefault();
    if (!activeCredentialProvider) return;
    const provider = activeCredentialProvider;

    setCredentialSubmitting(true);
    setErrorMessage(null);
    setNotice(null);

    try {
      const response = await fetch(`${clientApiBaseUrl}/api/train/credentials/${provider.toLowerCase()}`, {
        method: "POST",
        credentials: "include",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({
          username: normalizePhoneNumber(credentialForm.username),
          password: credentialForm.password,
        }),
      });

      const payload = (await response.json().catch(() => null)) as { detail?: string } | null;
      if (!response.ok) {
        setErrorMessage(payload?.detail ?? t("train.providerErrors.loginFailed", { provider }));
        return;
      }

      setCredentialForm((current) => ({ ...current, password: "" }));
      setOmittedProviders((current) => {
        const next = new Set(current);
        next.delete(provider);
        return next;
      });
      setNotice(t("train.notice.credentialsVerified", { provider }));
      setCredentialProvider(null);
      await loadCredentialStatus();
    } catch {
      setErrorMessage(t("train.providerErrors.verifyFailed", { provider }));
    } finally {
      setCredentialSubmitting(false);
    }
  };

  const continueWithoutProvider = (provider: CredentialProvider) => {
    setOmittedProviders((current) => {
      const next = new Set(current);
      next.add(provider);
      return next;
    });
    setCredentialProvider(null);
    setCredentialForm({ username: "", password: "" });
    setNotice(t("train.notice.continuingWithoutProvider", { provider }));
  };

  const signOutProvider = async (provider: CredentialProvider) => {
    const confirmed = window.confirm(t("train.confirm.signOutProvider", { provider }));
    if (!confirmed) return;

    setErrorMessage(null);
    setNotice(null);
    setSigningOutProvider(provider);

    try {
      const response = await fetch(`${clientApiBaseUrl}/api/train/credentials/${provider.toLowerCase()}/signout`, {
        method: "POST",
        credentials: "include",
      });
      if (!response.ok) {
        const detail = await parseApiErrorMessage(response, t("train.providerErrors.signOutFailed", { provider }));
        setErrorMessage(detail);
        return;
      }

      setOmittedProviders((current) => {
        const next = new Set(current);
        next.delete(provider);
        return next;
      });
      setCredentialProvider(null);
      setCredentialForm({ username: "", password: "" });
      setNotice(t("train.notice.signedOutProvider", { provider }));
      await loadCredentialStatus();
    } catch {
      setErrorMessage(t("train.providerErrors.signOutFailed", { provider }));
    } finally {
      setSigningOutProvider((current) => (current === provider ? null : current));
    }
  };

  const toggleSelectedSchedule = (scheduleId: string) => {
    setSelectedScheduleIds((current) => {
      if (current.includes(scheduleId)) {
        return current.filter((id) => id !== scheduleId);
      }
      return [...current, scheduleId];
    });
  };

  const moveRank = (index: number, direction: "up" | "down") => {
    setSelectedScheduleIds((current) => {
      const next = [...current];
      const target = direction === "up" ? index - 1 : index + 1;
      if (target < 0 || target >= current.length) {
        return current;
      }
      const item = next[index];
      next[index] = next[target];
      next[target] = item;
      return next;
    });
  };

  const createTask = async () => {
    setCreatingTask(true);
    setErrorMessage(null);
    setNotice(null);

    if (selectedSchedules.length === 0) {
      setErrorMessage(t("train.error.selectSchedule"));
      setCreatingTask(false);
      return;
    }

    if ((createForm.adults + createForm.children) < 1) {
      setErrorMessage(t("train.error.passengerMinimum"));
      setCreatingTask(false);
      return;
    }

    const ranked = selectedSchedules.map((schedule, index) => ({
      schedule_id: schedule.schedule_id,
      departure_at: schedule.departure_at,
      rank: index + 1,
      provider: schedule.provider,
    }));

    try {
      const response = await fetch(`${clientApiBaseUrl}/api/train/tasks`, {
        method: "POST",
        credentials: "include",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({
          dep: searchForm.dep,
          arr: searchForm.arr,
          date: searchForm.date,
          selected_trains_ranked: ranked,
          passengers: {
            adults: createForm.adults,
            children: createForm.children,
          },
          seat_class: createForm.seatClass,
          auto_pay: createForm.autoPay && autoPayAvailable,
          notify: createForm.notify,
        }),
      });

      if (!response.ok) {
        const detail = await parseApiErrorMessage(response, t("train.error.createTask"));
        setErrorMessage(detail);
        return;
      }

      const payload = (await response.json()) as {
        task: TrainTaskSummary;
        deduplicated: boolean;
      };

      setNotice(payload.deduplicated ? t("train.notice.taskDeduplicated") : t("train.notice.taskCreatedQueued"));
      await reloadTasks({ forceCompleted: true });
    } catch {
      setErrorMessage(t("train.error.createTask"));
    } finally {
      setCreatingTask(false);
    }
  };

  const sendTaskAction = async (taskId: string, action: "pause" | "resume" | "cancel" | "delete" | "retry") => {
    if (action === "cancel") {
      const confirmed = window.confirm(t("train.confirm.cancelTask"));
      if (!confirmed) return;
    }

    setErrorMessage(null);
    try {
      const response = await fetch(`${clientApiBaseUrl}/api/train/tasks/${taskId}/${action}`, {
        method: "POST",
        credentials: "include",
      });
      if (!response.ok) {
        const detail = await parseApiErrorMessage(response, t("train.task.actionFailed"));
        setErrorMessage(detail);
        return;
      }
      await reloadTasks({ forceCompleted: true });
    } catch {
      setErrorMessage(t("train.task.actionFailed"));
    }
  };

  const cancelTaskTicket = async (taskId: string) => {
    const confirmed = window.confirm(t("train.confirm.cancelTicket"));
    if (!confirmed) return;

    setErrorMessage(null);
    setNotice(null);
    setCancellingTaskId(taskId);

    try {
      const detailResponse = await fetch(`${clientApiBaseUrl}/api/train/tasks/${taskId}`, {
        credentials: "include",
        cache: "no-store",
      });
      if (!detailResponse.ok) {
        const detail = await parseApiErrorMessage(detailResponse, t("train.error.taskDetailLoad"));
        setErrorMessage(detail);
        return;
      }

      const detailPayload = (await detailResponse.json()) as { artifacts: TrainArtifact[] };
      const ticketArtifact = detailPayload.artifacts.find((artifact) => artifact.kind === "ticket");
      if (!ticketArtifact) {
        setErrorMessage(t("train.ticket.missingArtifact"));
        return;
      }

      const cancelResponse = await fetch(`${clientApiBaseUrl}/api/train/tickets/${ticketArtifact.id}/cancel`, {
        method: "POST",
        credentials: "include",
      });
      const cancelPayload = (await cancelResponse.json().catch(() => null)) as { detail?: string } | null;
      if (!cancelResponse.ok) {
        setErrorMessage(cancelPayload?.detail ?? t("train.error.ticketCancel"));
        return;
      }

      setNotice(cancelPayload?.detail ?? t("train.notice.ticketCancelDone"));
      await reloadTasks({ forceCompleted: true });
    } catch {
      setErrorMessage(t("train.error.ticketCancel"));
    } finally {
      setCancellingTaskId((current) => (current === taskId ? null : current));
    }
  };

  const payAwaitingPaymentTask = async (taskId: string) => {
    const confirmed = window.confirm(t("train.confirm.payNow"));
    if (!confirmed) return;

    setErrorMessage(null);
    setNotice(null);
    setPayingTaskId(taskId);
    try {
      const response = await fetch(`${clientApiBaseUrl}/api/train/tasks/${taskId}/pay`, {
        method: "POST",
        credentials: "include",
      });
      if (!response.ok) {
        const detail = await parseApiErrorMessage(response, t("train.error.paymentProcess"));
        setErrorMessage(detail);
        return;
      }

      setNotice(t("train.notice.paymentProcessed"));
      await reloadTasks({ refreshCompleted: true, forceCompleted: true });
    } catch {
      setErrorMessage(t("train.error.paymentProcess"));
    } finally {
      setPayingTaskId((current) => (current === taskId ? null : current));
    }
  };

  const onExpandSearchMobile = () => {
    setMobileSearchCollapsed(false);
    if (!isMobileViewport()) return;
    scrollElementToViewportTop(searchPanelRef.current);
  };

  return (
    <section className="space-y-8">
      {errorMessage ? (
        <p className="rounded-xl bg-rose-50 px-3 py-2 text-sm text-rose-700">{renderError(errorMessage)}</p>
      ) : null}
      {notice ? <p className="rounded-xl bg-emerald-50 px-3 py-2 text-sm text-emerald-700">{notice}</p> : null}

      <div ref={searchPanelRef} className="rounded-2xl border border-blossom-100 bg-white p-6 shadow-petal">
        <div className="flex items-center justify-between gap-3">
          <h2 className="text-lg font-semibold text-slate-800">{t("train.searchSchedules")}</h2>
          <button
            type="button"
            onClick={() => {
              setCredentialPanelOpen((current) => {
                const next = !current;
                if (!next) {
                  setCredentialProvider(null);
                }
                return next;
              });
            }}
            aria-label={credentialPanelOpen ? t("train.hideCredentials") : t("train.showCredentials")}
            title={credentialPanelOpen ? t("train.hideCredentials") : t("train.showCredentials")}
            className={`inline-flex h-10 w-10 items-center justify-center rounded-full border shadow-sm transition focus:outline-none focus:ring-2 ${
              searchUnlocked
                ? "border-emerald-200 bg-emerald-50 text-emerald-700 hover:bg-emerald-100 focus:ring-emerald-100"
                : "border-slate-200 bg-slate-100 text-slate-500 hover:bg-slate-200 focus:ring-slate-200"
            }`}
          >
            <svg viewBox="0 0 24 24" className="h-5 w-5" fill="none" stroke="currentColor" strokeWidth="1.8">
              <path d="M8 11V8a4 4 0 1 1 8 0v3" strokeLinecap="round" />
              <rect x="6" y="11" width="12" height="9" rx="2.5" />
            </svg>
          </button>
        </div>

        {!searchUnlocked ? (
          <p className="mt-2 text-xs text-amber-700">{t("train.unlockSearch")}</p>
        ) : null}

        {credentialPanelOpen ? (
          <div className="mt-4 rounded-2xl border border-blossom-100 bg-blossom-50/30 p-4">
            {credentialLoading ? (
              <p className="text-sm text-slate-500">{t("train.checkingCredentials")}</p>
            ) : (
              <>
                <div className="grid gap-3 md:grid-cols-2">
                  {(["KTX", "SRT"] as const).map((provider) => {
                    const statusInfo = provider === "KTX" ? credentialStatus?.ktx : credentialStatus?.srt;
                    const isVerified = Boolean(statusInfo?.verified);
                    const isSkipped = omittedProviders.has(provider) && !isVerified;
                    const username = statusInfo?.username || "-";
                    return (
                      <div key={provider} className="rounded-xl border border-blossom-100 bg-white px-3 py-3">
                        <div className="flex items-start justify-between gap-3">
                          <div className="min-w-0">
                            <p className="text-xs uppercase tracking-[0.14em] text-blossom-500">{provider}</p>
                            <p className="mt-1 break-all text-sm font-medium text-slate-700">{username}</p>
                            <p className={`text-xs ${isVerified ? "text-emerald-600" : "text-amber-700"}`}>
                              {isVerified
                                ? t("train.connected")
                                : isSkipped
                                  ? t("train.skippedDisabled")
                                  : statusInfo?.detail || t("train.notConnected")}
                            </p>
                          </div>
                          <div className="flex shrink-0 flex-col gap-2 self-center">
                            <button
                              type="button"
                              onClick={() => {
                                setCredentialProvider(provider);
                                setCredentialForm({
                                  username: statusInfo?.username ?? "",
                                  password: "",
                                });
                              }}
                              className={SMALL_BUTTON_CLASS}
                            >
                              {isVerified ? t("train.change") : t("train.connect")}
                            </button>
                            <button
                              type="button"
                              onClick={() => void signOutProvider(provider)}
                              disabled={!statusInfo?.configured || signingOutProvider === provider}
                              className={SMALL_DANGER_BUTTON_CLASS}
                            >
                              {signingOutProvider === provider ? t("auth.signingOut") : t("auth.signOut")}
                            </button>
                          </div>
                        </div>
                      </div>
                    );
                  })}
                </div>

                {activeCredentialProvider ? (
                  <form onSubmit={onSubmitCredentials} className="mt-4 rounded-2xl border border-blossom-100 bg-white p-4">
                    <h3 className="text-base font-semibold text-slate-800">
                      {t("train.providerLoginRequired", { provider: activeCredentialProvider })}
                    </h3>
                    <p className="mt-1 text-sm text-slate-500">
                      {t("train.providerCredentialsHint", { provider: activeCredentialProvider })}
                    </p>
                    <div className="mt-4 grid gap-3 md:grid-cols-2">
                      <label className="text-sm text-slate-700">
                        {t("train.usernameLabel", { provider: activeCredentialProvider })}
                        <input
                          type="text"
                          value={credentialForm.username}
                          onChange={(event) => setCredentialForm((current) => ({ ...current, username: event.target.value }))}
                          className={FIELD_BASE_CLASS}
                          placeholder={t("train.usernamePlaceholder")}
                          required
                        />
                      </label>
                      <label className="text-sm text-slate-700">
                        {t("train.passwordLabel", { provider: activeCredentialProvider })}
                        <input
                          type="password"
                          value={credentialForm.password}
                          onChange={(event) => setCredentialForm((current) => ({ ...current, password: event.target.value }))}
                          className={FIELD_BASE_CLASS}
                          required
                        />
                      </label>
                    </div>
                    <div className="mt-4 flex items-center gap-2">
                      <button
                        type="submit"
                        disabled={credentialSubmitting}
                        className={PRIMARY_BUTTON_CLASS}
                      >
                        {credentialSubmitting
                          ? t("train.verifying")
                          : t("train.connectProvider", { provider: activeCredentialProvider })}
                      </button>
                      <button
                        type="button"
                        onClick={() => continueWithoutProvider(activeCredentialProvider)}
                        disabled={credentialSubmitting}
                        className={SMALL_BUTTON_CLASS}
                      >
                        {t("train.continueWithoutProvider", { provider: activeCredentialProvider })}
                      </button>
                      <button
                        type="button"
                        onClick={() => setCredentialProvider(null)}
                        className={SMALL_BUTTON_CLASS}
                      >
                        {t("common.cancel")}
                      </button>
                    </div>
                  </form>
                ) : null}
              </>
            )}
          </div>
        ) : null}

        {searchUnlocked ? (
          <>
            <div
              data-testid="search-summary-mobile"
              aria-hidden={!showMobileSearchSummary}
              className={`md:hidden overflow-hidden transition-[max-height,opacity,transform,margin] duration-300 ease-out ${
                showMobileSearchSummary
                  ? "mt-4 max-h-72 opacity-100 translate-y-0 visible"
                  : "mt-0 max-h-0 opacity-0 -translate-y-1 invisible pointer-events-none"
              }`}
            >
              <div className="rounded-xl border border-blossom-100 bg-blossom-50/50 px-3 py-3">
                <div className="min-w-0">
                  <p className="text-[11px] font-medium uppercase tracking-[0.14em] text-blossom-500">
                    {t("train.searchSummary")}
                  </p>
                  <p className="mt-1 truncate text-sm font-medium text-slate-700">{searchSummaryRoute}</p>
                  <p className="mt-0.5 text-xs text-slate-600">{searchSummaryDateTime}</p>
                  <p className="mt-0.5 text-xs text-slate-600">
                    {t("train.provider")}: {searchSummaryProvider}
                  </p>
                  <p className="mt-0.5 text-xs text-slate-600">{searchSummaryPassengers}</p>
                </div>
                <div className="mt-3">
                  <button
                    type="button"
                    onClick={onExpandSearchMobile}
                    className="inline-flex h-11 w-full items-center justify-center rounded-xl bg-blossom-600 px-4 text-sm font-semibold text-white shadow-sm transition hover:bg-blossom-700 focus:outline-none focus:ring-2 focus:ring-blossom-200"
                  >
                    {t("train.editSearch")}
                  </button>
                </div>
              </div>
            </div>
            <form
              data-testid="train-search-form"
              onSubmit={onSearch}
              className={`mt-4 overflow-hidden transition-[max-height,opacity,transform] duration-300 ease-out md:overflow-visible ${
                mobileSearchCollapsed
                  ? "max-h-0 opacity-0 -translate-y-1 invisible pointer-events-none md:max-h-[5000px] md:opacity-100 md:translate-y-0 md:visible md:pointer-events-auto"
                  : "max-h-[5000px] opacity-100 translate-y-0 visible"
              }`}
            >
            <div className="grid gap-4 lg:grid-cols-[minmax(0,2fr)_minmax(0,1fr)]">
              <div className="rounded-2xl border border-blossom-100 bg-blossom-50/40 p-4">
                <p className="text-xs font-medium uppercase tracking-[0.14em] text-blossom-500">
                  {t("train.stationDateTime")}
                </p>
                <div className="mt-3 grid grid-cols-[minmax(0,1fr)_auto_minmax(0,1fr)] items-end gap-2 sm:gap-3">
                  <label className="text-sm text-slate-700">
                    {t("train.departureStation")}
                    <select
                      value={searchForm.dep}
                      onChange={(event) => setSearchForm((cur) => ({ ...cur, dep: event.target.value }))}
                      className={`${FIELD_BASE_CLASS} md:hidden`}
                      required
                      disabled={stationsLoading || stations.length === 0 || !searchUnlocked}
                    >
                      {stations.map((station) => (
                        <option key={station.name} value={station.name}>
                          {formatStationLabel(station.name, locale, { compact: true })}
                        </option>
                      ))}
                    </select>
                    <select
                      value={searchForm.dep}
                      onChange={(event) => setSearchForm((cur) => ({ ...cur, dep: event.target.value }))}
                      className={`${FIELD_BASE_CLASS} hidden md:block`}
                      required
                      disabled={stationsLoading || stations.length === 0 || !searchUnlocked}
                    >
                      {stations.map((station) => (
                        <option key={station.name} value={station.name}>
                          {formatStationLabel(station.name, locale)}
                        </option>
                      ))}
                    </select>
                  </label>
                  <div className="flex items-center justify-center self-end">
                    <button
                      type="button"
                      onClick={() =>
                        setSearchForm((cur) => ({
                          ...cur,
                          dep: cur.arr,
                          arr: cur.dep,
                        }))
                      }
                      disabled={stationsLoading || stations.length === 0 || !searchUnlocked}
                      aria-label={t("train.swapStations")}
                      title={t("train.swapStations")}
                      className="inline-flex h-10 w-10 items-center justify-center rounded-full border border-blossom-200 bg-blossom-50 text-blossom-700 shadow-sm transition hover:bg-blossom-100 focus:outline-none focus:ring-2 focus:ring-blossom-100 disabled:cursor-not-allowed disabled:opacity-60"
                    >
                      <svg viewBox="0 0 24 24" className="h-5 w-5" fill="none" stroke="currentColor" strokeWidth="1.8">
                        <path d="M4 8h13" strokeLinecap="round" />
                        <path d="m14 5 3 3-3 3" strokeLinecap="round" strokeLinejoin="round" />
                        <path d="M20 16H7" strokeLinecap="round" />
                        <path d="m10 13-3 3 3 3" strokeLinecap="round" strokeLinejoin="round" />
                      </svg>
                    </button>
                  </div>
                  <label className="text-sm text-slate-700">
                    {t("train.arrivalStation")}
                    <select
                      value={searchForm.arr}
                      onChange={(event) => setSearchForm((cur) => ({ ...cur, arr: event.target.value }))}
                      className={`${FIELD_BASE_CLASS} md:hidden`}
                      required
                      disabled={stationsLoading || stations.length === 0 || !searchUnlocked}
                    >
                      {stations.map((station) => (
                        <option key={station.name} value={station.name}>
                          {formatStationLabel(station.name, locale, { compact: true })}
                        </option>
                      ))}
                    </select>
                    <select
                      value={searchForm.arr}
                      onChange={(event) => setSearchForm((cur) => ({ ...cur, arr: event.target.value }))}
                      className={`${FIELD_BASE_CLASS} hidden md:block`}
                      required
                      disabled={stationsLoading || stations.length === 0 || !searchUnlocked}
                    >
                      {stations.map((station) => (
                        <option key={station.name} value={station.name}>
                          {formatStationLabel(station.name, locale)}
                        </option>
                      ))}
                    </select>
                  </label>
                </div>

                <div className="mt-3 grid gap-3 md:grid-cols-2">
                  <label className="text-sm text-slate-700">
                    {t("train.date")}
                    <input
                      type="date"
                      value={searchForm.date}
                      onChange={(event) => setSearchForm((cur) => ({ ...cur, date: event.target.value }))}
                      className={FIELD_BASE_CLASS}
                      required
                      disabled={!searchUnlocked}
                    />
                  </label>
                  <div className="grid grid-cols-2 gap-2">
                    <label className="text-sm text-slate-700">
                      {t("train.timeStart")}
                      <input
                        type="time"
                        value={searchForm.start}
                        onChange={(event) => setSearchForm((cur) => ({ ...cur, start: event.target.value }))}
                        className={FIELD_BASE_CLASS}
                        required
                        disabled={!searchUnlocked}
                      />
                    </label>
                    <label className="text-sm text-slate-700">
                      {t("train.timeEnd")}
                      <input
                        type="time"
                        value={searchForm.end}
                        onChange={(event) => setSearchForm((cur) => ({ ...cur, end: event.target.value }))}
                        className={FIELD_BASE_CLASS}
                        required
                        disabled={!searchUnlocked}
                      />
                    </label>
                  </div>
                </div>

                {!ktxVerified || !srtVerified ? (
                  <p className="mt-3 text-xs text-amber-700">{t("train.providersDisabled")}</p>
                ) : null}

                <div data-testid="provider-selector-mobile" className="mt-4 md:hidden">
                  <p className="text-xs font-medium uppercase tracking-[0.14em] text-blossom-500">{t("train.provider")}</p>
                  <div className="mt-2 grid grid-cols-2 gap-2">
                    {(["SRT", "KTX"] as const).map((provider) => {
                      const isSelectable = isProviderSelectable(provider);
                      const isSelected = isProviderSelected(provider);
                      return (
                        <button
                          key={provider}
                          type="button"
                          aria-pressed={isSelected}
                          disabled={!isSelectable}
                          onClick={() => toggleProviderSelection(provider)}
                          className={`inline-flex h-10 items-center justify-center rounded-xl border text-sm font-medium transition focus:outline-none focus:ring-2 focus:ring-blossom-100 ${
                            isSelectable
                              ? isSelected
                                ? "border-blossom-300 bg-blossom-100 text-blossom-700"
                                : "border-blossom-200 bg-white text-slate-700 hover:bg-blossom-50"
                              : "cursor-not-allowed border-slate-200 bg-slate-100 text-slate-400"
                          }`}
                        >
                          {provider}
                        </button>
                      );
                    })}
                  </div>
                </div>

                <div
                  data-testid="provider-selector-desktop"
                  className="mt-4 hidden flex-wrap items-center gap-4 text-sm text-slate-700 md:flex"
                >
                  <label className={`inline-flex items-center gap-2 ${!srtVerified ? "text-slate-400" : ""}`}>
                    <input
                      type="checkbox"
                      checked={searchForm.providers.SRT}
                      disabled={!srtVerified || !searchUnlocked}
                      onChange={(event) => setProviderSelected("SRT", event.target.checked)}
                    />
                    SRT
                  </label>
                  <label className={`inline-flex items-center gap-2 ${!ktxVerified ? "text-slate-400" : ""}`}>
                    <input
                      type="checkbox"
                      checked={searchForm.providers.KTX}
                      disabled={!ktxVerified || !searchUnlocked}
                      onChange={(event) => setProviderSelected("KTX", event.target.checked)}
                    />
                    KTX
                  </label>
                </div>
              </div>

              <div className="rounded-2xl border border-blossom-100 bg-blossom-50/40 p-4">
                <p className="text-xs font-medium uppercase tracking-[0.14em] text-blossom-500">
                  {t("train.passengerSeatClass")}
                </p>
                <div className="mt-3 space-y-3">
                  <label className="text-sm text-slate-700">
                    {t("train.seatClass")}
                    <select
                      value={createForm.seatClass}
                      onChange={(event) =>
                        setCreateForm((cur) => ({ ...cur, seatClass: event.target.value as TrainSeatClass }))
                      }
                      className={FIELD_BASE_CLASS}
                    >
                      <option value="general_preferred">{seatClassLabels.general_preferred}</option>
                      <option value="general">{seatClassLabels.general}</option>
                      <option value="special_preferred">{seatClassLabels.special_preferred}</option>
                      <option value="special">{seatClassLabels.special}</option>
                    </select>
                  </label>

                  <div className="grid grid-cols-1 gap-3 sm:grid-cols-2">
                    <div className="rounded-xl border border-blossom-100 bg-white/80 px-3 py-3">
                      <p className="text-sm text-slate-700">{t("train.adults")}</p>
                      <div className="mt-2 flex items-center justify-between gap-3 md:hidden">
                        <button
                          type="button"
                          aria-label={t("train.decrementAdults")}
                          disabled={createForm.adults <= 0 || (createForm.adults === 1 && createForm.children === 0)}
                          onClick={() => setAdults(createForm.adults - 1)}
                          className="inline-flex h-12 w-12 items-center justify-center rounded-xl border border-blossom-300 bg-white text-xl font-semibold leading-none text-slate-700 shadow-sm transition hover:bg-blossom-50 focus:outline-none focus:ring-2 focus:ring-blossom-100 disabled:cursor-not-allowed disabled:opacity-50"
                        >
                          -
                        </button>
                        <span
                          data-testid="adults-count-mobile"
                          className="min-w-10 text-center text-lg font-semibold tabular-nums text-slate-800"
                        >
                          {createForm.adults}
                        </span>
                        <button
                          type="button"
                          aria-label={t("train.incrementAdults")}
                          disabled={createForm.adults >= 9}
                          onClick={() => setAdults(createForm.adults + 1)}
                          className="inline-flex h-12 w-12 items-center justify-center rounded-xl border border-blossom-300 bg-white text-xl font-semibold leading-none text-slate-700 shadow-sm transition hover:bg-blossom-50 focus:outline-none focus:ring-2 focus:ring-blossom-100 disabled:cursor-not-allowed disabled:opacity-50"
                        >
                          +
                        </button>
                      </div>
                      <input
                        type="number"
                        min={0}
                        max={9}
                        inputMode="numeric"
                        aria-label={t("train.adults")}
                        value={createForm.adults}
                        onChange={(event) => setAdults(parsePassengerInputValue(event.target.value))}
                        className={`${FIELD_BASE_CLASS} hidden md:block`}
                      />
                    </div>
                    <div className="rounded-xl border border-blossom-100 bg-white/80 px-3 py-3">
                      <p className="text-sm text-slate-700">{t("train.children")}</p>
                      <div className="mt-2 flex items-center justify-between gap-3 md:hidden">
                        <button
                          type="button"
                          aria-label={t("train.decrementChildren")}
                          disabled={createForm.children <= 0 || (createForm.children === 1 && createForm.adults === 0)}
                          onClick={() => setChildren(createForm.children - 1)}
                          className="inline-flex h-12 w-12 items-center justify-center rounded-xl border border-blossom-300 bg-white text-xl font-semibold leading-none text-slate-700 shadow-sm transition hover:bg-blossom-50 focus:outline-none focus:ring-2 focus:ring-blossom-100 disabled:cursor-not-allowed disabled:opacity-50"
                        >
                          -
                        </button>
                        <span
                          data-testid="children-count-mobile"
                          className="min-w-10 text-center text-lg font-semibold tabular-nums text-slate-800"
                        >
                          {createForm.children}
                        </span>
                        <button
                          type="button"
                          aria-label={t("train.incrementChildren")}
                          disabled={createForm.children >= 9}
                          onClick={() => setChildren(createForm.children + 1)}
                          className="inline-flex h-12 w-12 items-center justify-center rounded-xl border border-blossom-300 bg-white text-xl font-semibold leading-none text-slate-700 shadow-sm transition hover:bg-blossom-50 focus:outline-none focus:ring-2 focus:ring-blossom-100 disabled:cursor-not-allowed disabled:opacity-50"
                        >
                          +
                        </button>
                      </div>
                      <input
                        type="number"
                        min={0}
                        max={9}
                        inputMode="numeric"
                        aria-label={t("train.children")}
                        value={createForm.children}
                        onChange={(event) => setChildren(parsePassengerInputValue(event.target.value))}
                        className={`${FIELD_BASE_CLASS} hidden md:block`}
                      />
                    </div>
                  </div>

                </div>
              </div>
            </div>

            <div className="mt-4 flex justify-end">
              <button
                type="submit"
                disabled={searchDisabled}
                className={PRIMARY_BUTTON_CLASS}
              >
                {searching ? t("train.searching") : t("train.search")}
              </button>
            </div>
            </form>
          </>
        ) : null}
      </div>

      {searchUnlocked && showRanking ? (
            <div ref={schedulePanelRef} className="rounded-2xl border border-blossom-100 bg-white p-6 shadow-petal">
              <h2 className="text-lg font-semibold text-slate-800">
                {t("train.selectSchedules", { date: selectedDateLabel })}
              </h2>
              <div data-testid="schedule-selector-mobile" className="mt-4 space-y-2 md:hidden">
                {schedules.map((schedule) => {
                  const checked = selectedScheduleIds.includes(schedule.schedule_id);
                  return (
                    <button
                      key={schedule.schedule_id}
                      type="button"
                      aria-pressed={checked}
                      aria-label={scheduleTrainLabel(schedule)}
                      onClick={() => toggleSelectedSchedule(schedule.schedule_id)}
                      className={`w-full rounded-xl border px-3 py-3 text-left transition focus:outline-none focus:ring-2 focus:ring-blossom-100 ${
                        checked
                          ? "border-blossom-300 bg-blossom-100/70"
                          : "border-blossom-100 bg-white hover:bg-blossom-50/60"
                      }`}
                    >
                      <div className="flex items-start justify-between gap-3">
                        <div>
                          <p className="text-sm font-semibold text-slate-800">
                            {scheduleTrainLabel(schedule)}
                          </p>
                          <p className="mt-1 text-xs text-slate-600">
                            {formatTimeKst(schedule.departure_at, locale)} - {formatTimeKst(schedule.arrival_at, locale)} ·{" "}
                            {formatTransitDuration(schedule.departure_at, schedule.arrival_at)}
                          </p>
                        </div>
                        <span
                          aria-hidden="true"
                          className={`inline-flex h-7 w-7 items-center justify-center rounded-full border text-xs font-semibold ${
                            checked
                              ? "border-blossom-500 bg-blossom-500 text-white"
                              : "border-slate-300 bg-white text-slate-400"
                          }`}
                        >
                          {checked ? "✓" : "+"}
                        </span>
                      </div>
                      <div className="mt-2 flex items-center gap-2">
                        <span
                          title={schedule.availability.general ? t("train.generalAvailable") : t("train.generalSoldOut")}
                          className={`inline-flex h-6 w-6 items-center justify-center rounded-full text-[11px] font-semibold ${
                            schedule.availability.general ? "bg-blossom-500 text-white" : "bg-slate-200 text-slate-500"
                          }`}
                        >
                          G
                        </span>
                        <span
                          title={schedule.availability.special ? t("train.specialAvailable") : t("train.specialSoldOut")}
                          className={`inline-flex h-6 w-6 items-center justify-center rounded-full text-[11px] font-semibold ${
                            schedule.availability.special ? "bg-blossom-500 text-white" : "bg-slate-200 text-slate-500"
                          }`}
                        >
                          S
                        </span>
                      </div>
                    </button>
                  );
                })}
              </div>
              <div data-testid="schedule-selector-desktop" className="mt-4 hidden overflow-x-auto md:block">
                <table className="min-w-full table-fixed text-left text-sm">
                  <thead>
                    <tr className="text-slate-500">
                      <th className="px-2 pb-2 text-center">{t("train.table.status")}</th>
                      <th className="px-2 pb-2">{t("train.table.train")}</th>
                      <th className="px-2 pb-2">{t("train.table.departure")}</th>
                      <th className="px-2 pb-2">{t("train.table.arrival")}</th>
                      <th className="px-2 pb-2">{t("train.table.duration")}</th>
                      <th className="px-2 pb-2 text-center" colSpan={2}>
                        {t("train.table.availability")}
                      </th>
                    </tr>
                  </thead>
                  <tbody>
                    {schedules.map((schedule) => {
                      const checked = selectedScheduleIds.includes(schedule.schedule_id);
                      return (
                        <tr
                          key={schedule.schedule_id}
                          role="button"
                          tabIndex={0}
                          aria-pressed={checked}
                          onClick={() => toggleSelectedSchedule(schedule.schedule_id)}
                          onKeyDown={(event) => {
                            if (event.key === "Enter" || event.key === " ") {
                              event.preventDefault();
                              toggleSelectedSchedule(schedule.schedule_id);
                            }
                          }}
                          className={`cursor-pointer border-t border-blossom-100 transition ${
                            checked ? "bg-blossom-100/70" : "hover:bg-blossom-50/50"
                          }`}
                        >
                          <td className="px-2 py-2 align-middle text-center">
                            <span
                              aria-hidden="true"
                              className={`mx-auto inline-flex h-5 w-5 items-center justify-center rounded-full border transition ${
                                checked
                                  ? "border-blossom-500 bg-blossom-500 text-white"
                                  : "border-slate-300 bg-slate-100 text-transparent"
                              }`}
                            >
                              {checked ? (
                                <svg
                                  viewBox="0 0 20 20"
                                  className="h-3.5 w-3.5"
                                  fill="none"
                                  stroke="currentColor"
                                  strokeWidth="2.2"
                                >
                                  <path d="M4 10.5 8 14l8-8" strokeLinecap="round" strokeLinejoin="round" />
                                </svg>
                              ) : null}
                            </span>
                          </td>
                          <td className="px-2 py-2">
                            {scheduleTrainLabel(schedule)}
                          </td>
                          <td className="px-2 py-2">{formatTimeKst(schedule.departure_at, locale)}</td>
                          <td className="px-2 py-2">{formatTimeKst(schedule.arrival_at, locale)}</td>
                          <td className="px-2 py-2">{formatTransitDuration(schedule.departure_at, schedule.arrival_at)}</td>
                          <td className="px-2 py-2 text-center">
                            <span
                              title={schedule.availability.general ? t("train.generalAvailable") : t("train.generalSoldOut")}
                              className={`inline-flex h-6 w-6 items-center justify-center rounded-full text-[11px] font-semibold ${
                                schedule.availability.general ? "bg-blossom-500 text-white" : "bg-slate-200 text-slate-500"
                              }`}
                            >
                              G
                            </span>
                          </td>
                          <td className="px-2 py-2 text-center">
                            <span
                              title={schedule.availability.special ? t("train.specialAvailable") : t("train.specialSoldOut")}
                              className={`inline-flex h-6 w-6 items-center justify-center rounded-full text-[11px] font-semibold ${
                                schedule.availability.special ? "bg-blossom-500 text-white" : "bg-slate-200 text-slate-500"
                              }`}
                            >
                              S
                            </span>
                          </td>
                        </tr>
                      );
                    })}
                  </tbody>
                </table>
              </div>

              <div className="mt-5 rounded-xl border border-blossom-100 bg-blossom-50/50 p-4">
                {selectedSchedules.length === 0 ? (
                  <p className="text-sm text-slate-500">{t("train.selectToCreateTask")}</p>
                ) : null}
                {selectedSchedules.length === 1 ? (
                  <div className="rounded-lg border border-blossom-100 bg-white px-3 py-2 text-sm text-slate-700">
                    <span className="font-medium">{t("train.selected")}</span> {scheduleTrainLabel(selectedSchedules[0])} ·{" "}
                    {formatDateTimeKst(selectedSchedules[0].departure_at, locale)}
                  </div>
                ) : null}
                {selectedSchedules.length > 1 ? (
                  <>
                    <p className="text-sm font-medium text-slate-700">{t("train.priorityOrder")}</p>
                    <ul className="mt-3 space-y-2 text-sm">
                      {selectedSchedules.map((schedule, index) => (
                        <li
                          key={schedule.schedule_id}
                          className="flex items-center justify-between rounded-lg border border-blossom-100 bg-white px-3 py-2"
                        >
                          <div className="flex items-center gap-2">
                            <span className="inline-flex h-6 min-w-6 items-center justify-center rounded-full bg-blossom-500 px-2 text-xs font-semibold text-white">
                              {index + 1}
                            </span>
                            <span>
                              {scheduleTrainLabel(schedule)} · {formatDateTimeKst(schedule.departure_at, locale)}
                            </span>
                          </div>
                          <div className="flex items-center gap-2">
                            <button
                              type="button"
                              onClick={() => moveRank(index, "up")}
                              className={SMALL_BUTTON_CLASS}
                            >
                              {t("train.up")}
                            </button>
                            <button
                              type="button"
                              onClick={() => moveRank(index, "down")}
                              className={SMALL_BUTTON_CLASS}
                            >
                              {t("train.down")}
                            </button>
                          </div>
                        </li>
                      ))}
                    </ul>
                  </>
                ) : null}

                <div className="mt-4 flex flex-wrap items-center justify-between gap-3 rounded-lg border border-blossom-100 bg-white px-3 py-3">
                  <div className="text-sm text-slate-600">
                    <p>
                      <span className="font-medium">{t("train.provider")}:</span>{" "}
                      {selectedSchedules.length > 0 ? selectedProviderList.join(" + ") : t("train.selectSchedulesFirst")}
                    </p>
                    <p>
                      <span className="font-medium">{t("train.seat")}:</span> {seatClassLabels[createForm.seatClass]} ·{" "}
                      <span className="font-medium">{t("train.passengers")}:</span> {createForm.adults} {t("train.adult")} /{" "}
                      {createForm.children} {t("train.child")}
                    </p>
                  </div>
                  <div className="flex flex-col gap-1">
                    <div className="flex flex-wrap items-center gap-3">
                      {TRAIN_AUTO_PAY_FEATURE_ENABLED ? (
                        <button
                          type="button"
                          role="switch"
                          aria-checked={createForm.autoPay}
                          onClick={() => {
                            if (!autoPayAvailable) return;
                            setCreateForm((cur) => ({ ...cur, autoPay: !cur.autoPay }));
                          }}
                          disabled={!autoPayAvailable}
                          title={autoPayAvailable ? t("train.autoPay") : t("train.autoPayHelp")}
                          className={`inline-flex items-center gap-2 rounded-full border px-3 py-1.5 text-xs font-medium transition focus:outline-none focus:ring-2 focus:ring-blossom-100 disabled:cursor-not-allowed disabled:opacity-60 ${
                            createForm.autoPay
                              ? "border-blossom-300 bg-blossom-50 text-blossom-700"
                              : "border-slate-200 bg-white text-slate-600"
                          }`}
                        >
                          <span>{t("train.autoPay")}</span>
                          <span
                            className={`relative inline-flex h-5 w-9 items-center rounded-full transition ${
                              createForm.autoPay ? "bg-blossom-500" : "bg-slate-300"
                            }`}
                          >
                            <span
                              className={`inline-block h-4 w-4 rounded-full bg-white shadow transition ${
                                createForm.autoPay ? "translate-x-4" : "translate-x-0.5"
                              }`}
                            />
                          </span>
                        </button>
                      ) : null}

                      <button
                        type="button"
                        role="switch"
                        aria-checked={createForm.notify}
                        onClick={() => setCreateForm((cur) => ({ ...cur, notify: !cur.notify }))}
                        className={`inline-flex items-center gap-2 rounded-full border px-3 py-1.5 text-xs font-medium transition focus:outline-none focus:ring-2 focus:ring-blossom-100 ${
                          createForm.notify
                            ? "border-blossom-300 bg-blossom-50 text-blossom-700"
                            : "border-slate-200 bg-white text-slate-600"
                        }`}
                      >
                        <span>{t("train.notify")}</span>
                        <span
                          className={`relative inline-flex h-5 w-9 items-center rounded-full transition ${
                            createForm.notify ? "bg-blossom-500" : "bg-slate-300"
                          }`}
                        >
                          <span
                            className={`inline-block h-4 w-4 rounded-full bg-white shadow transition ${
                              createForm.notify ? "translate-x-4" : "translate-x-0.5"
                            }`}
                          />
                        </span>
                      </button>

                      <button
                        type="button"
                        onClick={createTask}
                        disabled={createDisabled}
                        className={PRIMARY_BUTTON_CLASS}
                      >
                        {creatingTask ? t("train.creatingTask") : t("train.createTask")}
                      </button>
                    </div>

                  </div>
                </div>
                {TRAIN_AUTO_PAY_FEATURE_ENABLED && !autoPayAvailable ? (
                  <div className="mt-3">
                    <div className="inline-flex items-center gap-1 rounded-full border border-amber-200 bg-amber-50/90 px-3 py-1.5 text-xs text-amber-700 shadow-sm">
                      <span className="font-medium">{t("train.walletRequiredAutoPay")}</span>
                      <span>{t("train.configureIn")}</span>
                      <Link
                        href={ROUTES.settings.payment}
                        className="font-medium underline decoration-amber-300 underline-offset-2 hover:text-amber-800"
                      >
                        {t("nav.paymentSettings")}
                      </Link>
                      <span>.</span>
                    </div>
                  </div>
                ) : null}
              </div>
            </div>
          ) : searchUnlocked && hasSearched ? (
            <div className="rounded-2xl border border-blossom-100 bg-white p-6 text-sm text-slate-500 shadow-petal">
              {t("train.noSchedulesYet")}
            </div>
          ) : null}

      {TRAIN_DUMMY_TASKS_ENABLED ? (
        <div className="rounded-2xl border border-dashed border-blossom-200 bg-white p-4 shadow-petal">
          <div className="flex flex-wrap items-center justify-between gap-3">
            <p className="text-sm text-slate-600">
              <span className="font-medium text-slate-700">Dev test tools:</span> create dummy task cards across task
              states.
            </p>
            <div className="flex flex-wrap items-center gap-2">
              <button type="button" onClick={seedDummyTaskCards} className={SMALL_BUTTON_CLASS}>
                Load dummy task cards
              </button>
              <button
                type="button"
                onClick={() => {
                  void clearDummyTaskCards();
                }}
                disabled={!dummyTaskCardsMode}
                className={!dummyTaskCardsMode ? SMALL_DISABLED_BUTTON_CLASS : SMALL_BUTTON_CLASS}
              >
                Restore live tasks
              </button>
            </div>
          </div>
        </div>
      ) : null}

      <div className="grid gap-4 lg:grid-cols-2">
        <div className="lg:col-span-2">
          <div className="flex flex-wrap items-center justify-end gap-2 text-sm text-slate-600">
            <label htmlFor="task-schedule-sort" className="font-medium text-slate-700">
              {t("train.sort.label")}
            </label>
            <select
              id="task-schedule-sort"
              value={scheduleSortOrder}
              onChange={(event) => setScheduleSortOrder(event.target.value as ScheduleSortOrder)}
              className="rounded-lg border border-blossom-200 bg-white px-2 py-1 text-sm text-slate-700 focus:outline-none focus:ring-2 focus:ring-blossom-100"
            >
              <option value="asc">{t("train.sort.scheduleAscending")}</option>
              <option value="desc">{t("train.sort.scheduleDescending")}</option>
            </select>
          </div>
        </div>
        <div className="rounded-2xl border border-blossom-100 bg-white p-6 shadow-petal">
          <h2 className="text-lg font-semibold text-slate-800">{t("train.activeTasks")}</h2>
          <ul className="mt-4 space-y-3 text-sm">
            {sortedActiveTasks.length === 0 ? <li className="text-slate-500">{t("train.empty.activeTasks")}</li> : null}
            {sortedActiveTasks.map((task) => {
              const info = taskInfoFromSpec(task);
              const showRetryNow = task.state === "QUEUED" || task.state === "POLLING";
              const ticketBadge = getTaskTicketBadge(task);
              const isSeatUnavailable = task.last_attempt_error_code === "seat_unavailable";
              const lastError =
                task.last_attempt_ok === false
                  ? task.last_attempt_error_message_safe || task.last_attempt_error_code || "Unknown error"
                  : null;
              return (
                <li key={task.id} className="rounded-xl border border-blossom-100 p-3">
                <div className="flex items-center justify-between gap-2">
                  <div>
                    <div className="flex flex-wrap items-center gap-2">
                      <p className="font-medium text-slate-700">{task.state}</p>
                      {ticketBadge ? (
                        <span
                          className={`inline-flex rounded-full px-2 py-0.5 text-[11px] font-medium ${ticketBadge.className}`}
                        >
                          {renderMaybeKey(ticketBadge.label)}
                        </span>
                      ) : null}
                    </div>
                    <p className="text-xs text-slate-500">
                      {t("train.lastAttempt")} {task.last_attempt_at ? formatDateTimeKst(task.last_attempt_at, locale) : "-"}
                    </p>
	                    <p className="mt-1 text-xs text-slate-600">
	                      {t("train.label.schedule")} {info.scheduleLabel}
	                    </p>
	                    {task.state === "POLLING" && task.next_run_at ? (
	                      <p className="text-xs text-slate-500">Next check: {formatDateTimeKst(task.next_run_at, locale)}</p>
	                    ) : null}
	                    {lastError ? (
	                      <p className={`text-xs ${isSeatUnavailable ? "text-amber-700" : "text-rose-600"}`} title={lastError}>
	                        Last error: {lastError}
	                      </p>
	                    ) : null}
	                    <p className="text-xs text-slate-600">
	                      {t("train.label.route")} {formatStationLabel(info.dep, locale)} {"->"} {formatStationLabel(info.arr, locale)}
	                    </p>
                    <p className="text-xs text-slate-600">
                      {t("train.label.passengers")} {info.passengerLabel}
                    </p>
                  </div>
                  <Link href={`/modules/train/tasks/${task.id}`} className="text-xs font-medium text-blossom-600 hover:text-blossom-700">
                    {t("train.action.detail")}
                  </Link>
                </div>
                <div className="mt-3 flex flex-wrap gap-2">
                  {showRetryNow ? (
                    <button
                      type="button"
                      onClick={() => sendTaskAction(task.id, "retry")}
                      disabled={!task.retry_now_allowed}
                      title={task.retry_now_allowed ? "Retry now" : retryNowDisabledTitle(task)}
                      className={task.retry_now_allowed ? SMALL_BUTTON_CLASS : SMALL_DISABLED_BUTTON_CLASS}
                    >
                      Retry now
                    </button>
                  ) : null}
                  {task.state !== "PAUSED" ? (
                    <button
                      type="button"
                      onClick={() => sendTaskAction(task.id, "pause")}
                      className={SMALL_BUTTON_CLASS}
                    >
                      {t("train.action.pause")}
                    </button>
                  ) : (
                    <button
                      type="button"
                      onClick={() => sendTaskAction(task.id, "resume")}
                      className={SMALL_BUTTON_CLASS}
                    >
                      {t("train.action.resume")}
                    </button>
                  )}
                  {isAwaitingPaymentTask(task) ? (
                    <button
                      type="button"
                      onClick={() => void payAwaitingPaymentTask(task.id)}
                      disabled={payingTaskId === task.id}
                      className={payingTaskId === task.id ? SMALL_DISABLED_BUTTON_CLASS : SMALL_SUCCESS_BUTTON_CLASS}
                    >
                      {payingTaskId === task.id ? t("train.action.paying") : t("train.action.pay")}
                    </button>
                  ) : null}
                  {isAwaitingPaymentTask(task) ? (
                    <button
                      type="button"
                      onClick={() => void cancelTaskTicket(task.id)}
                      disabled={cancellingTaskId === task.id || payingTaskId === task.id}
                      className={SMALL_DANGER_BUTTON_CLASS}
                    >
                      {cancellingTaskId === task.id ? t("train.action.cancelling") : t("train.action.cancelReservation")}
                    </button>
                  ) : null}
                  <button
                    type="button"
                    onClick={() => sendTaskAction(task.id, "cancel")}
                    className={SMALL_DANGER_BUTTON_CLASS}
                  >
                    {t("train.action.cancel")}
                  </button>
                </div>
                </li>
              );
            })}
          </ul>
        </div>

        <div className="rounded-2xl border border-blossom-100 bg-white p-6 shadow-petal">
          <h2 className="text-lg font-semibold text-slate-800">{t("train.completedTasks")}</h2>
          <ul className="mt-4 space-y-3 text-sm">
            {sortedCompletedTasks.length === 0 ? <li className="text-slate-500">{t("train.empty.completedTasks")}</li> : null}
            {sortedCompletedTasks.map((task) => {
              const info = taskInfoFromSpec(task);
              const ticketBadge = getTaskTicketBadge(task);
              return (
                <li key={task.id} className="rounded-xl border border-blossom-100 p-3">
                <div className="flex items-center justify-between gap-2">
                  <div>
                    <div className="flex flex-wrap items-center gap-2">
                      <p className="font-medium text-slate-700">{task.state}</p>
                      {ticketBadge ? (
                        <span
                          className={`inline-flex rounded-full px-2 py-0.5 text-[11px] font-medium ${ticketBadge.className}`}
                        >
                          {renderMaybeKey(ticketBadge.label)}
                        </span>
                      ) : null}
                    </div>
                    <p className="text-xs text-slate-500">
                      {t("train.label.completed")} {task.completed_at ? formatDateTimeKst(task.completed_at, locale) : "-"}
                    </p>
                    <p className="mt-1 text-xs text-slate-600">
                      {t("train.label.schedule")} {info.scheduleLabel}
                    </p>
                    <p className="text-xs text-slate-600">
                      {t("train.label.route")} {formatStationLabel(info.dep, locale)} {"->"} {formatStationLabel(info.arr, locale)}
                    </p>
                    <p className="text-xs text-slate-600">
                      {t("train.label.passengers")} {info.passengerLabel}
                    </p>
                  </div>
                  <Link href={`/modules/train/tasks/${task.id}`} className="text-xs font-medium text-blossom-600 hover:text-blossom-700">
                    {t("train.action.detail")}
                  </Link>
                </div>
                <div className="mt-3 flex flex-wrap gap-2">
                  {isAwaitingPaymentTask(task) ? (
                    <button
                      type="button"
                      onClick={() => void payAwaitingPaymentTask(task.id)}
                      disabled={payingTaskId === task.id || !autoPayAvailable}
                      title={autoPayAvailable ? t("train.action.payNow") : t("train.hint.paymentSettingsRequired")}
                      className={
                        payingTaskId === task.id || !autoPayAvailable
                          ? SMALL_DISABLED_BUTTON_CLASS
                          : SMALL_SUCCESS_BUTTON_CLASS
                      }
                    >
                      {payingTaskId === task.id ? t("train.action.paying") : t("train.action.pay")}
                    </button>
                  ) : null}
                  {isAwaitingPaymentTask(task) ? (
                    <button
                      type="button"
                      onClick={() => void cancelTaskTicket(task.id)}
                      disabled={cancellingTaskId === task.id || payingTaskId === task.id}
                      className={SMALL_DANGER_BUTTON_CLASS}
                    >
                      {cancellingTaskId === task.id ? t("train.action.cancelling") : t("train.action.cancel")}
                    </button>
                  ) : shouldShowCompletedCancel(task) ? (
                    <button
                      type="button"
                      onClick={() => void cancelTaskTicket(task.id)}
                      disabled={cancellingTaskId === task.id}
                      className={SMALL_DANGER_BUTTON_CLASS}
                    >
                      {cancellingTaskId === task.id ? t("train.action.cancelling") : t("train.action.cancel")}
                    </button>
                  ) : (
                    <button
                      type="button"
                      onClick={() => sendTaskAction(task.id, "delete")}
                      className={SMALL_DANGER_BUTTON_CLASS}
                    >
                      {t("common.delete")}
                    </button>
                  )}
                </div>
                </li>
              );
            })}
          </ul>
        </div>
      </div>
    </section>
  );
}
