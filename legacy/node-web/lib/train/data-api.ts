import { NEXT_PUBLIC_TRAIN_READS_VIA_DATA_API } from "@/lib/feature-flags";
import { getSupabaseAccessToken, isSupabaseDirectAuthEnabled } from "@/lib/supabase-auth";
import type { TrainTaskSummary, TrainTaskState } from "@/lib/types";

const SUPABASE_URL = (process.env.NEXT_PUBLIC_SUPABASE_URL ?? "").trim().replace(/\/+$/, "");
const SUPABASE_ANON_KEY = (process.env.NEXT_PUBLIC_SUPABASE_ANON_KEY ?? "").trim();
const ACTIVE_STATES = new Set<TrainTaskState>(["QUEUED", "RUNNING", "POLLING", "RESERVING", "PAYING", "PAUSED"]);
const TERMINAL_STATES = new Set<TrainTaskState>(["COMPLETED", "CANCELLED", "EXPIRED", "FAILED"]);
const TASK_SUMMARY_SELECT = [
  "id",
  "module",
  "state",
  "deadline_at",
  "created_at",
  "updated_at",
  "paused_at",
  "cancelled_at",
  "completed_at",
  "failed_at",
  "hidden_at",
  "last_attempt_at",
  "last_attempt_action",
  "last_attempt_ok",
  "last_attempt_error_code",
  "last_attempt_error_message_safe",
  "last_attempt_finished_at",
  "next_run_at",
  "retry_now_allowed",
  "retry_now_reason",
  "retry_now_available_at",
  "spec_json",
  "ticket_status",
  "ticket_paid",
  "ticket_payment_deadline_at",
  "ticket_reservation_id",
  "ticket_train_no",
  "ticket_seat_count",
  "ticket_seats",
  "ticket_seat_classes",
  "list_bucket",
].join(",");

export type TaskListStatus = "active" | "completed";

type TrainTaskSummaryRow = {
  id?: unknown;
  module?: unknown;
  state?: unknown;
  deadline_at?: unknown;
  created_at?: unknown;
  updated_at?: unknown;
  paused_at?: unknown;
  cancelled_at?: unknown;
  completed_at?: unknown;
  failed_at?: unknown;
  hidden_at?: unknown;
  last_attempt_at?: unknown;
  last_attempt_action?: unknown;
  last_attempt_ok?: unknown;
  last_attempt_error_code?: unknown;
  last_attempt_error_message_safe?: unknown;
  last_attempt_finished_at?: unknown;
  next_run_at?: unknown;
  retry_now_allowed?: unknown;
  retry_now_reason?: unknown;
  retry_now_available_at?: unknown;
  spec_json?: unknown;
  ticket_status?: unknown;
  ticket_paid?: unknown;
  ticket_payment_deadline_at?: unknown;
  ticket_reservation_id?: unknown;
  ticket_train_no?: unknown;
  ticket_seat_count?: unknown;
  ticket_seats?: unknown;
  ticket_seat_classes?: unknown;
  list_bucket?: unknown;
};

function hasSupabaseTaskReadConfig(): boolean {
  return NEXT_PUBLIC_TRAIN_READS_VIA_DATA_API && isSupabaseDirectAuthEnabled() && SUPABASE_URL.length > 0 && SUPABASE_ANON_KEY.length > 0;
}

function normalizeState(value: unknown): TrainTaskState | null {
  if (typeof value !== "string") return null;
  const normalized = value.trim().toUpperCase() as TrainTaskState;
  if (ACTIVE_STATES.has(normalized) || TERMINAL_STATES.has(normalized)) return normalized;
  return null;
}

function normalizeString(value: unknown): string | null {
  if (typeof value !== "string") return null;
  const normalized = value.trim();
  return normalized.length > 0 ? normalized : null;
}

function normalizeBoolean(value: unknown): boolean | null {
  return typeof value === "boolean" ? value : null;
}

function normalizeNumber(value: unknown): number | null {
  if (typeof value !== "number" || !Number.isFinite(value)) return null;
  return value;
}

function normalizeNullableStringArray(value: unknown): string[] | null {
  if (!Array.isArray(value)) return null;
  const rows = value
    .filter((entry): entry is string => typeof entry === "string")
    .map((entry) => entry.trim())
    .filter((entry) => entry.length > 0);
  return rows.length > 0 ? Array.from(new Set(rows)) : null;
}

function normalizeSpecJson(value: unknown): Record<string, unknown> {
  if (typeof value !== "object" || value === null || Array.isArray(value)) return {};
  return value as Record<string, unknown>;
}

function normalizeTaskSummaryRow(row: TrainTaskSummaryRow): TrainTaskSummary | null {
  const id = normalizeString(row.id);
  const moduleName = normalizeString(row.module);
  const state = normalizeState(row.state);
  const deadlineAt = normalizeString(row.deadline_at);
  const createdAt = normalizeString(row.created_at);
  const updatedAt = normalizeString(row.updated_at);
  if (!id || !moduleName || !state || !deadlineAt || !createdAt || !updatedAt) {
    return null;
  }

  return {
    id,
    module: moduleName,
    state,
    deadline_at: deadlineAt,
    created_at: createdAt,
    updated_at: updatedAt,
    paused_at: normalizeString(row.paused_at),
    cancelled_at: normalizeString(row.cancelled_at),
    completed_at: normalizeString(row.completed_at),
    failed_at: normalizeString(row.failed_at),
    hidden_at: normalizeString(row.hidden_at),
    last_attempt_at: normalizeString(row.last_attempt_at),
    last_attempt_action: normalizeString(row.last_attempt_action),
    last_attempt_ok: normalizeBoolean(row.last_attempt_ok),
    last_attempt_error_code: normalizeString(row.last_attempt_error_code),
    last_attempt_error_message_safe: normalizeString(row.last_attempt_error_message_safe),
    last_attempt_finished_at: normalizeString(row.last_attempt_finished_at),
    next_run_at: normalizeString(row.next_run_at),
    retry_now_allowed: row.retry_now_allowed === true,
    retry_now_reason: normalizeString(row.retry_now_reason),
    retry_now_available_at: normalizeString(row.retry_now_available_at),
    spec_json: normalizeSpecJson(row.spec_json),
    ticket_status: normalizeString(row.ticket_status),
    ticket_paid: normalizeBoolean(row.ticket_paid),
    ticket_payment_deadline_at: normalizeString(row.ticket_payment_deadline_at),
    ticket_reservation_id: normalizeString(row.ticket_reservation_id),
    ticket_train_no: normalizeString(row.ticket_train_no),
    ticket_seat_count: normalizeNumber(row.ticket_seat_count),
    ticket_seats: normalizeNullableStringArray(row.ticket_seats),
    ticket_seat_classes: normalizeNullableStringArray(row.ticket_seat_classes),
  };
}

export async function fetchTrainTaskListViaDataApi(params: {
  status: TaskListStatus;
  limit: number;
}): Promise<TrainTaskSummary[] | null> {
  if (typeof window === "undefined" || !hasSupabaseTaskReadConfig()) {
    return null;
  }

  const accessToken = await getSupabaseAccessToken();
  if (!accessToken) return null;

  const limit = Math.max(1, Math.min(500, Math.trunc(params.limit)));
  const query = new URLSearchParams();
  query.set("select", TASK_SUMMARY_SELECT);
  query.set("list_bucket", `eq.${params.status}`);
  query.set("order", "created_at.desc,id.desc");
  query.set("limit", String(limit));

  try {
    const response = await fetch(`${SUPABASE_URL}/rest/v1/v_train_task_list_compact?${query.toString()}`, {
      method: "GET",
      headers: {
        apikey: SUPABASE_ANON_KEY,
        Authorization: `Bearer ${accessToken}`,
      },
      cache: "no-store",
      credentials: "omit",
    });
    if (!response.ok) {
      return null;
    }
    const payload = (await response.json().catch(() => null)) as TrainTaskSummaryRow[] | null;
    if (!Array.isArray(payload)) return null;

    const rows: TrainTaskSummary[] = [];
    for (const row of payload) {
      const normalized = normalizeTaskSummaryRow(row);
      if (normalized) {
        rows.push(normalized);
      }
    }
    if (payload.length > 0 && rows.length === 0) {
      return null;
    }
    return rows;
  } catch {
    return null;
  }
}
