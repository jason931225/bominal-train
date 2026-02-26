import React from "react";

import { act, fireEvent, render, screen, within } from "@testing-library/react";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";

import { LocaleProvider } from "@/components/locale-provider";
import { clearTrainDashboardFetchCaches, TrainDashboard } from "@/components/train/train-dashboard";
import type { TrainSchedule, TrainTaskSummary } from "@/lib/types";

function makeTask(id: string, state: TrainTaskSummary["state"], overrides: Partial<TrainTaskSummary> = {}): TrainTaskSummary {
  return {
    id,
    module: "train",
    state,
    deadline_at: "2026-02-22T12:00:00+09:00",
    created_at: "2026-02-22T11:00:00+09:00",
    updated_at: "2026-02-22T11:30:00+09:00",
    paused_at: state === "PAUSED" ? "2026-02-22T11:20:00+09:00" : null,
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
    ticket_train_no: null,
    ticket_seat_count: null,
    ticket_seats: null,
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

function createDeferred<T>() {
  let resolve!: (value: T | PromiseLike<T>) => void;
  let reject!: (reason?: unknown) => void;
  const promise = new Promise<T>((res, rej) => {
    resolve = res;
    reject = rej;
  });
  return { promise, resolve, reject };
}

async function flushAsyncEffects() {
  await act(async () => {
    await Promise.resolve();
    await Promise.resolve();
  });
}

describe("TrainDashboard action flows", () => {
  let activeTasks: TrainTaskSummary[];
  let completedTasks: TrainTaskSummary[];
  let schedules: TrainSchedule[];
  let fetchMock: ReturnType<typeof vi.fn>;

  beforeEach(() => {
    clearTrainDashboardFetchCaches();
    vi.useFakeTimers();
    vi.stubGlobal("confirm", vi.fn(() => true));
    activeTasks = [];
    completedTasks = [];
    schedules = [];
    fetchMock = vi.fn(async (input: RequestInfo | URL, init?: RequestInit) => {
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
          detail: null,
        });
      }
      if (url.includes("/api/train/stations")) {
        return jsonResponse({
          stations: [
            { name: "수서", srt_code: "0551", srt_supported: true },
            { name: "부산", srt_code: "0020", srt_supported: true },
            { name: "동탄", srt_code: "0552", srt_supported: true },
          ],
        });
      }
      if (url.includes("/api/train/tasks?")) {
        const parsed = new URL(url, "http://localhost");
        const status = parsed.searchParams.get("status");
        if (status === "active") return jsonResponse({ tasks: activeTasks });
        if (status === "completed") return jsonResponse({ tasks: completedTasks });
      }
      if (url.includes("/api/train/search") && method === "POST") {
        return jsonResponse({ schedules });
      }
      if (url.includes("/api/train/tasks/") && method === "GET") {
        return jsonResponse({
          artifacts: [{ id: "ticket-artifact-1", module: "train", kind: "ticket", data_json_safe: {}, created_at: "2026-02-22T11:00:00+09:00" }],
        });
      }
      if (url.includes("/api/train/tasks/duplicate-check") && method === "POST") {
        return jsonResponse({
          has_duplicate: false,
          summary: { already_reserved: 0, waiting: 0, polling: 0 },
          matches: [],
        });
      }
      if (url.includes("/api/train/tasks/") && method === "POST") {
        return jsonResponse({ ok: true, detail: "ok" });
      }
      if (url.includes("/api/train/tickets/") && method === "POST") {
        return jsonResponse({ detail: "Ticket cancellation request completed." });
      }
      if (url.includes("/api/train/tasks") && method === "POST") {
        return jsonResponse({
          task: makeTask("new-task", "QUEUED"),
          deduplicated: false,
        });
      }

      return jsonResponse({ detail: "not found" }, 404);
    });
    vi.stubGlobal("fetch", fetchMock);
  });

  afterEach(() => {
    clearTrainDashboardFetchCaches();
    vi.useRealTimers();
    vi.unstubAllGlobals();
    vi.restoreAllMocks();
  });

  async function renderDashboard() {
    render(
      <LocaleProvider initialLocale="en">
        <TrainDashboard />
      </LocaleProvider>,
    );
    await flushAsyncEffects();
  }

  function getTaskCard(taskId: string): HTMLElement {
    const detailLink = document.querySelector<HTMLAnchorElement>(`a[href="/modules/train/tasks/${taskId}"]`);
    expect(detailLink).not.toBeNull();
    const card = detailLink?.closest("li");
    expect(card).not.toBeNull();
    return card as HTMLElement;
  }

  it("renders unverified provider as disabled in the desktop selector", async () => {
    fetchMock.mockImplementation(async (input: RequestInfo | URL, init?: RequestInit) => {
      const url = String(input);
      const method = init?.method ?? "GET";

      if (url.includes("/api/train/credentials/status")) {
        return jsonResponse({
          ktx: { configured: true, verified: true, username: "01012345678", verified_at: null, detail: null },
          srt: { configured: true, verified: false, username: null, verified_at: null, detail: "needs login" },
        });
      }
      if (url.includes("/api/wallet/payment-card")) {
        return jsonResponse({
          configured: true,
          card_masked: "****-****-****-1234",
          expiry_month: 12,
          expiry_year: 2030,
          updated_at: "2026-02-22T10:00:00+09:00",
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
      if (url.includes("/api/train/tasks?")) {
        return jsonResponse({ tasks: [] });
      }
      if (url.includes("/api/train/search") && method === "POST") {
        return jsonResponse({ schedules: [] });
      }
      return jsonResponse({ detail: "not found" }, 404);
    });

    await renderDashboard();

    expect(screen.getByText("Connect to a provider to unlock search.")).toBeInTheDocument();
    const providerDesktop = within(screen.getByTestId("provider-selector-desktop"));
    expect(providerDesktop.getByRole("button", { name: "SRT" })).toBeDisabled();
    expect(providerDesktop.queryByRole("button", { name: "KTX" })).not.toBeInTheDocument();
  }, 45_000);

  it("renders sold-out availability badges and ignores non-activating desktop keydown", async () => {
    schedules = [
      makeSchedule({
        availability: {
          general: false,
          special: true,
        },
      }),
    ];

    await renderDashboard();

    fireEvent.click(screen.getByRole("button", { name: "Search" }));
    await flushAsyncEffects();

    const mobileSelector = screen.getByTestId("schedule-selector-mobile");
    expect(within(mobileSelector).getByTitle("General sold out")).toBeInTheDocument();

    const desktopSelector = screen.getByTestId("schedule-selector-desktop");
    expect(within(desktopSelector).getByTitle("General sold out")).toBeInTheDocument();

    const desktopRow = within(desktopSelector).getAllByRole("button")[0];
    expect(desktopRow).toHaveAttribute("aria-pressed", "false");
    fireEvent.keyDown(desktopRow, { key: "Escape" });
    expect(desktopRow).toHaveAttribute("aria-pressed", "false");
  });

  it("renders task error and completed-time fallbacks", async () => {
    activeTasks = [
      makeTask("active-message", "RUNNING", {
        last_attempt_ok: false,
        last_attempt_error_message_safe: "Friendly failure",
        last_attempt_error_code: "seat_unavailable",
      }),
      makeTask("active-code", "RUNNING", {
        last_attempt_ok: false,
        last_attempt_error_message_safe: null,
        last_attempt_error_code: "seat_unavailable",
      }),
      makeTask("active-soldout", "RUNNING", {
        last_attempt_ok: false,
        last_attempt_error_message_safe: null,
        last_attempt_error_code: "sold_out",
      }),
      makeTask("active-html", "POLLING", {
        last_attempt_ok: false,
        last_attempt_error_message_safe: "예약상태가 변경되었습니다.<br>승차권내역 및 예약내역을 반드시 확인하여 주십시오.",
        last_attempt_error_code: "provider_error",
      }),
      makeTask("active-unknown", "RUNNING", {
        last_attempt_ok: false,
        last_attempt_error_message_safe: null,
        last_attempt_error_code: null,
      }),
    ];
    completedTasks = [
      makeTask("completed-no-time", "COMPLETED", {
        completed_at: null,
        ticket_train_no: "311",
        ticket_seats: ["8-12A"],
      }),
    ];

    await renderDashboard();

    const seatUnavailableErrors = screen.getAllByText("No reservable seats are available for this schedule.");
    const seatUnavailableError = seatUnavailableErrors[0];
    const soldOutError = seatUnavailableErrors[1];
    expect(seatUnavailableError).toBeInTheDocument();
    expect(soldOutError).toBeInTheDocument();
    expect(seatUnavailableErrors.length).toBeGreaterThanOrEqual(3);
    expect(seatUnavailableError.closest("p")).toHaveClass("text-amber-700");
    expect(soldOutError.closest("p")).toHaveClass("text-amber-700");
    expect(screen.getByText("Reservation status changed. Please verify your reservation details.")).toBeInTheDocument();
    expect(screen.queryByText(/<br>/)).not.toBeInTheDocument();
    expect(screen.getByText("Unknown error")).toBeInTheDocument();
    expect(screen.getByText(/Completed:\s*-/)).toBeInTheDocument();
    expect(screen.getByText("Train:")).toBeInTheDocument();
    expect(screen.getByText("311")).toBeInTheDocument();
    expect(screen.getByText("Seats:")).toBeInTheDocument();
    expect(screen.getByText("Car 8 - 12A")).toBeInTheDocument();
  });

  it("renders multi-schedule polling options with departure-arrival entries", async () => {
    activeTasks = [
      makeTask("polling-multi-expand", "POLLING", {
        spec_json: {
          dep: "수서",
          arr: "마산",
          date: "2026-02-27",
          passengers: { adults: 1, children: 0 },
          selected_trains_ranked: [
            { rank: 1, departure_at: "2026-02-27T19:24:00+09:00", arrival_at: "2026-02-27T21:44:00+09:00", provider: "SRT" },
            { rank: 2, departure_at: "2026-02-27T19:38:00+09:00", arrival_at: "2026-02-27T21:58:00+09:00", provider: "SRT" },
            { rank: 3, departure_at: "2026-02-27T19:56:00+09:00", arrival_at: "2026-02-27T22:16:00+09:00", provider: "SRT" },
          ],
        },
      }),
    ];
    completedTasks = [];

    await renderDashboard();

    expect(screen.getByText("SRT 19:24 - 21:44")).toBeInTheDocument();
    expect(screen.getByText("SRT 19:38 - 21:58")).toBeInTheDocument();
    expect(screen.getByText("SRT 19:56 - 22:16")).toBeInTheDocument();
    expect(screen.queryByRole("button", { name: "Show schedule options" })).not.toBeInTheDocument();
    expect(screen.queryByRole("button", { name: "Hide schedule options" })).not.toBeInTheDocument();
    expect(screen.queryByText("02/27/2026 19:38")).not.toBeInTheDocument();
  });

  it("renders active and completed task-card payload details with always-visible schedule rows", async () => {
    activeTasks = [
      makeTask("active-pending-detail", "COMPLETED", {
        ticket_status: "awaiting_payment",
        ticket_paid: false,
        last_attempt_ok: false,
        last_attempt_error_code: "provider_error",
        last_attempt_error_message_safe: "Manual provider warning",
        spec_json: {
          dep: "수서",
          arr: "부산",
          date: "2026-03-01",
          passengers: { adults: 2, children: 1 },
          selected_trains_ranked: [
            { rank: 1, departure_at: "2026-03-01T09:10:00+09:00", arrival_at: "2026-03-01T11:20:00+09:00", provider: "KTX" },
            { rank: 2, departure_at: "2026-03-01T09:20:00+09:00", arrival_at: "2026-03-01T11:30:00+09:00", provider: "KTX" },
            { rank: 3, departure_at: "2026-03-01T09:45:00+09:00", arrival_at: "2026-03-01T11:55:00+09:00", provider: "KTX" },
          ],
        },
      }),
    ];
    completedTasks = [
      makeTask("completed-confirmed-detail", "COMPLETED", {
        completed_at: "2026-03-01T12:30:00+09:00",
        ticket_status: "ticket_issued",
        ticket_paid: true,
        ticket_train_no: "311",
        ticket_seats: ["8-12A"],
        spec_json: {
          dep: "수서",
          arr: "부산",
          date: "2026-03-01",
          passengers: { adults: 1, children: 0 },
          selected_trains_ranked: [
            { rank: 1, departure_at: "2026-03-01T11:30:00+09:00", arrival_at: "2026-03-01T13:40:00+09:00", provider: "SRT" },
            { rank: 2, departure_at: "2026-03-01T11:45:00+09:00", arrival_at: "2026-03-01T13:55:00+09:00", provider: "SRT" },
          ],
        },
      }),
    ];

    await renderDashboard();

    const activeCard = getTaskCard("active-pending-detail");
    expect(within(activeCard).getByText("PENDING")).toBeInTheDocument();
    expect(within(activeCard).getByText("Awaiting Payment")).toBeInTheDocument();
    expect(within(activeCard).getByText("03/01/2026 (Sunday)")).toBeInTheDocument();
    expect(within(activeCard).getByText("09:10")).toBeInTheDocument();
    expect(within(activeCard).getByText("11:20")).toBeInTheDocument();
    expect(within(activeCard).getByText("Manual provider warning")).toBeInTheDocument();
    expect(within(activeCard).getByText("KTX 09:10 - 11:20")).toBeInTheDocument();
    expect(within(activeCard).getByText("KTX 09:20 - 11:30")).toBeInTheDocument();
    expect(within(activeCard).getByText("KTX 09:45 - 11:55")).toBeInTheDocument();
    expect(within(activeCard).queryByRole("button", { name: "Show schedule options" })).not.toBeInTheDocument();
    expect(within(activeCard).queryByRole("button", { name: "Hide schedule options" })).not.toBeInTheDocument();
    expect(within(activeCard).queryByText("03/01/2026 09:20")).not.toBeInTheDocument();

    const completedCard = getTaskCard("completed-confirmed-detail");
    expect(within(completedCard).getByText("COMPLETED")).toBeInTheDocument();
    expect(within(completedCard).getByText("Confirmed")).toBeInTheDocument();
    expect(within(completedCard).getByText(/Completed:\s*03\/01\/2026,\s*12:30 KST/)).toBeInTheDocument();
    expect(within(completedCard).getByText("03/01/2026 (Sunday)")).toBeInTheDocument();
    expect(within(completedCard).getByText("11:30")).toBeInTheDocument();
    expect(within(completedCard).getByText("13:40")).toBeInTheDocument();
    expect(within(completedCard).getByText("SRT 311")).toBeInTheDocument();
    expect(within(completedCard).getByText("Car 8 - 12A")).toBeInTheDocument();
    expect(within(completedCard).getByText("SRT 11:30 - 13:40")).toBeInTheDocument();
    expect(within(completedCard).getByText("SRT 11:45 - 13:55")).toBeInTheDocument();
    expect(within(completedCard).queryByRole("button", { name: "Show schedule options" })).not.toBeInTheDocument();
    expect(within(completedCard).queryByRole("button", { name: "Hide schedule options" })).not.toBeInTheDocument();
    expect(within(completedCard).queryByText("03/01/2026 11:45")).not.toBeInTheDocument();
  });

  it("loads dummy task cards and shows multi-schedule rows without repeating date strings", async () => {
    await renderDashboard();

    fireEvent.click(screen.getByRole("button", { name: "Load dummy task cards" }));
    await flushAsyncEffects();
    expect(screen.getByText("Loaded dummy task cards for local UI testing.")).toBeInTheDocument();

    const queuedCard = getTaskCard("dummy-queued");
    expect(within(queuedCard).getByText("QUEUED")).toBeInTheDocument();
    const queuedRetry = within(queuedCard).getByRole("button", { name: "Retry" });
    expect(queuedRetry).toBeDisabled();
    expect(queuedRetry).toHaveAttribute("title", "Task is currently running.");

    const pausedCard = getTaskCard("dummy-paused");
    expect(within(pausedCard).getByText("PAUSED")).toBeInTheDocument();
    expect(within(pausedCard).getByRole("button", { name: "Resume" })).toBeInTheDocument();
    expect(within(pausedCard).queryByRole("button", { name: "Pause" })).not.toBeInTheDocument();

    const completedCard = getTaskCard("dummy-completed-confirmed");
    expect(within(completedCard).getByText("Train task")).toBeInTheDocument();
    expect(within(completedCard).getByText("Confirmed")).toBeInTheDocument();
    expect(within(completedCard).getByText("SRT 311")).toBeInTheDocument();
    expect(within(completedCard).getByText("Car 8 - 12A")).toBeInTheDocument();

    const multiScheduleCard = getTaskCard("dummy-polling-multi-schedule");
    expect(within(multiScheduleCard).getByText("POLLING")).toBeInTheDocument();
    expect(within(multiScheduleCard).getByText("#1")).toBeInTheDocument();
    expect(within(multiScheduleCard).getByText("#2")).toBeInTheDocument();
    expect(within(multiScheduleCard).getByText("#3")).toBeInTheDocument();
    expect(within(multiScheduleCard).queryByRole("button", { name: "Show schedule options" })).not.toBeInTheDocument();
    expect(within(multiScheduleCard).queryByRole("button", { name: "Hide schedule options" })).not.toBeInTheDocument();
    for (const text of Array.from(multiScheduleCard.querySelectorAll("ul li")).map((item) => item.textContent ?? "")) {
      expect(text).not.toMatch(/\d{2}\/\d{2}\/\d{4}/);
    }
  });

  it("shows task-card controls by state and performs representative actions", async () => {
    activeTasks = [
      makeTask("queued-control", "QUEUED"),
      makeTask("running-control", "RUNNING"),
      makeTask("paused-control", "PAUSED"),
      makeTask("awaiting-control", "COMPLETED", {
        ticket_status: "awaiting_payment",
        ticket_paid: false,
      }),
    ];
    completedTasks = [
      makeTask("completed-issued-control", "COMPLETED", {
        ticket_status: "ticket_issued",
        ticket_paid: true,
      }),
      makeTask("completed-cancelled-control", "COMPLETED", {
        ticket_status: "cancelled",
        ticket_paid: false,
      }),
      makeTask("expired-control", "EXPIRED", {
        ticket_status: "awaiting_payment",
        ticket_paid: false,
      }),
    ];

    await renderDashboard();

    const queuedCard = getTaskCard("queued-control");
    expect(within(queuedCard).getByRole("button", { name: "Retry" })).toBeEnabled();
    expect(within(queuedCard).getByRole("button", { name: "Pause" })).toBeInTheDocument();
    expect(within(queuedCard).getByRole("button", { name: "Cancel" })).toBeInTheDocument();
    expect(within(queuedCard).queryByRole("button", { name: "Resume" })).not.toBeInTheDocument();
    expect(within(queuedCard).queryByRole("button", { name: "Pay" })).not.toBeInTheDocument();
    expect(within(queuedCard).queryByRole("button", { name: "Cancel reservation" })).not.toBeInTheDocument();

    const pausedCard = getTaskCard("paused-control");
    expect(within(pausedCard).getByRole("button", { name: "Resume" })).toBeInTheDocument();
    expect(within(pausedCard).queryByRole("button", { name: "Pause" })).not.toBeInTheDocument();

    const awaitingCard = getTaskCard("awaiting-control");
    expect(within(awaitingCard).queryByRole("button", { name: "Retry" })).not.toBeInTheDocument();
    expect(within(awaitingCard).getByRole("button", { name: "Pay" })).toBeInTheDocument();
    expect(within(awaitingCard).getByRole("button", { name: "Cancel reservation" })).toBeInTheDocument();
    expect(within(awaitingCard).queryByRole("button", { name: "Cancel" })).not.toBeInTheDocument();

    const completedIssuedCard = getTaskCard("completed-issued-control");
    expect(within(completedIssuedCard).queryByRole("button", { name: "Pay" })).not.toBeInTheDocument();
    expect(within(completedIssuedCard).getByRole("button", { name: "Cancel" })).toBeInTheDocument();
    expect(within(completedIssuedCard).queryByRole("button", { name: "Delete" })).not.toBeInTheDocument();

    const completedCancelledCard = getTaskCard("completed-cancelled-control");
    expect(within(completedCancelledCard).getByRole("button", { name: "Delete" })).toBeInTheDocument();
    expect(within(completedCancelledCard).queryByRole("button", { name: "Cancel" })).not.toBeInTheDocument();
    expect(within(completedCancelledCard).queryByRole("button", { name: "Pay" })).not.toBeInTheDocument();

    const expiredCard = getTaskCard("expired-control");
    expect(within(expiredCard).getByRole("button", { name: "Retry" })).toBeInTheDocument();
    expect(within(expiredCard).getByRole("button", { name: "Delete" })).toBeInTheDocument();
    expect(within(expiredCard).queryByRole("button", { name: "Pay" })).not.toBeInTheDocument();
    expect(within(expiredCard).queryByRole("button", { name: "Cancel" })).not.toBeInTheDocument();
    expect(within(expiredCard).queryByRole("button", { name: "Cancel reservation" })).not.toBeInTheDocument();

    fireEvent.click(within(queuedCard).getByRole("button", { name: "Retry" }));
    await flushAsyncEffects();
    const runningCard = getTaskCard("running-control");
    fireEvent.click(within(runningCard).getByRole("button", { name: "Pause" }));
    await flushAsyncEffects();
    fireEvent.click(within(pausedCard).getByRole("button", { name: "Resume" }));
    await flushAsyncEffects();
    fireEvent.click(within(awaitingCard).getByRole("button", { name: "Pay" }));
    await flushAsyncEffects();
    fireEvent.click(within(awaitingCard).getByRole("button", { name: "Cancel reservation" }));
    await flushAsyncEffects();
    fireEvent.click(within(queuedCard).getByRole("button", { name: "Cancel" }));
    await flushAsyncEffects();
    fireEvent.click(within(completedCancelledCard).getByRole("button", { name: "Delete" }));
    await flushAsyncEffects();
    fireEvent.click(within(expiredCard).getByRole("button", { name: "Retry" }));
    await flushAsyncEffects();
    fireEvent.click(within(expiredCard).getByRole("button", { name: "Delete" }));
    await flushAsyncEffects();

    expect(fetchMock).toHaveBeenCalledWith(
      expect.stringContaining("/api/train/tasks/queued-control/retry"),
      expect.objectContaining({ method: "POST" }),
    );
    expect(fetchMock).toHaveBeenCalledWith(
      expect.stringContaining("/api/train/tasks/running-control/pause"),
      expect.objectContaining({ method: "POST" }),
    );
    expect(fetchMock).toHaveBeenCalledWith(
      expect.stringContaining("/api/train/tasks/paused-control/resume"),
      expect.objectContaining({ method: "POST" }),
    );
    expect(fetchMock).toHaveBeenCalledWith(
      expect.stringContaining("/api/train/tasks/awaiting-control/pay"),
      expect.objectContaining({ method: "POST" }),
    );
    expect(fetchMock).toHaveBeenCalledWith(
      expect.stringContaining("/api/train/tickets/ticket-artifact-1/cancel"),
      expect.objectContaining({ method: "POST" }),
    );
    expect(fetchMock).toHaveBeenCalledWith(
      expect.stringContaining("/api/train/tasks/queued-control/cancel"),
      expect.objectContaining({ method: "POST" }),
    );
    expect(fetchMock).toHaveBeenCalledWith(
      expect.stringContaining("/api/train/tasks/completed-cancelled-control/delete"),
      expect.objectContaining({ method: "POST" }),
    );
    expect(fetchMock).toHaveBeenCalledWith(
      expect.stringContaining("/api/train/tasks/expired-control/retry"),
      expect.objectContaining({ method: "POST" }),
    );
    expect(fetchMock).toHaveBeenCalledWith(
      expect.stringContaining("/api/train/tasks/expired-control/delete"),
      expect.objectContaining({ method: "POST" }),
    );
  });

  it("requires explicit confirmation before creating duplicate reservations for the same schedule", async () => {
    schedules = [makeSchedule()];
    let duplicateCheckCalled = 0;
    let createTaskCalled = 0;

    fetchMock.mockImplementation(async (input: RequestInfo | URL, init?: RequestInit) => {
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
      if (url.includes("/api/train/tasks?")) {
        const parsed = new URL(url, "http://localhost");
        const status = parsed.searchParams.get("status");
        if (status === "active") return jsonResponse({ tasks: activeTasks });
        if (status === "completed") return jsonResponse({ tasks: completedTasks });
      }
      if (url.includes("/api/train/search") && method === "POST") {
        return jsonResponse({ schedules });
      }
      if (url.includes("/api/train/tasks/duplicate-check") && method === "POST") {
        duplicateCheckCalled += 1;
        return jsonResponse({
          has_duplicate: true,
          summary: { already_reserved: 1, waiting: 1, polling: 1 },
          matches: [
            {
              task_id: "11111111-1111-1111-1111-111111111111",
              state: "COMPLETED",
              category: "already_reserved",
              departure_at: "2026-02-22T12:30:00+09:00",
              ticket_status: "ticket_issued",
            },
            {
              task_id: "22222222-2222-2222-2222-222222222222",
              state: "RUNNING",
              category: "waiting",
              departure_at: "2026-02-22T12:30:00+09:00",
              ticket_status: "awaiting_payment",
            },
            {
              task_id: "33333333-3333-3333-3333-333333333333",
              state: "POLLING",
              category: "polling",
              departure_at: "2026-02-22T12:30:00+09:00",
              ticket_status: null,
            },
          ],
        });
      }
      if (url.includes("/api/train/tasks") && method === "POST") {
        createTaskCalled += 1;
        return jsonResponse({
          task: makeTask("created-after-confirm", "QUEUED"),
          deduplicated: false,
        });
      }
      return jsonResponse({ detail: "not found" }, 404);
    });

    await renderDashboard();
    fireEvent.click(screen.getByRole("button", { name: "Search" }));
    await flushAsyncEffects();
    fireEvent.click(screen.getByRole("button", { name: "SRT 301" }));
    fireEvent.click(screen.getByRole("button", { name: "Continue" }));

    expect(screen.getByRole("heading", { name: "Review task before starting" })).toBeInTheDocument();
    expect(duplicateCheckCalled).toBe(0);
    fireEvent.click(screen.getByRole("button", { name: "Confirm" }));
    await flushAsyncEffects();

    expect(duplicateCheckCalled).toBe(1);
    expect(createTaskCalled).toBe(0);
    expect(screen.getByRole("heading", { name: "You have an existing reservation for this schedule." })).toBeInTheDocument();
    expect(screen.getByText("Reserved ticket (1)")).toBeInTheDocument();
    expect(screen.getByText("Waitlisted (1)")).toBeInTheDocument();
    expect(screen.getByText("Polling task (1)")).toBeInTheDocument();
    const cancelButton = screen.getByRole("button", { name: "Cancel" });
    expect(cancelButton.className).toContain("h-10");

    fireEvent.click(screen.getAllByRole("button", { name: "Detail" })[0]);
    expect(screen.getByRole("heading", { name: "Open task detail?" })).toBeInTheDocument();
    expect(
      screen.getByText("Open task detail now? You will leave this screen and need to search again."),
    ).toBeInTheDocument();

    fireEvent.click(screen.getByRole("button", { name: "Cancel" }));
    expect(screen.getByRole("heading", { name: "You have an existing reservation for this schedule." })).toBeInTheDocument();

    fireEvent.click(screen.getByRole("button", { name: "Create anyway" }));
    await flushAsyncEffects();
    expect(createTaskCalled).toBe(1);
    expect(screen.getByText("Task created and queued.")).toBeInTheDocument();
  });

  it("handles search, task creation, and active task action controls", async () => {
    activeTasks = [
      makeTask("polling-1", "POLLING", {
        last_attempt_ok: false,
        last_attempt_error_code: "seat_unavailable",
        last_attempt_error_message_safe: "No available seats in selected trains right now",
      }),
      makeTask("paused-1", "PAUSED"),
      makeTask("awaiting-1", "COMPLETED", {
        ticket_status: "awaiting_payment",
        ticket_paid: false,
        ticket_payment_deadline_at: "2026-02-22T13:00:00+09:00",
      }),
    ];
    completedTasks = [
      makeTask("completed-1", "COMPLETED", { ticket_status: "ticket_issued", ticket_paid: true }),
      makeTask("completed-awaiting", "COMPLETED", {
        ticket_status: "awaiting_payment",
        ticket_paid: false,
      }),
    ];
    schedules = [makeSchedule()];

    await renderDashboard();

    fireEvent.change(screen.getByLabelText("Date"), { target: { value: "2026-02-22" } });
    fireEvent.click(screen.getByRole("button", { name: "Search" }));
    await flushAsyncEffects();
    expect(screen.getByRole("heading", { name: /Select schedules \(/ })).toBeInTheDocument();

    const originalMatchMedia = window.matchMedia;
    Object.defineProperty(window, "innerWidth", { configurable: true, value: 500 });
    window.matchMedia = vi.fn().mockReturnValue({ matches: true }) as typeof window.matchMedia;
    fireEvent.click(screen.getByRole("button", { name: "Modify Search" }));
    await flushAsyncEffects();
    const departureInput = screen.getByLabelText("Departure station");
    const arrivalInput = screen.getByLabelText("Arrival station");
    fireEvent.focus(departureInput);
    fireEvent.change(departureInput, { target: { value: "동탄" } });
    fireEvent.blur(departureInput);
    fireEvent.focus(arrivalInput);
    fireEvent.change(arrivalInput, { target: { value: "수서" } });
    fireEvent.blur(arrivalInput);
    fireEvent.change(screen.getByLabelText("Seat class"), { target: { value: "special" } });
    const adultsDesktopInput = screen.getByLabelText("Adults");
    const childrenDesktopInput = screen.getByLabelText("Children");
    fireEvent.change(adultsDesktopInput, { target: { value: "0" } });
    expect(screen.getByTestId("adults-count-mobile")).toHaveTextContent("1");
    fireEvent.change(childrenDesktopInput, { target: { value: "1" } });
    expect(screen.getByTestId("children-count-mobile")).toHaveTextContent("1");
    fireEvent.change(adultsDesktopInput, { target: { value: "0" } });
    expect(screen.getByTestId("adults-count-mobile")).toHaveTextContent("0");
    fireEvent.change(childrenDesktopInput, { target: { value: "0" } });
    expect(screen.getByTestId("children-count-mobile")).toHaveTextContent("1");
    fireEvent.change(adultsDesktopInput, { target: { value: "1" } });
    fireEvent.click(screen.getByRole("button", { name: "Decrease children" }));
    expect(screen.getByTestId("children-count-mobile")).toHaveTextContent("0");
    fireEvent.click(screen.getByRole("button", { name: "Decrease adults" }));
    fireEvent.click(screen.getByRole("button", { name: "Increase children" }));
    fireEvent.click(screen.getByRole("button", { name: "Decrease adults" }));
    fireEvent.click(screen.getByRole("button", { name: "Decrease children" }));
    fireEvent.click(screen.getByRole("button", { name: "Increase adults" }));
    window.matchMedia = originalMatchMedia;

    fireEvent.click(screen.getByRole("button", { name: "SRT 301" }));
    expect(screen.getByRole("button", { name: "Continue" })).not.toBeDisabled();
    fireEvent.click(screen.getByRole("button", { name: "Continue" }));
    expect(screen.getByRole("heading", { name: "Review task before starting" })).toBeInTheDocument();
    fireEvent.click(screen.getByRole("button", { name: "Confirm" }));
    await flushAsyncEffects();
    expect(screen.getByText("Task created and queued.")).toBeInTheDocument();

    const desktopRows = within(screen.getByTestId("schedule-selector-desktop")).getAllByRole("button");
    fireEvent.click(desktopRows[0]);
    fireEvent.click(desktopRows[0]);

    fireEvent.click(screen.getByRole("button", { name: "Load dummy task cards" }));
    await flushAsyncEffects();
    expect(screen.getByText("Loaded dummy task cards for local UI testing.")).toBeInTheDocument();
    fireEvent.click(screen.getByRole("button", { name: "Restore live tasks" }));
    await flushAsyncEffects();

    fireEvent.click(screen.getAllByRole("button", { name: "Retry" })[0]);
    await flushAsyncEffects();

    fireEvent.click(screen.getAllByRole("button", { name: "Pause" })[0]);
    await flushAsyncEffects();

    fireEvent.click(screen.getAllByRole("button", { name: "Resume" })[0]);
    await flushAsyncEffects();

    const confirmMock = vi.spyOn(window, "confirm").mockReturnValue(true);
    fireEvent.click(screen.getAllByRole("button", { name: "Pay" })[0]);
    await flushAsyncEffects();
    fireEvent.click(screen.getAllByRole("button", { name: "Cancel reservation" })[0]);
    await flushAsyncEffects();
    fireEvent.click(screen.getAllByRole("button", { name: "Cancel" })[0]);
    await flushAsyncEffects();

    expect(confirmMock).toHaveBeenCalled();
    expect(fetchMock).toHaveBeenCalledWith(
      expect.stringContaining("/api/train/tasks/awaiting-1/pay"),
      expect.objectContaining({ method: "POST" }),
    );
    expect(fetchMock).toHaveBeenCalledWith(
      expect.stringContaining("/api/train/tasks/awaiting-1"),
      expect.objectContaining({ credentials: "include", cache: "no-store" }),
    );
    expect(fetchMock).toHaveBeenCalledWith(
      expect.stringContaining("/api/train/tasks/awaiting-1/pay"),
      expect.objectContaining({ method: "POST" }),
    );
  }, 45_000);

  it("handles credential submit/continue/sign-out branches and search unlock validation", async () => {
    let credentialStatusCall = 0;
    let credentialSubmitCount = 0;
    let signOutCount = 0;
    let searchCalls = 0;

    fetchMock.mockImplementation(async (input: RequestInfo | URL, init?: RequestInit) => {
      const url = String(input);
      const method = init?.method ?? "GET";

      if (url.includes("/api/train/credentials/status")) {
        credentialStatusCall += 1;
        const verified = credentialStatusCall > 1;
        return jsonResponse({
          ktx: { configured: true, verified, username: verified ? "01012345678" : null, verified_at: null, detail: null },
          srt: { configured: true, verified: false, username: null, verified_at: null, detail: "needs login" },
        });
      }
      if (url.includes("/api/wallet/payment-card")) {
        return jsonResponse({ configured: false, card_masked: null, expiry_month: null, expiry_year: null, updated_at: null, detail: null });
      }
      if (url.includes("/api/train/stations")) {
        return jsonResponse({
          stations: [
            { name: "수서", srt_code: "0551", srt_supported: true },
            { name: "부산", srt_code: "0020", srt_supported: true },
          ],
        });
      }
      if (url.includes("/api/train/tasks?")) {
        return jsonResponse({ tasks: [] });
      }
      if (url.includes("/api/train/search") && method === "POST") {
        searchCalls += 1;
        if (searchCalls === 1) {
          return jsonResponse({ detail: "Select at least one provider." }, 400);
        }
        return jsonResponse({ schedules: [] });
      }
      if (url.includes("/api/train/credentials/srt") && method === "POST") {
        credentialSubmitCount += 1;
        if (credentialSubmitCount === 1) {
          return jsonResponse({ detail: "bad credentials" }, 400);
        }
        return jsonResponse({ ok: true });
      }
      if (url.includes("/api/train/credentials/") && url.includes("/signout") && method === "POST") {
        signOutCount += 1;
        if (signOutCount === 1) {
          return jsonResponse({ detail: "signout failed" }, 400);
        }
        return jsonResponse({ ok: true });
      }
      return jsonResponse({ detail: "not found" }, 404);
    });

    await renderDashboard();

    fireEvent.click(screen.getByRole("button", { name: /Show provider credentials|Hide provider credentials/i }));
    fireEvent.click(screen.getByRole("button", { name: /Show provider credentials|Hide provider credentials/i }));

    fireEvent.click(screen.getAllByRole("button", { name: "Connect" })[0]);
    fireEvent.change(screen.getByLabelText("SRT username"), { target: { value: "010-1234-5678" } });
    fireEvent.change(screen.getByLabelText("SRT password"), { target: { value: "pw1234" } });
    const srtCredentialForm = screen.getByLabelText("SRT username").closest("form");
    expect(srtCredentialForm).not.toBeNull();
    fireEvent.click(within(srtCredentialForm!).getByRole("button", { name: "Connect" }));
    await flushAsyncEffects();
    expect(screen.getByText("bad credentials")).toBeInTheDocument();

    fireEvent.click(within(srtCredentialForm!).getByRole("button", { name: "Connect" }));
    await flushAsyncEffects();
    expect(screen.getByText("SRT credentials verified.")).toBeInTheDocument();

    fireEvent.click(screen.getAllByRole("button", { name: /Connect|Change/ })[0]);
    fireEvent.click(within(srtCredentialForm!).getByRole("button", { name: "Ignore" }));
    expect(screen.getByText("Continuing without SRT. SRT search is disabled.")).toBeInTheDocument();

    const confirmSpy = vi.spyOn(window, "confirm").mockReturnValueOnce(false).mockReturnValueOnce(true).mockReturnValueOnce(true);
    fireEvent.click(screen.getAllByRole("button", { name: "Sign out" })[0]);
    fireEvent.click(screen.getAllByRole("button", { name: "Sign out" })[0]);
    await flushAsyncEffects();
    fireEvent.click(screen.getAllByRole("button", { name: "Sign out" })[0]);
    await flushAsyncEffects();
    expect(screen.getByText("SRT signed out.")).toBeInTheDocument();
    expect(confirmSpy).toHaveBeenCalled();

    expect(screen.getByText("Connect to a provider to unlock search.")).toBeInTheDocument();
    expect(screen.queryByTestId("train-search-form")).not.toBeInTheDocument();
  }, 20_000);

  it("covers task list polling error states and action failure branches", async () => {
    activeTasks = [
      makeTask("awaiting-error", "COMPLETED", {
        ticket_status: "awaiting_payment",
        ticket_paid: false,
        ticket_payment_deadline_at: "2026-02-22T13:00:00+09:00",
      }),
      makeTask("running-error", "RUNNING"),
    ];
    completedTasks = [
      makeTask("completed-delete", "COMPLETED", { ticket_status: "cancelled", ticket_paid: false }),
      makeTask("completed-cancel", "COMPLETED", { ticket_status: "ticket_issued", ticket_paid: true }),
      makeTask("completed-awaiting", "COMPLETED", { ticket_status: "awaiting_payment", ticket_paid: false }),
    ];
    schedules = [
      makeSchedule(),
      makeSchedule({
        schedule_id: "KTX-123",
        provider: "KTX",
        train_no: "123",
        departure_at: "2026-02-22T12:40:00+09:00",
        arrival_at: "2026-02-22T14:20:00+09:00",
        metadata: { train_type_code: "10" },
      }),
    ];

    let firstActivePoll = true;
    let payAttempt = 0;
    let cancelReservationAttempt = 0;
    let cancelTaskAttempt = 0;
    let pauseAttempt = 0;

    fetchMock.mockImplementation(async (input: RequestInfo | URL, init?: RequestInit) => {
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
          detail: null,
        });
      }
      if (url.includes("/api/train/stations")) {
        return jsonResponse({
          stations: [
            { name: "수서", srt_code: "0551", srt_supported: true },
            { name: "부산", srt_code: "0020", srt_supported: true },
            { name: "동탄", srt_code: "0552", srt_supported: true },
          ],
        });
      }
      if (url.includes("/api/train/tasks?")) {
        const parsed = new URL(url, "http://localhost");
        const status = parsed.searchParams.get("status");
        if (status === "active") {
          if (firstActivePoll) {
            firstActivePoll = false;
            return jsonResponse({ detail: "server busy" }, 500);
          }
          return jsonResponse({ tasks: activeTasks });
        }
        if (status === "completed") {
          return jsonResponse({ tasks: completedTasks });
        }
      }
      if (url.includes("/api/train/search") && method === "POST") {
        return jsonResponse({ schedules });
      }
      if (url.includes("/api/train/tasks/awaiting-error/pay") && method === "POST") {
        payAttempt += 1;
        if (payAttempt === 1) return jsonResponse({ detail: "payment failed" }, 500);
        if (payAttempt === 2) throw new Error("payment network");
        return jsonResponse({ detail: "ok" });
      }
      if (url.includes("/api/train/tasks/completed-awaiting/pay") && method === "POST") {
        return jsonResponse({ detail: "pay disabled path" });
      }
      if (url.includes("/api/train/tasks/awaiting-error") && method === "GET") {
        cancelReservationAttempt += 1;
        if (cancelReservationAttempt === 1) return jsonResponse({ detail: "detail load failed" }, 500);
        if (cancelReservationAttempt === 2) return jsonResponse({ artifacts: [] });
        if (cancelReservationAttempt === 3) {
          return jsonResponse({
            artifacts: [{ id: "ticket-a", module: "train", kind: "ticket", data_json_safe: {}, created_at: "2026-02-22T11:00:00+09:00" }],
          });
        }
        throw new Error("detail fetch error");
      }
      if (url.includes("/api/train/tickets/ticket-a/cancel") && method === "POST") {
        if (cancelReservationAttempt === 3) return jsonResponse({ detail: "cancel failed" }, 500);
        return jsonResponse({ detail: "cancel ok" });
      }
      if (url.includes("/api/train/tasks/completed-cancel") && method === "GET") {
        return jsonResponse({
          artifacts: [{ id: "ticket-completed", module: "train", kind: "ticket", data_json_safe: {}, created_at: "2026-02-22T11:00:00+09:00" }],
        });
      }
      if (url.includes("/api/train/tickets/ticket-completed/cancel") && method === "POST") {
        return jsonResponse({ detail: "cancelled" });
      }
      if (url.includes("/api/train/tasks/") && url.includes("/cancel") && method === "POST") {
        cancelTaskAttempt += 1;
        if (cancelTaskAttempt === 1) return jsonResponse({ detail: "cancel action failed" }, 500);
        throw new Error("cancel action network");
      }
      if (url.includes("/api/train/tasks/") && url.includes("/pause") && method === "POST") {
        pauseAttempt += 1;
        if (pauseAttempt === 1) return jsonResponse({ detail: "pause failed" }, 500);
        throw new Error("pause network");
      }
      if (url.includes("/api/train/tasks/completed-delete/delete") && method === "POST") {
        return jsonResponse({ ok: true });
      }
      return jsonResponse({ detail: "not found" }, 404);
    });

    const confirmSpy = vi.spyOn(window, "confirm").mockReturnValue(true);
    await renderDashboard();
    expect(screen.getByText("Could not load task lists.")).toBeInTheDocument();
    await act(async () => {
      document.dispatchEvent(new Event("visibilitychange"));
      await Promise.resolve();
    });
    await flushAsyncEffects();

    fireEvent.change(screen.getByLabelText("Date"), { target: { value: "2026-02-22" } });
    fireEvent.click(screen.getByRole("button", { name: "Search" }));
    await flushAsyncEffects();

    fireEvent.click(screen.getByRole("button", { name: "SRT 301" }));
    fireEvent.click(screen.getByRole("button", { name: "KTX-산천 123" }));
    expect(screen.queryByText("Priority order")).not.toBeInTheDocument();

    const desktopRows = within(screen.getByTestId("schedule-selector-desktop")).getAllByRole("button");
    fireEvent.keyDown(desktopRows[0], { key: "Enter" });
    fireEvent.keyDown(desktopRows[1], { key: " " });

    const runningCard = screen.getAllByRole("button", { name: "Pause" })[0]?.closest("li");
    expect(runningCard).not.toBeNull();
    fireEvent.click(within(runningCard!).getByRole("button", { name: "Pause" }));
    await flushAsyncEffects();
    expect(screen.getByText("pause failed")).toBeInTheDocument();
    fireEvent.click(within(runningCard!).getByRole("button", { name: "Pause" }));
    await flushAsyncEffects();
    expect(screen.getByText("Could not update task.")).toBeInTheDocument();

    fireEvent.click(screen.getAllByRole("button", { name: "Cancel" })[0]);
    await flushAsyncEffects();

    const activePayButton = screen.getAllByRole("button", { name: "Pay" }).find((button) => !button.hasAttribute("disabled"));
    expect(activePayButton).toBeTruthy();
    fireEvent.click(activePayButton!);
    await flushAsyncEffects();
    expect(screen.getByText("payment failed")).toBeInTheDocument();
    fireEvent.click(activePayButton!);
    await flushAsyncEffects();
    expect(screen.getByText("Could not process payment.")).toBeInTheDocument();

    const cancelReservationButton = screen.getByRole("button", { name: "Cancel reservation" });
    fireEvent.click(cancelReservationButton);
    await flushAsyncEffects();
    expect(screen.getByText("detail load failed")).toBeInTheDocument();
    fireEvent.click(cancelReservationButton);
    await flushAsyncEffects();
    expect(screen.getByText("No ticket artifact found for this task.")).toBeInTheDocument();
    fireEvent.click(cancelReservationButton);
    await flushAsyncEffects();
    expect(screen.getByText("cancel failed")).toBeInTheDocument();
    fireEvent.click(cancelReservationButton);
    await flushAsyncEffects();
    expect(screen.getByText("Could not cancel ticket.")).toBeInTheDocument();

    const completedCancelButtons = screen.getAllByRole("button", { name: "Cancel" });
    fireEvent.click(completedCancelButtons[completedCancelButtons.length - 1]);
    await flushAsyncEffects();

    fireEvent.click(screen.getByRole("button", { name: "Delete" }));
    await flushAsyncEffects();
    expect(confirmSpy).toHaveBeenCalled();
  }, 20_000);

  it("uses ticket-cancel fallback notice when provider response has no JSON detail", async () => {
    activeTasks = [
      makeTask("awaiting-fallback", "COMPLETED", {
        ticket_status: "awaiting_payment",
        ticket_paid: false,
      }),
    ];
    completedTasks = [];

    fetchMock.mockImplementation(async (input: RequestInfo | URL, init?: RequestInit) => {
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
      if (url.includes("/api/train/tasks?")) {
        const parsed = new URL(url, "http://localhost");
        const status = parsed.searchParams.get("status");
        if (status === "active") return jsonResponse({ tasks: activeTasks });
        if (status === "completed") return jsonResponse({ tasks: completedTasks });
      }
      if (url.includes("/api/train/tasks/awaiting-fallback") && method === "GET") {
        return jsonResponse({
          artifacts: [{ id: "ticket-fallback", module: "train", kind: "ticket", data_json_safe: {}, created_at: "2026-02-22T11:00:00+09:00" }],
        });
      }
      if (url.includes("/api/train/tickets/ticket-fallback/cancel") && method === "POST") {
        return new Response("ok", { status: 200, headers: { "Content-Type": "text/plain" } });
      }
      return jsonResponse({ detail: "not found" }, 404);
    });

    await renderDashboard();
    fireEvent.click(screen.getByRole("button", { name: "Cancel reservation" }));
    await flushAsyncEffects();
    await flushAsyncEffects();
    expect(fetchMock).toHaveBeenCalledWith(
      expect.stringContaining("/api/train/tickets/ticket-fallback/cancel"),
      expect.objectContaining({ method: "POST" }),
    );
    expect(screen.getByText("Ticket cancellation request completed.")).toBeInTheDocument();
  }, 20_000);

  it("covers unlock/session-expired and additional form input branches", async () => {
    fetchMock.mockImplementation(async (input: RequestInfo | URL, init?: RequestInit) => {
      const url = String(input);
      const method = init?.method ?? "GET";

      if (url.includes("/api/train/credentials/status")) {
        return jsonResponse({
          ktx: { configured: true, verified: true, username: "01012345678", verified_at: null, detail: null },
          srt: { configured: true, verified: true, username: "srt-user", verified_at: null, detail: null },
        });
      }
      if (url.includes("/api/wallet/payment-card")) {
        throw new Error("wallet fail");
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
        return jsonResponse({ detail: "expired" }, 401);
      }
      if (url.includes("/api/train/tasks?status=completed")) {
        return jsonResponse({ tasks: [] });
      }
      if (url.includes("/api/train/search") && method === "POST") {
        throw new Error("search network");
      }
      return jsonResponse({ detail: "not found" }, 404);
    });

    await renderDashboard();
    expect(
      screen.getByText(/Session expired\. Please log in again\.|Could not load wallet status\./),
    ).toBeInTheDocument();

    fireEvent.submit(screen.getByTestId("train-search-form"));
    await flushAsyncEffects();
    expect(screen.getByText("Could not reach API for search.")).toBeInTheDocument();

    fireEvent.change(screen.getAllByLabelText("Departure station")[0], { target: { value: "부산" } });
    fireEvent.change(screen.getAllByLabelText("Arrival station")[0], { target: { value: "수서" } });
    fireEvent.click(screen.getByRole("button", { name: "Swap departure and arrival stations" }));

    const providerDesktop = within(screen.getByTestId("provider-selector-desktop"));
    fireEvent.click(providerDesktop.getByRole("button", { name: "SRT" }));
    fireEvent.submit(screen.getByTestId("train-search-form"));
    await flushAsyncEffects();
    expect(screen.getByText("Select at least one provider.")).toBeInTheDocument();
  }, 20_000);

  it("covers defensive and error branches for search/create/credentials/actions", async () => {
    const active = [
      makeTask("running-guard", "RUNNING", {
        ticket_status: "custom_state",
      }),
      makeTask("awaiting-guard", "COMPLETED", {
        ticket_status: "awaiting_payment",
        ticket_paid: false,
      }),
    ];
    const completed = [makeTask("completed-guard", "COMPLETED", { ticket_status: "ticket_issued", ticket_paid: true })];
    const schedulesLocal = [
      makeSchedule(),
      makeSchedule({
        schedule_id: "SRT-302",
        train_no: "302",
        departure_at: "2026-02-22T13:00:00+09:00",
        arrival_at: "2026-02-22T15:20:00+09:00",
      }),
    ];

    let searchCount = 0;
    let createCount = 0;
    let cancelTicketCount = 0;

    fetchMock.mockImplementation(async (input: RequestInfo | URL, init?: RequestInit) => {
      const url = String(input);
      const method = init?.method ?? "GET";

      if (url.includes("/api/train/credentials/status")) {
        return jsonResponse({
          ktx: { configured: true, verified: true, username: "01012345678", verified_at: null, detail: null },
          srt: { configured: true, verified: true, username: "srt-user", verified_at: null, detail: null },
        });
      }
      if (url.includes("/api/wallet/payment-card")) {
        return jsonResponse({ configured: false, card_masked: null, expiry_month: null, expiry_year: null, updated_at: null, detail: null });
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
        return jsonResponse({ tasks: active });
      }
      if (url.includes("/api/train/tasks?status=completed")) {
        return jsonResponse({ tasks: completed });
      }
      if (url.includes("/api/train/search") && method === "POST") {
        searchCount += 1;
        if (searchCount === 1) return jsonResponse({ detail: "search detail failure" }, 400);
        return jsonResponse({ schedules: schedulesLocal });
      }
      if (url.includes("/api/train/tasks/duplicate-check") && method === "POST") {
        return jsonResponse({
          has_duplicate: false,
          summary: { already_reserved: 0, waiting: 0, polling: 0 },
          matches: [],
        });
      }
      if (url.includes("/api/train/tasks") && method === "POST") {
        createCount += 1;
        if (createCount === 1) return jsonResponse({ detail: "create detail failure" }, 400);
        throw new Error("create network failure");
      }
      if (url.includes("/api/train/tasks/awaiting-guard") && method === "GET") {
        return jsonResponse({
          artifacts: [{ id: "ticket-guard", module: "train", kind: "ticket", data_json_safe: {}, created_at: "2026-02-22T11:00:00+09:00" }],
        });
      }
      if (url.includes("/api/train/tickets/ticket-guard/cancel") && method === "POST") {
        cancelTicketCount += 1;
        if (cancelTicketCount === 1) return new Response("not-json", { status: 500, headers: { "Content-Type": "text/plain" } });
        return jsonResponse({ detail: "cancel-ok" });
      }
      if (url.includes("/api/train/tasks/running-guard/cancel") && method === "POST") {
        return jsonResponse({ ok: true });
      }
      return jsonResponse({ detail: "not found" }, 404);
    });

    const confirmSpy = vi
      .spyOn(window, "confirm")
      .mockReturnValueOnce(false) // sendTaskAction(cancel) early return
      .mockReturnValueOnce(false) // payAwaitingPaymentTask early return
      .mockReturnValueOnce(false) // cancelTaskTicket early return
      .mockReturnValue(true); // cancel ticket non-ok json parse fallback

    await renderDashboard();

    fireEvent.click(screen.getByRole("button", { name: "Search" }));
    await flushAsyncEffects();
    expect(screen.getByText("search detail failure")).toBeInTheDocument();

    fireEvent.click(screen.getByRole("button", { name: "Search" }));
    await flushAsyncEffects();
    fireEvent.click(screen.getByRole("button", { name: "SRT 301" }));
    fireEvent.click(screen.getByRole("button", { name: "SRT 302" }));

    fireEvent.click(screen.getByRole("button", { name: "Continue" }));
    fireEvent.click(screen.getByRole("button", { name: "Confirm" }));
    await flushAsyncEffects();
    expect(screen.getByText("create detail failure")).toBeInTheDocument();
    fireEvent.click(screen.getByRole("button", { name: "Continue" }));
    fireEvent.click(screen.getByRole("button", { name: "Confirm" }));
    await flushAsyncEffects();
    expect(screen.getByText("Could not create Task.")).toBeInTheDocument();

    const runningGuardCard = screen.getByText("RUNNING").closest("li");
    expect(runningGuardCard).not.toBeNull();
    fireEvent.click(within(runningGuardCard!).getByRole("button", { name: "Cancel" })); // early return confirm false
    await flushAsyncEffects();
    const activePayButton = screen.getAllByRole("button", { name: "Pay" }).find((button) => !button.hasAttribute("disabled"));
    expect(activePayButton).toBeTruthy();
    fireEvent.click(activePayButton!); // early return confirm false
    await flushAsyncEffects();
    fireEvent.click(screen.getByRole("button", { name: "Cancel reservation" })); // early return confirm false
    await flushAsyncEffects();

    fireEvent.click(screen.getByRole("button", { name: "Cancel reservation" }));
    await flushAsyncEffects();
    expect(
      screen.queryByText("Could not cancel ticket.") ?? screen.queryByText("not found"),
    ).toBeInTheDocument();
    expect(confirmSpy).toHaveBeenCalled();
  }, 25_000);

  it("keeps polling deterministic for in-flight/dummy reload guards and preserves non-task-list errors", async () => {
    const activeDeferred = createDeferred<Response>();
    let activeRequestCount = 0;
    let failTaskList = false;

    fetchMock.mockImplementation(async (input: RequestInfo | URL, init?: RequestInit) => {
      const url = String(input);
      const method = init?.method ?? "GET";

      if (url.includes("/api/train/credentials/status")) {
        return jsonResponse({
          ktx: { configured: true, verified: true, username: "01012345678", verified_at: null, detail: null },
          srt: { configured: true, verified: true, username: "srt-user", verified_at: null, detail: null },
        });
      }
      if (url.includes("/api/wallet/payment-card")) {
        return jsonResponse({ configured: false, card_masked: null, expiry_month: null, expiry_year: null, updated_at: null, detail: null });
      }
      if (url.includes("/api/train/stations")) {
        return jsonResponse({
          stations: [
            { name: "수서", srt_code: "0551", srt_supported: true },
            { name: "부산", srt_code: "0020", srt_supported: true },
          ],
        });
      }
      if (url.includes("/api/train/tasks?")) {
        const parsed = new URL(url, "http://localhost");
        const status = parsed.searchParams.get("status");
        if (status === "active") {
          activeRequestCount += 1;
          if (activeRequestCount === 1) {
            return activeDeferred.promise;
          }
          if (failTaskList) {
            return jsonResponse({ detail: "task list failed" }, 500);
          }
          return jsonResponse({ tasks: [] });
        }
        if (status === "completed") {
          if (failTaskList) {
            return jsonResponse({ detail: "task list failed" }, 500);
          }
          return jsonResponse({ tasks: [] });
        }
      }
      if (url.includes("/api/train/search") && method === "POST") {
        return jsonResponse({ detail: "search detail preserved" }, 400);
      }
      return jsonResponse({ detail: "not found" }, 404);
    });

    await renderDashboard();

    await act(async () => {
      vi.advanceTimersByTime(60_000);
      await Promise.resolve();
      await Promise.resolve();
    });
    expect(activeRequestCount).toBe(1);

    activeDeferred.resolve(jsonResponse({ tasks: [] }));
    await flushAsyncEffects();
    expect(activeRequestCount).toBe(1);

    fireEvent.click(screen.getByRole("button", { name: "Load dummy task cards" }));
    await flushAsyncEffects();
    const requestsBeforeDummyTick = activeRequestCount;

    await act(async () => {
      vi.advanceTimersByTime(60_000);
      await Promise.resolve();
      await Promise.resolve();
    });
    expect(activeRequestCount).toBe(requestsBeforeDummyTick);

    fireEvent.click(screen.getByRole("button", { name: "Restore live tasks" }));
    await flushAsyncEffects();

    fireEvent.click(screen.getByRole("button", { name: "Search" }));
    await flushAsyncEffects();
    expect(screen.getByText("search detail preserved")).toBeInTheDocument();

    failTaskList = true;
    const requestsBeforeFailureTick = activeRequestCount;
    await act(async () => {
      vi.advanceTimersByTime(60_000);
      await Promise.resolve();
      await Promise.resolve();
    });
    await flushAsyncEffects();
    expect(activeRequestCount).toBe(requestsBeforeFailureTick);
    expect(screen.queryByText("Could not load task lists.")).not.toBeInTheDocument();
  }, 20_000);

  it("shows credential timeout on abort and blocks search while providers are not connected", async () => {
    fetchMock.mockImplementation(async (input: RequestInfo | URL, init?: RequestInit) => {
      const url = String(input);

      if (url.includes("/api/train/credentials/status")) {
        const signal = init?.signal as AbortSignal | undefined;
        return await new Promise<Response>((_resolve, reject) => {
          if (!signal) {
            reject(new Error("missing abort signal"));
            return;
          }
          const onAbort = () => reject(new DOMException("Aborted", "AbortError"));
          if (signal.aborted) {
            onAbort();
            return;
          }
          signal.addEventListener("abort", onAbort, { once: true });
        });
      }
      if (url.includes("/api/wallet/payment-card")) {
        return jsonResponse({ configured: false, card_masked: null, expiry_month: null, expiry_year: null, updated_at: null, detail: null });
      }
      if (url.includes("/api/train/stations")) {
        return jsonResponse({
          stations: [
            { name: "수서", srt_code: "0551", srt_supported: true },
            { name: "부산", srt_code: "0020", srt_supported: true },
          ],
        });
      }
      if (url.includes("/api/train/tasks?")) {
        return jsonResponse({ tasks: [] });
      }
      return jsonResponse({ detail: "not found" }, 404);
    });

    await renderDashboard();

    await act(async () => {
      vi.advanceTimersByTime(10_000);
      await Promise.resolve();
    });
    await flushAsyncEffects();
    expect(
      screen.getByText("Credential check exceeded 10 seconds. You can continue by connecting providers manually."),
    ).toBeInTheDocument();

    expect(screen.getByText("Connect to a provider to unlock search.")).toBeInTheDocument();
  }, 20_000);

  it("uses fallback error copy for non-JSON search and credential submit responses", async () => {
    fetchMock.mockImplementation(async (input: RequestInfo | URL, init?: RequestInit) => {
      const url = String(input);
      const method = init?.method ?? "GET";

      if (url.includes("/api/train/credentials/status")) {
        return jsonResponse({
          ktx: { configured: true, verified: false, username: null, verified_at: null, detail: "needs login" },
          srt: { configured: true, verified: true, username: "srt-user", verified_at: null, detail: null },
        });
      }
      if (url.includes("/api/wallet/payment-card")) {
        return jsonResponse({ configured: false, card_masked: null, expiry_month: null, expiry_year: null, updated_at: null, detail: null });
      }
      if (url.includes("/api/train/stations")) {
        return jsonResponse({
          stations: [
            { name: "수서", srt_code: "0551", srt_supported: true },
            { name: "부산", srt_code: "0020", srt_supported: true },
          ],
        });
      }
      if (url.includes("/api/train/tasks?")) {
        return jsonResponse({ tasks: [] });
      }
      if (url.includes("/api/train/search") && method === "POST") {
        return new Response("search upstream failure", { status: 500, headers: { "Content-Type": "text/plain" } });
      }
      return jsonResponse({ detail: "not found" }, 404);
    });

    await renderDashboard();

    fireEvent.click(screen.getByRole("button", { name: "Search" }));
    await flushAsyncEffects();
    expect(screen.getByText("Search failed.")).toBeInTheDocument();

  }, 20_000);

  it("ignores late station payloads after unmount without mutating state", async () => {
    const stationDeferred = createDeferred<Response>();

    fetchMock.mockImplementation(async (input: RequestInfo | URL) => {
      const url = String(input);

      if (url.includes("/api/train/credentials/status")) {
        return jsonResponse({
          ktx: { configured: true, verified: true, username: "01012345678", verified_at: null, detail: null },
          srt: { configured: true, verified: true, username: "srt-user", verified_at: null, detail: null },
        });
      }
      if (url.includes("/api/wallet/payment-card")) {
        return jsonResponse({ configured: false, card_masked: null, expiry_month: null, expiry_year: null, updated_at: null, detail: null });
      }
      if (url.includes("/api/train/stations")) {
        return stationDeferred.promise;
      }
      if (url.includes("/api/train/tasks?")) {
        return jsonResponse({ tasks: [] });
      }
      return jsonResponse({ detail: "not found" }, 404);
    });

    const view = render(
      <LocaleProvider initialLocale="en">
        <TrainDashboard />
      </LocaleProvider>,
    );
    await flushAsyncEffects();
    view.unmount();

    stationDeferred.resolve(
      jsonResponse({
        stations: [
          { name: "수서", srt_code: "0551", srt_supported: true },
          { name: "부산", srt_code: "0020", srt_supported: true },
        ],
      }),
    );
    await flushAsyncEffects();
    expect(fetchMock.mock.calls.some(([input]) => String(input).includes("/api/train/stations"))).toBe(true);
  }, 20_000);

  it("keeps station selects disabled when station payload is empty", async () => {
    fetchMock.mockImplementation(async (input: RequestInfo | URL) => {
      const url = String(input);
      if (url.includes("/api/train/credentials/status")) {
        return jsonResponse({
          ktx: { configured: true, verified: true, username: "01012345678", verified_at: null, detail: null },
          srt: { configured: true, verified: true, username: "srt-user", verified_at: null, detail: null },
        });
      }
      if (url.includes("/api/wallet/payment-card")) {
        return jsonResponse({ configured: false, card_masked: null, expiry_month: null, expiry_year: null, updated_at: null, detail: null });
      }
      if (url.includes("/api/train/stations")) {
        return jsonResponse({ stations: [] });
      }
      if (url.includes("/api/train/tasks?")) {
        return jsonResponse({ tasks: [] });
      }
      return jsonResponse({ detail: "not found" }, 404);
    });

    await renderDashboard();

    const departureMobile = screen.getAllByLabelText("Departure station")[0] as HTMLInputElement;
    const arrivalMobile = screen.getAllByLabelText("Arrival station")[0] as HTMLInputElement;
    expect(departureMobile.disabled).toBe(true);
    expect(arrivalMobile.disabled).toBe(true);
  });

  it("maps non-Error task polling rejections to generic task list error", async () => {
    fetchMock.mockImplementation(async (input: RequestInfo | URL) => {
      const url = String(input);
      if (url.includes("/api/train/credentials/status")) {
        return jsonResponse({
          ktx: { configured: true, verified: true, username: "01012345678", verified_at: null, detail: null },
          srt: { configured: true, verified: true, username: "srt-user", verified_at: null, detail: null },
        });
      }
      if (url.includes("/api/wallet/payment-card")) {
        return jsonResponse({ configured: false, card_masked: null, expiry_month: null, expiry_year: null, updated_at: null, detail: null });
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
        throw "non-error rejection";
      }
      if (url.includes("/api/train/tasks?status=completed")) {
        return jsonResponse({ tasks: [] });
      }
      return jsonResponse({ detail: "not found" }, 404);
    });

    await renderDashboard();

    expect(screen.getByText("Could not load task lists.")).toBeInTheDocument();
  });

  it("autocorrects station input on blur for strong matches and reverts no-match input", async () => {
    fetchMock.mockImplementation(async (input: RequestInfo | URL) => {
      const url = String(input);
      if (url.includes("/api/train/credentials/status")) {
        return jsonResponse({
          ktx: { configured: true, verified: true, username: "01012345678", verified_at: null, detail: null },
          srt: { configured: true, verified: true, username: "srt-user", verified_at: null, detail: null },
        });
      }
      if (url.includes("/api/wallet/payment-card")) {
        return jsonResponse({ configured: false, card_masked: null, expiry_month: null, expiry_year: null, updated_at: null, detail: null });
      }
      if (url.includes("/api/train/stations")) {
        return jsonResponse({
          stations: [
            { name: "수서", srt_code: "0551", srt_supported: true },
            { name: "부산", srt_code: "0020", srt_supported: true },
            { name: "동탄", srt_code: "0552", srt_supported: true },
            { name: "서울", srt_code: null, srt_supported: false },
          ],
        });
      }
      if (url.includes("/api/train/tasks?")) {
        return jsonResponse({ tasks: [] });
      }
      return jsonResponse({ detail: "not found" }, 404);
    });

    await renderDashboard();

    const departure = screen.getAllByLabelText("Departure station")[0] as HTMLInputElement;
    fireEvent.focus(departure);
    fireEvent.change(departure, { target: { value: "tntj" } });
    expect(within(screen.getByRole("listbox")).getAllByRole("option").length).toBeLessThanOrEqual(3);
    fireEvent.blur(departure);
    expect(departure.value).toContain("Suseo");

    fireEvent.focus(departure);
    fireEvent.change(departure, { target: { value: "zzzzqqq" } });
    expect(screen.getByText("No matching stations")).toBeInTheDocument();
    fireEvent.blur(departure);
    expect(departure.value).toContain("Suseo");
  });

});
