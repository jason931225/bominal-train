import { NEXT_PUBLIC_TRAIN_DETAIL_VIA_GRAPHQL } from "@/lib/feature-flags";
import { getSupabaseAccessToken, isSupabaseDirectAuthEnabled } from "@/lib/supabase-auth";
import type { TrainArtifact, TrainTaskAttempt, TrainTaskSummary, TrainTaskState } from "@/lib/types";

const SUPABASE_URL = (process.env.NEXT_PUBLIC_SUPABASE_URL ?? "").trim().replace(/\/+$/, "");
const SUPABASE_ANON_KEY = (process.env.NEXT_PUBLIC_SUPABASE_ANON_KEY ?? "").trim();

const DETAIL_QUERY = `
query TrainTaskDetail($taskId: UUID!) {
  tasksCollection(
    first: 1
    filter: {
      id: { eq: $taskId }
      module: { eq: "train" }
      hiddenAt: { isNull: true }
    }
  ) {
    edges {
      node {
        id
        module
        state
        deadlineAt
        createdAt
        updatedAt
        pausedAt
        cancelledAt
        completedAt
        failedAt
        hiddenAt
        specJson
        taskAttemptsCollection(first: 500) {
          edges {
            node {
              id
              action
              provider
              ok
              retryable
              errorCode
              errorMessageSafe
              durationMs
              metaJsonSafe
              startedAt
              finishedAt
            }
          }
        }
        artifactsCollection(first: 200, filter: { module: { eq: "train" } }) {
          edges {
            node {
              id
              module
              kind
              dataJsonSafe
              createdAt
            }
          }
        }
      }
    }
  }
}
`;

type GraphqlTaskNode = {
  id?: unknown;
  module?: unknown;
  state?: unknown;
  deadlineAt?: unknown;
  createdAt?: unknown;
  updatedAt?: unknown;
  pausedAt?: unknown;
  cancelledAt?: unknown;
  completedAt?: unknown;
  failedAt?: unknown;
  hiddenAt?: unknown;
  specJson?: unknown;
  taskAttemptsCollection?: { edges?: Array<{ node?: Record<string, unknown> | null } | null> } | null;
  artifactsCollection?: { edges?: Array<{ node?: Record<string, unknown> | null } | null> } | null;
};

type TaskDetailPayload = {
  task: TrainTaskSummary;
  attempts: TrainTaskAttempt[];
  artifacts: TrainArtifact[];
};

const VALID_STATES = new Set<TrainTaskState>([
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

function hasSupabaseGraphqlConfig(): boolean {
  return NEXT_PUBLIC_TRAIN_DETAIL_VIA_GRAPHQL && isSupabaseDirectAuthEnabled() && SUPABASE_URL.length > 0 && SUPABASE_ANON_KEY.length > 0;
}

function readString(value: unknown): string | null {
  if (typeof value !== "string") return null;
  const normalized = value.trim();
  return normalized.length > 0 ? normalized : null;
}

function readBoolean(value: unknown): boolean | null {
  return typeof value === "boolean" ? value : null;
}

function readNumber(value: unknown): number | null {
  if (typeof value !== "number" || !Number.isFinite(value)) return null;
  return value;
}

function readRecord(value: unknown): Record<string, unknown> {
  if (typeof value !== "object" || value === null || Array.isArray(value)) return {};
  return value as Record<string, unknown>;
}

function normalizeTicketSeatClass(value: unknown): "general" | "special" | null {
  if (typeof value !== "string") return null;
  const normalized = value.trim().toLowerCase();
  if (!normalized) return null;
  if (normalized === "1") return "general";
  if (normalized === "2") return "special";
  if (normalized.includes("general") || normalized.includes("일반")) return "general";
  if (normalized.includes("special") || normalized.includes("특실")) return "special";
  return null;
}

function extractTicketSeatClasses(ticketData: Record<string, unknown>): string[] | null {
  const seen = new Set<"general" | "special">();
  if (Array.isArray(ticketData.seat_classes)) {
    for (const row of ticketData.seat_classes) {
      const seatClass = normalizeTicketSeatClass(row);
      if (seatClass) {
        seen.add(seatClass);
      }
    }
  }

  if (Array.isArray(ticketData.tickets)) {
    for (const row of ticketData.tickets) {
      if (typeof row !== "object" || row === null || Array.isArray(row)) continue;
      const rowRecord = row as Record<string, unknown>;
      const seatClassCode = normalizeTicketSeatClass(rowRecord.seat_class_code);
      const seatClassName =
        normalizeTicketSeatClass(rowRecord.seat_class_name) ?? normalizeTicketSeatClass(rowRecord.seat_class);
      if (seatClassCode) {
        seen.add(seatClassCode);
      } else if (seatClassName) {
        seen.add(seatClassName);
      }
    }
  }

  const ordered = (["general", "special"] as const).filter((seatClass) => seen.has(seatClass));
  return ordered.length > 0 ? ordered : null;
}

function mapTaskSummary(node: GraphqlTaskNode): TrainTaskSummary | null {
  const id = readString(node.id);
  const moduleName = readString(node.module);
  const deadlineAt = readString(node.deadlineAt);
  const createdAt = readString(node.createdAt);
  const updatedAt = readString(node.updatedAt);
  const stateRaw = readString(node.state)?.toUpperCase() as TrainTaskState | undefined;
  if (!id || !moduleName || !deadlineAt || !createdAt || !updatedAt || !stateRaw || !VALID_STATES.has(stateRaw)) {
    return null;
  }

  return {
    id,
    module: moduleName,
    state: stateRaw,
    deadline_at: deadlineAt,
    created_at: createdAt,
    updated_at: updatedAt,
    paused_at: readString(node.pausedAt),
    cancelled_at: readString(node.cancelledAt),
    completed_at: readString(node.completedAt),
    failed_at: readString(node.failedAt),
    hidden_at: readString(node.hiddenAt),
    last_attempt_at: null,
    last_attempt_action: null,
    last_attempt_ok: null,
    last_attempt_error_code: null,
    last_attempt_error_message_safe: null,
    last_attempt_finished_at: null,
    next_run_at: null,
    retry_now_allowed: false,
    retry_now_reason: null,
    retry_now_available_at: null,
    spec_json: readRecord(node.specJson),
    ticket_status: null,
    ticket_paid: null,
    ticket_payment_deadline_at: null,
    ticket_reservation_id: null,
    ticket_train_no: null,
    ticket_seat_count: null,
    ticket_seats: null,
    ticket_seat_classes: null,
  };
}

function mapTaskAttempt(node: Record<string, unknown>): TrainTaskAttempt | null {
  const id = readString(node.id);
  const action = readString(node.action);
  const provider = readString(node.provider);
  const ok = readBoolean(node.ok);
  const retryable = readBoolean(node.retryable);
  const durationMs = readNumber(node.durationMs);
  const startedAt = readString(node.startedAt);
  const finishedAt = readString(node.finishedAt);
  if (!id || !action || !provider || ok == null || retryable == null || durationMs == null || !startedAt || !finishedAt) {
    return null;
  }
  return {
    id,
    action,
    provider,
    ok,
    retryable,
    error_code: readString(node.errorCode),
    error_message_safe: readString(node.errorMessageSafe),
    duration_ms: durationMs,
    meta_json_safe: readRecord(node.metaJsonSafe),
    started_at: startedAt,
    finished_at: finishedAt,
  };
}

function mapArtifact(node: Record<string, unknown>): TrainArtifact | null {
  const id = readString(node.id);
  const moduleName = readString(node.module);
  const kind = readString(node.kind);
  const createdAt = readString(node.createdAt);
  if (!id || !moduleName || !kind || !createdAt) return null;
  return {
    id,
    module: moduleName,
    kind,
    data_json_safe: readRecord(node.dataJsonSafe),
    created_at: createdAt,
  };
}

export async function fetchTrainTaskDetailViaGraphql(taskId: string): Promise<TaskDetailPayload | null> {
  if (typeof window === "undefined" || !hasSupabaseGraphqlConfig()) return null;

  const accessToken = await getSupabaseAccessToken();
  if (!accessToken) return null;

  try {
    const response = await fetch(`${SUPABASE_URL}/graphql/v1`, {
      method: "POST",
      headers: {
        "Content-Type": "application/json",
        apikey: SUPABASE_ANON_KEY,
        Authorization: `Bearer ${accessToken}`,
      },
      credentials: "omit",
      cache: "no-store",
      body: JSON.stringify({
        query: DETAIL_QUERY,
        variables: { taskId },
      }),
    });
    if (!response.ok) {
      return null;
    }

    const payload = (await response.json().catch(() => null)) as
      | { data?: { tasksCollection?: { edges?: Array<{ node?: GraphqlTaskNode | null } | null> } | null } | null; errors?: unknown[] }
      | null;
    if (!payload || !payload.data?.tasksCollection?.edges?.length) return null;
    if (Array.isArray(payload.errors) && payload.errors.length > 0) return null;

    const node = payload.data.tasksCollection.edges[0]?.node ?? null;
    if (!node) return null;
    const task = mapTaskSummary(node);
    if (!task) return null;

    const attempts: TrainTaskAttempt[] = [];
    const attemptEdges = node.taskAttemptsCollection?.edges ?? [];
    for (const edge of attemptEdges) {
      const mapped = edge?.node ? mapTaskAttempt(edge.node) : null;
      if (mapped) {
        attempts.push(mapped);
      }
    }
    attempts.sort((left, right) => Date.parse(left.started_at) - Date.parse(right.started_at));

    const artifacts: TrainArtifact[] = [];
    const artifactEdges = node.artifactsCollection?.edges ?? [];
    for (const edge of artifactEdges) {
      const mapped = edge?.node ? mapArtifact(edge.node) : null;
      if (mapped) {
        artifacts.push(mapped);
      }
    }
    artifacts.sort((left, right) => Date.parse(left.created_at) - Date.parse(right.created_at));

    // Populate summary ticket and last-attempt fields from nested rows to preserve
    // task-card semantics when GraphQL is the primary detail source.
    const latestAttempt = attempts.length > 0 ? attempts[attempts.length - 1] : null;
    if (latestAttempt) {
      task.last_attempt_at = latestAttempt.finished_at;
      task.last_attempt_finished_at = latestAttempt.finished_at;
      task.last_attempt_action = latestAttempt.action;
      task.last_attempt_ok = latestAttempt.ok;
      task.last_attempt_error_code = latestAttempt.error_code;
      task.last_attempt_error_message_safe = latestAttempt.error_message_safe;
    }
    const ticketArtifact = artifacts.filter((artifact) => artifact.kind === "ticket").at(-1);
    const ticketData = ticketArtifact?.data_json_safe ?? {};
    task.ticket_status = readString(ticketData.status);
    task.ticket_paid = readBoolean(ticketData.paid);
    task.ticket_payment_deadline_at = readString(ticketData.payment_deadline_at);
    task.ticket_reservation_id = readString(ticketData.reservation_id);
    task.ticket_train_no = readString(ticketData.train_no);
    task.ticket_seat_count = readNumber(ticketData.seat_count);
    if (Array.isArray(ticketData.tickets)) {
      const seatRows = ticketData.tickets
        .filter((row): row is Record<string, unknown> => typeof row === "object" && row !== null)
        .map((row) => {
          const seatNo = readString(row.seat_no);
          const carNo = readString(row.car_no);
          if (!seatNo) return null;
          return carNo ? `${carNo}-${seatNo}` : seatNo;
        })
        .filter((value): value is string => typeof value === "string");
      task.ticket_seats = seatRows.length > 0 ? Array.from(new Set(seatRows)) : null;
    }
    task.ticket_seat_classes = extractTicketSeatClasses(ticketData);

    return { task, attempts, artifacts };
  } catch {
    return null;
  }
}
