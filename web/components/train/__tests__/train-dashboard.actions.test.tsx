import React from "react";

import { act, fireEvent, render, screen, within } from "@testing-library/react";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";

import { LocaleProvider } from "@/components/locale-provider";
import { TrainDashboard } from "@/components/train/train-dashboard";
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
          cvv_cached_until: null,
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
      if (url.includes("/api/train/tasks?")) {
        return jsonResponse({ tasks: [] });
      }
      if (url.includes("/api/train/search") && method === "POST") {
        return jsonResponse({ schedules: [] });
      }
      return jsonResponse({ detail: "not found" }, 404);
    });

    await renderDashboard();

    expect(screen.getByText("Providers without verified credentials are disabled for search.")).toBeInTheDocument();
    const providerDesktop = within(screen.getByTestId("provider-selector-desktop"));
    expect(providerDesktop.getByRole("checkbox", { name: "SRT" })).toBeDisabled();
    expect(providerDesktop.getByRole("checkbox", { name: "KTX" })).not.toBeDisabled();
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
    fireEvent.click(screen.getByRole("button", { name: "Create Task" }));
    await flushAsyncEffects();

    expect(duplicateCheckCalled).toBe(1);
    expect(createTaskCalled).toBe(0);
    expect(screen.getByRole("heading", { name: "You have an existing reservation for this schedule." })).toBeInTheDocument();
    expect(screen.getByText("Reserved ticket (1)")).toBeInTheDocument();
    expect(screen.getByText("Waiting list (1)")).toBeInTheDocument();
    expect(screen.getByText("Polling task (1)")).toBeInTheDocument();

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
    const searchForm = screen.getByTestId("train-search-form");
    const desktopStationSelects = searchForm.querySelectorAll("select.hidden.md\\:block");
    fireEvent.change(desktopStationSelects[0] as HTMLSelectElement, { target: { value: "동탄" } });
    fireEvent.change(desktopStationSelects[1] as HTMLSelectElement, { target: { value: "수서" } });
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
    expect(screen.getByRole("button", { name: "Create Task" })).not.toBeDisabled();
    fireEvent.click(screen.getByRole("button", { name: "Create Task" }));
    await flushAsyncEffects();
    expect(screen.getByText("Task created and queued.")).toBeInTheDocument();

    const desktopRows = within(screen.getByTestId("schedule-selector-desktop")).getAllByRole("button");
    fireEvent.click(desktopRows[0]);
    fireEvent.click(desktopRows[0]);

    fireEvent.change(screen.getByLabelText("Sort:"), { target: { value: "desc" } });
    fireEvent.change(screen.getByLabelText("Sort:"), { target: { value: "asc" } });

    fireEvent.click(screen.getByRole("button", { name: "Load dummy task cards" }));
    await flushAsyncEffects();
    expect(screen.getByText("Loaded dummy task cards for local UI testing.")).toBeInTheDocument();
    fireEvent.click(screen.getByRole("button", { name: "Restore live tasks" }));
    await flushAsyncEffects();

    fireEvent.click(screen.getByRole("button", { name: "Retry now" }));
    await flushAsyncEffects();

    fireEvent.click(screen.getAllByRole("button", { name: "Pause" })[0]);
    await flushAsyncEffects();

    fireEvent.click(screen.getByRole("button", { name: "Resume" }));
    await flushAsyncEffects();

    const confirmMock = vi.spyOn(window, "confirm").mockReturnValue(true);
    fireEvent.click(screen.getAllByRole("button", { name: "Pay" })[0]);
    await flushAsyncEffects();
    fireEvent.click(screen.getByRole("button", { name: "Cancel reservation" }));
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
        return jsonResponse({ configured: false, card_masked: null, expiry_month: null, expiry_year: null, updated_at: null, cvv_cached_until: null, detail: null });
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
      if (url.includes("/api/train/credentials/ktx") && method === "POST") {
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
    fireEvent.change(screen.getByLabelText("KTX username"), { target: { value: "010-1234-5678" } });
    fireEvent.change(screen.getByLabelText("KTX password"), { target: { value: "pw1234" } });
    fireEvent.click(screen.getByRole("button", { name: "Connect KTX" }));
    await flushAsyncEffects();
    expect(screen.getByText("bad credentials")).toBeInTheDocument();

    fireEvent.click(screen.getByRole("button", { name: "Connect KTX" }));
    await flushAsyncEffects();
    expect(screen.getByText("KTX credentials verified.")).toBeInTheDocument();

    const connectButtons = screen.getAllByRole("button", { name: "Connect" });
    fireEvent.click(connectButtons[connectButtons.length - 1]);
    fireEvent.click(screen.getByRole("button", { name: "Continue without SRT" }));
    expect(screen.getByText("Continuing without SRT. SRT search is disabled.")).toBeInTheDocument();

    const confirmSpy = vi.spyOn(window, "confirm").mockReturnValueOnce(false).mockReturnValueOnce(true).mockReturnValueOnce(true);
    fireEvent.click(screen.getAllByRole("button", { name: "Sign out" })[0]);
    fireEvent.click(screen.getAllByRole("button", { name: "Sign out" })[0]);
    await flushAsyncEffects();
    fireEvent.click(screen.getAllByRole("button", { name: "Sign out" })[0]);
    await flushAsyncEffects();
    expect(screen.getByText("KTX signed out.")).toBeInTheDocument();
    expect(confirmSpy).toHaveBeenCalled();

    const srtToggle = within(screen.getByTestId("provider-selector-mobile")).getByRole("button", { name: "SRT" });
    fireEvent.click(srtToggle);
    fireEvent.submit(screen.getByTestId("train-search-form"));
    await flushAsyncEffects();
    expect(screen.getByText("Select at least one provider.")).toBeInTheDocument();
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
          cvv_cached_until: null,
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
    expect(screen.getByText("Priority order")).toBeInTheDocument();
    fireEvent.click(screen.getAllByRole("button", { name: "Up" })[1]);
    fireEvent.click(screen.getAllByRole("button", { name: "Down" })[0]);
    fireEvent.click(screen.getByRole("switch", { name: "Notify" }));

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
      if (url.includes("/api/train/credentials/ktx") && method === "POST") {
        return jsonResponse({ ok: true });
      }
      return jsonResponse({ detail: "not found" }, 404);
    });

    await renderDashboard();
    expect(screen.getByText("Session expired. Please log in again.")).toBeInTheDocument();

    fireEvent.click(screen.getByRole("button", { name: /Show provider credentials|Hide provider credentials/i }));
    fireEvent.click(screen.getAllByRole("button", { name: "Change" })[0]);
    fireEvent.change(screen.getByLabelText("KTX username"), { target: { value: "010-1234-5678" } });
    fireEvent.change(screen.getByLabelText("KTX password"), { target: { value: "pw1234" } });
    fireEvent.click(screen.getByRole("button", { name: "Cancel" }));
    await flushAsyncEffects();

    fireEvent.submit(screen.getByTestId("train-search-form"));
    await flushAsyncEffects();
    expect(screen.getByText("Could not reach API for search.")).toBeInTheDocument();

    fireEvent.change(screen.getAllByLabelText("Departure station")[0], { target: { value: "부산" } });
    fireEvent.change(screen.getAllByLabelText("Arrival station")[0], { target: { value: "수서" } });
    fireEvent.click(screen.getByRole("button", { name: "Swap departure and arrival stations" }));
    fireEvent.change(screen.getByLabelText("Time start"), { target: { value: "07:00" } });
    fireEvent.change(screen.getByLabelText("Time end"), { target: { value: "21:00" } });

    const providerDesktop = within(screen.getByTestId("provider-selector-desktop"));
    fireEvent.click(providerDesktop.getByRole("checkbox", { name: "SRT" }));
    fireEvent.click(providerDesktop.getByRole("checkbox", { name: "KTX" }));
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
    let credentialSubmitCount = 0;
    let signOutCount = 0;
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
        return jsonResponse({ configured: false, card_masked: null, expiry_month: null, expiry_year: null, updated_at: null, cvv_cached_until: null, detail: null });
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
      if (url.includes("/api/train/credentials/ktx/signout") && method === "POST") {
        signOutCount += 1;
        if (signOutCount === 1) return jsonResponse({ detail: "signout detail failure" }, 400);
        throw new Error("signout network failure");
      }
      if (url.includes("/api/train/credentials/ktx") && method === "POST") {
        credentialSubmitCount += 1;
        if (credentialSubmitCount === 1) return jsonResponse({ detail: "credential detail failure" }, 400);
        throw new Error("credential network failure");
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
      .mockReturnValue(true) // sign out non-ok
      .mockReturnValue(true) // sign out catch
      .mockReturnValue(true); // cancel ticket non-ok json parse fallback

    await renderDashboard();

    fireEvent.click(screen.getByRole("button", { name: "Search" }));
    await flushAsyncEffects();
    expect(screen.getByText("search detail failure")).toBeInTheDocument();

    fireEvent.click(screen.getByRole("button", { name: "Search" }));
    await flushAsyncEffects();
    fireEvent.click(screen.getByRole("button", { name: "SRT 301" }));
    fireEvent.click(screen.getByRole("button", { name: "SRT 302" }));
    fireEvent.click(screen.getAllByRole("button", { name: "Up" })[0]);
    fireEvent.click(screen.getAllByRole("button", { name: "Down" })[1]);

    fireEvent.click(screen.getByRole("button", { name: "Create Task" }));
    await flushAsyncEffects();
    expect(screen.getByText("create detail failure")).toBeInTheDocument();
    fireEvent.click(screen.getByRole("button", { name: "Create Task" }));
    await flushAsyncEffects();
    expect(screen.getByText("Could not create Task.")).toBeInTheDocument();

    fireEvent.click(screen.getByRole("button", { name: /Show provider credentials|Hide provider credentials/i }));
    fireEvent.click(screen.getAllByRole("button", { name: "Change" })[0]);
    fireEvent.change(screen.getByLabelText("KTX username"), { target: { value: "01012345678" } });
    fireEvent.change(screen.getByLabelText("KTX password"), { target: { value: "pw1234" } });
    fireEvent.click(screen.getByRole("button", { name: "Connect KTX" }));
    await flushAsyncEffects();
    expect(screen.getByText("credential detail failure")).toBeInTheDocument();
    fireEvent.click(screen.getByRole("button", { name: "Connect KTX" }));
    await flushAsyncEffects();
    expect(screen.getByText("Could not verify KTX credentials.")).toBeInTheDocument();

    const runningGuardCard = screen.getByText("RUNNING").closest("li");
    expect(runningGuardCard).not.toBeNull();
    fireEvent.click(within(runningGuardCard!).getByRole("button", { name: "Cancel" })); // early return confirm false
    await flushAsyncEffects();
    fireEvent.click(screen.getByRole("button", { name: "Pay" })); // early return confirm false
    await flushAsyncEffects();
    fireEvent.click(screen.getByRole("button", { name: "Cancel reservation" })); // early return confirm false
    await flushAsyncEffects();

    fireEvent.click(screen.getAllByRole("button", { name: "Sign out" })[0]);
    await flushAsyncEffects();
    expect(screen.getByText("signout detail failure")).toBeInTheDocument();
    fireEvent.click(screen.getAllByRole("button", { name: "Sign out" })[0]);
    await flushAsyncEffects();
    expect(screen.getByText("Could not sign out KTX.")).toBeInTheDocument();

    fireEvent.click(screen.getByRole("button", { name: "Cancel reservation" }));
    await flushAsyncEffects();
    expect(screen.getByText("Could not cancel ticket.")).toBeInTheDocument();
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
        return jsonResponse({ configured: false, card_masked: null, expiry_month: null, expiry_year: null, updated_at: null, cvv_cached_until: null, detail: null });
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
    await act(async () => {
      vi.advanceTimersByTime(60_000);
      await Promise.resolve();
      await Promise.resolve();
    });
    await flushAsyncEffects();
    expect(screen.getByText("search detail preserved")).toBeInTheDocument();
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
        return jsonResponse({ configured: false, card_masked: null, expiry_month: null, expiry_year: null, updated_at: null, cvv_cached_until: null, detail: null });
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
        return jsonResponse({ configured: false, card_masked: null, expiry_month: null, expiry_year: null, updated_at: null, cvv_cached_until: null, detail: null });
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
      if (url.includes("/api/train/credentials/ktx") && method === "POST") {
        return new Response("credential upstream failure", { status: 500, headers: { "Content-Type": "text/plain" } });
      }
      return jsonResponse({ detail: "not found" }, 404);
    });

    await renderDashboard();

    fireEvent.click(screen.getByRole("button", { name: "Search" }));
    await flushAsyncEffects();
    expect(screen.getByText("Search failed.")).toBeInTheDocument();

    fireEvent.click(screen.getByRole("button", { name: /Show provider credentials|Hide provider credentials/i }));
    await flushAsyncEffects();
    if (!screen.queryByRole("button", { name: "Connect" })) {
      fireEvent.click(screen.getByRole("button", { name: /Show provider credentials|Hide provider credentials/i }));
      await flushAsyncEffects();
    }
    fireEvent.click(screen.getAllByRole("button", { name: "Connect" })[0]);
    fireEvent.change(screen.getByLabelText("KTX username"), { target: { value: "010-5555-0000" } });
    fireEvent.change(screen.getByLabelText("KTX password"), { target: { value: "pw1234" } });
    fireEvent.click(screen.getByRole("button", { name: "Connect KTX" }));
    await flushAsyncEffects();
    expect(screen.getByText("KTX login failed.")).toBeInTheDocument();
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
        return jsonResponse({ configured: false, card_masked: null, expiry_month: null, expiry_year: null, updated_at: null, cvv_cached_until: null, detail: null });
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
        return jsonResponse({ configured: false, card_masked: null, expiry_month: null, expiry_year: null, updated_at: null, cvv_cached_until: null, detail: null });
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

    const departureMobile = screen.getAllByLabelText("Departure station")[0] as HTMLSelectElement;
    const arrivalMobile = screen.getAllByLabelText("Arrival station")[0] as HTMLSelectElement;
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
        return jsonResponse({ configured: false, card_masked: null, expiry_month: null, expiry_year: null, updated_at: null, cvv_cached_until: null, detail: null });
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

});
