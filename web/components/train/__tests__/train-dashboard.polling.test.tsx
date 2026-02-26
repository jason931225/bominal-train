import React from "react";

import { act, fireEvent, render, screen, within } from "@testing-library/react";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";

import { LocaleProvider } from "@/components/locale-provider";
import { clearTrainDashboardFetchCaches, TrainDashboard } from "@/components/train/train-dashboard";
import type { TrainTaskSummary } from "@/lib/types";

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

function makeTask(id: string, state: TrainTaskSummary["state"]): TrainTaskSummary {
  return {
    id,
    module: "train",
    state,
    deadline_at: "2026-02-15T12:00:00+09:00",
    created_at: "2026-02-15T11:00:00+09:00",
    updated_at: "2026-02-15T11:30:00+09:00",
    paused_at: null,
    cancelled_at: null,
    completed_at: state === "COMPLETED" ? "2026-02-15T11:40:00+09:00" : null,
    failed_at: null,
    hidden_at: null,
    last_attempt_at: null,
    last_attempt_action: null,
    last_attempt_ok: null,
    last_attempt_error_code: null,
    last_attempt_error_message_safe: null,
    last_attempt_finished_at: null,
    next_run_at: null,
    retry_now_allowed: true,
    retry_now_reason: null,
    retry_now_available_at: null,
    spec_json: {
      dep: "수서",
      arr: "부산",
      date: "2026-02-15",
      passengers: { adults: 1, children: 0 },
      selected_trains_ranked: [
        {
          rank: 1,
          departure_at: "2026-02-15T13:10:00+09:00",
        },
      ],
    },
    ticket_status: state === "COMPLETED" ? "awaiting_payment" : null,
    ticket_paid: state === "COMPLETED" ? false : null,
    ticket_payment_deadline_at: null,
    ticket_reservation_id: null,
  };
}

describe("TrainDashboard polling behavior", () => {
  let activeCalls = 0;
  let completedCalls = 0;
  let pauseCalls = 0;
  let activeTaskState: TrainTaskSummary["state"] = "RUNNING";
  let visibilityState: DocumentVisibilityState = "visible";
  let searchStatus = 200;
  let searchBody: Record<string, unknown> = { schedules: [] };

  beforeEach(() => {
    clearTrainDashboardFetchCaches();
    activeCalls = 0;
    completedCalls = 0;
    pauseCalls = 0;
    activeTaskState = "RUNNING";
    visibilityState = "visible";
    searchStatus = 200;
    searchBody = { schedules: [] };
    MockEventSource.instances = [];
    vi.stubGlobal("EventSource", MockEventSource as unknown as typeof EventSource);

    Object.defineProperty(document, "visibilityState", {
      configurable: true,
      get: () => visibilityState,
    });

    vi.stubGlobal(
      "fetch",
      vi.fn(async (input: RequestInfo | URL, init?: RequestInit) => {
        const url = typeof input === "string" ? input : input.toString();

        if (url.includes("/api/train/credentials/status")) {
          return new Response(
            JSON.stringify({
              ktx: { configured: true, verified: true, username: "ktx-user", verified_at: null, detail: null },
              srt: { configured: true, verified: true, username: "srt-user", verified_at: null, detail: null },
            }),
            { status: 200, headers: { "Content-Type": "application/json" } },
          );
        }
        if (url.includes("/api/wallet/payment-card")) {
          return new Response(
            JSON.stringify({
              configured: true,
              card_masked: "****-****-****-1234",
              expiry_month: 12,
              expiry_year: 2030,
              updated_at: "2026-02-15T11:00:00+09:00",
              detail: null,
            }),
            { status: 200, headers: { "Content-Type": "application/json" } },
          );
        }
        if (url.includes("/api/train/stations")) {
          return new Response(
            JSON.stringify({
              stations: [
                { name: "수서", srt_code: "0551", srt_supported: true },
                { name: "부산", srt_code: "0020", srt_supported: true },
              ],
            }),
            { status: 200, headers: { "Content-Type": "application/json" } },
          );
        }
        if (url.includes("/api/train/tasks?")) {
          const parsed = new URL(url, "http://localhost");
          const status = parsed.searchParams.get("status");
          if (status === "active") {
            activeCalls += 1;
            return new Response(JSON.stringify({ tasks: [makeTask("active-1", activeTaskState)] }), {
              status: 200,
              headers: { "Content-Type": "application/json" },
            });
          }
          if (status === "completed") {
            completedCalls += 1;
            return new Response(JSON.stringify({ tasks: [makeTask("done-1", "COMPLETED")] }), {
              status: 200,
              headers: { "Content-Type": "application/json" },
            });
          }
        }
        if (url.includes("/api/train/tasks/active-1/pause") && (init?.method ?? "GET") === "POST") {
          pauseCalls += 1;
          return new Response(JSON.stringify({ ok: true }), {
            status: 200,
            headers: { "Content-Type": "application/json" },
          });
        }
        if (url.includes("/api/train/search") && (init?.method ?? "GET") === "POST") {
          return new Response(JSON.stringify(searchBody), {
            status: searchStatus,
            headers: { "Content-Type": "application/json" },
          });
        }
        return new Response(JSON.stringify({ detail: "not found" }), {
          status: 404,
          headers: { "Content-Type": "application/json" },
        });
      }),
    );
  });

  afterEach(() => {
    clearTrainDashboardFetchCaches();
    vi.unstubAllGlobals();
  });

  async function flushAsyncEffects() {
    await act(async () => {
      await Promise.resolve();
      await Promise.resolve();
    });
  }

  async function renderDashboard() {
    render(
      <LocaleProvider initialLocale="en">
        <TrainDashboard />
      </LocaleProvider>,
    );
    await flushAsyncEffects();
    expect(activeCalls).toBeGreaterThanOrEqual(1);
    expect(completedCalls).toBeGreaterThanOrEqual(1);
  }

  it("does not register interval-based task polling timers", async () => {
    const setIntervalSpy = vi.spyOn(window, "setInterval");
    await renderDashboard();
    expect(setIntervalSpy.mock.calls.some(([, delay]) => delay === 10000 || delay === 60000)).toBe(false);
    setIntervalSpy.mockRestore();
  });

  it("refreshes task lists only for terminal state events", async () => {
    await renderDashboard();
    const baselineActiveCalls = activeCalls;
    const baselineCompletedCalls = completedCalls;

    MockEventSource.emit("task_state", { task_id: "active-1", state: "POLLING" });
    await flushAsyncEffects();
    expect(activeCalls).toBe(baselineActiveCalls);
    expect(completedCalls).toBe(baselineCompletedCalls);

    MockEventSource.emit("task_state", { task_id: "active-1", state: "COMPLETED" });
    await flushAsyncEffects();
    expect(activeCalls).toBe(baselineActiveCalls + 1);
    expect(completedCalls).toBe(baselineCompletedCalls + 1);
  });

  it("refreshes task lists for ticket-status payloads even when delivered on task_state channel", async () => {
    await renderDashboard();
    const baselineActiveCalls = activeCalls;
    const baselineCompletedCalls = completedCalls;

    MockEventSource.emit("task_state", {
      type: "task_ticket_status_changed",
      task_id: "active-1",
      state: "POLLING",
      previous_ticket_status: "waiting",
      ticket_status: "awaiting_payment",
    });
    await flushAsyncEffects();

    expect(activeCalls).toBe(baselineActiveCalls + 1);
    expect(completedCalls).toBe(baselineCompletedCalls + 1);
  });

  it("treats lowercase terminal states as terminal refresh events", async () => {
    await renderDashboard();
    const baselineActiveCalls = activeCalls;
    const baselineCompletedCalls = completedCalls;

    MockEventSource.emit("task_state", { task_id: "active-1", state: "completed" });
    await flushAsyncEffects();

    expect(activeCalls).toBe(baselineActiveCalls + 1);
    expect(completedCalls).toBe(baselineCompletedCalls + 1);
  });

  it("reconciles task lists when receiving non-terminal events for unknown task ids", async () => {
    await renderDashboard();
    const baselineActiveCalls = activeCalls;
    const baselineCompletedCalls = completedCalls;

    MockEventSource.emit("task_state", { task_id: "unknown-task", state: "QUEUED" });
    await flushAsyncEffects();

    expect(activeCalls).toBe(baselineActiveCalls + 1);
    expect(completedCalls).toBe(baselineCompletedCalls + 1);
  });

  it("forces completed refresh when visibility changes back to visible", async () => {
    await renderDashboard();
    const baselineCompletedCalls = completedCalls;

    visibilityState = "hidden";
    fireEvent(document, new Event("visibilitychange"));
    expect(completedCalls).toBe(baselineCompletedCalls);

    visibilityState = "visible";
    fireEvent(document, new Event("visibilitychange"));
    await flushAsyncEffects();
    expect(completedCalls).toBe(baselineCompletedCalls + 1);
  });

  it("forces completed refresh after active task pause action", async () => {
    await renderDashboard();
    const baselineActive = activeCalls;
    const baselineCompleted = completedCalls;

    const pauseButton = screen.getByRole("button", { name: "Pause" });
    fireEvent.click(pauseButton);

    await flushAsyncEffects();
    expect(pauseCalls).toBe(1);
    expect(activeCalls).toBeGreaterThan(baselineActive);
    expect(completedCalls).toBeGreaterThan(baselineCompleted);
  }, 15_000);

  it("shows mobile provider toggles and keeps desktop selector desktop-only", async () => {
    await renderDashboard();

    const mobileProviderSelector = screen.getByTestId("provider-selector-mobile");
    const desktopProviderSelector = screen.getByTestId("provider-selector-desktop");

    expect(mobileProviderSelector.className).toContain("md:hidden");
    expect(desktopProviderSelector.className).toContain("hidden");
    expect(desktopProviderSelector.className).toContain("md:flex");

    const srtToggle = within(mobileProviderSelector).getByRole("button", { name: "SRT" });
    expect(srtToggle).toHaveAttribute("aria-pressed", "true");

    fireEvent.click(srtToggle);
    expect(srtToggle).toHaveAttribute("aria-pressed", "false");
  });

  it("shows no-schedules notice (not error) when search returns empty schedules with provider errors", async () => {
    searchStatus = 200;
    searchBody = {
      schedules: [],
      provider_errors: {
        SRT: {
          error_code: "provider_unreachable",
          error_message: "temporary provider error",
        },
      },
    };

    await renderDashboard();
    const searchButton = screen.getByRole("button", { name: "Search" });
    fireEvent.click(searchButton);
    await flushAsyncEffects();

    expect(screen.getByText("No schedules in this window.")).toBeInTheDocument();
    expect(screen.queryByText("SRT: temporary provider error")).not.toBeInTheDocument();
  });

  it("keeps search panel visible after search and keeps Modify Search available", async () => {
    searchStatus = 200;
    searchBody = {
      schedules: [
        {
          schedule_id: "SRT-301",
          provider: "SRT",
          departure_at: "2026-02-15T13:10:00+09:00",
          arrival_at: "2026-02-15T15:20:00+09:00",
          train_no: "301",
          dep: "수서",
          arr: "부산",
          availability: { general: true, special: false },
          metadata: {},
        },
      ],
    };

    await renderDashboard();
    expect(screen.getByTestId("train-search-form")).toBeInTheDocument();
    expect(screen.queryByTestId("search-summary-inline")).not.toBeInTheDocument();

    fireEvent.click(screen.getByRole("button", { name: "Search" }));
    await flushAsyncEffects();

    expect(screen.getByTestId("train-search-form")).toBeInTheDocument();
    expect(screen.getByTestId("search-summary-inline")).toBeInTheDocument();

    fireEvent.click(screen.getByRole("button", { name: "Modify Search" }));
    await flushAsyncEffects();

    expect(screen.getByTestId("train-search-form")).toBeInTheDocument();
  });

  it("hides auto-pay badge and toggle while auto-pay feature is disabled", async () => {
    await renderDashboard();

    expect(screen.queryByText("Wallet required for auto-pay.")).not.toBeInTheDocument();
    expect(screen.queryByRole("button", { name: "Auto-pay" })).not.toBeInTheDocument();
  });

  it("keeps selected schedule date tied to last search results, not unsaved calendar edits", async () => {
    searchStatus = 200;
    searchBody = {
      schedules: [
        {
          schedule_id: "SRT-301",
          provider: "SRT",
          departure_at: "2026-02-15T13:10:00+09:00",
          arrival_at: "2026-02-15T15:20:00+09:00",
          train_no: "301",
          dep: "수서",
          arr: "부산",
          availability: { general: true, special: false },
          metadata: {},
        },
      ],
    };

    await renderDashboard();

    fireEvent.change(screen.getByLabelText("Date"), { target: { value: "2026-02-15" } });
    fireEvent.click(screen.getByRole("button", { name: "Search" }));
    await flushAsyncEffects();

    expect(screen.getByRole("heading", { name: "Select schedules (02/15/2026)" })).toBeInTheDocument();

    fireEvent.change(screen.getByLabelText("Date"), { target: { value: "2026-02-20" } });

    expect(screen.getByRole("heading", { name: "Select schedules (02/15/2026)" })).toBeInTheDocument();
    expect(screen.queryByRole("heading", { name: "Select schedules (02/20/2026)" })).not.toBeInTheDocument();
  });

  it("shows mobile schedule cards and keeps desktop schedule table desktop-only", async () => {
    searchStatus = 200;
    searchBody = {
      schedules: [
        {
          schedule_id: "KTX-301",
          provider: "KTX",
          departure_at: "2026-02-15T13:10:00+09:00",
          arrival_at: "2026-02-15T15:20:00+09:00",
          train_no: "301",
          dep: "수서",
          arr: "부산",
          availability: { general: true, special: false },
          metadata: { train_type_name: "KTX-산천" },
        },
        {
          schedule_id: "KTX-302",
          provider: "KTX",
          departure_at: "2026-02-15T14:10:00+09:00",
          arrival_at: "2026-02-15T16:20:00+09:00",
          train_no: "302",
          dep: "수서",
          arr: "부산",
          availability: { general: true, special: true },
          metadata: { train_type_name: "KTX-산천" },
        },
      ],
    };

    await renderDashboard();
    fireEvent.click(screen.getByRole("button", { name: "Search" }));
    await flushAsyncEffects();

    const mobileSelector = screen.getByTestId("schedule-selector-mobile");
    const desktopSelector = screen.getByTestId("schedule-selector-desktop");
    expect(mobileSelector.className).toContain("md:hidden");
    expect(desktopSelector.className).toContain("hidden");
    expect(desktopSelector.className).toContain("md:block");

    const mobileCard = within(mobileSelector).getByRole("button", { name: /KTX-산천 301/ });
    const mobileCardSecond = within(mobileSelector).getByRole("button", { name: /KTX-산천 302/ });
    expect(mobileCard).toHaveAttribute("aria-pressed", "false");
    fireEvent.click(mobileCard);
    expect(mobileCard).toHaveAttribute("aria-pressed", "true");
    const firstIndicatorSingle = mobileCard.querySelector("span[aria-hidden='true'].inline-flex.h-7.w-7") as HTMLSpanElement | null;
    expect(firstIndicatorSingle?.textContent).toBe("✓");

    fireEvent.click(mobileCardSecond);
    const firstIndicatorRanked = mobileCard.querySelector("span[aria-hidden='true'].inline-flex.h-7.w-7") as HTMLSpanElement | null;
    const secondIndicatorRanked = mobileCardSecond.querySelector("span[aria-hidden='true'].inline-flex.h-7.w-7") as HTMLSpanElement | null;
    expect(firstIndicatorRanked?.textContent).toBe("1");
    expect(secondIndicatorRanked?.textContent).toBe("2");
  });

  it("renders train type from metadata code plus train number in search results", async () => {
    searchStatus = 200;
    searchBody = {
      schedules: [
        {
          schedule_id: "KTX-305",
          provider: "KTX",
          departure_at: "2026-02-16T08:30:00+09:00",
          arrival_at: "2026-02-16T10:50:00+09:00",
          train_no: "305",
          dep: "수서",
          arr: "부산",
          availability: { general: true, special: true },
          metadata: { train_type_code: "07" },
        },
      ],
    };

    await renderDashboard();
    fireEvent.click(screen.getByRole("button", { name: "Search" }));
    await flushAsyncEffects();

    expect(screen.getByRole("button", { name: "KTX-산천 305" })).toBeInTheDocument();
    expect(screen.queryByRole("button", { name: "KTX 305" })).not.toBeInTheDocument();
  });

  it("accepts direct passenger count input with bounds", async () => {
    await renderDashboard();

    const adultInput = screen.getByRole("spinbutton", { name: "Adults" }) as HTMLInputElement;
    const childInput = screen.getByRole("spinbutton", { name: "Children" }) as HTMLInputElement;

    fireEvent.change(childInput, { target: { value: "2" } });
    fireEvent.change(adultInput, { target: { value: "0" } });

    expect(adultInput.value).toBe("0");
    expect(childInput.value).toBe("2");

    fireEvent.change(adultInput, { target: { value: "12" } });
    fireEvent.change(childInput, { target: { value: "-1" } });

    expect(adultInput.value).toBe("9");
    expect(childInput.value).toBe("0");
  });

  it("uses +/- passenger controls on mobile layout", async () => {
    await renderDashboard();

    const decAdults = screen.getByRole("button", { name: "Decrease adults" });
    const incAdults = screen.getByRole("button", { name: "Increase adults" });
    const decChildren = screen.getByRole("button", { name: "Decrease children" });
    const incChildren = screen.getByRole("button", { name: "Increase children" });

    expect(screen.getByTestId("adults-count-mobile")).toHaveTextContent("1");
    expect(screen.getByTestId("children-count-mobile")).toHaveTextContent("0");
    expect(decAdults).toBeDisabled();
    expect(decChildren).toBeDisabled();

    fireEvent.click(incChildren);
    expect(screen.getByTestId("children-count-mobile")).toHaveTextContent("1");
    expect(decAdults).not.toBeDisabled();

    fireEvent.click(decAdults);
    expect(screen.getByTestId("adults-count-mobile")).toHaveTextContent("0");
    expect(screen.getByTestId("children-count-mobile")).toHaveTextContent("1");
  });
});
