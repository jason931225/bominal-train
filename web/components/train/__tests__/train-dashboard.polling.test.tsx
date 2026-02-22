import React from "react";

import { act, fireEvent, render, screen } from "@testing-library/react";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";

import { LocaleProvider } from "@/components/locale-provider";
import { TrainDashboard } from "@/components/train/train-dashboard";
import type { TrainTaskSummary } from "@/lib/types";

const POLL_MS = 12_000;

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
  let visibilityState: DocumentVisibilityState = "visible";

  beforeEach(() => {
    activeCalls = 0;
    completedCalls = 0;
    pauseCalls = 0;
    visibilityState = "visible";
    vi.useFakeTimers();

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
              cvv_cached_until: null,
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
            return new Response(JSON.stringify({ tasks: [makeTask("active-1", "RUNNING")] }), {
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
        return new Response(JSON.stringify({ detail: "not found" }), {
          status: 404,
          headers: { "Content-Type": "application/json" },
        });
      }),
    );
  });

  afterEach(() => {
    vi.unstubAllGlobals();
    vi.useRealTimers();
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

  it("fetches completed tasks every third poll tick while active tasks fetch every tick", async () => {
    await renderDashboard();
    expect(activeCalls).toBe(1);
    expect(completedCalls).toBe(1);

    await act(async () => {
      vi.advanceTimersByTime(POLL_MS);
      await Promise.resolve();
      await Promise.resolve();
    });
    expect(activeCalls).toBe(2);
    expect(completedCalls).toBe(1);

    await act(async () => {
      vi.advanceTimersByTime(POLL_MS);
      await Promise.resolve();
      await Promise.resolve();
    });
    expect(activeCalls).toBe(3);
    expect(completedCalls).toBe(1);

    await act(async () => {
      vi.advanceTimersByTime(POLL_MS);
      await Promise.resolve();
      await Promise.resolve();
    });
    expect(activeCalls).toBe(4);
    expect(completedCalls).toBe(2);
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
  });
});
