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

type TrainTaskEventsStore = {
  source: EventSource | null;
  listeners: Set<TrainTaskEventListener>;
  onTaskState: ((event: MessageEvent<string>) => void) | null;
  onAttentionSnapshot: ((event: MessageEvent<string>) => void) | null;
  onTaskTicketStatus: ((event: MessageEvent<string>) => void) | null;
};

declare global {
  // eslint-disable-next-line no-var
  var __bominalTrainTaskEventsStore: TrainTaskEventsStore | undefined;
}

function getStore(): TrainTaskEventsStore {
  if (typeof window === "undefined") {
    return { source: null, listeners: new Set(), onTaskState: null, onAttentionSnapshot: null, onTaskTicketStatus: null };
  }
  if (!window.__bominalTrainTaskEventsStore) {
    window.__bominalTrainTaskEventsStore = {
      source: null,
      listeners: new Set(),
      onTaskState: null,
      onAttentionSnapshot: null,
      onTaskTicketStatus: null,
    };
  }
  return window.__bominalTrainTaskEventsStore;
}

function dispatchEventPayload(store: TrainTaskEventsStore, event: MessageEvent<string>): void {
  let parsed: unknown;
  try {
    parsed = JSON.parse(event.data) as unknown;
  } catch {
    // Ignore malformed payloads from stream.
    return;
  }

  let payload: TrainTaskEventPayload = {};
  if (typeof parsed === "object" && parsed !== null && !Array.isArray(parsed)) {
    payload = parsed as TrainTaskEventPayload;
  }
  if (!payload.type) {
    payload.type = event.type;
  }

  for (const listener of Array.from(store.listeners)) {
    listener(payload, event);
  }
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

function ensureSource(store: TrainTaskEventsStore): void {
  if (store.source || typeof window === "undefined" || !("EventSource" in window)) {
    return;
  }
  const eventsBase = resolveEventsBaseUrl();
  const source = new EventSource(`${eventsBase}/api/train/tasks/events`, { withCredentials: true });
  const onTaskState = (event: MessageEvent<string>) => dispatchEventPayload(store, event);
  const onAttentionSnapshot = (event: MessageEvent<string>) => dispatchEventPayload(store, event);
  const onTaskTicketStatus = (event: MessageEvent<string>) => dispatchEventPayload(store, event);

  source.addEventListener("task_state", onTaskState as EventListener);
  source.addEventListener("attention_snapshot", onAttentionSnapshot as EventListener);
  source.addEventListener("task_ticket_status", onTaskTicketStatus as EventListener);
  store.source = source;
  store.onTaskState = onTaskState;
  store.onAttentionSnapshot = onAttentionSnapshot;
  store.onTaskTicketStatus = onTaskTicketStatus;
}

function closeSource(store: TrainTaskEventsStore): void {
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

export function subscribeTrainTaskEvents(listener: TrainTaskEventListener): () => void {
  const store = getStore();
  if (typeof window === "undefined" || !("EventSource" in window)) {
    return () => undefined;
  }

  store.listeners.add(listener);
  ensureSource(store);

  return () => {
    store.listeners.delete(listener);
    if (store.listeners.size === 0) {
      closeSource(store);
    }
  };
}
