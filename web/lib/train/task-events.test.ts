import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";

class MockEventSource {
  static instances: MockEventSource[] = [];

  readonly url: string;
  readonly withCredentials: boolean;
  closed = false;
  private listeners = new Map<string, Set<(event: MessageEvent<string>) => void>>();

  constructor(url: string, init?: EventSourceInit) {
    this.url = url;
    this.withCredentials = Boolean(init?.withCredentials);
    MockEventSource.instances.push(this);
  }

  addEventListener(type: string, listener: EventListenerOrEventListenerObject) {
    const callback =
      typeof listener === "function"
        ? (listener as (event: MessageEvent<string>) => void)
        : (event: MessageEvent<string>) => listener.handleEvent(event as Event);
    if (!this.listeners.has(type)) {
      this.listeners.set(type, new Set());
    }
    this.listeners.get(type)?.add(callback);
  }

  removeEventListener(type: string, listener: EventListenerOrEventListenerObject) {
    const callback =
      typeof listener === "function"
        ? (listener as (event: MessageEvent<string>) => void)
        : (event: MessageEvent<string>) => listener.handleEvent(event as Event);
    this.listeners.get(type)?.delete(callback);
  }

  close() {
    this.closed = true;
    this.listeners.clear();
  }
}

class MockRealtimeChannel {
  private statusCallback: ((status: string) => void) | null = null;
  private readonly initialStatuses: string[];
  private payloadHandler: ((payload: unknown) => void) | null = null;

  readonly on = vi.fn(
    (
      _eventType: string,
      _filter: Record<string, unknown>,
      handler: (payload: unknown) => void,
    ): MockRealtimeChannel => {
      this.payloadHandler = handler;
      return this;
    },
  );

  readonly subscribe = vi.fn((callback: (status: string) => void): MockRealtimeChannel => {
    this.statusCallback = callback;
    for (const status of this.initialStatuses) {
      callback(status);
    }
    return this;
  });

  readonly unsubscribe = vi.fn();

  constructor(initialStatuses: string[]) {
    this.initialStatuses = initialStatuses;
  }

  emitPayload(payload: unknown) {
    this.payloadHandler?.(payload);
  }

  emitStatus(status: string) {
    this.statusCallback?.(status);
  }
}

type LoadModuleOptions = {
  token: string | null;
  realtimeEnabled: boolean;
  canaryPercent: number;
  retrySeconds: number;
  supabaseUrl?: string | undefined;
  supabaseAnonKey?: string | undefined;
  subscribeStatusByAttempt: string[][];
};

type LoadModuleResult = {
  channels: MockRealtimeChannel[];
  createClientMock: ReturnType<typeof vi.fn>;
  fetchTaskListBootstrapMock: ReturnType<typeof vi.fn>;
  subscribeTrainTaskEvents: (
    listener: (payload: Record<string, unknown>, event: MessageEvent<string>) => void,
  ) => () => void;
};

const BASE64URL_PADDING = /={1,2}$/u;

function base64UrlEncode(value: string): string {
  return Buffer.from(value, "utf-8")
    .toString("base64")
    .replace(/\+/gu, "-")
    .replace(/\//gu, "_")
    .replace(BASE64URL_PADDING, "");
}

function makeJwt(sub: string): string {
  const payload = base64UrlEncode(JSON.stringify({ sub }));
  return `header.${payload}.signature`;
}

async function flushAsyncEffects(iterations = 4): Promise<void> {
  for (let index = 0; index < iterations; index += 1) {
    await Promise.resolve();
  }
}

async function loadTaskEventsModule(options: LoadModuleOptions): Promise<LoadModuleResult> {
  process.env.NEXT_PUBLIC_SUPABASE_URL = options.supabaseUrl ?? "https://bominal-test.supabase.co";
  process.env.NEXT_PUBLIC_SUPABASE_ANON_KEY = options.supabaseAnonKey ?? "test-anon-key";

  const channels: MockRealtimeChannel[] = [];

  const createClientMock = vi.fn(() => {
    const statusSequence = options.subscribeStatusByAttempt[channels.length] ?? ["SUBSCRIBED"];
    const channel = new MockRealtimeChannel(statusSequence);
    channels.push(channel);
    return {
      channel: vi.fn(() => channel),
      removeChannel: vi.fn(async () => "ok"),
    };
  });

  const fetchTaskListBootstrapMock = vi.fn(async () => ({ tasks: [] }));

  vi.doMock("@/lib/feature-flags", () => ({
    NEXT_PUBLIC_TRAIN_EVENTS_REALTIME_ENABLED: options.realtimeEnabled,
    NEXT_PUBLIC_TRAIN_EVENTS_REALTIME_CANARY_PERCENT: options.canaryPercent,
    NEXT_PUBLIC_TRAIN_EVENTS_REALTIME_RETRY_SECONDS: options.retrySeconds,
  }));
  vi.doMock("@/lib/supabase-auth", () => ({
    getSupabaseAccessToken: vi.fn(async () => options.token),
  }));
  vi.doMock("@supabase/supabase-js", () => ({
    createClient: createClientMock,
  }));
  vi.doMock("@/lib/train/task-list-bootstrap", () => ({
    fetchTaskListBootstrap: fetchTaskListBootstrapMock,
  }));
  vi.doMock("@/lib/api-base", () => ({
    clientApiBaseUrl: "",
    clientApiEventsBaseUrl: "",
  }));

  const importedTaskEventsModule = await import("@/lib/train/task-events");
  return {
    channels,
    createClientMock,
    fetchTaskListBootstrapMock,
    subscribeTrainTaskEvents: importedTaskEventsModule.subscribeTrainTaskEvents,
  };
}

describe("train task event transport manager", () => {
  const originalSupabaseUrl = process.env.NEXT_PUBLIC_SUPABASE_URL;
  const originalSupabaseAnonKey = process.env.NEXT_PUBLIC_SUPABASE_ANON_KEY;

  beforeEach(() => {
    vi.resetModules();
    vi.restoreAllMocks();
    vi.unstubAllGlobals();
    MockEventSource.instances = [];
    vi.stubGlobal("EventSource", MockEventSource as unknown as typeof EventSource);
    Object.defineProperty(window, "EventSource", {
      configurable: true,
      writable: true,
      value: MockEventSource,
    });
    delete (window as Window & { __bominalTrainTaskEventsStore?: unknown }).__bominalTrainTaskEventsStore;
  });

  afterEach(() => {
    vi.useRealTimers();
    vi.restoreAllMocks();
    vi.unstubAllGlobals();
    delete (window as Window & { __bominalTrainTaskEventsStore?: unknown }).__bominalTrainTaskEventsStore;
    process.env.NEXT_PUBLIC_SUPABASE_URL = originalSupabaseUrl;
    process.env.NEXT_PUBLIC_SUPABASE_ANON_KEY = originalSupabaseAnonKey;
  });

  it("uses realtime as primary transport when subscription succeeds", async () => {
    const token = makeJwt("user-primary");
    const { channels, createClientMock, fetchTaskListBootstrapMock, subscribeTrainTaskEvents } =
      await loadTaskEventsModule({
        token,
        realtimeEnabled: true,
        canaryPercent: 100,
        retrySeconds: 60,
        subscribeStatusByAttempt: [["SUBSCRIBED"]],
      });
    const listener = vi.fn();

    const unsubscribe = subscribeTrainTaskEvents(listener);
    await flushAsyncEffects();

    expect(createClientMock).toHaveBeenCalledTimes(1);
    expect(MockEventSource.instances).toHaveLength(0);
    expect(fetchTaskListBootstrapMock).toHaveBeenCalledTimes(1);

    channels[0]?.emitPayload({
      eventType: "UPDATE",
      new: {
        task_id: "task-1",
        state: "RUNNING",
        updated_at: "2026-03-01T00:00:00Z",
        ticket_status: "awaiting_payment",
      },
    });
    await flushAsyncEffects();

    const eventTypes = listener.mock.calls.map((call) => String(call[0]?.type ?? ""));
    expect(eventTypes).toContain("task_state_changed");
    expect(eventTypes).toContain("task_ticket_status_changed");

    unsubscribe();
    await flushAsyncEffects();
    expect(channels[0]?.unsubscribe).toHaveBeenCalledTimes(1);
  });

  it("falls back to SSE when realtime subscribe fails", async () => {
    const token = makeJwt("user-fallback");
    const { createClientMock, subscribeTrainTaskEvents } = await loadTaskEventsModule({
      token,
      realtimeEnabled: true,
      canaryPercent: 100,
      retrySeconds: 60,
      subscribeStatusByAttempt: [["CHANNEL_ERROR"]],
    });

    const unsubscribe = subscribeTrainTaskEvents(() => undefined);
    await flushAsyncEffects(20);

    expect(createClientMock).toHaveBeenCalledTimes(1);
    expect(MockEventSource.instances).toHaveLength(1);
    expect(MockEventSource.instances[0]?.url).toMatch(/\/api\/train\/tasks\/events$/u);
    expect(MockEventSource.instances[0]?.withCredentials).toBe(true);

    unsubscribe();
  });

  it("retries realtime while on SSE and cuts back over after recovery", async () => {
    vi.useFakeTimers();
    const token = makeJwt("user-recovery");
    const { createClientMock, subscribeTrainTaskEvents } = await loadTaskEventsModule({
      token,
      realtimeEnabled: true,
      canaryPercent: 100,
      retrySeconds: 5,
      subscribeStatusByAttempt: [["CHANNEL_ERROR"], ["SUBSCRIBED"]],
    });

    const unsubscribe = subscribeTrainTaskEvents(() => undefined);
    await flushAsyncEffects(20);

    expect(createClientMock).toHaveBeenCalledTimes(1);
    expect(MockEventSource.instances).toHaveLength(1);
    expect(MockEventSource.instances[0]?.closed).toBe(false);

    await vi.advanceTimersByTimeAsync(5000);
    await flushAsyncEffects();

    expect(createClientMock).toHaveBeenCalledTimes(2);
    expect(MockEventSource.instances[0]?.closed).toBe(true);

    unsubscribe();
  });

  it("falls back to SSE when realtime channel errors after initial subscribe", async () => {
    const token = makeJwt("user-channel-error");
    const { channels, subscribeTrainTaskEvents } = await loadTaskEventsModule({
      token,
      realtimeEnabled: true,
      canaryPercent: 100,
      retrySeconds: 60,
      subscribeStatusByAttempt: [["SUBSCRIBED"]],
    });

    const unsubscribe = subscribeTrainTaskEvents(() => undefined);
    await flushAsyncEffects();
    expect(MockEventSource.instances).toHaveLength(0);

    channels[0]?.emitStatus("CHANNEL_ERROR");
    await flushAsyncEffects(20);
    expect(MockEventSource.instances).toHaveLength(1);

    unsubscribe();
  });

  it("starts on SSE immediately when realtime token is unavailable", async () => {
    const { createClientMock, subscribeTrainTaskEvents } = await loadTaskEventsModule({
      token: null,
      realtimeEnabled: true,
      canaryPercent: 100,
      retrySeconds: 60,
      subscribeStatusByAttempt: [],
    });

    const unsubscribe = subscribeTrainTaskEvents(() => undefined);
    await flushAsyncEffects();

    expect(createClientMock).not.toHaveBeenCalled();
    expect(MockEventSource.instances).toHaveLength(1);

    unsubscribe();
  });

  it("does not crash when EventSource constructor throws", async () => {
    class ThrowingEventSource {
      constructor() {
        throw new DOMException("Mixed content blocked", "SecurityError");
      }
    }

    vi.stubGlobal("EventSource", ThrowingEventSource as unknown as typeof EventSource);
    Object.defineProperty(window, "EventSource", {
      configurable: true,
      writable: true,
      value: ThrowingEventSource,
    });

    const { createClientMock, subscribeTrainTaskEvents } = await loadTaskEventsModule({
      token: null,
      realtimeEnabled: false,
      canaryPercent: 0,
      retrySeconds: 60,
      subscribeStatusByAttempt: [],
    });

    const unsubscribe = subscribeTrainTaskEvents(() => undefined);
    await flushAsyncEffects(20);

    expect(createClientMock).not.toHaveBeenCalled();

    unsubscribe();
  });
});
