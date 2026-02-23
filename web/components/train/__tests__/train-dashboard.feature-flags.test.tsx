import React from "react";

import { act, fireEvent, render, screen } from "@testing-library/react";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import type { TrainSchedule, TrainTaskSummary } from "@/lib/types";

function makeTask(id: string, state: TrainTaskSummary["state"], overrides: Partial<TrainTaskSummary> = {}): TrainTaskSummary {
  return {
    id,
    module: "train",
    state,
    deadline_at: "2026-02-22T12:00:00+09:00",
    created_at: "2026-02-22T11:00:00+09:00",
    updated_at: "2026-02-22T11:30:00+09:00",
    paused_at: null,
    cancelled_at: null,
    completed_at: state === "COMPLETED" ? "2026-02-22T11:40:00+09:00" : null,
    failed_at: null,
    hidden_at: null,
    last_attempt_at: "2026-02-22T11:45:00+09:00",
    last_attempt_action: "SEARCH",
    last_attempt_ok: true,
    last_attempt_error_code: null,
    last_attempt_error_message_safe: null,
    last_attempt_finished_at: "2026-02-22T11:45:10+09:00",
    next_run_at: "2026-02-22T11:46:00+09:00",
    retry_now_allowed: true,
    retry_now_reason: null,
    retry_now_available_at: null,
    spec_json: {
      dep: "수서",
      arr: "부산",
      date: "2026-02-22",
      passengers: { adults: 1, children: 0 },
      selected_trains_ranked: [{ rank: 1, departure_at: "2026-02-22T12:30:00+09:00" }],
    },
    ticket_status: null,
    ticket_paid: null,
    ticket_payment_deadline_at: null,
    ticket_reservation_id: null,
    ...overrides,
  };
}

function makeSchedule(overrides: Partial<TrainSchedule> = {}): TrainSchedule {
  return {
    schedule_id: "SRT-301",
    provider: "SRT",
    departure_at: "2026-02-22T12:30:00+09:00",
    arrival_at: "2026-02-22T15:00:00+09:00",
    train_no: "301",
    dep: "수서",
    arr: "부산",
    availability: { general: true, special: false },
    metadata: {},
    ...overrides,
  };
}

function jsonResponse(payload: unknown, status = 200): Response {
  return new Response(JSON.stringify(payload), {
    status,
    headers: { "Content-Type": "application/json" },
  });
}

async function flushAsyncEffects() {
  await act(async () => {
    await Promise.resolve();
    await Promise.resolve();
  });
}

describe("TrainDashboard feature-flag and fallback branches", () => {
  beforeEach(() => {
    vi.useFakeTimers();
    vi.stubGlobal("confirm", vi.fn(() => true));
  });

  afterEach(() => {
    vi.useRealTimers();
    vi.unstubAllGlobals();
    vi.unstubAllEnvs();
    vi.restoreAllMocks();
  });

  it("covers auto-pay enabled flow and completed awaiting-payment controls", async () => {
    vi.stubEnv("NEXT_PUBLIC_TRAIN_AUTO_PAY_ENABLED", "true");
    vi.resetModules();
    const { LocaleProvider } = await import("@/components/locale-provider");
    const { TrainDashboard } = await import("@/components/train/train-dashboard");

    const activeTasks: TrainTaskSummary[] = [];
    const completedTasks: TrainTaskSummary[] = [
      makeTask("completed-awaiting", "COMPLETED", {
        ticket_status: "awaiting_payment",
        ticket_paid: false,
        ticket_payment_deadline_at: "2026-02-22T13:00:00+09:00",
      }),
      makeTask("completed-plain", "COMPLETED", {
        ticket_status: "cancelled",
        ticket_paid: false,
      }),
    ];
    const schedules = [makeSchedule()];
    let createTaskAutoPay: boolean | null = null;

    const fetchMock = vi.fn(async (input: RequestInfo | URL, init?: RequestInit) => {
      const url = String(input);
      const method = init?.method ?? "GET";

      if (url.includes("/api/train/credentials/status")) {
        return jsonResponse({
          ktx: { configured: true, verified: true, username: "01012345678", verified_at: null, detail: null },
          srt: { configured: true, verified: true, username: "srt-user", verified_at: null, detail: null },
        });
      }
      if (url.includes("/api/wallet/payment-card")) {
        return jsonResponse({
          configured: true,
          card_masked: "****-****-****-1234",
          expiry_month: 12,
          expiry_year: 2030,
          updated_at: "2026-02-22T10:00:00+09:00",
          cvv_cached_until: null,
          detail: null,
        });
      }
      if (url.includes("/api/train/stations")) {
        return jsonResponse({
          stations: [
            { name: "수서", srt_code: "0551", srt_supported: true },
            { name: "부산", srt_code: "0020", srt_supported: true },
          ],
        });
      }
      if (url.includes("/api/train/tasks?status=active")) {
        return jsonResponse({ tasks: activeTasks });
      }
      if (url.includes("/api/train/tasks?status=completed")) {
        return jsonResponse({ tasks: completedTasks });
      }
      if (url.includes("/api/train/search") && method === "POST") {
        return jsonResponse({ schedules });
      }
      if (url.endsWith("/api/train/tasks") && method === "POST") {
        const payload = JSON.parse(String(init?.body ?? "{}")) as { auto_pay?: boolean };
        createTaskAutoPay = Boolean(payload.auto_pay);
        return jsonResponse({ task: makeTask("dedup-task", "QUEUED"), deduplicated: true });
      }
      if (url.includes("/api/train/tasks/completed-awaiting/pay") && method === "POST") {
        return jsonResponse({ detail: "Payment processed." });
      }
      if (url.includes("/api/train/tasks/completed-awaiting") && method === "GET") {
        return jsonResponse({
          artifacts: [{ id: "ticket-completed", module: "train", kind: "ticket", data_json_safe: {}, created_at: "2026-02-22T11:00:00+09:00" }],
        });
      }
      if (url.includes("/api/train/tickets/ticket-completed/cancel") && method === "POST") {
        return jsonResponse({ detail: "Ticket cancellation request completed." });
      }
      return jsonResponse({ detail: "not found" }, 404);
    });
    vi.stubGlobal("fetch", fetchMock);
    vi.spyOn(window, "confirm").mockReturnValue(true);

    render(
      <LocaleProvider initialLocale="en">
        <TrainDashboard />
      </LocaleProvider>,
    );
    await flushAsyncEffects();

    fireEvent.click(screen.getByRole("button", { name: "Search" }));
    await flushAsyncEffects();
    fireEvent.click(screen.getByRole("button", { name: "SRT 301" }));

    const autoPaySwitch = screen.getByRole("switch", { name: "Auto-pay" });
    fireEvent.click(autoPaySwitch);
    fireEvent.click(autoPaySwitch);
    fireEvent.click(screen.getByRole("button", { name: "Create Task" }));
    await flushAsyncEffects();
    expect(screen.getByText("Task already active (deduplicated).")).toBeInTheDocument();
    expect(createTaskAutoPay).toBe(true);

    const completedPayButton = screen.getAllByRole("button", { name: "Pay" })[0];
    fireEvent.click(completedPayButton);
    await flushAsyncEffects();
    expect(fetchMock).toHaveBeenCalledWith(
      expect.stringContaining("/api/train/tasks/completed-awaiting/pay"),
      expect.objectContaining({ method: "POST" }),
    );

    fireEvent.click(screen.getAllByRole("button", { name: "Cancel" })[0]);
    await flushAsyncEffects();
    expect(fetchMock).toHaveBeenCalledWith(
      expect.stringContaining("/api/train/tickets/ticket-completed/cancel"),
      expect.objectContaining({ method: "POST" }),
    );

    // Hidden visibility state should short-circuit polling tick handler.
    Object.defineProperty(document, "visibilityState", { configurable: true, value: "hidden" });
    await act(async () => {
      vi.advanceTimersByTime(60_000);
      await Promise.resolve();
    });
  }, 20_000);

  it("covers credential/wallet/station failure fallbacks with auto-pay flag enabled", async () => {
    vi.stubEnv("NEXT_PUBLIC_TRAIN_AUTO_PAY_ENABLED", "true");
    vi.resetModules();
    const { LocaleProvider } = await import("@/components/locale-provider");
    const { TrainDashboard } = await import("@/components/train/train-dashboard");

    const fetchMock = vi.fn(async (input: RequestInfo | URL) => {
      const url = String(input);
      if (url.includes("/api/train/credentials/status")) {
        return jsonResponse({ detail: "credential status failed" }, 500);
      }
      if (url.includes("/api/wallet/payment-card")) {
        return jsonResponse({ detail: "wallet failed" }, 500);
      }
      if (url.includes("/api/train/stations")) {
        return jsonResponse({ detail: "stations failed" }, 500);
      }
      if (url.includes("/api/train/tasks?status=active")) {
        return jsonResponse({ tasks: [] });
      }
      if (url.includes("/api/train/tasks?status=completed")) {
        return jsonResponse({ tasks: [] });
      }
      return jsonResponse({ detail: "not found" }, 404);
    });
    vi.stubGlobal("fetch", fetchMock);

    render(
      <LocaleProvider initialLocale="en">
        <TrainDashboard />
      </LocaleProvider>,
    );
    await flushAsyncEffects();

    expect(screen.getByText("Could not load provider credential status.")).toBeInTheDocument();
  });

  it("shows wallet status error when wallet load fails without an existing error", async () => {
    vi.stubEnv("NEXT_PUBLIC_TRAIN_AUTO_PAY_ENABLED", "true");
    vi.resetModules();
    const { LocaleProvider } = await import("@/components/locale-provider");
    const { TrainDashboard } = await import("@/components/train/train-dashboard");

    const fetchMock = vi.fn(async (input: RequestInfo | URL) => {
      const url = String(input);
      if (url.includes("/api/train/credentials/status")) {
        return jsonResponse({
          ktx: { configured: true, verified: true, username: "01012345678", verified_at: null, detail: null },
          srt: { configured: true, verified: true, username: "srt-user", verified_at: null, detail: null },
        });
      }
      if (url.includes("/api/wallet/payment-card")) {
        return jsonResponse({ detail: "wallet failed" }, 500);
      }
      if (url.includes("/api/train/stations")) {
        return jsonResponse({
          stations: [
            { name: "수서", srt_code: "0551", srt_supported: true },
            { name: "부산", srt_code: "0020", srt_supported: true },
          ],
        });
      }
      if (url.includes("/api/train/tasks?status=active")) {
        return jsonResponse({ tasks: [] });
      }
      if (url.includes("/api/train/tasks?status=completed")) {
        return jsonResponse({ tasks: [] });
      }
      return jsonResponse({ detail: "not found" }, 404);
    });
    vi.stubGlobal("fetch", fetchMock);

    render(
      <LocaleProvider initialLocale="en">
        <TrainDashboard />
      </LocaleProvider>,
    );
    await flushAsyncEffects();

    expect(screen.getByText("Could not load wallet status.")).toBeInTheDocument();
  });

  it("defaults auto-pay feature to disabled when env key is unset", async () => {
    const originalFlag = process.env.NEXT_PUBLIC_TRAIN_AUTO_PAY_ENABLED;
    try {
      delete process.env.NEXT_PUBLIC_TRAIN_AUTO_PAY_ENABLED;
      vi.resetModules();
      const { LocaleProvider } = await import("@/components/locale-provider");
      const { TrainDashboard } = await import("@/components/train/train-dashboard");

      const fetchMock = vi.fn(async (input: RequestInfo | URL, init?: RequestInit) => {
        const url = String(input);
        const method = init?.method ?? "GET";

        if (url.includes("/api/train/credentials/status")) {
          return jsonResponse({
            ktx: { configured: true, verified: true, username: "01012345678", verified_at: null, detail: null },
            srt: { configured: true, verified: true, username: "srt-user", verified_at: null, detail: null },
          });
        }
        if (url.includes("/api/wallet/payment-card")) {
          return jsonResponse({
            configured: true,
            card_masked: "****-****-****-1234",
            expiry_month: 12,
            expiry_year: 2030,
            updated_at: "2026-02-22T10:00:00+09:00",
            cvv_cached_until: null,
            detail: null,
          });
        }
        if (url.includes("/api/train/stations")) {
          return jsonResponse({
            stations: [
              { name: "수서", srt_code: "0551", srt_supported: true },
              { name: "부산", srt_code: "0020", srt_supported: true },
            ],
          });
        }
        if (url.includes("/api/train/tasks?status=active")) {
          return jsonResponse({ tasks: [] });
        }
        if (url.includes("/api/train/tasks?status=completed")) {
          return jsonResponse({ tasks: [] });
        }
        if (url.includes("/api/train/search") && method === "POST") {
          return jsonResponse({ schedules: [makeSchedule()] });
        }
        return jsonResponse({ detail: "not found" }, 404);
      });
      vi.stubGlobal("fetch", fetchMock);

      render(
        <LocaleProvider initialLocale="en">
          <TrainDashboard />
        </LocaleProvider>,
      );
      await flushAsyncEffects();

      fireEvent.click(screen.getByRole("button", { name: "Search" }));
      await flushAsyncEffects();
      fireEvent.click(screen.getByRole("button", { name: "SRT 301" }));

      expect(screen.queryByRole("switch", { name: "Auto-pay" })).not.toBeInTheDocument();
    } finally {
      if (originalFlag !== undefined) {
        process.env.NEXT_PUBLIC_TRAIN_AUTO_PAY_ENABLED = originalFlag;
      } else {
        delete process.env.NEXT_PUBLIC_TRAIN_AUTO_PAY_ENABLED;
      }
    }
  });

  it("shows the wallet-required warning when auto-pay is enabled but no card is configured", async () => {
    vi.stubEnv("NEXT_PUBLIC_TRAIN_AUTO_PAY_ENABLED", "true");
    vi.resetModules();
    const { LocaleProvider } = await import("@/components/locale-provider");
    const { TrainDashboard } = await import("@/components/train/train-dashboard");

    const fetchMock = vi.fn(async (input: RequestInfo | URL, init?: RequestInit) => {
      const url = String(input);
      const method = init?.method ?? "GET";

      if (url.includes("/api/train/credentials/status")) {
        return jsonResponse({
          ktx: { configured: true, verified: true, username: "01012345678", verified_at: null, detail: null },
          srt: { configured: true, verified: true, username: "srt-user", verified_at: null, detail: null },
        });
      }
      if (url.includes("/api/wallet/payment-card")) {
        return jsonResponse({
          configured: false,
          card_masked: null,
          expiry_month: null,
          expiry_year: null,
          updated_at: null,
          cvv_cached_until: null,
          detail: null,
        });
      }
      if (url.includes("/api/train/stations")) {
        return jsonResponse({
          stations: [
            { name: "수서", srt_code: "0551", srt_supported: true },
            { name: "부산", srt_code: "0020", srt_supported: true },
          ],
        });
      }
      if (url.includes("/api/train/tasks?status=active")) {
        return jsonResponse({ tasks: [] });
      }
      if (url.includes("/api/train/tasks?status=completed")) {
        return jsonResponse({ tasks: [] });
      }
      if (url.includes("/api/train/search") && method === "POST") {
        return jsonResponse({ schedules: [makeSchedule()] });
      }
      return jsonResponse({ detail: "not found" }, 404);
    });
    vi.stubGlobal("fetch", fetchMock);

    render(
      <LocaleProvider initialLocale="en">
        <TrainDashboard />
      </LocaleProvider>,
    );
    await flushAsyncEffects();

    fireEvent.click(screen.getByRole("button", { name: "Search" }));
    await flushAsyncEffects();
    fireEvent.click(screen.getByRole("button", { name: "SRT 301" }));

    expect(screen.getByRole("switch", { name: "Auto-pay" })).toBeDisabled();
    expect(screen.getByText("Wallet required for auto-pay.")).toBeInTheDocument();
    expect(screen.getByRole("link", { name: "Payment settings" })).toHaveAttribute("href", "/settings/payment");
  });

  it("hides dummy task tools when loaded in production mode", async () => {
    vi.stubEnv("NODE_ENV", "production");
    vi.resetModules();
    const { LocaleProvider } = await import("@/components/locale-provider");
    const { TrainDashboard } = await import("@/components/train/train-dashboard");

    const fetchMock = vi.fn(async (input: RequestInfo | URL) => {
      const url = String(input);
      if (url.includes("/api/train/credentials/status")) {
        return jsonResponse({
          ktx: { configured: true, verified: true, username: "01012345678", verified_at: null, detail: null },
          srt: { configured: true, verified: true, username: "srt-user", verified_at: null, detail: null },
        });
      }
      if (url.includes("/api/wallet/payment-card")) {
        return jsonResponse({
          configured: true,
          card_masked: "****-****-****-1234",
          expiry_month: 12,
          expiry_year: 2030,
          updated_at: "2026-02-22T10:00:00+09:00",
          cvv_cached_until: null,
          detail: null,
        });
      }
      if (url.includes("/api/train/stations")) {
        return jsonResponse({
          stations: [
            { name: "수서", srt_code: "0551", srt_supported: true },
            { name: "부산", srt_code: "0020", srt_supported: true },
          ],
        });
      }
      if (url.includes("/api/train/tasks?status=active")) {
        return jsonResponse({ tasks: [] });
      }
      if (url.includes("/api/train/tasks?status=completed")) {
        return jsonResponse({ tasks: [] });
      }
      return jsonResponse({ detail: "not found" }, 404);
    });
    vi.stubGlobal("fetch", fetchMock);

    render(
      <LocaleProvider initialLocale="en">
        <TrainDashboard />
      </LocaleProvider>,
    );
    await flushAsyncEffects();

    expect(screen.queryByRole("button", { name: "Load dummy task cards" })).not.toBeInTheDocument();
    expect(screen.queryByText(/Dev test tools:/i)).not.toBeInTheDocument();
  });
});
