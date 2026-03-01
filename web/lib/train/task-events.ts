import { createClient, type RealtimeChannel, type SupabaseClient } from "@supabase/supabase-js";

import {
  NEXT_PUBLIC_TRAIN_EVENTS_REALTIME_CANARY_PERCENT,
  NEXT_PUBLIC_TRAIN_EVENTS_REALTIME_ENABLED,
  NEXT_PUBLIC_TRAIN_EVENTS_REALTIME_RETRY_SECONDS,
} from "@/lib/feature-flags";
import { getSupabaseAccessToken } from "@/lib/supabase-auth";
import { fetchTaskListBootstrap } from "@/lib/train/task-list-bootstrap";
import { clientApiBaseUrl, clientApiEventsBaseUrl } from "@/lib/api-base";

type TrainTaskEventPayload = Record<string, unknown> & {
  type?: string;
  task_id?: string;
  state?: string;
  updated_at?: string;
  ticket_status?: string;
  previous_ticket_status?: string;
  tasks?: unknown[];
};

type TrainTaskEventListener = (payload: TrainTaskEventPayload, event: MessageEvent<string>) => void;

type TaskRealtimeRow = {
  task_id?: string;
  user_id?: string;
  state?: string;
  updated_at?: string;
  ticket_status?: string | null;
};

type TaskRealtimeSnapshot = {
  ticketStatus: string;
};

type TrainTaskEventsStore = {
  source: EventSource | null;
  listeners: Set<TrainTaskEventListener>;
  onTaskState: ((event: MessageEvent<string>) => void) | null;
  onAttentionSnapshot: ((event: MessageEvent<string>) => void) | null;
  onTaskTicketStatus: ((event: MessageEvent<string>) => void) | null;
  realtimeClient: SupabaseClient | null;
  realtimeChannel: RealtimeChannel | null;
  realtimeRetryTimer: number | null;
  startPromise: Promise<void> | null;
  ticketSnapshotsByTaskId: Map<string, TaskRealtimeSnapshot>;
};

const SUPABASE_URL = (process.env.NEXT_PUBLIC_SUPABASE_URL ?? "").trim().replace(/\/+$/, "");
const SUPABASE_ANON_KEY = (process.env.NEXT_PUBLIC_SUPABASE_ANON_KEY ?? "").trim();
const REALTIME_CHANNEL_SCHEMA = "public";
const REALTIME_CHANNEL_TABLE = "task_realtime_events";
const REALTIME_SUBSCRIBE_TIMEOUT_MS = 5000;

declare global {
  var __bominalTrainTaskEventsStore: TrainTaskEventsStore | undefined;
}

function createEmptyStore(): TrainTaskEventsStore {
  return {
    source: null,
    listeners: new Set(),
    onTaskState: null,
    onAttentionSnapshot: null,
    onTaskTicketStatus: null,
    realtimeClient: null,
    realtimeChannel: null,
    realtimeRetryTimer: null,
    startPromise: null,
    ticketSnapshotsByTaskId: new Map(),
  };
}

function getStore(): TrainTaskEventsStore {
  if (typeof window === "undefined") {
    return createEmptyStore();
  }
  if (!window.__bominalTrainTaskEventsStore) {
    window.__bominalTrainTaskEventsStore = createEmptyStore();
  }
  return window.__bominalTrainTaskEventsStore;
}

function parseEventPayload(event: MessageEvent<string>): TrainTaskEventPayload | null {
  let parsed: unknown;
  try {
    parsed = JSON.parse(event.data) as unknown;
  } catch {
    return null;
  }
  if (typeof parsed !== "object" || parsed === null || Array.isArray(parsed)) {
    return null;
  }
  const payload = parsed as TrainTaskEventPayload;
  if (!payload.type) {
    payload.type = event.type;
  }
  return payload;
}

function dispatchPayload(store: TrainTaskEventsStore, eventType: string, payload: TrainTaskEventPayload): void {
  const normalizedPayload = { ...payload };
  if (!normalizedPayload.type) {
    normalizedPayload.type = eventType;
  }
  const event = new MessageEvent<string>(eventType, {
    data: JSON.stringify(normalizedPayload),
  });
  for (const listener of Array.from(store.listeners)) {
    listener(normalizedPayload, event);
  }
}

function dispatchSseEvent(store: TrainTaskEventsStore, event: MessageEvent<string>): void {
  const payload = parseEventPayload(event);
  if (!payload) {
    return;
  }
  for (const listener of Array.from(store.listeners)) {
    listener(payload, event);
  }
}

function clearRealtimeRetry(store: TrainTaskEventsStore): void {
  if (store.realtimeRetryTimer == null || typeof window === "undefined") {
    return;
  }
  window.clearTimeout(store.realtimeRetryTimer);
  store.realtimeRetryTimer = null;
}

function normalizeNullableString(value: unknown): string | null {
  if (typeof value !== "string") {
    return null;
  }
  const normalized = value.trim();
  return normalized.length > 0 ? normalized : null;
}

function resolveEventsBaseUrl(): string {
  if (!clientApiEventsBaseUrl) {
    return clientApiBaseUrl;
  }
  if (typeof window === "undefined") {
    return clientApiEventsBaseUrl;
  }
  // In same-origin cookie mode (`clientApiBaseUrl === ""`), avoid cross-host SSE
  // (for example 127.0.0.1 page with localhost events base), which drops cookies.
  if (!clientApiBaseUrl) {
    try {
      const candidate = new URL(clientApiEventsBaseUrl, window.location.origin);
      if (candidate.hostname !== window.location.hostname) {
        return clientApiBaseUrl;
      }
    } catch {
      return clientApiBaseUrl;
    }
  }
  return clientApiEventsBaseUrl;
}

function ensureSseSource(store: TrainTaskEventsStore): void {
  if (store.source || typeof window === "undefined" || !("EventSource" in window)) {
    return;
  }
  const eventsBase = resolveEventsBaseUrl();
  const source = new EventSource(`${eventsBase}/api/train/tasks/events`, { withCredentials: true });
  const onTaskState = (event: MessageEvent<string>) => dispatchSseEvent(store, event);
  const onAttentionSnapshot = (event: MessageEvent<string>) => dispatchSseEvent(store, event);
  const onTaskTicketStatus = (event: MessageEvent<string>) => dispatchSseEvent(store, event);

  source.addEventListener("task_state", onTaskState as EventListener);
  source.addEventListener("attention_snapshot", onAttentionSnapshot as EventListener);
  source.addEventListener("task_ticket_status", onTaskTicketStatus as EventListener);
  store.source = source;
  store.onTaskState = onTaskState;
  store.onAttentionSnapshot = onAttentionSnapshot;
  store.onTaskTicketStatus = onTaskTicketStatus;
}

function closeSseSource(store: TrainTaskEventsStore): void {
  if (!store.source) {
    return;
  }
  if (store.onTaskState) {
    store.source.removeEventListener("task_state", store.onTaskState as EventListener);
  }
  if (store.onAttentionSnapshot) {
    store.source.removeEventListener("attention_snapshot", store.onAttentionSnapshot as EventListener);
  }
  if (store.onTaskTicketStatus) {
    store.source.removeEventListener("task_ticket_status", store.onTaskTicketStatus as EventListener);
  }
  store.source.close();
  store.source = null;
  store.onTaskState = null;
  store.onAttentionSnapshot = null;
  store.onTaskTicketStatus = null;
}

function hasRealtimeBrowserConfig(): boolean {
  return SUPABASE_URL.length > 0 && SUPABASE_ANON_KEY.length > 0;
}

function decodeBase64UrlToUtf8(value: string): string | null {
  if (!value) {
    return null;
  }
  const normalized = value.replace(/-/g, "+").replace(/_/g, "/");
  const padding = normalized.length % 4 === 0 ? "" : "=".repeat(4 - (normalized.length % 4));
  const input = normalized + padding;
  try {
    if (typeof atob === "function") {
      return atob(input);
    }
    if (typeof Buffer !== "undefined") {
      return Buffer.from(input, "base64").toString("utf-8");
    }
  } catch {
    return null;
  }
  return null;
}

function extractIdentityFromAccessToken(accessToken: string): string | null {
  const token = accessToken.trim();
  if (!token) {
    return null;
  }
  const tokenParts = token.split(".");
  if (tokenParts.length < 2) {
    return null;
  }
  const payloadRaw = decodeBase64UrlToUtf8(tokenParts[1] ?? "");
  if (!payloadRaw) {
    return null;
  }
  try {
    const payload = JSON.parse(payloadRaw) as Record<string, unknown>;
    const sub = normalizeNullableString(payload.sub);
    return sub;
  } catch {
    return null;
  }
}

function stableBucketPercent(value: string): number {
  let hash = 0x811c9dc5;
  for (let index = 0; index < value.length; index += 1) {
    hash ^= value.charCodeAt(index);
    hash = Math.imul(hash, 0x01000193);
  }
  return Math.abs(hash >>> 0) % 100;
}

type RealtimeEligibilityResult = {
  enabled: boolean;
  accessToken: string | null;
  shouldRetry: boolean;
};

async function evaluateRealtimeEligibility(): Promise<RealtimeEligibilityResult> {
  if (!NEXT_PUBLIC_TRAIN_EVENTS_REALTIME_ENABLED) {
    return { enabled: false, accessToken: null, shouldRetry: false };
  }
  if (NEXT_PUBLIC_TRAIN_EVENTS_REALTIME_CANARY_PERCENT <= 0) {
    return { enabled: false, accessToken: null, shouldRetry: false };
  }
  if (!hasRealtimeBrowserConfig()) {
    return { enabled: false, accessToken: null, shouldRetry: true };
  }
  const accessToken = await getSupabaseAccessToken();
  if (!accessToken) {
    return { enabled: false, accessToken: null, shouldRetry: true };
  }
  if (NEXT_PUBLIC_TRAIN_EVENTS_REALTIME_CANARY_PERCENT >= 100) {
    return { enabled: true, accessToken, shouldRetry: true };
  }
  const identity = extractIdentityFromAccessToken(accessToken);
  if (!identity) {
    return { enabled: false, accessToken, shouldRetry: true };
  }
  const bucket = stableBucketPercent(identity);
  return {
    enabled: bucket < NEXT_PUBLIC_TRAIN_EVENTS_REALTIME_CANARY_PERCENT,
    accessToken,
    shouldRetry: false,
  };
}

function dispatchRealtimeTaskState(store: TrainTaskEventsStore, row: TaskRealtimeRow): void {
  const taskId = normalizeNullableString(row.task_id);
  const state = normalizeNullableString(row.state);
  const updatedAt = normalizeNullableString(row.updated_at);
  if (!taskId || !state || !updatedAt) {
    return;
  }
  const userId = normalizeNullableString(row.user_id);
  dispatchPayload(store, "task_state", {
    type: "task_state_changed",
    task_id: taskId,
    state,
    updated_at: updatedAt,
    user_id: userId ?? undefined,
  });
}

function dispatchRealtimeTicketStatus(store: TrainTaskEventsStore, row: TaskRealtimeRow): void {
  const taskId = normalizeNullableString(row.task_id);
  const state = normalizeNullableString(row.state);
  const updatedAt = normalizeNullableString(row.updated_at);
  if (!taskId || !state || !updatedAt) {
    return;
  }

  const nextTicketStatus = normalizeNullableString(row.ticket_status) ?? "";
  const previousSnapshot = store.ticketSnapshotsByTaskId.get(taskId);
  const previousTicketStatus = previousSnapshot?.ticketStatus ?? "";
  store.ticketSnapshotsByTaskId.set(taskId, { ticketStatus: nextTicketStatus });

  if (nextTicketStatus === previousTicketStatus) {
    return;
  }

  dispatchPayload(store, "task_ticket_status", {
    type: "task_ticket_status_changed",
    task_id: taskId,
    state,
    ticket_status: nextTicketStatus,
    previous_ticket_status: previousTicketStatus,
    updated_at: updatedAt,
  });
}

function handleRealtimePayload(store: TrainTaskEventsStore, payload: { eventType?: unknown; new?: unknown; old?: unknown }): void {
  const eventType = String(payload.eventType ?? "").toUpperCase();
  const newRow = (payload.new && typeof payload.new === "object" ? payload.new : null) as TaskRealtimeRow | null;
  const oldRow = (payload.old && typeof payload.old === "object" ? payload.old : null) as TaskRealtimeRow | null;
  if (eventType === "DELETE") {
    const deletedTaskId = normalizeNullableString(oldRow?.task_id);
    if (deletedTaskId) {
      store.ticketSnapshotsByTaskId.delete(deletedTaskId);
    }
    return;
  }
  const row = newRow ?? oldRow;
  if (!row) {
    return;
  }
  dispatchRealtimeTaskState(store, row);
  dispatchRealtimeTicketStatus(store, row);
}

async function closeRealtimeSubscription(store: TrainTaskEventsStore): Promise<void> {
  const channel = store.realtimeChannel;
  const client = store.realtimeClient;
  store.realtimeChannel = null;
  store.realtimeClient = null;
  if (channel) {
    try {
      channel.unsubscribe();
    } catch {
      // Best-effort cleanup.
    }
  }
  if (client && channel) {
    try {
      await client.removeChannel(channel);
    } catch {
      // Best-effort cleanup.
    }
  }
}

async function emitAttentionSnapshot(store: TrainTaskEventsStore): Promise<void> {
  try {
    const snapshot = await fetchTaskListBootstrap({ refreshCompleted: false });
    dispatchPayload(store, "attention_snapshot", {
      type: "attention_snapshot",
      tasks: snapshot.tasks,
    });
  } catch {
    // Best-effort snapshot; top-nav has a remote fallback timer.
  }
}

async function tryStartRealtime(store: TrainTaskEventsStore, accessToken: string): Promise<boolean> {
  if (typeof window === "undefined") {
    return false;
  }

  await closeRealtimeSubscription(store);

  const realtimeClient = createClient(SUPABASE_URL, SUPABASE_ANON_KEY, {
    auth: {
      persistSession: false,
      autoRefreshToken: false,
      detectSessionInUrl: false,
    },
    global: {
      headers: {
        Authorization: `Bearer ${accessToken}`,
      },
    },
  });

  const channelName = `train-task-events-${Date.now().toString(36)}-${Math.random().toString(36).slice(2, 8)}`;
  const channel = realtimeClient
    .channel(channelName)
    .on(
      "postgres_changes",
      { event: "*", schema: REALTIME_CHANNEL_SCHEMA, table: REALTIME_CHANNEL_TABLE },
      (payload) => {
        handleRealtimePayload(store, payload as { eventType?: unknown; new?: unknown; old?: unknown });
      },
    );

  const subscribed = await new Promise<boolean>((resolve) => {
    let settled = false;
    let initialSubscribed = false;
    const timeoutId = window.setTimeout(() => {
      if (settled) {
        return;
      }
      settled = true;
      resolve(false);
    }, REALTIME_SUBSCRIBE_TIMEOUT_MS);

    const finish = (result: boolean) => {
      if (settled) {
        return;
      }
      settled = true;
      window.clearTimeout(timeoutId);
      resolve(result);
    };

    channel.subscribe((status) => {
      const normalized = String(status ?? "").toUpperCase();
      if (normalized === "SUBSCRIBED") {
        initialSubscribed = true;
        finish(true);
        return;
      }
      if (initialSubscribed && (normalized === "CLOSED" || normalized === "CHANNEL_ERROR" || normalized === "TIMED_OUT")) {
        void closeRealtimeSubscription(store).then(() => {
          ensureSseSource(store);
          scheduleRealtimeRetry(store);
        });
        return;
      }
      if (normalized === "TIMED_OUT" || normalized === "CHANNEL_ERROR" || normalized === "CLOSED") {
        finish(false);
      }
    });
  });

  if (!subscribed) {
    try {
      channel.unsubscribe();
    } catch {
      // Best-effort cleanup.
    }
    try {
      await realtimeClient.removeChannel(channel);
    } catch {
      // Best-effort cleanup.
    }
    return false;
  }

  store.realtimeClient = realtimeClient;
  store.realtimeChannel = channel;
  clearRealtimeRetry(store);
  closeSseSource(store);
  await emitAttentionSnapshot(store);
  return true;
}

function scheduleRealtimeRetry(store: TrainTaskEventsStore): void {
  if (typeof window === "undefined" || store.realtimeRetryTimer != null || store.listeners.size === 0) {
    return;
  }
  store.realtimeRetryTimer = window.setTimeout(() => {
    store.realtimeRetryTimer = null;
    if (store.listeners.size === 0) {
      return;
    }
    void establishPrimaryTransport(store);
  }, NEXT_PUBLIC_TRAIN_EVENTS_REALTIME_RETRY_SECONDS * 1000);
}

async function establishPrimaryTransport(store: TrainTaskEventsStore): Promise<void> {
  if (!NEXT_PUBLIC_TRAIN_EVENTS_REALTIME_ENABLED) {
    ensureSseSource(store);
    clearRealtimeRetry(store);
    return;
  }
  if (NEXT_PUBLIC_TRAIN_EVENTS_REALTIME_CANARY_PERCENT <= 0) {
    ensureSseSource(store);
    clearRealtimeRetry(store);
    return;
  }

  const eligibility = await evaluateRealtimeEligibility();
  if (eligibility.enabled && eligibility.accessToken) {
    const started = await tryStartRealtime(store, eligibility.accessToken);
    if (started) {
      return;
    }
    ensureSseSource(store);
    scheduleRealtimeRetry(store);
    return;
  }

  ensureSseSource(store);
  if (eligibility.shouldRetry) {
    scheduleRealtimeRetry(store);
  } else {
    clearRealtimeRetry(store);
  }
}

function ensureTransport(store: TrainTaskEventsStore): void {
  if (store.startPromise) {
    return;
  }
  store.startPromise = (async () => {
    if (store.listeners.size === 0) {
      return;
    }
    await establishPrimaryTransport(store);
  })().finally(() => {
    store.startPromise = null;
  });
}

function shutdownTransport(store: TrainTaskEventsStore): void {
  clearRealtimeRetry(store);
  closeSseSource(store);
  void closeRealtimeSubscription(store);
  store.ticketSnapshotsByTaskId.clear();
}

export function subscribeTrainTaskEvents(listener: TrainTaskEventListener): () => void {
  const store = getStore();
  if (typeof window === "undefined") {
    return () => undefined;
  }

  store.listeners.add(listener);
  ensureTransport(store);

  return () => {
    store.listeners.delete(listener);
    if (store.listeners.size === 0) {
      shutdownTransport(store);
    }
  };
}
