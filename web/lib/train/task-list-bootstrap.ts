import { clientApiBaseUrl } from "@/lib/api-base";
import type { TrainTaskSummary } from "@/lib/types";

export type TaskListStatus = "active" | "completed";

export type FetchTasksByStatusOptions = {
  refreshCompleted?: boolean;
  limit?: number;
};

export type TaskListBootstrapOptions = {
  refreshCompleted?: boolean;
  force?: boolean;
  cacheTtlMs?: number;
  activeLimit?: number;
  completedLimit?: number;
};

export type TaskListBootstrapSnapshot = {
  active: TrainTaskSummary[];
  completed: TrainTaskSummary[];
  tasks: TrainTaskSummary[];
  fetchedAt: number;
};

export const TASK_LIST_ERROR_MESSAGE = "task_list_error";
export const SESSION_EXPIRED_MESSAGE = "session_expired";
export const ACTIVE_TASK_FETCH_LIMIT = 60;
export const COMPLETED_TASK_FETCH_LIMIT = 80;

const DEFAULT_BOOTSTRAP_CACHE_TTL_MS = 5000;

let bootstrapSnapshot: TaskListBootstrapSnapshot | null = null;
let bootstrapInFlight: Promise<TaskListBootstrapSnapshot> | null = null;

function mergeTasks(active: TrainTaskSummary[], completed: TrainTaskSummary[]): TrainTaskSummary[] {
  const merged = new Map<string, TrainTaskSummary>();
  for (const task of active) {
    merged.set(task.id, task);
  }
  for (const task of completed) {
    if (!merged.has(task.id)) {
      merged.set(task.id, task);
    }
  }
  return Array.from(merged.values());
}

function buildSnapshot(active: TrainTaskSummary[], completed: TrainTaskSummary[]): TaskListBootstrapSnapshot {
  return {
    active,
    completed,
    tasks: mergeTasks(active, completed),
    fetchedAt: Date.now(),
  };
}

function isFresh(snapshot: TaskListBootstrapSnapshot, ttlMs: number): boolean {
  return Date.now() - snapshot.fetchedAt <= ttlMs;
}

export function readTaskListBootstrapSnapshot(): TaskListBootstrapSnapshot | null {
  return bootstrapSnapshot;
}

export function clearTaskListBootstrapSnapshot(): void {
  bootstrapSnapshot = null;
  bootstrapInFlight = null;
}

export async function fetchTasksByStatus(status: TaskListStatus, options?: FetchTasksByStatusOptions): Promise<TrainTaskSummary[]> {
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

export async function fetchTaskListBootstrap(options?: TaskListBootstrapOptions): Promise<TaskListBootstrapSnapshot> {
  const ttlMs = Math.max(0, options?.cacheTtlMs ?? DEFAULT_BOOTSTRAP_CACHE_TTL_MS);
  if (!options?.force && bootstrapSnapshot && isFresh(bootstrapSnapshot, ttlMs)) {
    return bootstrapSnapshot;
  }

  if (bootstrapInFlight) {
    return bootstrapInFlight;
  }

  const activeLimit = options?.activeLimit ?? ACTIVE_TASK_FETCH_LIMIT;
  const completedLimit = options?.completedLimit ?? COMPLETED_TASK_FETCH_LIMIT;

  bootstrapInFlight = (async () => {
    const [active, completed] = await Promise.all([
      fetchTasksByStatus("active", { limit: activeLimit }),
      fetchTasksByStatus("completed", {
        refreshCompleted: options?.refreshCompleted,
        limit: completedLimit,
      }),
    ]);

    const next = buildSnapshot(active, completed);
    bootstrapSnapshot = next;
    return next;
  })();

  try {
    return await bootstrapInFlight;
  } finally {
    bootstrapInFlight = null;
  }
}
