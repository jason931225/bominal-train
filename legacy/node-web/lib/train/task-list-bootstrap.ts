import { clientApiBaseUrl } from "@/lib/api-base";
import { fetchTrainTaskListViaDataApi } from "@/lib/train/data-api";
import type { TrainTaskSummary } from "@/lib/types";

export type TaskListStatus = "active" | "completed";

export type FetchTasksByStatusOptions = {
  refreshCompleted?: boolean;
  limit?: number;
  view?: "full" | "compact";
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
const taskListEtagCache = new Map<string, { etag: string; tasks: TrainTaskSummary[] }>();

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
  taskListEtagCache.clear();
}

export async function fetchTasksByStatus(status: TaskListStatus, options?: FetchTasksByStatusOptions): Promise<TrainTaskSummary[]> {
  const resolvedLimit = options?.limit ?? (status === "active" ? ACTIVE_TASK_FETCH_LIMIT : COMPLETED_TASK_FETCH_LIMIT);
  const dataApiRows = await fetchTrainTaskListViaDataApi({
    status,
    limit: resolvedLimit,
  });
  if (dataApiRows) {
    return dataApiRows;
  }

  const query = new URLSearchParams({ status });
  query.set("view", options?.view ?? "compact");
  if (status === "completed" && options?.refreshCompleted) {
    query.set("refresh_completed", "true");
  }
  query.set("limit", String(resolvedLimit));
  const cacheKey = query.toString();
  const cachedEntry = taskListEtagCache.get(cacheKey);

  const headers = new Headers();
  if (cachedEntry?.etag) {
    headers.set("If-None-Match", cachedEntry.etag);
  }

  const response = await fetch(`${clientApiBaseUrl}/api/train/tasks?${query.toString()}`, {
    credentials: "include",
    cache: "no-store",
    headers,
  });
  if (response.status === 304 && cachedEntry) {
    return cachedEntry.tasks;
  }

  if (!response.ok) {
    if (response.status === 401) {
      throw new Error(SESSION_EXPIRED_MESSAGE);
    }
    throw new Error(TASK_LIST_ERROR_MESSAGE);
  }

  const payload = (await response.json()) as { tasks: TrainTaskSummary[] };
  const etag = response.headers.get("etag");
  if (etag && payload.tasks) {
    taskListEtagCache.set(cacheKey, { etag, tasks: payload.tasks });
  }
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
