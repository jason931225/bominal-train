import React from "react";

import { act, render, waitFor } from "@testing-library/react";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";

import { LocaleProvider } from "@/components/locale-provider";
import { TopNavTaskAttention } from "@/components/top-nav-task-attention";
import { clearTaskListBootstrapSnapshot } from "@/lib/train/task-list-bootstrap";
import type { TrainTaskSummary } from "@/lib/types";

vi.mock("next/navigation", async () => {
  const actual = await vi.importActual<typeof import("next/navigation")>("next/navigation");
  return {
    ...actual,
    usePathname: () => "/dashboard",
  };
});

class MockEventSource {
  static instances: MockEventSource[] = [];

  readonly url: string;
  readonly withCredentials: boolean;
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
    this.listeners.clear();
  }

  static emit(type: string, payload: Record<string, unknown>) {
    const event = new MessageEvent<string>(type, { data: JSON.stringify(payload) });
    for (const instance of MockEventSource.instances) {
      const callbacks = instance.listeners.get(type);
      if (!callbacks) continue;
      for (const callback of callbacks) {
        callback(event);
      }
    }
  }
}

function makeTaskListResponse(id: string = "t-1"): Response {
  return new Response(JSON.stringify({ tasks: [makeAttentionTask(id)] }), {
    status: 200,
    headers: { "Content-Type": "application/json" },
  });
}

function createDeferred<T>(): {
  promise: Promise<T>;
  resolve: (value: T) => void;
  reject: (reason?: unknown) => void;
} {
  let resolve!: (value: T) => void;
  let reject!: (reason?: unknown) => void;
  const promise = new Promise<T>((res, rej) => {
    resolve = res;
    reject = rej;
  });
  return { promise, resolve, reject };
}

function makeAttentionTask(id: string): TrainTaskSummary {
  return {
    id,
    module: "train",
    state: "COMPLETED",
    deadline_at: "2026-02-15T12:00:00+09:00",
    created_at: "2026-02-15T11:00:00+09:00",
    updated_at: "2026-02-15T11:30:00+09:00",
    paused_at: null,
    cancelled_at: null,
    completed_at: "2026-02-15T11:40:00+09:00",
    failed_at: null,
    hidden_at: null,
    last_attempt_at: null,
    last_attempt_action: null,
    last_attempt_ok: null,
    last_attempt_error_code: null,
    last_attempt_error_message_safe: null,
    last_attempt_finished_at: null,
    next_run_at: null,
    retry_now_allowed: false,
    retry_now_reason: "terminal_state",
    retry_now_available_at: null,
    spec_json: {
      dep: "수서",
      arr: "부산",
      date: "2026-02-15",
      passengers: { adults: 1, children: 0 },
      selected_trains_ranked: [{ rank: 1, departure_at: "2026-02-15T13:10:00+09:00" }],
    },
    ticket_status: "awaiting_payment",
    ticket_paid: false,
    ticket_payment_deadline_at: null,
    ticket_reservation_id: "PNR123",
  };
}

describe("TopNavTaskAttention", () => {
  const fetchMock = vi.fn<typeof fetch>();
  let visibilityState: DocumentVisibilityState = "visible";

  beforeEach(() => {
    MockEventSource.instances = [];
    visibilityState = "visible";
    vi.clearAllMocks();
    clearTaskListBootstrapSnapshot();
    vi.stubGlobal("EventSource", MockEventSource as unknown as typeof EventSource);
    vi.stubGlobal("fetch", fetchMock);
    Object.defineProperty(document, "visibilityState", {
      configurable: true,
      get: () => visibilityState,
    });

    fetchMock.mockImplementation(() => Promise.resolve(makeTaskListResponse("t-1")));
  });

  afterEach(() => {
    vi.unstubAllGlobals();
  });

  it("boots from SSE snapshot without initial list fetch and refreshes on terminal state events", async () => {
    const setIntervalSpy = vi.spyOn(window, "setInterval");

    render(
      <LocaleProvider initialLocale="en">
        <TopNavTaskAttention userId="user-1" displayName="Jason" />
      </LocaleProvider>,
    );

    expect(setIntervalSpy.mock.calls.some(([, delay]) => delay === 60000)).toBe(false);
    expect(MockEventSource.instances).toHaveLength(1);
    expect(MockEventSource.instances[0]?.url).toMatch(/\/api\/train\/tasks\/events$/);
    expect(fetchMock).toHaveBeenCalledTimes(0);

    await act(async () => {
      MockEventSource.emit("attention_snapshot", { tasks: [makeAttentionTask("t-1")] });
    });
    expect(fetchMock).toHaveBeenCalledTimes(0);

    await act(async () => {
      MockEventSource.emit("task_state", { task_id: "t-1", state: "COMPLETED" });
    });

    await waitFor(() => {
      expect(fetchMock).toHaveBeenCalledTimes(2);
    });
  });

  it("ignores non-terminal state events for attention refresh", async () => {
    render(
      <LocaleProvider initialLocale="en">
        <TopNavTaskAttention userId="user-1" displayName="Jason" />
      </LocaleProvider>,
    );

    await act(async () => {
      MockEventSource.emit("task_state", { task_id: "t-1", state: "RUNNING" });
    });

    expect(fetchMock).toHaveBeenCalledTimes(0);
  });

  it("coalesces burst terminal-state refresh events into a single queued follow-up request", async () => {
    render(
      <LocaleProvider initialLocale="en">
        <TopNavTaskAttention userId="user-3" displayName="Jason" />
      </LocaleProvider>,
    );

    const deferredFetch = createDeferred<Response>();
    fetchMock.mockImplementationOnce(() => deferredFetch.promise);

    await act(async () => {
      MockEventSource.emit("task_state", { task_id: "t-3", state: "COMPLETED" });
      MockEventSource.emit("task_state", { task_id: "t-3", state: "FAILED" });
      MockEventSource.emit("task_state", { task_id: "t-3", state: "EXPIRED" });
    });

    await waitFor(() => {
      expect(fetchMock).toHaveBeenCalledTimes(2);
    });

    await act(async () => {
      deferredFetch.resolve(makeTaskListResponse("t-3"));
    });

    await waitFor(() => {
      expect(fetchMock).toHaveBeenCalledTimes(4);
    });
  });

  it("skips SSE-triggered refresh while document is hidden", async () => {
    render(
      <LocaleProvider initialLocale="en">
        <TopNavTaskAttention userId="user-2" displayName="Jason" />
      </LocaleProvider>,
    );

    visibilityState = "hidden";
    await act(async () => {
      MockEventSource.emit("task_state", { task_id: "t-2", state: "FAILED" });
    });

    expect(fetchMock).toHaveBeenCalledTimes(0);
  });
});
