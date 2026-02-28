"use client";

import Link from "next/link";
import { createPortal } from "react-dom";
import { FormEvent, KeyboardEvent, UIEvent, useCallback, useEffect, useLayoutEffect, useMemo, useRef, useState } from "react";
import { AnimatePresence, motion } from "framer-motion";

import { useLocale } from "@/components/locale-provider";
import { clientApiBaseUrl } from "@/lib/api-base";
import type { Locale } from "@/lib/i18n";
import { formatDateTimeKst, kstDateInputValue } from "@/lib/kst";
import { ROUTES } from "@/lib/routes";
import { rankStationCandidates, shouldAutoCommitTopSuggestion } from "@/lib/train/station-search";
import { UI_BUTTON_OUTLINE, UI_BUTTON_OUTLINE_SM, UI_BUTTON_PRIMARY, UI_BUTTON_DANGER_SM, UI_FIELD } from "@/lib/ui";
import {
  clearStoredDummyTaskCards,
  setDummyTaskCardsModeEnabled,
  storeDummyTaskCards,
  TRAIN_DUMMY_TASKS_ENABLED,
} from "@/lib/train/dummy-task-cards";
import { getTrainStationsCached } from "@/lib/train/stations-cache";
import { subscribeTrainTaskEvents } from "@/lib/train/task-events";
import {
  ACTIVE_TASK_FETCH_LIMIT,
  clearTaskListBootstrapSnapshot,
  COMPLETED_TASK_FETCH_LIMIT,
  fetchTaskListBootstrap,
  fetchTasksByStatus as fetchTasksByStatusFromApi,
  SESSION_EXPIRED_MESSAGE,
  TASK_LIST_ERROR_MESSAGE,
  type TaskListStatus,
} from "@/lib/train/task-list-bootstrap";
import { formatStationLabel } from "@/lib/train/stations-i18n";
import type {
  TrainArtifact,
  TrainCredentialStatusResponse,
  TrainSchedule,
  TrainSeatClass,
  TrainStation,
  TrainTaskState,
  TrainTaskSummary,
  WalletPaymentCardConfigured,
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
  retryOnExpiry: boolean;
  notify: boolean;
};

type CredentialFormState = {
  username: string;
  password: string;
};

type CredentialProvider = "KTX" | "SRT";
type ScheduleSortOrder = "asc" | "desc";
type DuplicateMatchCategory = "already_reserved" | "waiting" | "polling";

type TrainTaskDuplicateMatch = {
  task_id: string;
  state: TrainTaskState;
  category: DuplicateMatchCategory;
  departure_at: string;
  ticket_status?: string | null;
};

type TrainTaskDuplicateSummary = {
  already_reserved: number;
  waiting: number;
  polling: number;
};

type TrainTaskDuplicateCheckResponse = {
  has_duplicate: boolean;
  summary: TrainTaskDuplicateSummary;
  matches: TrainTaskDuplicateMatch[];
};

type TrainTaskCreatePayload = {
  dep: string;
  arr: string;
  date: string;
  selected_trains_ranked: Array<{
    schedule_id: string;
    departure_at: string;
    arrival_at: string;
    rank: number;
    provider: "SRT" | "KTX";
  }>;
  passengers: {
    adults: number;
    children: number;
  };
  seat_class: TrainSeatClass;
  auto_pay: boolean;
  retry_on_expiry: boolean;
  notify: boolean;
  confirm_duplicate?: boolean;
};

const CREDENTIAL_STATUS_TIMEOUT_MS = 10000;
const DEFAULT_DEP_STATION = "수서";
const DEFAULT_ARR_STATION = "마산";
const ACTIVE_TASK_STATES_FOR_LIST = new Set<TrainTaskState>([
  "QUEUED",
  "RUNNING",
  "POLLING",
  "RESERVING",
  "PAYING",
  "PAUSED",
]);
const VALID_TASK_EVENT_STATES = new Set<TrainTaskState>([
  "QUEUED",
  "RUNNING",
  "POLLING",
  "RESERVING",
  "PAYING",
  "PAUSED",
  "COMPLETED",
  "CANCELLED",
  "EXPIRED",
  "FAILED",
]);
const TASK_LIST_REFRESH_EVENT_STATES = new Set<TrainTaskState>(["COMPLETED", "CANCELLED", "EXPIRED", "FAILED"]);
const TASK_LIST_PENDING_TICKET_STATUSES = new Set(["awaiting_payment", "waiting"]);
const SOLD_OUT_LIKE_ERROR_CODES = new Set(["seat_unavailable", "sold_out"]);
const DUPLICATE_CATEGORY_ORDER: DuplicateMatchCategory[] = ["already_reserved", "waiting", "polling"];
const TRAIN_AUTO_PAY_FEATURE_ENABLED = ["1", "true", "yes", "on"].includes(
  (process.env.NEXT_PUBLIC_TRAIN_AUTO_PAY_ENABLED ?? "false").trim().toLowerCase(),
);
const TRAIN_KTX_FEATURE_ENABLED = ["1", "true", "yes", "on"].includes(
  (process.env.NEXT_PUBLIC_TRAIN_KTX_ENABLED ?? "false").trim().toLowerCase(),
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
const TRAIN_DASHBOARD_FETCH_DEDUPE_TTL_MS = 5_000;
let credentialStatusCache: { value: TrainCredentialStatusResponse | null; fetchedAt: number } = {
  value: null,
  fetchedAt: 0,
};
let credentialStatusInFlight: Promise<TrainCredentialStatusResponse> | null = null;
let paymentCardConfiguredCache: { value: boolean | null; fetchedAt: number } = {
  value: null,
  fetchedAt: 0,
};
let paymentCardConfiguredInFlight: Promise<boolean> | null = null;

export function clearTrainDashboardFetchCaches(): void {
  credentialStatusCache = { value: null, fetchedAt: 0 };
  credentialStatusInFlight = null;
  paymentCardConfiguredCache = { value: null, fetchedAt: 0 };
  paymentCardConfiguredInFlight = null;
  clearTaskListBootstrapSnapshot();
}

function isDashboardFetchCacheFresh(fetchedAt: number): boolean {
  return fetchedAt > 0 && Date.now() - fetchedAt <= TRAIN_DASHBOARD_FETCH_DEDUPE_TTL_MS;
}

async function fetchCredentialStatusForDashboard(
  options?: { force?: boolean; signal?: AbortSignal },
): Promise<TrainCredentialStatusResponse> {
  const force = Boolean(options?.force);
  if (!force && credentialStatusCache.value && isDashboardFetchCacheFresh(credentialStatusCache.fetchedAt)) {
    return credentialStatusCache.value;
  }
  if (credentialStatusInFlight) {
    return credentialStatusInFlight;
  }

  credentialStatusInFlight = (async () => {
    const response = await fetch(`${clientApiBaseUrl}/api/train/credentials/status`, {
      credentials: "include",
      cache: "no-store",
      signal: options?.signal,
    });
    if (!response.ok) {
      throw new Error("failed");
    }
    const payload = (await response.json()) as TrainCredentialStatusResponse;
    credentialStatusCache = { value: payload, fetchedAt: Date.now() };
    return payload;
  })();

  try {
    return await credentialStatusInFlight;
  } finally {
    credentialStatusInFlight = null;
  }
}

async function fetchPaymentCardConfiguredForDashboard(options?: { force?: boolean }): Promise<boolean> {
  const force = Boolean(options?.force);
  if (!force && paymentCardConfiguredCache.value != null && isDashboardFetchCacheFresh(paymentCardConfiguredCache.fetchedAt)) {
    return paymentCardConfiguredCache.value;
  }
  if (paymentCardConfiguredInFlight) {
    return paymentCardConfiguredInFlight;
  }

  paymentCardConfiguredInFlight = (async () => {
    const response = await fetch(`${clientApiBaseUrl}/api/wallet/payment-card/configured`, {
      credentials: "include",
      cache: "no-store",
    });
    if (!response.ok) {
      throw new Error("failed");
    }
    const payload = (await response.json()) as WalletPaymentCardConfigured;
    const configured = Boolean(payload.configured);
    paymentCardConfiguredCache = { value: configured, fetchedAt: Date.now() };
    return configured;
  })();

  try {
    return await paymentCardConfiguredInFlight;
  } finally {
    paymentCardConfiguredInFlight = null;
  }
}

/**
 * Normalize Korean phone numbers to 11-digit format (e.g., 01012345678).
 * Handles: 010-1234-5678, 010 1234 5678, +82-10-1234-5678, +8210-1234-5678, etc.
 * Returns original input if not a recognizable phone pattern.
 */
export function normalizePhoneNumber(input: string): string {
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
const SEARCH_SECTION_LABEL_CLASS = "text-xs font-medium uppercase tracking-[0.14em] text-blossom-500";
const PRIMARY_BUTTON_CLASS = UI_BUTTON_PRIMARY;
const SECONDARY_BUTTON_CLASS = UI_BUTTON_OUTLINE;
const SMALL_BUTTON_CLASS = UI_BUTTON_OUTLINE_SM;
const SMALL_DANGER_BUTTON_CLASS = UI_BUTTON_DANGER_SM;
const SMALL_SUCCESS_BUTTON_CLASS =
  "inline-flex h-11 items-center justify-center rounded-full border border-emerald-200 bg-emerald-50 px-3 text-sm font-medium text-emerald-700 shadow-sm transition hover:bg-emerald-100 focus:outline-none focus:ring-2 focus:ring-emerald-100 disabled:cursor-not-allowed disabled:opacity-60 sm:h-8 sm:px-2.5 sm:text-xs";
const SMALL_DISABLED_BUTTON_CLASS =
  "inline-flex h-11 items-center justify-center rounded-full border border-slate-200 bg-slate-100 px-3 text-sm font-medium text-slate-500 shadow-sm transition focus:outline-none focus:ring-2 focus:ring-slate-100 sm:h-8 sm:px-2.5 sm:text-xs";
const TASK_ACTION_BUTTON_SIZE_CLASS = "h-11 min-w-[88px] px-3 sm:h-9";
type TranslateFn = (key: string, vars?: Record<string, string | number>) => string;
const TASK_CARD_ENTER_EXIT_TRANSITION = { duration: 0.22, ease: [0.22, 1, 0.36, 1] as const };
const TASK_CARD_INITIAL_ANIMATION = { opacity: 0, y: 10, scale: 0.985 };
const TASK_CARD_ANIMATE_ANIMATION = { opacity: 1, y: 0, scale: 1 };
const TASK_CARD_EXIT_ANIMATION = { opacity: 0, y: -8, scale: 0.985 };
const EMPTY_CREDENTIAL_STATUS: TrainCredentialStatusResponse = {
  ktx: { configured: false, verified: false, detail: null },
  srt: { configured: false, verified: false, detail: null },
};

export function isUnpaidAwaitingPaymentTicket(task: Pick<TrainTaskSummary, "ticket_status" | "ticket_paid">): boolean {
  return task.ticket_status === "awaiting_payment" && task.ticket_paid !== true;
}

export function isPendingTicketForActiveList(task: Pick<TrainTaskSummary, "ticket_status" | "ticket_paid">): boolean {
  const status = String(task.ticket_status || "").trim().toLowerCase();
  return (status === "awaiting_payment" || status === "waiting") && task.ticket_paid !== true;
}

export function isActiveTaskForList(task: TrainTaskSummary): boolean {
  if (ACTIVE_TASK_STATES_FOR_LIST.has(task.state)) {
    return true;
  }
  return task.state === "COMPLETED" && isPendingTicketForActiveList(task);
}

function readTaskEventState(value: unknown): TrainTaskState | null {
  if (typeof value !== "string") return null;
  const normalized = value.trim().toUpperCase();
  return VALID_TASK_EVENT_STATES.has(normalized as TrainTaskState) ? (normalized as TrainTaskState) : null;
}

function readTaskEventType(payloadType: unknown, rawEventType: unknown): "task_state" | "task_ticket_status" | null {
  const normalize = (value: unknown): "task_state" | "task_ticket_status" | null => {
    if (typeof value !== "string") return null;
    const normalized = value.trim().toLowerCase();
    if (normalized === "task_state" || normalized === "task_state_changed") return "task_state";
    if (normalized === "task_ticket_status" || normalized === "task_ticket_status_changed") return "task_ticket_status";
    return null;
  };
  return normalize(payloadType) ?? normalize(rawEventType);
}

function applyTaskEventState(
  tasks: TrainTaskSummary[],
  options: {
    taskId: string;
    state: TrainTaskState;
    updatedAt: string | null;
  },
): TrainTaskSummary[] {
  const { taskId, state, updatedAt } = options;
  let changed = false;
  const next = tasks.map((task) => {
    if (task.id !== taskId) return task;
    changed = true;
    return {
      ...task,
      state,
      updated_at: updatedAt ?? task.updated_at,
    };
  });
  return changed ? next : tasks;
}

function hasTaskById(tasks: TrainTaskSummary[], taskId: string): boolean {
  return tasks.some((task) => task.id === taskId);
}

export function buildDummyTaskCards(now: Date = new Date()): { active: TrainTaskSummary[]; completed: TrainTaskSummary[] } {
  const isoAtOffsetMinutes = (minutesFromNow: number): string =>
    new Date(now.getTime() + minutesFromNow * 60_000).toISOString();
  const isoWithOffsetMinutes = (iso: string, offsetMinutes: number): string => {
    const parsed = new Date(iso).getTime();
    if (Number.isNaN(parsed)) {
      return iso;
    }
    return new Date(parsed + offsetMinutes * 60_000).toISOString();
  };

  const makeSpec = (
    provider: "SRT" | "KTX",
    departureAtIso: string,
    rankedDepartureAtIso: string[] = [departureAtIso],
    dep: string = "수서",
    arr: string = "부산",
  ): Record<string, unknown> => ({
    dep,
    arr,
    date: departureAtIso.slice(0, 10),
    passengers: { adults: 1, children: 0 },
    seat_class: "general_preferred",
    selected_trains_ranked: rankedDepartureAtIso.map((departureAt, index) => ({
      rank: index + 1,
      departure_at: departureAt,
      arrival_at: isoWithOffsetMinutes(departureAt, 150),
      provider,
      schedule_id: `dummy-${provider.toLowerCase()}-${index + 1}-${departureAt}`,
    })),
  });

  const makeTask = (
    id: string,
    state: TrainTaskState,
    options: {
      provider?: "SRT" | "KTX";
      dep?: string;
      arr?: string;
      departureOffsetMinutes?: number;
      additionalDepartureOffsetMinutes?: number[];
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
      ticketTrainNo?: string | null;
      ticketSeatCount?: number | null;
      ticketSeats?: string[] | null;
    } = {},
  ): TrainTaskSummary => {
    const provider = options.provider ?? "SRT";
    const departureOffsets = [options.departureOffsetMinutes ?? 120, ...(options.additionalDepartureOffsetMinutes ?? [])];
    const rankedDepartureAt = departureOffsets.map((offsetMinutes) => isoAtOffsetMinutes(offsetMinutes));
    const departureAt = rankedDepartureAt[0];
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
      spec_json: makeSpec(provider, departureAt, rankedDepartureAt, options.dep ?? "수서", options.arr ?? "부산"),
      ticket_status: options.ticketStatus ?? null,
      ticket_paid: options.ticketPaid ?? null,
      ticket_payment_deadline_at:
        options.ticketPaymentDeadlineOffsetMinutes == null
          ? null
          : isoAtOffsetMinutes(options.ticketPaymentDeadlineOffsetMinutes),
      ticket_reservation_id: options.ticketReservationId ?? null,
      ticket_train_no: options.ticketTrainNo ?? null,
      ticket_seat_count: options.ticketSeatCount ?? null,
      ticket_seats: options.ticketSeats ?? null,
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
    makeTask("polling-multi-schedule", "POLLING", {
      provider: "SRT",
      dep: "수서",
      arr: "마산",
      departureOffsetMinutes: 12 * 60 + 24,
      additionalDepartureOffsetMinutes: [12 * 60 + 38, 12 * 60 + 56],
      deadlineOffsetMinutes: 18 * 60,
      lastAttemptOk: false,
      lastAttemptErrorCode: "seat_unavailable",
      lastAttemptErrorMessageSafe: "No available seats in selected trains right now",
      nextRunOffsetMinutes: 2,
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
      ticketTrainNo: "311",
      ticketSeatCount: 1,
      ticketSeats: ["8-12A"],
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
    makeTask("queued-defaults", "QUEUED"),
  ];

  return {
    active: allSampleTasks.filter((task) => isActiveTaskForList(task)),
    completed: allSampleTasks.filter((task) => !isActiveTaskForList(task)),
  };
}

export async function fetchTasksByStatus(
  status: TaskListStatus,
  options?: { refreshCompleted?: boolean; limit?: number },
) {
  return fetchTasksByStatusFromApi(status, options);
}

export async function parseApiErrorMessage(response: Response, fallback: string): Promise<string> {
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

export function formatTransitDuration(departureAt: string, arrivalAt: string): string {
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

export function formatTimeKst(value: string, locale: string): string {
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

export function formatTicketStatus(value: string | null | undefined): string | null {
  if (!value) return null;
  const words = value.split("_").filter(Boolean);
  if (words.length === 0) return null;
  return words.map((word) => word[0].toUpperCase() + word.slice(1)).join(" ");
}

export function getTaskTicketBadge(task: TrainTaskSummary): { label: string; className: string } | null {
  const status = task.ticket_status ?? null;
  const paid = task.ticket_paid === true;

  if (!status && !paid) return null;
  if (task.state === "EXPIRED" && (status === "awaiting_payment" || status === "waiting") && !paid) {
    return null;
  }
  if (status === "cancelled") {
    return { label: "train.badge.cancelled", className: "bg-slate-100 text-slate-700" };
  }
  if (status === "reservation_not_found") {
    return { label: "train.badge.reservationNotFound", className: "bg-rose-100 text-rose-700" };
  }
  if (status === "awaiting_payment" && !paid) {
    return { label: "train.badge.awaitingPayment", className: "bg-amber-100 text-amber-700" };
  }
  if (status === "waiting") {
    return { label: "train.badge.waitlisted", className: "bg-sky-100 text-sky-700" };
  }
  if (paid) {
    return { label: "train.badge.confirmed", className: "bg-emerald-100 text-emerald-700" };
  }
  const fallbackStatus = status as string;
  return {
    label: formatTicketStatus(fallbackStatus) ?? fallbackStatus,
    className: "bg-slate-100 text-slate-700",
  };
}

export function taskDisplayState(task: Pick<TrainTaskSummary, "state" | "ticket_status" | "ticket_paid">): string {
  if (task.state === "EXPIRED") {
    return "EXPIRED";
  }
  const status = task.ticket_status ?? null;
  if (status === "awaiting_payment" && task.ticket_paid !== true) {
    return "PENDING";
  }
  if (status === "waiting") {
    return "PENDING";
  }
  return task.state;
}

export function isAwaitingPaymentTask(task: TrainTaskSummary): boolean {
  return task.state === "COMPLETED" && isUnpaidAwaitingPaymentTicket(task);
}

export function shouldShowCompletedCancel(task: TrainTaskSummary): boolean {
  if (task.state !== "COMPLETED") return false;
  if (task.ticket_status === "cancelled") return false;
  if (task.ticket_status === "reservation_not_found" && task.ticket_paid !== true) return false;
  return true;
}

function isRetryDeleteTerminalTask(task: TrainTaskSummary): boolean {
  return task.state === "EXPIRED" || task.state === "CANCELLED" || task.state === "FAILED";
}

export function retryNowDisabledTitle(task: TrainTaskSummary): string {
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

export function retryNowDisabledTitleLocalized(
  task: TrainTaskSummary,
  options?: { locale?: string; t?: TranslateFn },
): string {
  const reason = task.retry_now_reason ?? null;
  const locale = options?.locale ?? "en";
  const t = options?.t;
  const translate = (key: string, fallback: string, vars?: Record<string, string | number>) =>
    t ? t(key, vars) : fallback;

  if (!reason) return translate("train.retryDisabled.default", "Retry is not available.");
  if (reason === "cooldown_active" && task.retry_now_available_at) {
    const time = formatDateTimeKst(task.retry_now_available_at, locale);
    return translate("train.retryDisabled.cooldownActive", `Retry available at ${time}.`, { time });
  }
  if (reason === "deadline_passed") return translate("train.retryDisabled.deadlinePassed", "Task deadline has passed.");
  if (reason === "paused_use_resume") {
    return translate("train.retryDisabled.pausedUseResume", "Task is paused. Use Resume instead.");
  }
  if (reason === "task_running") return translate("train.retryDisabled.taskRunning", "Task is currently running.");
  if (reason === "terminal_state") return translate("train.retryDisabled.terminalState", "Task is already finished.");
  if (reason === "not_eligible_state") {
    return translate("train.retryDisabled.notEligibleState", "Task is not eligible for retry.");
  }
  return translate("train.retryDisabled.default", "Retry is not available.");
}

function parseScheduleDateParts(value: string): { year: number; month: number; day: number; asUtcDate: Date } | null {
  const trimmed = value.trim();
  if (!trimmed) return null;

  let normalized = trimmed;
  if (/^\d{8}$/.test(normalized)) {
    normalized = `${normalized.slice(0, 4)}-${normalized.slice(4, 6)}-${normalized.slice(6, 8)}`;
  } else if (/^\d{4}-\d{2}-\d{2}T/.test(normalized)) {
    normalized = normalized.slice(0, 10);
  }

  const match = normalized.match(/^(\d{4})-(\d{2})-(\d{2})$/);
  if (!match) return null;

  const year = Number.parseInt(match[1], 10);
  const month = Number.parseInt(match[2], 10);
  const day = Number.parseInt(match[3], 10);
  if (!Number.isInteger(year) || !Number.isInteger(month) || !Number.isInteger(day)) {
    return null;
  }

  const asUtcDate = new Date(Date.UTC(year, month - 1, day));
  if (
    Number.isNaN(asUtcDate.getTime()) ||
    asUtcDate.getUTCFullYear() !== year ||
    asUtcDate.getUTCMonth() + 1 !== month ||
    asUtcDate.getUTCDate() !== day
  ) {
    return null;
  }

  return { year, month, day, asUtcDate };
}

export function formatScheduleTitleDate(value: string): string {
  if (!value) return "MM/DD/YYYY";
  const parts = parseScheduleDateParts(value);
  if (!parts) return "MM/DD/YYYY";
  return `${String(parts.month).padStart(2, "0")}/${String(parts.day).padStart(2, "0")}/${String(parts.year)}`;
}

export function formatScheduleDateWithWeekday(value: string): string {
  if (!value) return "MM/DD/YYYY (Weekday)";
  const parts = parseScheduleDateParts(value);
  if (!parts) return "MM/DD/YYYY (Weekday)";
  const weekday = parts.asUtcDate.toLocaleDateString("en-US", {
    weekday: "long",
    timeZone: "UTC",
  });
  return `${String(parts.month).padStart(2, "0")}/${String(parts.day).padStart(2, "0")}/${String(parts.year)} (${weekday})`;
}

export function isRecord(value: unknown): value is Record<string, unknown> {
  return typeof value === "object" && value !== null && !Array.isArray(value);
}

export function readInteger(value: unknown): number | null {
  return typeof value === "number" && Number.isFinite(value) ? Math.trunc(value) : null;
}

export function clampPassengerCount(value: number, min: number, max: number): number {
  const normalized = Number.isFinite(value) ? Math.trunc(value) : min;
  return Math.min(max, Math.max(min, normalized));
}

export function parsePassengerInputValue(value: string): number {
  const trimmed = value.trim();
  if (trimmed.length === 0) return 0;
  const parsed = Number.parseInt(trimmed, 10);
  return Number.isNaN(parsed) ? 0 : parsed;
}

export function isMobileViewport(): boolean {
  if (typeof window === "undefined") return false;
  if (typeof window.matchMedia !== "function") {
    return window.innerWidth < 768;
  }
  return window.matchMedia("(max-width: 767px)").matches;
}

export function scrollElementToViewportCenter(element: HTMLElement | null): void {
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

export function scrollElementToViewportTop(
  element: HTMLElement | null,
  options?: { offsetPx?: number },
): void {
  if (!element || typeof window === "undefined") return;
  window.requestAnimationFrame(() => {
    window.requestAnimationFrame(() => {
      const rect = element.getBoundingClientRect();
      const offsetPx = Math.max(0, options?.offsetPx ?? 0);
      const absoluteTop = Math.max(0, rect.top + window.scrollY - offsetPx);
      window.scrollTo({ top: absoluteTop, behavior: "smooth" });
    });
  });
}

export function metadataString(
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

export function scheduleTrainLabel(schedule: TrainSchedule): string {
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

export function taskInfoFromSpec(task: TrainTaskSummary, locale: string = "en"): {
  travelDateLabel: string;
  primaryDepartureLabel: string;
  primaryArrivalLabel: string;
  scheduleTimeOptions: Array<{
    rank: number;
    provider: string | null;
    departureLabel: string;
    arrivalLabel: string;
  }>;
  scheduleLabel: string;
  scheduleOptionCount: number;
  scheduleOptions: Array<{ rank: number; timeLabel: string; provider: string | null }>;
  dep: string;
  arr: string;
  passengerLabel: string;
} {
  const fallback = {
    travelDateLabel: "MM/DD/YYYY (Weekday)",
    primaryDepartureLabel: "-",
    primaryArrivalLabel: "-",
    scheduleTimeOptions: [],
    scheduleLabel: "-",
    scheduleOptionCount: 0,
    scheduleOptions: [],
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

  const rankedRaw = Array.isArray(task.spec_json.selected_trains_ranked) ? task.spec_json.selected_trains_ranked : [];
  const ranked = rankedRaw
    .filter(isRecord)
    .map((row) => ({
      rank: readInteger(row.rank) ?? 999,
      departureAt: typeof row.departure_at === "string" ? row.departure_at : "",
      arrivalAt: typeof row.arrival_at === "string" ? row.arrival_at : "",
      provider: typeof row.provider === "string" && row.provider.trim().length > 0 ? row.provider.trim() : null,
    }))
    .filter((row) => row.departureAt.length > 0)
    .sort((a, b) => a.rank - b.rank);

  const effectiveDateString =
    parseScheduleDateParts(dateString) != null
      ? dateString
      : ranked.length > 0
        ? ranked[0].departureAt.slice(0, 10)
        : "";
  const dateLabel = formatScheduleTitleDate(effectiveDateString);
  const travelDateLabel = formatScheduleDateWithWeekday(effectiveDateString);

  const scheduleTimeOptions = ranked
    .map((row) => ({
      rank: row.rank,
      provider: row.provider,
      departureLabel: formatTimeKst(row.departureAt, "ko"),
      arrivalLabel: row.arrivalAt ? formatTimeKst(row.arrivalAt, "ko") : "-",
    }))
    .filter((row) => row.departureLabel !== "-");

  const scheduleOptions = scheduleTimeOptions.map((row) => ({
    rank: row.rank,
    provider: row.provider,
    timeLabel: row.departureLabel,
  }));

  let scheduleLabel = "-";
  if (scheduleOptions.length > 0) {
    const firstTime = scheduleOptions[0].timeLabel;
    scheduleLabel = `${dateLabel} ${firstTime}`;
  } else if (effectiveDateString) {
    scheduleLabel = dateLabel;
  }

  const passengersRaw = isRecord(task.spec_json.passengers) ? task.spec_json.passengers : {};
  const adults = Math.max(0, readInteger(passengersRaw.adults) ?? 0);
  const children = Math.max(0, readInteger(passengersRaw.children) ?? 0);
  const isKorean = locale.toLowerCase().startsWith("ko");
  const passengerParts: string[] = [];
  if (adults > 0) {
    passengerParts.push(isKorean ? `성인 ${adults}` : `${adults} adult${adults === 1 ? "" : "s"}`);
  }
  if (children > 0) {
    passengerParts.push(isKorean ? `아동 ${children}` : `${children} child${children === 1 ? "" : "ren"}`);
  }

  return {
    travelDateLabel,
    primaryDepartureLabel: scheduleTimeOptions[0]?.departureLabel ?? "-",
    primaryArrivalLabel: scheduleTimeOptions[0]?.arrivalLabel ?? "-",
    scheduleTimeOptions,
    scheduleLabel,
    scheduleOptionCount: ranked.length,
    scheduleOptions,
    dep,
    arr,
    passengerLabel: passengerParts.length > 0 ? passengerParts.join(", ") : "-",
  };
}

function taskPrimaryProviderFromSpec(task: TrainTaskSummary): string | null {
  if (!isRecord(task.spec_json)) {
    return null;
  }

  if (typeof task.spec_json.provider === "string" && task.spec_json.provider.trim().length > 0) {
    return task.spec_json.provider.trim();
  }

  if (Array.isArray(task.spec_json.providers)) {
    for (const value of task.spec_json.providers) {
      if (typeof value === "string" && value.trim().length > 0) {
        return value.trim();
      }
    }
  }

  const rankedRaw = Array.isArray(task.spec_json.selected_trains_ranked) ? task.spec_json.selected_trains_ranked : [];
  const ranked = rankedRaw
    .filter(isRecord)
    .map((row) => ({
      rank: readInteger(row.rank) ?? 999,
      provider: typeof row.provider === "string" ? row.provider.trim() : "",
    }))
    .filter((row) => row.provider.length > 0)
    .sort((a, b) => a.rank - b.rank);

  return ranked.length > 0 ? ranked[0].provider : null;
}

export function taskTicketTrainLabel(task: TrainTaskSummary): string | null {
  if (typeof task.ticket_train_no !== "string") {
    return null;
  }
  const rawTrainNo = task.ticket_train_no.trim();
  const trainNo = /^\d+$/.test(rawTrainNo) ? String(parseInt(rawTrainNo, 10)) : rawTrainNo;
  if (!trainNo) {
    return null;
  }
  const provider = taskPrimaryProviderFromSpec(task);
  return provider ? `${provider} ${trainNo}` : trainNo;
}

function formatTicketSeatEntry(rawSeat: string, locale: string): string {
  const seat = rawSeat.trim();
  const match = seat.match(/^(\d+)\s*-\s*(.+)$/);
  if (!match) {
    return seat;
  }
  const carNo = match[1];
  const seatNo = match[2].trim();
  if (!seatNo) {
    return seat;
  }
  if (locale.toLowerCase().startsWith("ko")) {
    return `${carNo}호차 - ${seatNo}`;
  }
  return `Car ${carNo} - ${seatNo}`;
}

export function taskTicketSeatLabel(
  task: Pick<TrainTaskSummary, "ticket_seat_count" | "ticket_seats">,
  locale: string = "en",
): string | null {
  if (Array.isArray(task.ticket_seats)) {
    const seats = Array.from(
      new Set(
        task.ticket_seats
          .filter((seat): seat is string => typeof seat === "string")
          .map((seat) => seat.trim())
          .filter((seat) => seat.length > 0),
      ),
    );
    if (seats.length > 0) {
      return seats.map((seat) => formatTicketSeatEntry(seat, locale)).join(", ");
    }
  }

  if (typeof task.ticket_seat_count === "number" && Number.isFinite(task.ticket_seat_count)) {
    const count = Math.max(0, Math.trunc(task.ticket_seat_count));
    if (count > 0) {
      return String(count);
    }
  }

  return null;
}

function parseDateMs(value: string | null | undefined): number | null {
  if (!value) return null;
  const parsed = new Date(value).getTime();
  return Number.isNaN(parsed) ? null : parsed;
}

export function formatCountdownSeconds(valueMs: number): string {
  const totalSeconds = Math.max(0, Math.ceil(valueMs / 1000));
  const hours = Math.floor(totalSeconds / 3600);
  const minutes = Math.floor((totalSeconds % 3600) / 60);
  const seconds = totalSeconds % 60;

  if (hours > 0) {
    return `${hours}h ${minutes}m`;
  }
  if (minutes > 0) {
    return `${minutes}m ${String(seconds).padStart(2, "0")}s`;
  }
  return `${seconds}s`;
}

export type TaskRetryCountdown = {
  remainingMs: number;
  elapsedMs: number | null;
  windowMs: number | null;
  progressPercent: number | null;
  isDue: boolean;
};

export function taskRetryCountdown(task: TrainTaskSummary, nowMs: number): TaskRetryCountdown | null {
  const nextRunMs = parseDateMs(task.next_run_at);
  if (nextRunMs === null) return null;

  const remainingMs = Math.max(0, nextRunMs - nowMs);
  const isDue = remainingMs === 0;
  const lastAttemptMs = parseDateMs(task.last_attempt_finished_at ?? task.last_attempt_at);
  if (lastAttemptMs === null || lastAttemptMs >= nextRunMs) {
    return {
      remainingMs,
      elapsedMs: null,
      windowMs: null,
      progressPercent: isDue ? 100 : null,
      isDue,
    };
  }

  const windowMs = nextRunMs - lastAttemptMs;
  const elapsedMs = Math.min(windowMs, Math.max(0, nowMs - lastAttemptMs));
  const progressPercent = Math.max(0, Math.min(100, Math.round((elapsedMs / windowMs) * 100)));
  return {
    remainingMs,
    elapsedMs,
    windowMs,
    progressPercent,
    isDue,
  };
}

export function taskSummaryRenderKey(task: TrainTaskSummary): string {
  return [
    task.id,
    task.state,
    task.updated_at,
    task.last_attempt_at ?? "",
    task.last_attempt_action ?? "",
    task.last_attempt_ok == null ? "" : String(task.last_attempt_ok),
    task.last_attempt_error_code ?? "",
    task.last_attempt_finished_at ?? "",
    task.next_run_at ?? "",
    task.retry_now_allowed ? "1" : "0",
    task.retry_now_reason ?? "",
    task.retry_now_available_at ?? "",
    task.ticket_status ?? "",
    task.ticket_paid == null ? "" : String(task.ticket_paid),
    task.ticket_payment_deadline_at ?? "",
    task.ticket_reservation_id ?? "",
    task.ticket_train_no ?? "",
    task.ticket_seat_count == null ? "" : String(task.ticket_seat_count),
    Array.isArray(task.ticket_seats) ? task.ticket_seats.join(",") : "",
  ].join("|");
}

export function taskListRenderKey(tasks: TrainTaskSummary[]): string {
  return tasks.map((task) => taskSummaryRenderKey(task)).join(";");
}

export function normalizeTaskErrorMessage(value: string): string {
  return value
    .replace(/<br\s*\/?>/gi, "\n")
    .replace(/<\/?[^>]+>/g, "")
    .split("\n")
    .map((line) => line.trim())
    .filter(Boolean)
    .join(" ");
}

function isDuplicateCheckResponse(value: unknown): value is TrainTaskDuplicateCheckResponse {
  if (!isRecord(value)) return false;
  if (typeof value.has_duplicate !== "boolean") return false;
  if (!isRecord(value.summary) || !Array.isArray(value.matches)) return false;
  return true;
}

export function taskPrimaryDepartureAtMs(task: TrainTaskSummary): number | null {
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

export function sortTasksByScheduleProximity(tasks: TrainTaskSummary[], order: ScheduleSortOrder): TrainTaskSummary[] {
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

export function sortActiveTasksByImminence(tasks: TrainTaskSummary[]): TrainTaskSummary[] {
  return [...tasks].sort((a, b) => {
    const aSchedule = taskPrimaryDepartureAtMs(a);
    const bSchedule = taskPrimaryDepartureAtMs(b);

    if (aSchedule !== null && bSchedule !== null && aSchedule !== bSchedule) {
      return aSchedule - bSchedule;
    }
    if (aSchedule !== null && bSchedule === null) return -1;
    if (aSchedule === null && bSchedule !== null) return 1;

    const aCreated = new Date(a.created_at).getTime();
    const bCreated = new Date(b.created_at).getTime();
    const aFallback = Number.isNaN(aCreated) ? 0 : aCreated;
    const bFallback = Number.isNaN(bCreated) ? 0 : bCreated;
    return aFallback - bFallback;
  });
}

export function validateCreateTaskInputs(
  selectedScheduleCount: number,
  totalPassengers: number,
): "train.error.selectSchedule" | "train.error.passengerMinimum" | null {
  if (selectedScheduleCount < 1) {
    return "train.error.selectSchedule";
  }
  if (totalPassengers < 1) {
    return "train.error.passengerMinimum";
  }
  return null;
}

export function resolveSearchStations(
  currentDep: string,
  currentArr: string,
  stations: TrainStation[],
): { dep: string; arr: string } {
  if (stations.length < 1) {
    return { dep: currentDep, arr: currentArr };
  }
  const names = new Set(stations.map((station) => station.name));
  const dep = names.has(currentDep)
    ? currentDep
    : names.has(DEFAULT_DEP_STATION)
      ? DEFAULT_DEP_STATION
      : stations[0].name;
  const arr = names.has(currentArr)
    ? currentArr
    : names.has(DEFAULT_ARR_STATION)
      ? DEFAULT_ARR_STATION
      : stations[Math.min(1, stations.length - 1)].name;
  return { dep, arr };
}

type StationAutocompleteFieldProps = {
  label: string;
  locale: Locale;
  stationName: string;
  stations: TrainStation[];
  disabled: boolean;
  noMatchesLabel: string;
  onStationChange: (value: string) => void;
};

function StationAutocompleteField({
  label,
  locale,
  stationName,
  stations,
  disabled,
  noMatchesLabel,
  onStationChange,
}: StationAutocompleteFieldProps) {
  const [focused, setFocused] = useState(false);
  const [draft, setDraft] = useState("");

  const suggestions = useMemo(
    () => (focused ? rankStationCandidates(draft, stations, { locale, limit: 3 }) : []),
    [draft, focused, locale, stations],
  );

  const commitTopSuggestion = useCallback(
    (value: string) => {
      const ranked = rankStationCandidates(value, stations, { locale, limit: 3 });
      if (!shouldAutoCommitTopSuggestion(ranked, value)) {
        return;
      }
      const top = ranked[0];
      if (top) {
        onStationChange(top.station.name);
      }
    },
    [locale, onStationChange, stations],
  );

  const selectStation = useCallback(
    (value: string) => {
      onStationChange(value);
      setDraft(formatStationLabel(value, locale, { compact: true }));
      setFocused(false);
    },
    [locale, onStationChange],
  );

  const handleFocus = useCallback(() => {
    setFocused(true);
    setDraft(formatStationLabel(stationName, locale, { compact: true }));
  }, [locale, stationName]);

  const handleBlur = useCallback(() => {
    commitTopSuggestion(draft);
    setFocused(false);
    setDraft("");
  }, [commitTopSuggestion, draft]);

  const handleChange = useCallback(
    (nextValue: string) => {
      setDraft(nextValue);
    },
    [],
  );

  const handleKeyDown = useCallback(
    (event: KeyboardEvent<HTMLInputElement>) => {
      if (event.key === "Enter") {
        commitTopSuggestion(draft);
        setFocused(false);
      }
    },
    [commitTopSuggestion, draft],
  );

  const showNoMatches = focused && draft.trim().length > 0 && suggestions.length < 1;
  const visibleValue = focused ? draft : formatStationLabel(stationName, locale);

  return (
    <label className="relative text-sm text-slate-700">
      <span className="hidden md:inline">{label}</span>
      <input
        aria-label={label}
        value={visibleValue}
        onFocus={handleFocus}
        onBlur={handleBlur}
        onChange={(event) => handleChange(event.target.value)}
        onKeyDown={handleKeyDown}
        className={FIELD_BASE_CLASS}
        required
        disabled={disabled}
        autoComplete="off"
      />
      {focused && !disabled ? (
        <div className="absolute z-20 mt-1 w-full overflow-hidden rounded-xl border border-slate-200 bg-white shadow-lg">
          {showNoMatches ? (
            <p className="px-3 py-2 text-xs text-slate-500">{noMatchesLabel}</p>
          ) : (
            <ul role="listbox" className="max-h-52 overflow-auto py-1">
              {suggestions.map((match) => (
                <li key={match.station.name} className="px-1">
                  <button
                    type="button"
                    role="option"
                    aria-selected={match.station.name === stationName}
                    onMouseDown={(event) => {
                      event.preventDefault();
                      selectStation(match.station.name);
                    }}
                    className={`flex w-full flex-col rounded-lg px-2 py-1.5 text-left transition ${
                      match.station.name === stationName
                        ? "bg-blossom-50 text-blossom-700"
                        : "text-slate-700 hover:bg-slate-50"
                    }`}
                  >
                    <span className="text-sm font-medium">{match.primaryLabel}</span>
                    {match.secondaryLabel ? <span className="text-xs text-slate-500">{match.secondaryLabel}</span> : null}
                  </button>
                </li>
              ))}
            </ul>
          )}
        </div>
      ) : null}
    </label>
  );
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
    start: "00:01",
    end: "23:59",
    providers: { SRT: true, KTX: TRAIN_KTX_FEATURE_ENABLED },
  });
  const [createForm, setCreateForm] = useState<CreateTaskState>({
    seatClass: "general_preferred",
    adults: 1,
    children: 0,
    autoPay: false,
    retryOnExpiry: true,
    notify: false,
  });
  const [duplicateWarning, setDuplicateWarning] = useState<TrainTaskDuplicateCheckResponse | null>(null);
  const [duplicateDetailTargetTaskId, setDuplicateDetailTargetTaskId] = useState<string | null>(null);
  const [createTaskReviewOpen, setCreateTaskReviewOpen] = useState(false);
  const [searching, setSearching] = useState(false);
  const [hasSearched, setHasSearched] = useState(false);
  const [shouldScrollToSearchResultsSection, setShouldScrollToSearchResultsSection] = useState(false);
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
  const [paymentCardConfigured, setPaymentCardConfigured] = useState(false);
  const [credentialLoading, setCredentialLoading] = useState(false);
  const [credentialSubmitting, setCredentialSubmitting] = useState(false);
  const [credentialProvider, setCredentialProvider] = useState<CredentialProvider | null>(null);
  const [credentialPanelOpen, setCredentialPanelOpen] = useState(false);
  const [omittedProviders, setOmittedProviders] = useState<Set<CredentialProvider>>(new Set());
  const [credentialForm, setCredentialForm] = useState<CredentialFormState>({ username: "", password: "" });
  const [errorMessage, setErrorMessage] = useState<string | null>(null);
  const [notice, setNotice] = useState<string | null>(null);
  const [dummyTaskCardsMode, setDummyTaskCardsMode] = useState(false);
  const [reviewScheduleScrollIndex, setReviewScheduleScrollIndex] = useState(0);
  const [stickyBaseTopPx, setStickyBaseTopPx] = useState(64);
  const [statusBannerHeightPx, setStatusBannerHeightPx] = useState(0);
  const tasksLoadInFlight = useRef(false);
  const activeTasksRef = useRef<TrainTaskSummary[]>([]);
  const completedTasksRef = useRef<TrainTaskSummary[]>([]);
  const queuedTaskReload = useRef<{ pending: boolean; refreshCompleted: boolean; force: boolean }>({
    pending: false,
    refreshCompleted: false,
    force: false,
  });
  const dummyTaskCardsModeRef = useRef(false);
  const searchPanelRef = useRef<HTMLDivElement | null>(null);
  const schedulePanelRef = useRef<HTMLDivElement | null>(null);
  const searchSummaryCardRef = useRef<HTMLDivElement | null>(null);
  const statusBannerRef = useRef<HTMLDivElement | null>(null);
  const reviewSchedulesCarouselRef = useRef<HTMLDivElement | null>(null);

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
  const ktxVerified = Boolean(credentialStatus?.ktx.verified);
  const srtVerified = Boolean(credentialStatus?.srt.verified);
  const ktxTemporarilyUnavailable = String(credentialStatus?.ktx.detail ?? "")
    .trim()
    .toLowerCase()
    .includes("temporarily unavailable");
  const ktxProviderEnabled = TRAIN_KTX_FEATURE_ENABLED && !ktxTemporarilyUnavailable;
  const searchProviderOptions: CredentialProvider[] = ktxProviderEnabled ? ["SRT", "KTX"] : ["SRT"];
  const credentialProviderOptions: CredentialProvider[] = ktxProviderEnabled ? ["KTX", "SRT"] : ["SRT"];
  const selectedSearchProviders = useMemo(
    () => searchProviderOptions.filter((provider) => searchForm.providers[provider]),
    [searchForm.providers, searchProviderOptions],
  );
  const selectedDateLabel = useMemo(
    () => formatScheduleTitleDate(lastSearchResultDate ?? searchForm.date),
    [lastSearchResultDate, searchForm.date],
  );
  const searchSummaryDep = formatStationLabel(searchForm.dep, locale, { compact: true });
  const searchSummaryArr = formatStationLabel(searchForm.arr, locale, { compact: true });
  const searchSummaryTimeRange = `${searchForm.start} - ${searchForm.end}`;
  const searchSummaryProvider = selectedSearchProviders.length > 0 ? selectedSearchProviders.join(" + ") : "-";
  const searchSummaryPassengerParts: string[] = [];
  if (createForm.adults > 0) {
    searchSummaryPassengerParts.push(
      locale.toLowerCase().startsWith("ko")
        ? `성인 ${createForm.adults}`
        : `${createForm.adults} ${t("train.adult")}${createForm.adults === 1 ? "" : "s"}`,
    );
  }
  if (createForm.children > 0) {
    searchSummaryPassengerParts.push(
      locale.toLowerCase().startsWith("ko")
        ? `아동 ${createForm.children}`
        : `${createForm.children} ${t("train.child")}${createForm.children === 1 ? "" : "ren"}`,
    );
  }
  const searchSummaryPassengers = searchSummaryPassengerParts.length > 0 ? searchSummaryPassengerParts.join(", ") : "-";

  const selectedProviderCount = searchProviderOptions.reduce(
    (count, provider) => count + Number(searchForm.providers[provider]),
    0,
  );
  const hasSearchResults = schedules.length > 0;
  const showRanking = hasSearched && hasSearchResults;
  const suggestedCredentialProvider =
    credentialStatus == null
      ? credentialProviderOptions[0] ?? null
      : credentialProviderOptions.find((provider) => {
          const providerVerified = provider === "SRT" ? srtVerified : ktxVerified;
          return !providerVerified && !omittedProviders.has(provider);
        }) ?? null;
  const activeCredentialProvider = credentialProvider ?? suggestedCredentialProvider;
  const hasAnyConnectedProvider = credentialProviderOptions.some((provider) =>
    provider === "SRT" ? srtVerified : ktxVerified,
  );
  const searchUnlocked = credentialStatus != null && hasAnyConnectedProvider;
  const searchDisabled = searching || selectedProviderCount === 0 || !searchUnlocked;
  const totalPassengers = createForm.adults + createForm.children;
  const createDisabled = !showRanking || selectedSchedules.length === 0 || creatingTask || totalPassengers < 1;
  const statusBannerCount = Number(Boolean(errorMessage)) + Number(Boolean(notice));
  const hasStatusBanner = statusBannerCount > 0;
  const searchSummaryStickyTopPx = stickyBaseTopPx + statusBannerHeightPx;
  const sortedActiveTasks = useMemo(() => sortActiveTasksByImminence(activeTasks), [activeTasks]);
  const sortedCompletedTasks = useMemo(() => sortTasksByScheduleProximity(completedTasks, "asc"), [completedTasks]);
  const autoPayAvailable = TRAIN_AUTO_PAY_FEATURE_ENABLED && paymentCardConfigured;
  const reviewScheduleProviders = useMemo(() => {
    const seen = new Set<string>();
    const providers: string[] = [];
    for (const schedule of selectedSchedules) {
      if (!seen.has(schedule.provider)) {
        seen.add(schedule.provider);
        providers.push(schedule.provider);
      }
    }
    return providers.join(" + ");
  }, [selectedSchedules]);
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

  useEffect(() => {
    activeTasksRef.current = activeTasks;
  }, [activeTasks]);

  useEffect(() => {
    completedTasksRef.current = completedTasks;
  }, [completedTasks]);

  const toggleProviderSelection = (provider: CredentialProvider) => {
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

  const handleReviewSchedulesScroll = useCallback(
    (event: UIEvent<HTMLDivElement>) => {
      if (selectedSchedules.length <= 1) {
        setReviewScheduleScrollIndex(0);
        return;
      }
      const container = event.currentTarget;
      const viewportWidth = container.clientWidth;
      if (viewportWidth <= 0) {
        return;
      }
      const nextIndex = Math.min(
        selectedSchedules.length - 1,
        Math.max(0, Math.round(container.scrollLeft / viewportWidth)),
      );
      setReviewScheduleScrollIndex(nextIndex);
    },
    [selectedSchedules.length],
  );

  const reloadTasks = useCallback(async (options?: { refreshCompleted?: boolean; force?: boolean }) => {
    if (dummyTaskCardsModeRef.current) return;
    if (tasksLoadInFlight.current) {
      queuedTaskReload.current = {
        pending: true,
        refreshCompleted: queuedTaskReload.current.refreshCompleted || Boolean(options?.refreshCompleted),
        force: queuedTaskReload.current.force || Boolean(options?.force),
      };
      return;
    }
    tasksLoadInFlight.current = true;
    try {
      const { active, completed } = await fetchTaskListBootstrap({
        refreshCompleted: options?.refreshCompleted,
        force: Boolean(options?.force),
        activeLimit: ACTIVE_TASK_FETCH_LIMIT,
        completedLimit: COMPLETED_TASK_FETCH_LIMIT,
      });
      const nextActiveKey = taskListRenderKey(active);
      setActiveTasks((current) => (taskListRenderKey(current) === nextActiveKey ? current : active));
      const nextCompletedKey = taskListRenderKey(completed);
      setCompletedTasks((current) => (taskListRenderKey(current) === nextCompletedKey ? current : completed));
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
      if (queuedTaskReload.current.pending) {
        const queued = queuedTaskReload.current;
        queuedTaskReload.current = { pending: false, refreshCompleted: false, force: false };
        void reloadTasks({ refreshCompleted: queued.refreshCompleted, force: queued.force });
      }
    }
  }, []);

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

  const duplicateCategoryLabel = (category: DuplicateMatchCategory): string => {
    if (category === "already_reserved") return t("train.duplicateCheck.category.already_reserved");
    if (category === "waiting") return t("train.duplicateCheck.category.waiting");
    return t("train.duplicateCheck.category.polling");
  };

  const duplicateCategoryClass = (category: DuplicateMatchCategory): string => {
    if (category === "already_reserved") {
      return "border-emerald-200 bg-emerald-50 text-emerald-700";
    }
    if (category === "waiting") {
      return "border-amber-200 bg-amber-50 text-amber-700";
    }
    return "border-sky-200 bg-sky-50 text-sky-700";
  };

  const duplicateWarningDetailHref = duplicateDetailTargetTaskId
    ? `${ROUTES.modules.train}/tasks/${duplicateDetailTargetTaskId}`
    : null;

  const taskErrorPresentation = (
    task: TrainTaskSummary,
  ): { message: string; className: string } | null => {
    if (task.last_attempt_ok !== false) return null;

    const normalizedRawMessage = normalizeTaskErrorMessage(task.last_attempt_error_message_safe ?? "");
    const fallbackCode = task.last_attempt_error_code ?? "";
    const lowerMessage = normalizedRawMessage.toLowerCase();
    const lowerCode = fallbackCode.toLowerCase();

    const isSeatUnavailable =
      SOLD_OUT_LIKE_ERROR_CODES.has(lowerCode) ||
      lowerMessage.includes("no available seats") ||
      lowerMessage.includes("no reservable seats") ||
      (lowerMessage.includes("좌석") && lowerMessage.includes("없"));

    if (isSeatUnavailable) {
      return {
        message: t("train.taskError.seatUnavailable"),
        className: "border-amber-200 bg-amber-50 text-amber-700",
      };
    }

    const reservationChanged =
      lowerMessage.includes("reservation status") ||
      lowerMessage.includes("예약상태가 변경되었습니다");
    if (reservationChanged) {
      return {
        message: t("train.taskError.reservationChanged"),
        className: "border-sky-200 bg-sky-50 text-sky-700",
      };
    }

    return {
      message: normalizedRawMessage || fallbackCode || t("train.taskError.unknown"),
      className: "border-rose-200 bg-rose-50 text-rose-700",
    };
  };

  const seedDummyTaskCards = () => {
    const dummy = buildDummyTaskCards();
    setDummyTaskCardsModeEnabled(true);
    storeDummyTaskCards([...dummy.active, ...dummy.completed]);
    dummyTaskCardsModeRef.current = true;
    setDummyTaskCardsMode(true);
    setActiveTasks(dummy.active);
    setCompletedTasks(dummy.completed);
    setErrorMessage(null);
    setNotice(t("train.notice.dummyTaskCardsLoaded"));
  };

  const clearDummyTaskCards = async () => {
    clearStoredDummyTaskCards();
    dummyTaskCardsModeRef.current = false;
    setDummyTaskCardsMode(false);
    setNotice(t("train.notice.liveTaskCardsRestored"));
    await reloadTasks({ force: true, refreshCompleted: true });
  };

  const loadCredentialStatus = useCallback(async (options?: { force?: boolean }) => {
    setCredentialLoading(true);
    const abortController = new AbortController();
    const timeoutHandle = window.setTimeout(() => abortController.abort(), CREDENTIAL_STATUS_TIMEOUT_MS);
    try {
      const payload = await fetchCredentialStatusForDashboard({
        force: Boolean(options?.force),
        signal: abortController.signal,
      });
      setCredentialStatus(payload);
      setOmittedProviders((current) => {
        const next = new Set(current);
        if (payload.ktx.verified) next.delete("KTX");
        if (payload.srt.verified) next.delete("SRT");
        return next;
      });

      setCredentialProvider(null);
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
      setCredentialLoading(false);
    }
  }, [t]);

  const loadPaymentCardConfigured = useCallback(async (options?: { force?: boolean }) => {
    try {
      const configured = await fetchPaymentCardConfiguredForDashboard({ force: Boolean(options?.force) });
      setPaymentCardConfigured(configured);
    } catch {
      setErrorMessage((current) => current ?? t("train.error.walletStatusLoad"));
    }
  }, [t]);

  useEffect(() => {
    void loadCredentialStatus();
  }, [loadCredentialStatus]);

  useEffect(() => {
    if (!TRAIN_AUTO_PAY_FEATURE_ENABLED) {
      setPaymentCardConfigured(false);
      return;
    }
    void loadPaymentCardConfigured();
  }, [loadPaymentCardConfigured]);

  useLayoutEffect(() => {
    if (typeof window === "undefined") return;

    const computeOffsets = () => {
      const topNav = document.querySelector<HTMLElement>('[data-top-nav="true"]');
      const navBottom = topNav ? topNav.getBoundingClientRect().bottom : 56;
      const nextBaseTop = Math.max(0, Math.round(navBottom + 8));
      setStickyBaseTopPx((current) => (current === nextBaseTop ? current : nextBaseTop));

      if (!hasStatusBanner || !statusBannerRef.current) {
        setStatusBannerHeightPx((current) => (current === 0 ? current : 0));
        return;
      }

      const nextStatusHeight = Math.max(0, Math.round(statusBannerRef.current.getBoundingClientRect().height + 8));
      setStatusBannerHeightPx((current) => (current === nextStatusHeight ? current : nextStatusHeight));
    };

    computeOffsets();

    const animationFrame = window.requestAnimationFrame(computeOffsets);
    const onResize = () => {
      computeOffsets();
    };
    window.addEventListener("resize", onResize);

    const observer =
      typeof ResizeObserver !== "undefined"
        ? new ResizeObserver(() => {
            computeOffsets();
          })
        : null;
    const topNav = document.querySelector<HTMLElement>('[data-top-nav="true"]');
    if (topNav) observer?.observe(topNav);
    if (statusBannerRef.current) observer?.observe(statusBannerRef.current);

    return () => {
      window.cancelAnimationFrame(animationFrame);
      window.removeEventListener("resize", onResize);
      observer?.disconnect();
    };
  }, [hasStatusBanner, errorMessage, notice]);

  useEffect(() => {
    if (!shouldScrollToSearchResultsSection || !hasSearched) return;
    const animationFrame = window.requestAnimationFrame(() => {
      const stickyOffset = showRanking ? searchSummaryStickyTopPx : 0;

      // When schedules are visible, scroll so the schedule section sits right
      // below the sticky search summary (instead of stopping too early).
      if (showRanking && schedulePanelRef.current && searchSummaryCardRef.current) {
        const summaryHeight = searchSummaryCardRef.current.getBoundingClientRect().height;
        const summaryGapPx = 16; // matches `space-y-4`
        scrollElementToViewportTop(schedulePanelRef.current, {
          offsetPx: stickyOffset + summaryHeight + summaryGapPx,
        });
      } else {
        scrollElementToViewportTop(searchSummaryCardRef.current, { offsetPx: stickyOffset });
      }
      setShouldScrollToSearchResultsSection(false);
    });
    return () => window.cancelAnimationFrame(animationFrame);
  }, [hasSearched, shouldScrollToSearchResultsSection, showRanking, searchSummaryStickyTopPx]);

  useEffect(() => {
    if (!errorMessage) return;
    const timeout = window.setTimeout(() => {
      setErrorMessage((current) => (current === errorMessage ? null : current));
    }, 5_000);
    return () => window.clearTimeout(timeout);
  }, [errorMessage]);

  useEffect(() => {
    if (!notice) return;
    const timeout = window.setTimeout(() => {
      setNotice((current) => (current === notice ? null : current));
    }, 5_000);
    return () => window.clearTimeout(timeout);
  }, [notice]);

  useEffect(() => {
    if (showRanking && selectedSchedules.length > 0) return;
    setCreateTaskReviewOpen(false);
  }, [selectedSchedules.length, showRanking]);

  useEffect(() => {
    if (!createTaskReviewOpen) {
      setReviewScheduleScrollIndex(0);
      return;
    }
    const animationFrame = window.requestAnimationFrame(() => {
      const carousel = reviewSchedulesCarouselRef.current;
      if (carousel) {
        carousel.scrollTo({ left: 0, top: 0, behavior: "auto" });
      }
      setReviewScheduleScrollIndex(0);
    });
    return () => window.cancelAnimationFrame(animationFrame);
  }, [createTaskReviewOpen, selectedSchedules.length]);

  useEffect(() => {
    setCreateForm((current) => {
      const nextAutoPay = autoPayAvailable;
      if (current.autoPay === nextAutoPay) {
        return current;
      }
      return { ...current, autoPay: nextAutoPay };
    });
  }, [autoPayAvailable]);

  useEffect(() => {
    if (!credentialStatus) return;

    setSearchForm((current) => ({
      ...current,
      providers: {
        SRT: srtVerified,
        KTX: ktxProviderEnabled ? ktxVerified : false,
      },
    }));
  }, [credentialStatus, ktxProviderEnabled, ktxVerified, srtVerified]);

  useEffect(() => {
    if (!hasAnyConnectedProvider) {
      setCredentialPanelOpen(true);
      return;
    }
    const allVerified = credentialProviderOptions.every((provider) =>
      provider === "SRT" ? srtVerified : ktxVerified,
    );
    if (allVerified) {
      setCredentialPanelOpen(false);
    }
  }, [credentialProviderOptions, hasAnyConnectedProvider, ktxVerified, srtVerified]);

  useEffect(() => {
    const refreshIfVisible = async (options?: { refreshCompleted?: boolean }) => {
      if (document.visibilityState === "hidden") return;
      await reloadTasks({ refreshCompleted: options?.refreshCompleted, force: true });
    };

    void reloadTasks({ refreshCompleted: true });
    const unsubscribeTaskEvents = subscribeTrainTaskEvents((payload, event) => {
      const eventType = readTaskEventType(payload.type, event.type);
      if (eventType === "task_ticket_status") {
        const ticketStatus = String(payload.ticket_status || "").trim().toLowerCase();
        const previousTicketStatus = String(payload.previous_ticket_status || "").trim().toLowerCase();
        if (
          TASK_LIST_PENDING_TICKET_STATUSES.has(ticketStatus) ||
          TASK_LIST_PENDING_TICKET_STATUSES.has(previousTicketStatus)
        ) {
          void refreshIfVisible({ refreshCompleted: true });
        }
        return;
      }
      if (eventType !== "task_state") return;
      const state = readTaskEventState(payload.state);
      if (!state) return;
      if (TASK_LIST_REFRESH_EVENT_STATES.has(state)) {
        void refreshIfVisible({ refreshCompleted: true });
        return;
      }
      const taskId = typeof payload.task_id === "string" ? payload.task_id : null;
      if (!taskId) return;
      const knownTask =
        hasTaskById(activeTasksRef.current, taskId) || hasTaskById(completedTasksRef.current, taskId);
      if (!knownTask) {
        // Task list can be stale across tabs/sessions; reconcile once on unknown events.
        void refreshIfVisible({ refreshCompleted: true });
        return;
      }
      const updatedAt = typeof payload.updated_at === "string" ? payload.updated_at : null;
      setActiveTasks((current) => applyTaskEventState(current, { taskId, state, updatedAt }));
      setCompletedTasks((current) => applyTaskEventState(current, { taskId, state, updatedAt }));
    });

    const onVisibilityChange = () => {
      if (document.visibilityState === "visible") {
        void refreshIfVisible({ refreshCompleted: true });
      }
    };

    document.addEventListener("visibilitychange", onVisibilityChange);
    return () => {
      unsubscribeTaskEvents();
      document.removeEventListener("visibilitychange", onVisibilityChange);
    };
  }, [reloadTasks]);

  useEffect(() => {
    let alive = true;
    const loadStations = async () => {
      setStationsLoading(true);
      try {
        const loadedStations = await getTrainStationsCached();
        if (!alive) {
          return;
        }
        setStations(loadedStations);
        if (loadedStations.length > 0) {
          setSearchForm((current) => ({
            ...current,
            ...resolveSearchStations(current.dep, current.arr, loadedStations),
          }));
        }
      } catch {
        // Keep default stations when station list cannot be loaded.
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

    const providers = searchProviderOptions.filter((provider) => searchForm.providers[provider]);

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
      setShouldScrollToSearchResultsSection(true);
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
    const provider = activeCredentialProvider as CredentialProvider;

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
      await loadCredentialStatus({ force: true });
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
      await loadCredentialStatus({ force: true });
    } catch {
      setErrorMessage(t("train.providerErrors.signOutFailed", { provider }));
    } finally {
      setSigningOutProvider(null);
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

  const buildCreatePayload = useCallback(
    (confirmDuplicate: boolean): TrainTaskCreatePayload => {
      const ranked = selectedSchedules.map((schedule, index) => ({
        schedule_id: schedule.schedule_id,
        departure_at: schedule.departure_at,
        arrival_at: schedule.arrival_at,
        rank: index + 1,
        provider: schedule.provider,
      }));
      return {
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
        retry_on_expiry: createForm.retryOnExpiry,
        notify: createForm.notify,
        confirm_duplicate: confirmDuplicate,
      };
    },
    [
      autoPayAvailable,
      createForm.adults,
      createForm.autoPay,
      createForm.children,
      createForm.notify,
      createForm.retryOnExpiry,
      createForm.seatClass,
      searchForm.arr,
      searchForm.date,
      searchForm.dep,
      selectedSchedules,
    ],
  );

  const submitCreateTask = useCallback(
    async (payload: TrainTaskCreatePayload): Promise<boolean> => {
      const response = await fetch(`${clientApiBaseUrl}/api/train/tasks`, {
        method: "POST",
        credentials: "include",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify(payload),
      });
      if (!response.ok) {
        const detail = await parseApiErrorMessage(response, t("train.error.createTask"));
        setErrorMessage(detail);
        return false;
      }

      const createResponsePayload = (await response.json()) as {
        task: TrainTaskSummary;
        deduplicated: boolean;
      };

      setNotice(createResponsePayload.deduplicated ? t("train.notice.taskDeduplicated") : t("train.notice.taskCreatedQueued"));
      await reloadTasks({ force: true });
      return true;
    },
    [reloadTasks, t],
  );

  const openCreateTaskReview = async () => {
    const validationError = validateCreateTaskInputs(selectedSchedules.length, totalPassengers);
    if (validationError) {
      setErrorMessage(t(validationError));
      return;
    }
    setErrorMessage(null);
    setNotice(null);
    if (TRAIN_AUTO_PAY_FEATURE_ENABLED) {
      await loadPaymentCardConfigured({ force: true });
    }
    setCreateTaskReviewOpen(true);
  };

  const createTask = async () => {
    setCreateTaskReviewOpen(false);
    setCreatingTask(true);
    setErrorMessage(null);
    setNotice(null);
    setDuplicateWarning(null);
    setDuplicateDetailTargetTaskId(null);
    const payload = buildCreatePayload(false);

    try {
      let duplicatePayload: TrainTaskDuplicateCheckResponse | null = null;
      try {
        const duplicateResponse = await fetch(`${clientApiBaseUrl}/api/train/tasks/duplicate-check`, {
          method: "POST",
          credentials: "include",
          headers: { "Content-Type": "application/json" },
          body: JSON.stringify(payload),
        });
        if (duplicateResponse.ok) {
          const duplicateJson = (await duplicateResponse.json().catch(() => null)) as unknown;
          if (isDuplicateCheckResponse(duplicateJson)) {
            duplicatePayload = duplicateJson;
          }
        }
      } catch {
        // Duplicate warning preflight is best-effort; creation can continue on transport failures.
      }

      if (duplicatePayload?.has_duplicate) {
        setDuplicateWarning(duplicatePayload);
        setDuplicateDetailTargetTaskId(null);
        return;
      }

      await submitCreateTask(payload);
    } catch {
      setErrorMessage(t("train.error.createTask"));
    } finally {
      setCreatingTask(false);
    }
  };

  const createTaskWithDuplicateOverride = async () => {
    setCreatingTask(true);
    setErrorMessage(null);
    setNotice(null);
    try {
      const ok = await submitCreateTask(buildCreatePayload(true));
      if (ok) {
        setDuplicateWarning(null);
        setDuplicateDetailTargetTaskId(null);
      }
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
      await reloadTasks({ force: true });
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
      await reloadTasks({ force: true });
    } catch {
      setErrorMessage(t("train.error.ticketCancel"));
    } finally {
      setCancellingTaskId(null);
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
      await reloadTasks({ refreshCompleted: true, force: true });
    } catch {
      setErrorMessage(t("train.error.paymentProcess"));
    } finally {
      setPayingTaskId(null);
    }
  };

  const onExpandSearchMobile = () => {
    if (typeof window !== "undefined") {
      window.scrollTo({ top: 0, behavior: "smooth" });
      return;
    }
    scrollElementToViewportTop(searchPanelRef.current);
  };

  const createTaskReviewModal =
    createTaskReviewOpen && typeof document !== "undefined"
      ? createPortal(
          <div className="fixed inset-0 z-[230] flex items-center justify-center bg-slate-900/60 px-4 py-8">
            <div
              role="dialog"
              aria-modal="true"
              aria-labelledby="create-task-review-title"
              className="w-full max-w-2xl rounded-2xl border border-blossom-200 bg-white p-5 shadow-xl"
            >
              <h3 id="create-task-review-title" className="text-lg font-semibold text-slate-800">
                {t("train.reviewTask.title")}
              </h3>
              <p className="mt-2 text-sm text-slate-600">{t("train.reviewTask.body")}</p>

              <div className="mt-4 overflow-hidden rounded-2xl border border-blossom-100 bg-white">
                <div className="border-b border-dashed border-blossom-200 bg-gradient-to-r from-blossom-50 via-white to-sky-50 px-4 py-3">
                  <div className="flex items-center justify-between gap-3 text-xs font-medium text-slate-500">
                    <p className="truncate">{selectedDateLabel}</p>
                    <p className="truncate text-right">{searchSummaryPassengers}</p>
                  </div>
                  <div className="mt-2 grid grid-cols-[minmax(0,1fr)_auto_minmax(0,1fr)] items-center gap-2">
                    <p className="truncate text-center text-sm font-semibold text-slate-800">
                      {formatStationLabel(searchForm.dep, locale)}
                    </p>
                    <span className="inline-flex items-center gap-1 text-slate-400">
                      <span className="h-px w-4 bg-blossom-200 sm:w-6" />
                      <svg
                        aria-hidden="true"
                        viewBox="0 0 20 20"
                        className="h-4 w-4 text-slate-400"
                        fill="none"
                        stroke="currentColor"
                        strokeWidth="1.8"
                        strokeLinecap="round"
                        strokeLinejoin="round"
                      >
                        <path d="M3 10h12" />
                        <path d="m11 6 4 4-4 4" />
                      </svg>
                      <span className="h-px w-4 bg-blossom-200 sm:w-6" />
                    </span>
                    <p className="truncate text-center text-sm font-semibold text-slate-800">
                      {formatStationLabel(searchForm.arr, locale)}
                    </p>
                  </div>
                </div>

                <dl className="divide-y divide-blossom-100/70 px-4 py-1 text-sm text-slate-700">
                  <div className="flex min-h-12 items-center justify-between gap-3 py-2">
                    <dt className="text-slate-500">{t("train.seatClass")}</dt>
                    <dd className="text-right font-medium text-slate-800">{seatClassLabels[createForm.seatClass]}</dd>
                  </div>
                  <div className="flex min-h-12 items-center justify-between gap-3 py-2">
                    <dt className="text-slate-500">{t("train.provider")}</dt>
                    <dd className="text-right font-medium text-slate-800">{reviewScheduleProviders || "-"}</dd>
                  </div>
                  <div className="flex min-h-12 items-center justify-between gap-3 py-2">
                    <dt className="text-slate-500">{t("train.autoPay")}</dt>
                    <dd className="flex items-center">
                      <button
                        type="button"
                        role="switch"
                        aria-checked={createForm.autoPay && autoPayAvailable}
                        aria-label={t("train.autoPay")}
                        disabled={!autoPayAvailable}
                        onClick={() =>
                          setCreateForm((current) => ({
                            ...current,
                            autoPay: autoPayAvailable ? !current.autoPay : false,
                          }))
                        }
                        className={`relative inline-flex h-6 w-11 shrink-0 items-center rounded-full transition focus:outline-none focus:ring-2 focus:ring-blossom-100 ${
                          createForm.autoPay && autoPayAvailable ? "bg-blossom-500" : "bg-slate-300"
                        } ${!autoPayAvailable ? "cursor-not-allowed opacity-50" : ""}`}
                      >
                        <span
                          aria-hidden="true"
                          className={`inline-block h-5 w-5 transform rounded-full bg-white shadow-sm transition ${
                            createForm.autoPay && autoPayAvailable ? "translate-x-5" : "translate-x-0.5"
                          }`}
                        />
                      </button>
                    </dd>
                  </div>
                  <div className="flex min-h-12 items-center justify-between gap-3 py-2">
                    <dt className="text-slate-500">{t("train.notify")}</dt>
                    <dd className="flex items-center">
                      <button
                        type="button"
                        role="switch"
                        aria-checked={createForm.notify}
                        aria-label={t("train.notify")}
                        onClick={() =>
                          setCreateForm((current) => ({
                            ...current,
                            notify: !current.notify,
                          }))
                        }
                        className={`relative inline-flex h-6 w-11 shrink-0 items-center rounded-full transition focus:outline-none focus:ring-2 focus:ring-blossom-100 ${
                          createForm.notify ? "bg-blossom-500" : "bg-slate-300"
                        }`}
                      >
                        <span
                          aria-hidden="true"
                          className={`inline-block h-5 w-5 transform rounded-full bg-white shadow-sm transition ${
                            createForm.notify ? "translate-x-5" : "translate-x-0.5"
                          }`}
                        />
                      </button>
                    </dd>
                  </div>
                  <div className="flex min-h-12 items-center justify-between gap-3 py-2">
                    <dt className="text-slate-500">{t("train.retryOnExpiry")}</dt>
                    <dd className="flex items-center">
                      <button
                        type="button"
                        role="switch"
                        aria-checked={createForm.retryOnExpiry}
                        aria-label={t("train.retryOnExpiry")}
                        onClick={() =>
                          setCreateForm((current) => ({
                            ...current,
                            retryOnExpiry: !current.retryOnExpiry,
                          }))
                        }
                        className={`relative inline-flex h-6 w-11 shrink-0 items-center rounded-full transition focus:outline-none focus:ring-2 focus:ring-blossom-100 ${
                          createForm.retryOnExpiry ? "bg-blossom-500" : "bg-slate-300"
                        }`}
                      >
                        <span
                          aria-hidden="true"
                          className={`inline-block h-5 w-5 transform rounded-full bg-white shadow-sm transition ${
                            createForm.retryOnExpiry ? "translate-x-5" : "translate-x-0.5"
                          }`}
                        />
                      </button>
                    </dd>
                  </div>
                </dl>
                {TRAIN_AUTO_PAY_FEATURE_ENABLED && !autoPayAvailable ? (
                  <p className="px-4 pb-3 text-xs text-slate-500">{t("train.walletRequiredAutoPay")}</p>
                ) : null}
              </div>

              <div className="mt-4 rounded-xl border border-blossom-100 bg-blossom-50/40 p-3">
                <p className="text-xs font-medium uppercase tracking-[0.14em] text-blossom-500">
                  {t("train.reviewTask.selectedSchedules")}
                </p>
                <div
                  ref={reviewSchedulesCarouselRef}
                  onScroll={handleReviewSchedulesScroll}
                  className="mt-2 flex snap-x snap-mandatory overflow-x-auto [scrollbar-width:none] [-ms-overflow-style:none] [&::-webkit-scrollbar]:hidden"
                >
                  {selectedSchedules.map((schedule, index) => (
                    <article key={schedule.schedule_id} className="w-full shrink-0 snap-start pr-2 last:pr-0">
                      <div className="rounded-lg border border-blossom-100 bg-white px-3 py-2">
                        <div className="flex flex-wrap items-center justify-between gap-2">
                          <p className="text-sm font-medium text-slate-800">
                            {index + 1}. {scheduleTrainLabel(schedule)}
                          </p>
                          <p className="text-xs text-slate-500">{schedule.provider}</p>
                        </div>
                        <p className="mt-1 text-xs text-slate-600">
                          {formatTimeKst(schedule.departure_at, locale)} - {formatTimeKst(schedule.arrival_at, locale)} ·{" "}
                          {formatTransitDuration(schedule.departure_at, schedule.arrival_at)}
                        </p>
                      </div>
                    </article>
                  ))}
                </div>
                {selectedSchedules.length > 1 ? (
                  <div className="mt-2 flex items-center justify-center gap-1.5">
                    {selectedSchedules.map((schedule, index) => (
                      <span
                        key={schedule.schedule_id}
                        aria-hidden="true"
                        className={`rounded-full transition ${
                          index === reviewScheduleScrollIndex ? "h-1.5 w-4 bg-blossom-500" : "h-1.5 w-1.5 bg-blossom-200"
                        }`}
                      />
                    ))}
                  </div>
                ) : null}
              </div>

              <div className="mt-5 flex flex-wrap justify-end gap-2">
                <button
                  type="button"
                  onClick={() => setCreateTaskReviewOpen(false)}
                  className={SECONDARY_BUTTON_CLASS}
                  disabled={creatingTask}
                >
                  {t("common.cancel")}
                </button>
                <button
                  type="button"
                  onClick={() => void createTask()}
                  className={PRIMARY_BUTTON_CLASS}
                  disabled={creatingTask}
                >
                  {creatingTask ? t("train.creatingTask") : t("train.reviewTask.confirm")}
                </button>
              </div>
            </div>
          </div>,
          document.body,
        )
      : null;

  const duplicateWarningModal =
    duplicateWarning && typeof document !== "undefined"
      ? createPortal(
          <div className="fixed inset-0 z-[220] flex items-center justify-center bg-slate-900/60 px-4 py-8">
            {duplicateDetailTargetTaskId ? (
              <div
                role="dialog"
                aria-modal="true"
                aria-labelledby="duplicate-warning-leave-title"
                className="w-full max-w-lg rounded-2xl border border-blossom-200 bg-white p-5 shadow-xl"
              >
                <h3 id="duplicate-warning-leave-title" className="text-lg font-semibold text-slate-800">
                  {t("train.duplicateCheck.leaveDetailTitle")}
                </h3>
                <p className="mt-2 text-sm text-slate-600">{t("train.confirm.leaveDuplicateWarningDetail")}</p>
                <div className="mt-5 flex flex-wrap justify-end gap-2">
                  <button
                    type="button"
                    onClick={() => setDuplicateDetailTargetTaskId(null)}
                    className={SECONDARY_BUTTON_CLASS}
                  >
                    {t("common.cancel")}
                  </button>
                  {duplicateWarningDetailHref ? (
                    <Link href={duplicateWarningDetailHref} className={PRIMARY_BUTTON_CLASS}>
                      {t("train.action.detail")}
                    </Link>
                  ) : null}
                </div>
              </div>
            ) : (
              <div
                role="dialog"
                aria-modal="true"
                aria-labelledby="duplicate-warning-title"
                className="w-full max-w-2xl rounded-2xl border border-blossom-200 bg-white p-5 shadow-xl"
              >
                <h3 id="duplicate-warning-title" className="text-lg font-semibold text-slate-800">
                  {t("train.duplicateCheck.title")}
                </h3>
                <p className="mt-2 text-sm text-slate-600">{t("train.duplicateCheck.body")}</p>
                <ul className="mt-4 grid gap-2 md:grid-cols-3">
                  {DUPLICATE_CATEGORY_ORDER.map((category) => {
                    const count = duplicateWarning.summary[category];
                    if (count < 1) return null;
                    return (
                      <li
                        key={category}
                        className={`rounded-xl border px-3 py-2 text-sm font-medium ${duplicateCategoryClass(category)}`}
                      >
                        {duplicateCategoryLabel(category)} ({count})
                      </li>
                    );
                  })}
                </ul>
                <div className="mt-4 max-h-56 space-y-2 overflow-y-auto rounded-xl border border-blossom-100 bg-blossom-50/40 p-3">
                  {duplicateWarning.matches.slice(0, 8).map((match) => (
                    <div key={match.task_id} className="flex flex-wrap items-center justify-between gap-2 rounded-lg bg-white px-3 py-2">
                      <div className="min-w-0">
                        <p className="text-sm font-medium text-slate-700">{duplicateCategoryLabel(match.category)}</p>
                        <p className="text-xs text-slate-500">{formatDateTimeKst(match.departure_at, locale)}</p>
                      </div>
                      <button
                        type="button"
                        onClick={() => setDuplicateDetailTargetTaskId(match.task_id)}
                        className="text-xs font-medium text-blossom-600 hover:text-blossom-700"
                      >
                        {t("train.action.detail")}
                      </button>
                    </div>
                  ))}
                </div>
                <div className="mt-5 flex flex-wrap justify-end gap-2">
                  <button
                    type="button"
                    onClick={() => {
                      setDuplicateWarning(null);
                      setDuplicateDetailTargetTaskId(null);
                    }}
                    className={SECONDARY_BUTTON_CLASS}
                    disabled={creatingTask}
                  >
                    {t("common.cancel")}
                  </button>
                  <button
                    type="button"
                    onClick={() => void createTaskWithDuplicateOverride()}
                    className={PRIMARY_BUTTON_CLASS}
                    disabled={creatingTask}
                  >
                    {creatingTask ? t("train.creatingTask") : t("train.duplicateCheck.createAnyway")}
                  </button>
                </div>
              </div>
            )}
          </div>,
          document.body,
        )
      : null;

  const searchSummaryCard =
    searchUnlocked && hasSearched ? (
      <div
        ref={searchSummaryCardRef}
        data-testid="search-summary-inline"
        className={showRanking ? "sticky z-30" : ""}
        style={showRanking ? { top: searchSummaryStickyTopPx } : undefined}
      >
        <div
          className={`min-w-0 rounded-xl border border-blossom-100 bg-white px-3 py-3 shadow-petal ${
            showRanking ? "bg-white/95 backdrop-blur supports-[backdrop-filter]:bg-white/80" : ""
          }`}
        >
          <p className="text-[11px] font-medium uppercase tracking-[0.14em] text-blossom-500">
            {t("train.searchSummary")}
          </p>
          <div className="mt-2 flex items-start justify-between gap-3">
            <p className="truncate text-xs font-medium text-slate-600">{selectedDateLabel}</p>
            <p className="truncate text-xs text-slate-600">{searchSummaryPassengers}</p>
          </div>
          <div className="mt-2 grid grid-cols-[minmax(0,1fr)_auto_minmax(0,1fr)] items-center gap-2">
            <div className="min-w-0 text-center">
              <p className="text-[10px] font-semibold uppercase tracking-[0.12em] text-blossom-500">
                {t("train.fromShort")}
              </p>
              <p className="truncate text-sm font-semibold text-slate-800">{searchSummaryDep}</p>
            </div>
            <span className="inline-flex items-center gap-1 text-slate-400">
              <span className="h-px w-4 bg-blossom-200" />
              <svg
                aria-hidden="true"
                viewBox="0 0 20 20"
                className="h-4 w-4 text-slate-400"
                fill="none"
                stroke="currentColor"
                strokeWidth="1.8"
                strokeLinecap="round"
                strokeLinejoin="round"
              >
                <path d="M3 10h12" />
                <path d="m12 6 4 4-4 4" />
              </svg>
            </span>
            <div className="min-w-0 text-center">
              <p className="text-[10px] font-semibold uppercase tracking-[0.12em] text-blossom-500">
                {t("train.toShort")}
              </p>
              <p className="truncate text-sm font-semibold text-slate-800">{searchSummaryArr}</p>
            </div>
          </div>
          <div className="mt-2 flex items-center justify-between gap-3 text-xs text-slate-600">
            <p className="truncate">
              {`${t("train.timeShort")}: ${searchSummaryTimeRange}`}
            </p>
            <p className="truncate">{`${t("train.provider")}: ${searchSummaryProvider}`}</p>
          </div>
        </div>
      </div>
    ) : null;

  return (
    <section className="space-y-8">
      {hasStatusBanner ? (
        <div ref={statusBannerRef} className="sticky z-40 space-y-2" style={{ top: stickyBaseTopPx }}>
          {errorMessage ? (
            <p className="rounded-xl border border-rose-100 bg-rose-50 px-3 py-2 text-sm text-rose-700 shadow-sm">
              {renderError(errorMessage)}
            </p>
          ) : null}
          {notice ? (
            <p className="rounded-xl border border-emerald-100 bg-emerald-50 px-3 py-2 text-sm text-emerald-700 shadow-sm">
              {notice}
            </p>
          ) : null}
        </div>
      ) : null}
      {createTaskReviewModal}
      {duplicateWarningModal}

      <div ref={searchPanelRef} className="rounded-2xl border border-blossom-100 bg-white p-6 shadow-petal">
        <div className="flex items-center justify-between gap-3">
          <h2 className="text-lg font-semibold text-slate-800">{t("train.searchSchedules")}</h2>
          <div className="flex items-center gap-2">
            <div data-testid="provider-selector-desktop" className="hidden items-center gap-2 md:flex">
              {searchProviderOptions.map((provider) => {
                const isSelectable = isProviderSelectable(provider);
                const isSelected = isProviderSelected(provider);
                return (
                  <button
                    key={provider}
                    type="button"
                    aria-pressed={isSelected}
                    disabled={!isSelectable}
                    onClick={() => toggleProviderSelection(provider)}
                    className={`inline-flex h-10 items-center justify-center rounded-xl border px-4 text-sm font-medium transition focus:outline-none focus:ring-2 focus:ring-blossom-100 ${
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
                  {credentialProviderOptions.map((provider) => {
                    const statusInfo = provider === "KTX" ? credentialStatus?.ktx : credentialStatus?.srt;
                    const isVerified = Boolean(statusInfo?.verified);
                    const isSkipped = omittedProviders.has(provider) && !isVerified;
                    return (
                      <div key={provider} className="rounded-xl border border-blossom-100 bg-white px-3 py-3">
                        <div className="flex items-start justify-between gap-3">
                          <div className="min-w-0">
                            <p className="text-xs uppercase tracking-[0.14em] text-blossom-500">{provider}</p>
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
                                  username: "",
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
                    <div className="mt-4 grid grid-cols-3 gap-2">
                      <button
                        type="button"
                        onClick={() => continueWithoutProvider(activeCredentialProvider)}
                        disabled={credentialSubmitting}
                        className={`${SECONDARY_BUTTON_CLASS} h-10 w-full justify-center`}
                      >
                        {t("train.ignore")}
                      </button>
                      <button
                        type="button"
                        onClick={() => setCredentialProvider(null)}
                        className={`${SECONDARY_BUTTON_CLASS} h-10 w-full justify-center`}
                      >
                        {t("common.cancel")}
                      </button>
                      <button
                        type="submit"
                        disabled={credentialSubmitting}
                        className={`${PRIMARY_BUTTON_CLASS} h-10 w-full justify-center`}
                      >
                        {credentialSubmitting ? t("train.verifying") : t("train.connect")}
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
            <form
              data-testid="train-search-form"
              onSubmit={onSearch}
              className="mt-4"
            >
            <div className="grid gap-4 lg:grid-cols-[minmax(0,2fr)_minmax(0,1fr)]">
              <div className="rounded-2xl border border-blossom-100 bg-blossom-50/40 p-4">
                <p className={SEARCH_SECTION_LABEL_CLASS}>{t("train.station")}</p>
                <div className="mt-3 grid grid-cols-[minmax(0,1fr)_auto_minmax(0,1fr)] items-end gap-2 sm:gap-3">
                  <StationAutocompleteField
                    label={t("train.departureStation")}
                    locale={locale}
                    stationName={searchForm.dep}
                    stations={stations}
                    disabled={stationsLoading || stations.length === 0 || !searchUnlocked}
                    noMatchesLabel={t("train.noMatchingStations")}
                    onStationChange={(value) => setSearchForm((cur) => ({ ...cur, dep: value }))}
                  />
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
                  <StationAutocompleteField
                    label={t("train.arrivalStation")}
                    locale={locale}
                    stationName={searchForm.arr}
                    stations={stations}
                    disabled={stationsLoading || stations.length === 0 || !searchUnlocked}
                    noMatchesLabel={t("train.noMatchingStations")}
                    onStationChange={(value) => setSearchForm((cur) => ({ ...cur, arr: value }))}
                  />
                </div>

                <div className="mt-3 grid gap-3 md:grid-cols-2">
                  <label className="min-w-0 text-sm text-slate-700">
                    <span className={SEARCH_SECTION_LABEL_CLASS}>{t("train.date")}</span>
                    <input
                      aria-label={t("train.date")}
                      type="date"
                      value={searchForm.date}
                      onChange={(event) => setSearchForm((cur) => ({ ...cur, date: event.target.value }))}
                      className={FIELD_BASE_CLASS}
                      required
                      disabled={!searchUnlocked}
                    />
                  </label>
                  <div className="min-w-0 grid grid-cols-2 gap-2">
                    <p className={`col-span-2 ${SEARCH_SECTION_LABEL_CLASS}`}>
                      {t("train.timeShort")}
                    </p>
                    <label className="min-w-0 text-sm text-slate-700">
                      <span className="sr-only">{t("train.timeStart")}</span>
                      <input
                        aria-label={t("train.timeStart")}
                        type="time"
                        value={searchForm.start}
                        onChange={(event) => setSearchForm((cur) => ({ ...cur, start: event.target.value }))}
                        className={FIELD_BASE_CLASS}
                        required
                        disabled={!searchUnlocked}
                      />
                    </label>
                    <label className="min-w-0 text-sm text-slate-700">
                      <span className="sr-only">{t("train.timeEnd")}</span>
                      <input
                        aria-label={t("train.timeEnd")}
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

                {credentialProviderOptions.some((provider) => !isProviderVerified(provider)) ? (
                  <p className="mt-3 text-xs text-amber-700">{t("train.providersDisabled")}</p>
                ) : null}

                <div data-testid="provider-selector-mobile" className="mt-4 md:hidden">
                  <p className={SEARCH_SECTION_LABEL_CLASS}>{t("train.provider")}</p>
                  <div className="mt-2 grid grid-cols-2 gap-2">
                    {searchProviderOptions.map((provider) => {
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

              </div>

              <div className="rounded-2xl border border-blossom-100 bg-blossom-50/40 p-4">
                <p className="text-xs font-medium uppercase tracking-[0.14em] text-blossom-500">
                  {t("train.passengerSeatClass")}
                </p>
                <div className="mt-4 space-y-4">
                  <div className="grid grid-cols-1 gap-4 sm:grid-cols-2">
                    <div className="rounded-xl border border-blossom-100 bg-white/80 px-3 py-3">
                      <p className="text-sm text-slate-700">{t("train.adults")}</p>
                      <div className="mt-2 flex items-center justify-between gap-3 md:hidden">
                        <button
                          type="button"
                          aria-label={t("train.decrementAdults")}
                          disabled={createForm.adults <= 0 || (createForm.adults === 1 && createForm.children === 0)}
                          onClick={() => setAdults(createForm.adults - 1)}
                          className="inline-flex h-10 w-10 items-center justify-center rounded-xl border border-blossom-300 bg-white text-lg font-semibold leading-none text-slate-700 shadow-sm transition hover:bg-blossom-50 focus:outline-none focus:ring-2 focus:ring-blossom-100 disabled:cursor-not-allowed disabled:opacity-50"
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
                          className="inline-flex h-10 w-10 items-center justify-center rounded-xl border border-blossom-300 bg-white text-lg font-semibold leading-none text-slate-700 shadow-sm transition hover:bg-blossom-50 focus:outline-none focus:ring-2 focus:ring-blossom-100 disabled:cursor-not-allowed disabled:opacity-50"
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
                          className="inline-flex h-10 w-10 items-center justify-center rounded-xl border border-blossom-300 bg-white text-lg font-semibold leading-none text-slate-700 shadow-sm transition hover:bg-blossom-50 focus:outline-none focus:ring-2 focus:ring-blossom-100 disabled:cursor-not-allowed disabled:opacity-50"
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
                          className="inline-flex h-10 w-10 items-center justify-center rounded-xl border border-blossom-300 bg-white text-lg font-semibold leading-none text-slate-700 shadow-sm transition hover:bg-blossom-50 focus:outline-none focus:ring-2 focus:ring-blossom-100 disabled:cursor-not-allowed disabled:opacity-50"
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

                  <div>
                    <label htmlFor="train-seat-class" className="sr-only">
                      {t("train.seatClass")}
                    </label>
                    <select
                      id="train-seat-class"
                      aria-label={t("train.seatClass")}
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

      {searchUnlocked && hasSearched ? (
        <div className="space-y-4">
          {searchSummaryCard}
          {showRanking ? (
              <div ref={schedulePanelRef} className="rounded-2xl border border-blossom-100 bg-white p-6 shadow-petal">
                <h2 className="text-lg font-semibold text-slate-800">
                  {t("train.selectSchedules", { date: selectedDateLabel })}
                </h2>
                <div data-testid="schedule-selector-mobile" className="mt-4 space-y-2 md:hidden">
                  {schedules.map((schedule) => {
                    const checked = selectedScheduleIds.includes(schedule.schedule_id);
                    const selectedRank = selectedScheduleIds.indexOf(schedule.schedule_id) + 1;
                    const showSelectedRank = checked && selectedScheduleIds.length > 1;
                    return (
                      <button
                        key={schedule.schedule_id}
                        type="button"
                        aria-pressed={checked}
                        aria-label={`${scheduleTrainLabel(schedule)} ${formatTimeKst(schedule.departure_at, locale)} ${formatTimeKst(schedule.arrival_at, locale)}`}
                        onClick={() => toggleSelectedSchedule(schedule.schedule_id)}
                        className={`w-full overflow-hidden rounded-2xl border text-left transition focus:outline-none focus:ring-2 focus:ring-blossom-100 ${
                          checked
                            ? "border-blossom-300 bg-blossom-50 shadow-sm"
                            : "border-blossom-100 bg-white hover:bg-blossom-50/60"
                        }`}
                      >
                        <div className="border-b border-dashed border-blossom-200 bg-gradient-to-r from-blossom-50 via-white to-sky-50 px-3 py-2.5">
                          <div className="flex items-center justify-between gap-3">
                            <p className="truncate text-sm font-semibold text-slate-800">{scheduleTrainLabel(schedule)}</p>
                            <span
                              aria-hidden="true"
                              className={`inline-flex h-7 w-7 items-center justify-center rounded-full border text-xs font-semibold ${
                                checked
                                  ? "border-blossom-500 bg-blossom-500 text-white"
                                  : "border-slate-300 bg-white text-slate-400"
                              }`}
                            >
                              {checked ? (showSelectedRank ? selectedRank : "✓") : "+"}
                            </span>
                          </div>
                        </div>
                        <div className="px-3 py-3">
                          <div className="grid grid-cols-[minmax(0,1fr)_auto_minmax(0,1fr)] items-center gap-2">
                            <div className="min-w-0 text-center">
                              <p className="truncate text-sm font-semibold text-slate-800">
                                {formatStationLabel(schedule.dep, locale, { compact: true })}
                              </p>
                              <p className="mt-0.5 text-xs tabular-nums text-slate-600">{formatTimeKst(schedule.departure_at, locale)}</p>
                            </div>
                            <span className="inline-flex items-center gap-1 text-slate-400">
                              <span className="h-px w-3 bg-blossom-200" />
                              <svg
                                aria-hidden="true"
                                viewBox="0 0 20 20"
                                className="h-4 w-4"
                                fill="none"
                                stroke="currentColor"
                                strokeWidth="1.8"
                                strokeLinecap="round"
                                strokeLinejoin="round"
                              >
                                <path d="M3 10h12" />
                                <path d="m12 6 4 4-4 4" />
                              </svg>
                              <span className="h-px w-3 bg-blossom-200" />
                            </span>
                            <div className="min-w-0 text-center">
                              <p className="truncate text-sm font-semibold text-slate-800">
                                {formatStationLabel(schedule.arr, locale, { compact: true })}
                              </p>
                              <p className="mt-0.5 text-xs tabular-nums text-slate-600">{formatTimeKst(schedule.arrival_at, locale)}</p>
                            </div>
                          </div>
                          <div className="mt-2 flex items-center justify-between gap-2">
                            <p className="text-xs text-slate-500">{formatTransitDuration(schedule.departure_at, schedule.arrival_at)}</p>
                            <div className="flex items-center gap-2">
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
                          </div>
                        </div>
                      </button>
                    );
                  })}
                </div>
                <div data-testid="schedule-selector-desktop" className="mt-4 hidden md:block">
                  <div className="grid gap-3 md:grid-cols-2">
                  {schedules.map((schedule) => {
                    const checked = selectedScheduleIds.includes(schedule.schedule_id);
                    const selectedRank = selectedScheduleIds.indexOf(schedule.schedule_id) + 1;
                    const showSelectedRank = checked && selectedScheduleIds.length > 1;
                    return (
                      <button
                        key={schedule.schedule_id}
                        type="button"
                        aria-pressed={checked}
                        aria-label={scheduleTrainLabel(schedule)}
                        onClick={() => toggleSelectedSchedule(schedule.schedule_id)}
                        className={`w-full overflow-hidden rounded-2xl border text-left transition focus:outline-none focus:ring-2 focus:ring-blossom-100 ${
                          checked
                            ? "border-blossom-300 bg-blossom-50 shadow-sm"
                            : "border-blossom-100 bg-white hover:bg-blossom-50/60"
                        }`}
                      >
                        <div className="border-b border-dashed border-blossom-200 bg-gradient-to-r from-blossom-50 via-white to-sky-50 px-3 py-2.5">
                          <div className="flex items-center justify-between gap-3">
                            <p className="truncate text-sm font-semibold text-slate-800">{scheduleTrainLabel(schedule)}</p>
                            <span
                              aria-hidden="true"
                              className={`inline-flex h-7 w-7 items-center justify-center rounded-full border text-xs font-semibold ${
                                checked
                                  ? "border-blossom-500 bg-blossom-500 text-white"
                                  : "border-slate-300 bg-white text-slate-400"
                              }`}
                            >
                              {checked ? (showSelectedRank ? selectedRank : "✓") : "+"}
                            </span>
                          </div>
                        </div>
                        <div className="px-3 py-3">
                          <div className="grid grid-cols-[minmax(0,1fr)_auto_minmax(0,1fr)] items-center gap-2">
                            <div className="min-w-0 text-center">
                              <p className="truncate text-sm font-semibold text-slate-800">
                                {formatStationLabel(schedule.dep, locale, { compact: true })}
                              </p>
                              <p className="mt-0.5 text-xs tabular-nums text-slate-600">{formatTimeKst(schedule.departure_at, locale)}</p>
                            </div>
                            <span className="inline-flex items-center gap-1 text-slate-400">
                              <span className="h-px w-3 bg-blossom-200" />
                              <svg
                                aria-hidden="true"
                                viewBox="0 0 20 20"
                                className="h-4 w-4"
                                fill="none"
                                stroke="currentColor"
                                strokeWidth="1.8"
                                strokeLinecap="round"
                                strokeLinejoin="round"
                              >
                                <path d="M3 10h12" />
                                <path d="m12 6 4 4-4 4" />
                              </svg>
                              <span className="h-px w-3 bg-blossom-200" />
                            </span>
                            <div className="min-w-0 text-center">
                              <p className="truncate text-sm font-semibold text-slate-800">
                                {formatStationLabel(schedule.arr, locale, { compact: true })}
                              </p>
                              <p className="mt-0.5 text-xs tabular-nums text-slate-600">{formatTimeKst(schedule.arrival_at, locale)}</p>
                            </div>
                          </div>
                          <div className="mt-2 flex items-center justify-between gap-2">
                            <p className="text-xs text-slate-500">{formatTransitDuration(schedule.departure_at, schedule.arrival_at)}</p>
                            <div className="flex items-center gap-2">
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
                          </div>
                        </div>
                      </button>
                    );
                  })}
                  </div>
                </div>

                <div className="mt-5 flex flex-wrap items-center gap-2">
                  <div className="ml-auto flex w-full items-center gap-2 sm:w-auto">
                    <button
                      type="button"
                      onClick={onExpandSearchMobile}
                      className={`${SECONDARY_BUTTON_CLASS} h-11 flex-1 px-4 sm:min-w-[148px] sm:flex-none`}
                    >
                      {t("train.editSearch")}
                    </button>
                    <button
                      type="button"
                      onClick={() => void openCreateTaskReview()}
                      disabled={createDisabled}
                      className={`${PRIMARY_BUTTON_CLASS} h-11 flex-1 px-4 sm:min-w-[148px] sm:flex-none`}
                    >
                      {creatingTask ? t("train.creatingTask") : t("train.continueTask")}
                    </button>
                  </div>
                </div>
              </div>
            ) : (
              <div className="rounded-2xl border border-blossom-100 bg-white p-6 text-sm text-slate-500 shadow-petal">
                <p className="mt-4">{t("train.noSchedulesYet")}</p>
                <div className="mt-4 flex justify-end">
                  <button
                    type="button"
                    onClick={onExpandSearchMobile}
                    className={SECONDARY_BUTTON_CLASS}
                  >
                    {t("train.editSearch")}
                  </button>
                </div>
              </div>
            )}
        </div>
      ) : null}

      {TRAIN_DUMMY_TASKS_ENABLED ? (
        <div className="rounded-2xl border border-dashed border-blossom-200 bg-white p-4 shadow-petal">
          <div className="flex flex-wrap items-center justify-between gap-3">
            <p className="text-sm text-slate-600">
              <span className="font-medium text-slate-700">{t("train.devTools.heading")}</span> {t("train.devTools.body")}
            </p>
            <div className="flex flex-wrap items-center gap-2">
              <button type="button" onClick={seedDummyTaskCards} className={SMALL_BUTTON_CLASS}>
                {t("train.devTools.loadDummy")}
              </button>
              <button
                type="button"
                onClick={() => {
                  void clearDummyTaskCards();
                }}
                disabled={!dummyTaskCardsMode}
                className={!dummyTaskCardsMode ? SMALL_DISABLED_BUTTON_CLASS : SMALL_BUTTON_CLASS}
              >
                {t("train.devTools.restoreLive")}
              </button>
            </div>
          </div>
        </div>
      ) : null}

      <div className="grid gap-4 lg:grid-cols-2">
        <div className="rounded-2xl border border-blossom-100 bg-white p-6 shadow-petal">
          <h2 className="text-lg font-semibold text-slate-800">{t("train.activeTasks")}</h2>
          <ul className="mt-4 space-y-3 text-sm">
            {sortedActiveTasks.length === 0 ? <li className="text-slate-500">{t("train.empty.activeTasks")}</li> : null}
            <AnimatePresence initial={false}>
            {sortedActiveTasks.map((task) => {
              const info = taskInfoFromSpec(task, locale);
              const showRetryNow = task.state === "QUEUED" || task.state === "POLLING";
              const canControlRuntimeTask = ACTIVE_TASK_STATES_FOR_LIST.has(task.state);
              const ticketBadge = getTaskTicketBadge(task);
              const displayState = taskDisplayState(task);
              const taskError = taskErrorPresentation(task);
              return (
                <motion.li
                  key={task.id}
                  layout
                  initial={TASK_CARD_INITIAL_ANIMATION}
                  animate={TASK_CARD_ANIMATE_ANIMATION}
                  exit={TASK_CARD_EXIT_ANIMATION}
                  transition={TASK_CARD_ENTER_EXIT_TRANSITION}
                  className="overflow-hidden rounded-2xl border border-blossom-200 bg-white shadow-sm"
                >
                  <div className="border-b border-dashed border-blossom-200 bg-gradient-to-r from-blossom-50 via-white to-sky-50 px-4 py-3 sm:px-5">
                    <div className="flex items-start justify-between gap-3">
                      <div className="min-w-0">
                        <p className="text-[11px] font-medium uppercase tracking-[0.14em] text-blossom-500">
                          {t("train.ticketStyle.activeTask")}
                        </p>
                        <div className="mt-1 flex flex-wrap items-center gap-2">
                          <p className="font-medium text-slate-700">{displayState}</p>
                          {ticketBadge ? (
                            <span
                              className={`inline-flex rounded-full px-2 py-0.5 text-[11px] font-medium ${ticketBadge.className}`}
                            >
                              {renderMaybeKey(ticketBadge.label)}
                            </span>
                          ) : null}
                        </div>
                      </div>
                      <Link href={`/modules/train/tasks/${task.id}`} className={SMALL_BUTTON_CLASS}>
                        {t("train.action.detail")}
                      </Link>
                    </div>

                    <div className="mt-3 rounded-2xl border border-blossom-100 bg-white/90 p-3 sm:p-4">
                      <div className="flex items-center justify-between gap-3 text-xs font-medium text-slate-500">
                        <p className="truncate">{info.travelDateLabel}</p>
                        {info.passengerLabel !== "-" ? (
                          <p className="truncate text-right">{`${t("train.label.passengers")} ${info.passengerLabel}`}</p>
                        ) : (
                          <p aria-hidden="true" className="text-transparent">
                            {"\u00A0"}
                          </p>
                        )}
                      </div>
                      <div className="mt-2 grid grid-cols-[minmax(0,1fr)_auto_minmax(0,1fr)] items-center gap-x-2 gap-y-2 sm:gap-x-3">
                        <p className="truncate text-center text-sm font-semibold text-slate-800 sm:text-base">
                          {formatStationLabel(info.dep, locale)}
                        </p>
                        <span className="inline-flex items-center gap-1 text-slate-400">
                          <span className="h-px w-4 bg-blossom-200 sm:w-6" />
                          <svg
                            aria-hidden="true"
                            viewBox="0 0 20 20"
                            className="h-4 w-4 text-slate-400"
                            fill="none"
                            stroke="currentColor"
                            strokeWidth="1.8"
                            strokeLinecap="round"
                            strokeLinejoin="round"
                          >
                            <path d="M3 10h12" />
                            <path d="m11 6 4 4-4 4" />
                          </svg>
                          <span className="h-px w-4 bg-blossom-200 sm:w-6" />
                        </span>
                        <p className="truncate text-center text-sm font-semibold text-slate-800 sm:text-base">
                          {formatStationLabel(info.arr, locale)}
                        </p>
                        <p className="truncate text-center text-sm font-medium text-slate-700">{info.primaryDepartureLabel}</p>
                        <span className="invisible inline-flex items-center gap-1">
                          <span className="h-px w-4 sm:w-6" />
                          <svg
                            aria-hidden="true"
                            viewBox="0 0 20 20"
                            className="h-4 w-4"
                            fill="none"
                            stroke="currentColor"
                            strokeWidth="1.8"
                            strokeLinecap="round"
                            strokeLinejoin="round"
                          >
                            <path d="M3 10h12" />
                            <path d="m11 6 4 4-4 4" />
                          </svg>
                          <span className="h-px w-4 sm:w-6" />
                        </span>
                        <p className="truncate text-center text-sm font-medium text-slate-700">{info.primaryArrivalLabel}</p>
                      </div>
                      {info.scheduleTimeOptions.length > 1 ? (
                        <ul className="mt-2 space-y-1 rounded-lg bg-slate-50 px-2 py-1.5 text-xs text-slate-600">
                          {info.scheduleTimeOptions.map((option, index) => (
                            <li key={`${task.id}-schedule-${index}`} className="flex items-center justify-between gap-2">
                              <span className="font-medium text-slate-500">{`#${index + 1}`}</span>
                              <span>
                                {option.provider ? `${option.provider} ` : ""}
                                {option.departureLabel} - {option.arrivalLabel}
                              </span>
                            </li>
                          ))}
                        </ul>
                      ) : null}
                    </div>
                  </div>

                  {taskError ? (
                    <div className="px-4 py-3">
                      <p className={`rounded-xl border px-3 py-2 text-xs ${taskError.className}`}>
                        <span className="font-medium">{t("train.label.message")} </span>
                        <span>{taskError.message}</span>
                      </p>
                    </div>
                  ) : null}

                  <div className="relative flex flex-wrap gap-2 border-t border-dashed border-blossom-200 px-4 py-3">
                    <span
                      aria-hidden="true"
                      className="pointer-events-none absolute -left-3 top-0 h-6 w-6 -translate-y-1/2 rounded-full border border-blossom-200 bg-white"
                    />
                    <span
                      aria-hidden="true"
                      className="pointer-events-none absolute -right-3 top-0 h-6 w-6 -translate-y-1/2 rounded-full border border-blossom-200 bg-white"
                    />
                    {showRetryNow ? (
                      <button
                        type="button"
                        onClick={() => sendTaskAction(task.id, "retry")}
                        disabled={!task.retry_now_allowed}
                        title={
                          task.retry_now_allowed
                            ? t("train.action.retry")
                            : retryNowDisabledTitleLocalized(task, { locale, t })
                        }
                        className={`${task.retry_now_allowed ? SMALL_BUTTON_CLASS : SMALL_DISABLED_BUTTON_CLASS} ${TASK_ACTION_BUTTON_SIZE_CLASS}`}
                      >
                        {t("train.action.retry")}
                      </button>
                    ) : null}
                    {canControlRuntimeTask && task.state !== "PAUSED" ? (
                      <button
                        type="button"
                        onClick={() => sendTaskAction(task.id, "pause")}
                        className={`${SMALL_BUTTON_CLASS} ${TASK_ACTION_BUTTON_SIZE_CLASS}`}
                      >
                        {t("train.action.pause")}
                      </button>
                    ) : null}
                    {canControlRuntimeTask && task.state === "PAUSED" ? (
                      <button
                        type="button"
                        onClick={() => sendTaskAction(task.id, "resume")}
                        className={`${SMALL_BUTTON_CLASS} ${TASK_ACTION_BUTTON_SIZE_CLASS}`}
                      >
                        {t("train.action.resume")}
                      </button>
                    ) : null}
                    {isAwaitingPaymentTask(task) ? (
                      <button
                        type="button"
                        onClick={() => void payAwaitingPaymentTask(task.id)}
                        disabled={payingTaskId === task.id}
                        className={`${payingTaskId === task.id ? SMALL_DISABLED_BUTTON_CLASS : SMALL_SUCCESS_BUTTON_CLASS} ${TASK_ACTION_BUTTON_SIZE_CLASS}`}
                      >
                        {payingTaskId === task.id ? t("train.action.paying") : t("train.action.pay")}
                      </button>
                    ) : null}
                    {isAwaitingPaymentTask(task) ? (
                      <button
                        type="button"
                        onClick={() => void cancelTaskTicket(task.id)}
                        disabled={cancellingTaskId === task.id || payingTaskId === task.id}
                        className={`${SMALL_DANGER_BUTTON_CLASS} ${TASK_ACTION_BUTTON_SIZE_CLASS}`}
                      >
                        {cancellingTaskId === task.id ? t("train.action.cancelling") : t("train.action.cancelReservation")}
                      </button>
                    ) : null}
                    {canControlRuntimeTask ? (
                      <button
                        type="button"
                        onClick={() => sendTaskAction(task.id, "cancel")}
                        className={`${SMALL_DANGER_BUTTON_CLASS} ${TASK_ACTION_BUTTON_SIZE_CLASS}`}
                      >
                        {t("train.action.cancel")}
                      </button>
                    ) : null}
                  </div>
                </motion.li>
              );
            })}
            </AnimatePresence>
          </ul>
        </div>

        <div className="rounded-2xl border border-blossom-100 bg-white p-6 shadow-petal">
          <h2 className="text-lg font-semibold text-slate-800">{t("train.completedTasks")}</h2>
          <ul className="mt-4 space-y-3 text-sm">
            {sortedCompletedTasks.length === 0 ? <li className="text-slate-500">{t("train.empty.completedTasks")}</li> : null}
            <AnimatePresence initial={false}>
            {sortedCompletedTasks.map((task) => {
              const info = taskInfoFromSpec(task, locale);
              const ticketBadge = getTaskTicketBadge(task);
              const displayState = taskDisplayState(task);
              const ticketTrainLabel = taskTicketTrainLabel(task);
              const ticketSeatLabel = taskTicketSeatLabel(task, locale);
              return (
                <motion.li
                  key={task.id}
                  layout
                  initial={TASK_CARD_INITIAL_ANIMATION}
                  animate={TASK_CARD_ANIMATE_ANIMATION}
                  exit={TASK_CARD_EXIT_ANIMATION}
                  transition={TASK_CARD_ENTER_EXIT_TRANSITION}
                  className="overflow-hidden rounded-2xl border border-blossom-200 bg-white shadow-sm"
                >
                  <div className="border-b border-dashed border-blossom-200 bg-gradient-to-r from-slate-50 via-white to-blossom-50 px-4 py-3 sm:px-5">
                    <div className="flex items-start justify-between gap-3">
                      <div className="min-w-0">
                        <p className="text-[11px] font-medium uppercase tracking-[0.14em] text-slate-500">
                          {t("train.ticketStyle.activeTask")}
                        </p>
                        <div className="mt-1 flex flex-wrap items-center gap-2">
                          <p className="font-medium text-slate-700">{displayState}</p>
                          {ticketBadge ? (
                            <span
                              className={`inline-flex rounded-full px-2 py-0.5 text-[11px] font-medium ${ticketBadge.className}`}
                            >
                              {renderMaybeKey(ticketBadge.label)}
                            </span>
                          ) : null}
                        </div>
                        <p className="mt-1 text-xs text-slate-500">
                          {t("train.label.completed")} {task.completed_at ? formatDateTimeKst(task.completed_at, locale) : "-"}
                        </p>
                      </div>
                      <Link href={`/modules/train/tasks/${task.id}`} className={SMALL_BUTTON_CLASS}>
                        {t("train.action.detail")}
                      </Link>
                    </div>

                    <div className="mt-3 rounded-2xl border border-blossom-100 bg-white/90 p-3 sm:p-4">
                      <div className="flex items-center justify-between gap-3 text-xs font-medium text-slate-500">
                        <p className="truncate">{info.travelDateLabel}</p>
                        {info.passengerLabel !== "-" ? (
                          <p className="truncate text-right">{`${t("train.label.passengers")} ${info.passengerLabel}`}</p>
                        ) : (
                          <p aria-hidden="true" className="text-transparent">
                            {"\u00A0"}
                          </p>
                        )}
                      </div>
                      <div className="mt-2 grid grid-cols-[minmax(0,1fr)_auto_minmax(0,1fr)] items-center gap-x-2 gap-y-2 sm:gap-x-3">
                        <p className="truncate text-center text-sm font-semibold text-slate-800 sm:text-base">
                          {formatStationLabel(info.dep, locale)}
                        </p>
                        <span className="inline-flex items-center gap-1 text-slate-400">
                          <span className="h-px w-4 bg-blossom-200 sm:w-6" />
                          <svg
                            aria-hidden="true"
                            viewBox="0 0 20 20"
                            className="h-4 w-4 text-slate-400"
                            fill="none"
                            stroke="currentColor"
                            strokeWidth="1.8"
                            strokeLinecap="round"
                            strokeLinejoin="round"
                          >
                            <path d="M3 10h12" />
                            <path d="m11 6 4 4-4 4" />
                          </svg>
                          <span className="h-px w-4 bg-blossom-200 sm:w-6" />
                        </span>
                        <p className="truncate text-center text-sm font-semibold text-slate-800 sm:text-base">
                          {formatStationLabel(info.arr, locale)}
                        </p>
                        <p className="truncate text-center text-sm font-medium text-slate-700">{info.primaryDepartureLabel}</p>
                        <span className="invisible inline-flex items-center gap-1">
                          <span className="h-px w-4 sm:w-6" />
                          <svg
                            aria-hidden="true"
                            viewBox="0 0 20 20"
                            className="h-4 w-4"
                            fill="none"
                            stroke="currentColor"
                            strokeWidth="1.8"
                            strokeLinecap="round"
                            strokeLinejoin="round"
                          >
                            <path d="M3 10h12" />
                            <path d="m11 6 4 4-4 4" />
                          </svg>
                          <span className="h-px w-4 sm:w-6" />
                        </span>
                        <p className="truncate text-center text-sm font-medium text-slate-700">{info.primaryArrivalLabel}</p>
                      </div>
                      {info.scheduleTimeOptions.length > 1 ? (
                        <ul className="mt-2 space-y-1 rounded-lg bg-slate-50 px-2 py-1.5 text-xs text-slate-600">
                          {info.scheduleTimeOptions.map((option, index) => (
                            <li key={`${task.id}-schedule-${index}`} className="flex items-center justify-between gap-2">
                              <span className="font-medium text-slate-500">{`#${index + 1}`}</span>
                              <span>
                                {option.provider ? `${option.provider} ` : ""}
                                {option.departureLabel} - {option.arrivalLabel}
                              </span>
                            </li>
                          ))}
                        </ul>
                      ) : null}

                      <div className="mt-3 grid gap-2 sm:grid-cols-2">
                        {ticketTrainLabel ? (
                          <div className="rounded-xl border border-blossom-100 bg-white px-3 py-2">
                            <p className="text-[11px] font-medium uppercase tracking-[0.08em] text-slate-500">
                              {t("train.label.train")}
                            </p>
                            <p className="mt-1 text-sm font-medium text-slate-700">{ticketTrainLabel}</p>
                          </div>
                        ) : null}
                        {ticketSeatLabel ? (
                          <div className="rounded-xl border border-blossom-100 bg-white px-3 py-2">
                            <p className="text-[11px] font-medium uppercase tracking-[0.08em] text-slate-500">
                              {t("train.label.seats")}
                            </p>
                            <p className="mt-1 text-sm font-medium text-slate-700">{ticketSeatLabel}</p>
                          </div>
                        ) : null}
                      </div>
                    </div>
                  </div>

                  <div className="relative flex flex-wrap gap-2 border-t border-dashed border-blossom-200 px-4 py-3">
                    <span
                      aria-hidden="true"
                      className="pointer-events-none absolute -left-3 top-0 h-6 w-6 -translate-y-1/2 rounded-full border border-blossom-200 bg-white"
                    />
                    <span
                      aria-hidden="true"
                      className="pointer-events-none absolute -right-3 top-0 h-6 w-6 -translate-y-1/2 rounded-full border border-blossom-200 bg-white"
                    />
                    {isRetryDeleteTerminalTask(task) ? (
                      <button
                        type="button"
                        onClick={() => sendTaskAction(task.id, "retry")}
                        disabled={!task.retry_now_allowed}
                        title={
                          task.retry_now_allowed
                            ? t("train.action.retry")
                            : retryNowDisabledTitleLocalized(task, { locale, t })
                        }
                        className={`${task.retry_now_allowed ? SMALL_BUTTON_CLASS : SMALL_DISABLED_BUTTON_CLASS} ${TASK_ACTION_BUTTON_SIZE_CLASS}`}
                      >
                        {t("train.action.retry")}
                      </button>
                    ) : null}
                    {isAwaitingPaymentTask(task) ? (
                      <button
                        type="button"
                        onClick={() => void payAwaitingPaymentTask(task.id)}
                        disabled={payingTaskId === task.id || !autoPayAvailable}
                        title={autoPayAvailable ? t("train.action.payNow") : t("train.hint.paymentSettingsRequired")}
                        className={
                          `${payingTaskId === task.id || !autoPayAvailable ? SMALL_DISABLED_BUTTON_CLASS : SMALL_SUCCESS_BUTTON_CLASS} ${TASK_ACTION_BUTTON_SIZE_CLASS}`
                        }
                      >
                        {payingTaskId === task.id ? t("train.action.paying") : t("train.action.pay")}
                      </button>
                    ) : null}
                    {isRetryDeleteTerminalTask(task) ? null : isAwaitingPaymentTask(task) ? (
                      <button
                        type="button"
                        onClick={() => void cancelTaskTicket(task.id)}
                        disabled={cancellingTaskId === task.id || payingTaskId === task.id}
                        className={`${SMALL_DANGER_BUTTON_CLASS} ${TASK_ACTION_BUTTON_SIZE_CLASS}`}
                      >
                        {cancellingTaskId === task.id ? t("train.action.cancelling") : t("train.action.cancel")}
                      </button>
                    ) : shouldShowCompletedCancel(task) ? (
                      <button
                        type="button"
                        onClick={() => void cancelTaskTicket(task.id)}
                        disabled={cancellingTaskId === task.id}
                        className={`${SMALL_DANGER_BUTTON_CLASS} ${TASK_ACTION_BUTTON_SIZE_CLASS}`}
                      >
                        {cancellingTaskId === task.id ? t("train.action.cancelling") : t("train.action.cancel")}
                      </button>
                    ) : (
                      <button
                        type="button"
                        onClick={() => sendTaskAction(task.id, "delete")}
                        className={`${SMALL_DANGER_BUTTON_CLASS} ${TASK_ACTION_BUTTON_SIZE_CLASS}`}
                      >
                        {t("common.delete")}
                      </button>
                    )}
                    {isRetryDeleteTerminalTask(task) ? (
                      <button
                        type="button"
                        onClick={() => sendTaskAction(task.id, "delete")}
                        className={`${SMALL_DANGER_BUTTON_CLASS} ${TASK_ACTION_BUTTON_SIZE_CLASS}`}
                      >
                        {t("common.delete")}
                      </button>
                    ) : null}
                  </div>
                </motion.li>
              );
            })}
            </AnimatePresence>
          </ul>
        </div>
      </div>
    </section>
  );
}
