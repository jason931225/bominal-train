import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";

import {
  buildDummyTaskCards,
  clampPassengerCount,
  fetchTasksByStatus,
  formatScheduleTitleDate,
  formatTicketStatus,
  formatTimeKst,
  formatTransitDuration,
  getTaskTicketBadge,
  isActiveTaskForList,
  isAwaitingPaymentTask,
  isMobileViewport,
  isRecord,
  isUnpaidAwaitingPaymentTicket,
  metadataString,
  normalizePhoneNumber,
  parseApiErrorMessage,
  parsePassengerInputValue,
  readInteger,
  retryNowDisabledTitle,
  resolveSearchStations,
  scheduleTrainLabel,
  scrollElementToViewportCenter,
  scrollElementToViewportTop,
  shouldShowCompletedCancel,
  sortTasksByScheduleProximity,
  taskInfoFromSpec,
  taskListRenderKey,
  taskPrimaryDepartureAtMs,
  taskSummaryRenderKey,
  validateCreateTaskInputs,
} from "@/components/train/train-dashboard";
import type { TrainSchedule, TrainTaskSummary } from "@/lib/types";

function makeTask(
  id: string,
  state: TrainTaskSummary["state"],
  overrides: Partial<TrainTaskSummary> = {},
): TrainTaskSummary {
  return {
    id,
    module: "train",
    state,
    deadline_at: "2026-02-22T12:00:00+09:00",
    created_at: "2026-02-22T10:00:00+09:00",
    updated_at: "2026-02-22T10:10:00+09:00",
    paused_at: null,
    cancelled_at: null,
    completed_at: state === "COMPLETED" ? "2026-02-22T10:20:00+09:00" : null,
    failed_at: state === "FAILED" ? "2026-02-22T10:20:00+09:00" : null,
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
      date: "2026-02-22",
      passengers: { adults: 1, children: 0 },
      selected_trains_ranked: [
        { rank: 2, departure_at: "2026-02-22T11:00:00+09:00" },
        { rank: 1, departure_at: "2026-02-22T10:30:00+09:00" },
      ],
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
    departure_at: "2026-02-22T10:30:00+09:00",
    arrival_at: "2026-02-22T12:40:00+09:00",
    train_no: "301",
    dep: "수서",
    arr: "부산",
    availability: { general: true, special: false },
    metadata: {},
    ...overrides,
  };
}

describe("train dashboard helpers", () => {
  beforeEach(() => {
    vi.restoreAllMocks();
  });

  afterEach(() => {
    vi.unstubAllGlobals();
  });

  it("normalizes Korean phone numbers and preserves non-phone text", () => {
    expect(normalizePhoneNumber("010-1234-5678")).toBe("01012345678");
    expect(normalizePhoneNumber("+82-10-1234-5678")).toBe("01012345678");
    expect(normalizePhoneNumber("+8210-1234-5678")).toBe("01012345678");
    expect(normalizePhoneNumber("821012345678")).toBe("01012345678");
    expect(normalizePhoneNumber("user_name")).toBe("user_name");
  });

  it("classifies active/awaiting-payment task rows", () => {
    const polling = makeTask("poll", "POLLING");
    const awaiting = makeTask("awaiting", "COMPLETED", {
      ticket_status: "awaiting_payment",
      ticket_paid: false,
    });
    const completedPaid = makeTask("done", "COMPLETED", { ticket_status: "ticket_issued", ticket_paid: true });

    expect(isUnpaidAwaitingPaymentTicket(awaiting)).toBe(true);
    expect(isAwaitingPaymentTask(awaiting)).toBe(true);
    expect(isActiveTaskForList(polling)).toBe(true);
    expect(isActiveTaskForList(awaiting)).toBe(true);
    expect(isActiveTaskForList(completedPaid)).toBe(false);
  });

  it("builds dummy task cards split into active and completed groups", () => {
    const cards = buildDummyTaskCards(new Date("2026-02-22T00:00:00Z"));
    expect(cards.active.length).toBeGreaterThan(0);
    expect(cards.completed.length).toBeGreaterThan(0);
    expect(cards.active.some((task) => task.ticket_status === "awaiting_payment")).toBe(true);
    expect(cards.completed.some((task) => task.state === "FAILED")).toBe(true);
  });

  it("fetches task lists with proper query params and maps auth/errors", async () => {
    const fetchMock = vi.fn(async (input: RequestInfo | URL) => {
      const url = String(input);
      if (url.includes("status=active")) {
        return new Response(JSON.stringify({ tasks: [makeTask("a", "RUNNING")] }), {
          status: 200,
          headers: { "Content-Type": "application/json" },
        });
      }
      return new Response(JSON.stringify({ detail: "unauthorized" }), {
        status: 401,
        headers: { "Content-Type": "application/json" },
      });
    });
    vi.stubGlobal("fetch", fetchMock);

    const active = await fetchTasksByStatus("active", { limit: 5 });
    expect(active).toHaveLength(1);
    expect(fetchMock.mock.calls[0][0]).toContain("status=active");
    expect(fetchMock.mock.calls[0][0]).toContain("limit=5");

    await expect(fetchTasksByStatus("completed", { refreshCompleted: true, limit: 12 })).rejects.toThrow(
      "session_expired",
    );
    expect(fetchMock.mock.calls[1][0]).toContain("refresh_completed=true");
    expect(fetchMock.mock.calls[1][0]).toContain("limit=12");

    fetchMock.mockImplementationOnce(async () =>
      new Response(JSON.stringify({ detail: "boom" }), {
        status: 500,
        headers: { "Content-Type": "application/json" },
      }),
    );
    await expect(fetchTasksByStatus("active")).rejects.toThrow("task_list_error");

    fetchMock.mockImplementationOnce(async (input: RequestInfo | URL) => {
      const url = String(input);
      expect(url).toContain("status=completed");
      expect(url).toContain("limit=80");
      return new Response(JSON.stringify({ tasks: [makeTask("c", "COMPLETED")] }), {
        status: 200,
        headers: { "Content-Type": "application/json" },
      });
    });
    const completed = await fetchTasksByStatus("completed");
    expect(completed).toHaveLength(1);
  });

  it("parses API error message from json/text fallbacks", async () => {
    const jsonResponse = new Response(JSON.stringify({ detail: "json detail" }), {
      status: 400,
      headers: { "Content-Type": "application/json" },
    });
    expect(await parseApiErrorMessage(jsonResponse, "fallback")).toBe("json detail");

    const jsonNoDetail = new Response(JSON.stringify({ ok: false }), {
      status: 400,
      headers: { "Content-Type": "application/json" },
    });
    expect(await parseApiErrorMessage(jsonNoDetail, "fallback-json")).toBe("fallback-json");

    const textResponse = new Response("short text", {
      status: 400,
      headers: { "Content-Type": "text/plain" },
    });
    expect(await parseApiErrorMessage(textResponse, "fallback-text")).toBe("short text");

    const longText = new Response("x".repeat(200), {
      status: 400,
      headers: { "Content-Type": "text/plain" },
    });
    expect(await parseApiErrorMessage(longText, "fallback-long")).toHaveLength(163);

    const emptyText = new Response("", {
      status: 400,
      headers: { "Content-Type": "text/plain" },
    });
    expect(await parseApiErrorMessage(emptyText, "fallback-empty")).toBe("fallback-empty");

    const malformedJsonResponse = {
      headers: { get: () => "application/json" },
      json: async () => {
        throw new Error("invalid json");
      },
    } as unknown as Response;
    expect(await parseApiErrorMessage(malformedJsonResponse, "fallback-json-parse")).toBe("fallback-json-parse");

    const rejectedTextResponse = {
      headers: { get: () => "text/plain" },
      text: async () => {
        throw new Error("text fail");
      },
    } as unknown as Response;
    expect(await parseApiErrorMessage(rejectedTextResponse, "fallback-text-parse")).toBe("fallback-text-parse");

    const nullHeaderResponse = {
      headers: { get: () => null },
      text: async () => "",
    } as unknown as Response;
    expect(await parseApiErrorMessage(nullHeaderResponse, "fallback-null-header")).toBe("fallback-null-header");
  });

  it("formats transit/time/ticket strings and badges", () => {
    expect(formatTransitDuration("2026-02-22T10:00:00+09:00", "2026-02-22T11:30:00+09:00")).toBe("1h 30m");
    expect(formatTransitDuration("2026-02-22T10:00:00+09:00", "2026-02-22T10:45:00+09:00")).toBe("45m");
    expect(formatTransitDuration("2026-02-22T10:00:00+09:00", "2026-02-22T11:00:00+09:00")).toBe("1h");
    expect(formatTransitDuration("invalid", "2026-02-22T11:30:00+09:00")).toBe("-");

    expect(formatTimeKst("2026-02-22T11:30:00+09:00", "en")).toMatch(/\d{2}:\d{2}/);
    expect(formatTimeKst("invalid", "en")).toBe("-");

    expect(formatTicketStatus("awaiting_payment")).toBe("Awaiting Payment");
    expect(formatTicketStatus("_")).toBeNull();
    expect(formatTicketStatus("")).toBeNull();
    expect(formatTicketStatus(null)).toBeNull();

    expect(getTaskTicketBadge(makeTask("none", "RUNNING"))).toBeNull();
    expect(getTaskTicketBadge(makeTask("cancelled", "COMPLETED", { ticket_status: "cancelled" }))?.label).toBe(
      "train.badge.cancelled",
    );
    expect(
      getTaskTicketBadge(makeTask("missing", "COMPLETED", { ticket_status: "reservation_not_found", ticket_paid: false }))
        ?.label,
    ).toBe("train.badge.reservationNotFound");
    expect(
      getTaskTicketBadge(makeTask("awaiting", "COMPLETED", { ticket_status: "awaiting_payment", ticket_paid: false }))
        ?.label,
    ).toBe("train.badge.awaitingPayment");
    expect(getTaskTicketBadge(makeTask("paid", "COMPLETED", { ticket_status: "ticket_issued", ticket_paid: true }))?.label).toBe(
      "train.badge.confirmed",
    );
    expect(getTaskTicketBadge(makeTask("other", "COMPLETED", { ticket_status: "custom_state" }))?.label).toBe(
      "Custom State",
    );
    expect(getTaskTicketBadge(makeTask("other-empty", "COMPLETED", { ticket_status: "_" }))?.label).toBe("_");
  });

  it("computes completion cancel visibility and retry disabled titles", () => {
    expect(shouldShowCompletedCancel(makeTask("running", "RUNNING"))).toBe(false);
    expect(shouldShowCompletedCancel(makeTask("cancelled", "COMPLETED", { ticket_status: "cancelled" }))).toBe(false);
    expect(
      shouldShowCompletedCancel(
        makeTask("missing", "COMPLETED", { ticket_status: "reservation_not_found", ticket_paid: false }),
      ),
    ).toBe(false);
    expect(shouldShowCompletedCancel(makeTask("ok", "COMPLETED", { ticket_status: "ticket_issued", ticket_paid: true }))).toBe(
      true,
    );

    const byReason: Array<[string | null, string]> = [
      [null, "Retry is not available."],
      ["cooldown_active", "Retry available at"],
      ["deadline_passed", "Task deadline has passed."],
      ["paused_use_resume", "Task is paused. Use Resume instead."],
      ["task_running", "Task is currently running."],
      ["terminal_state", "Task is already finished."],
      ["not_eligible_state", "Task is not eligible for retry."],
      ["other", "Retry is not available."],
    ];
    for (const [reason, expected] of byReason) {
      const task = makeTask("r", "RUNNING", {
        retry_now_reason: reason,
        retry_now_available_at: "2026-02-22T11:00:00+09:00",
      });
      expect(retryNowDisabledTitle(task)).toContain(expected);
    }
  });

  it("handles schedule-title formatting and primitive helpers", () => {
    expect(formatScheduleTitleDate("2026-02-22")).toBe("02/22/2026");
    expect(formatScheduleTitleDate("")).toBe("MM/DD/YYYY");
    expect(formatScheduleTitleDate("bad")).toBe("MM/DD/YYYY");

    expect(isRecord({ a: 1 })).toBe(true);
    expect(isRecord([])).toBe(false);
    expect(isRecord(null)).toBe(false);

    expect(readInteger(2.8)).toBe(2);
    expect(readInteger("2")).toBeNull();

    expect(clampPassengerCount(12, 0, 9)).toBe(9);
    expect(clampPassengerCount(-2, 0, 9)).toBe(0);
    expect(clampPassengerCount(Number.NaN, 1, 9)).toBe(1);

    expect(parsePassengerInputValue(" 5 ")).toBe(5);
    expect(parsePassengerInputValue("")).toBe(0);
    expect(parsePassengerInputValue("abc")).toBe(0);
  });

  it("validates create-task invariants", () => {
    expect(validateCreateTaskInputs(0, 1)).toBe("train.error.selectSchedule");
    expect(validateCreateTaskInputs(1, 0)).toBe("train.error.passengerMinimum");
    expect(validateCreateTaskInputs(2, 2)).toBeNull();
  });

  it("resolves departure/arrival fallbacks from available stations", () => {
    const noStations = resolveSearchStations("수서", "부산", []);
    expect(noStations).toEqual({ dep: "수서", arr: "부산" });

    const withDefaults = [
      { name: "수서", srt_code: "0551", srt_supported: true },
      { name: "부산", srt_code: "0020", srt_supported: true },
      { name: "마산", srt_code: "0059", srt_supported: true },
    ];
    expect(resolveSearchStations("수서", "부산", withDefaults)).toEqual({ dep: "수서", arr: "부산" });
    expect(resolveSearchStations("없는역", "없는역", withDefaults)).toEqual({ dep: "수서", arr: "마산" });

    const withoutDefaults = [
      { name: "동탄", srt_code: "0552", srt_supported: true },
      { name: "천안아산", srt_code: "0502", srt_supported: true },
    ];
    expect(resolveSearchStations("없는역", "없는역", withoutDefaults)).toEqual({ dep: "동탄", arr: "천안아산" });
    expect(resolveSearchStations("없는역", "없는역", [withoutDefaults[0]])).toEqual({ dep: "동탄", arr: "동탄" });
  });

  it("detects mobile viewport and performs scroll helpers safely", () => {
    Object.defineProperty(window, "innerWidth", { configurable: true, value: 500 });
    const originalMatchMedia = window.matchMedia;
    // @ts-expect-error test override
    window.matchMedia = undefined;
    expect(isMobileViewport()).toBe(true);

    window.matchMedia = vi.fn().mockReturnValue({ matches: false }) as typeof window.matchMedia;
    expect(isMobileViewport()).toBe(false);
    window.matchMedia = vi.fn().mockReturnValue({ matches: true }) as typeof window.matchMedia;
    expect(isMobileViewport()).toBe(true);
    window.matchMedia = originalMatchMedia;

    const raf = vi.spyOn(window, "requestAnimationFrame").mockImplementation((cb: FrameRequestCallback) => {
      cb(0);
      return 1;
    });
    const scrollTo = vi.spyOn(window, "scrollTo").mockImplementation(() => undefined);
    const element = document.createElement("div");
    element.getBoundingClientRect = () =>
      ({
        top: 100,
        height: 20,
      }) as DOMRect;

    scrollElementToViewportCenter(element);
    scrollElementToViewportTop(element);
    scrollElementToViewportCenter(null);
    scrollElementToViewportTop(null);

    expect(raf).toHaveBeenCalled();
    expect(scrollTo).toHaveBeenCalled();
  });

  it("extracts metadata strings and computes schedule labels", () => {
    const scheduleWithName = makeSchedule({ metadata: { train_type_name: "KTX-산천" }, train_no: "305", provider: "KTX" });
    expect(scheduleTrainLabel(scheduleWithName)).toBe("KTX-산천 305");

    const scheduleWithCode = makeSchedule({ metadata: { train_type_code: "17" }, train_no: "301", provider: "SRT" });
    expect(scheduleTrainLabel(scheduleWithCode)).toBe("SRT 301");

    const scheduleFallback = makeSchedule({ metadata: { anything: "else" }, train_no: "999", provider: "KTX" });
    expect(scheduleTrainLabel(scheduleFallback)).toBe("KTX 999");

    expect(metadataString(scheduleWithName.metadata, "train_type_name")).toBe("KTX-산천");
    expect(metadataString({ count: 3 }, "count")).toBe("3");
    expect(metadataString({ blank: "   " }, "blank")).toBeNull();
    expect(metadataString({ enabled: true }, "enabled")).toBeNull();
    expect(metadataString(undefined, "missing")).toBeNull();
  });

  it("builds task info summary and deterministic task render keys", () => {
    const task = makeTask("task-1", "POLLING");
    const info = taskInfoFromSpec(task);
    expect(info.dep).toBe("수서");
    expect(info.arr).toBe("부산");
    expect(info.scheduleLabel).toContain("02/22/2026");
    expect(info.passengerLabel).toBe("1 adult, 0 children");

    const fallback = taskInfoFromSpec(makeTask("task-2", "POLLING", { spec_json: "invalid" as unknown as Record<string, unknown> }));
    expect(fallback.scheduleLabel).toBe("-");

    const dateOnly = taskInfoFromSpec(
      makeTask("task-3", "POLLING", {
        spec_json: {
          dep: "수서",
          arr: "부산",
          date: "2026-02-22",
          passengers: { adults: 1, children: 0 },
          selected_trains_ranked: [],
        },
      }),
    );
    expect(dateOnly.scheduleLabel).toBe("02/22/2026");

    const fullyFallback = taskInfoFromSpec(
      makeTask("task-4", "POLLING", {
        spec_json: {
          dep: "",
          arr: "",
          date: "",
          passengers: "bad",
          selected_trains_ranked: [{ rank: "bad", departure_at: "" }],
        } as unknown as Record<string, unknown>,
      }),
    );
    expect(fullyFallback.dep).toBe("-");
    expect(fullyFallback.arr).toBe("-");
    expect(fullyFallback.scheduleLabel).toBe("-");
    expect(fullyFallback.passengerLabel).toBe("0 adults, 0 children");

    const oneChild = taskInfoFromSpec(
      makeTask("task-5", "POLLING", {
        spec_json: {
          dep: "수서",
          arr: "부산",
          date: "2026-02-22",
          passengers: { adults: 0, children: 1 },
          selected_trains_ranked: [],
        },
      }),
    );
    expect(oneChild.passengerLabel).toBe("0 adults, 1 child");

    const nonStringDateAndRanks = taskInfoFromSpec(
      makeTask("task-6", "POLLING", {
        spec_json: {
          dep: 123,
          arr: false,
          date: 20260222,
          passengers: { adults: "x", children: "y" },
          selected_trains_ranked: "bad",
        } as unknown as Record<string, unknown>,
      }),
    );
    expect(nonStringDateAndRanks.dep).toBe("-");
    expect(nonStringDateAndRanks.arr).toBe("-");
    expect(nonStringDateAndRanks.scheduleLabel).toBe("-");
    expect(nonStringDateAndRanks.passengerLabel).toBe("0 adults, 0 children");

    const nonStringDepartureAt = taskInfoFromSpec(
      makeTask("task-7", "POLLING", {
        spec_json: {
          dep: "수서",
          arr: "부산",
          date: "2026-02-22",
          passengers: { adults: 1, children: 0 },
          selected_trains_ranked: [{ rank: 1, departure_at: 12345 }],
        } as unknown as Record<string, unknown>,
      }),
    );
    expect(nonStringDepartureAt.scheduleLabel).toBe("02/22/2026");

    const key1 = taskSummaryRenderKey(task);
    const key2 = taskSummaryRenderKey({ ...task, last_attempt_error_code: "x" });
    expect(key1).not.toBe(key2);
    expect(taskListRenderKey([task]).length).toBeGreaterThan(0);
  });

  it("sorts tasks by schedule proximity with awaiting-payment pinned first", () => {
    const awaiting = makeTask("awaiting", "COMPLETED", {
      ticket_status: "awaiting_payment",
      ticket_paid: false,
      spec_json: {
        selected_trains_ranked: [{ rank: 1, departure_at: "2026-02-22T09:30:00+09:00" }],
      },
    });
    const early = makeTask("early", "POLLING", {
      created_at: "2026-02-22T08:00:00+09:00",
      spec_json: {
        selected_trains_ranked: [{ rank: 1, departure_at: "2026-02-22T10:30:00+09:00" }],
      },
    });
    const late = makeTask("late", "POLLING", {
      created_at: "2026-02-22T09:00:00+09:00",
      spec_json: {
        selected_trains_ranked: [{ rank: 1, departure_at: "2026-02-22T11:30:00+09:00" }],
      },
    });
    const noSchedule = makeTask("none", "POLLING", { spec_json: { selected_trains_ranked: [] } });

    const asc = sortTasksByScheduleProximity([late, noSchedule, awaiting, early], "asc").map((task) => task.id);
    expect(asc[0]).toBe("awaiting");
    expect(asc[1]).toBe("early");
    expect(asc[2]).toBe("late");

    const desc = sortTasksByScheduleProximity([late, noSchedule, awaiting, early], "desc").map((task) => task.id);
    expect(desc[0]).toBe("awaiting");
    expect(desc[1]).toBe("late");
    expect(taskPrimaryDepartureAtMs(early)).not.toBeNull();
    expect(taskPrimaryDepartureAtMs(noSchedule)).toBeNull();
    expect(taskPrimaryDepartureAtMs(makeTask("bad", "POLLING", { spec_json: "bad" as unknown as Record<string, unknown> }))).toBeNull();

    const unsortedRanks = makeTask("ranked", "POLLING", {
      spec_json: {
        selected_trains_ranked: [
          { rank: 3, departure_at: "2026-02-22T12:30:00+09:00" },
          { rank: 1, departure_at: "2026-02-22T10:30:00+09:00" },
        ],
      },
    });
    expect(taskPrimaryDepartureAtMs(unsortedRanks)).toBe(new Date("2026-02-22T10:30:00+09:00").getTime());

    const invalidDeparture = makeTask("invalid-dep", "POLLING", {
      spec_json: {
        selected_trains_ranked: [{ rank: 1, departure_at: "not-a-date" }],
      },
    });
    expect(taskPrimaryDepartureAtMs(invalidDeparture)).toBeNull();

    const nonArrayDepartureList = makeTask("invalid-dep-list", "POLLING", {
      spec_json: {
        selected_trains_ranked: "bad",
      } as unknown as Record<string, unknown>,
    });
    expect(taskPrimaryDepartureAtMs(nonArrayDepartureList)).toBeNull();

    const nonStringDepartureValue = makeTask("invalid-dep-type", "POLLING", {
      spec_json: {
        selected_trains_ranked: [{ rank: 1, departure_at: 9999 }],
      } as unknown as Record<string, unknown>,
    });
    expect(taskPrimaryDepartureAtMs(nonStringDepartureValue)).toBeNull();

    const invalidRankFallback = makeTask("invalid-rank", "POLLING", {
      spec_json: {
        selected_trains_ranked: [{ rank: "bad-rank", departure_at: "2026-02-22T10:30:00+09:00" }],
      } as unknown as Record<string, unknown>,
    });
    expect(taskPrimaryDepartureAtMs(invalidRankFallback)).toBe(new Date("2026-02-22T10:30:00+09:00").getTime());

    const invalidCreatedA = makeTask("invalid-a", "POLLING", {
      created_at: "bad-created-a",
      spec_json: { selected_trains_ranked: [] },
    });
    const invalidCreatedB = makeTask("invalid-b", "POLLING", {
      created_at: "bad-created-b",
      spec_json: { selected_trains_ranked: [] },
    });
    const fallbackSorted = sortTasksByScheduleProximity([invalidCreatedB, invalidCreatedA], "asc").map((task) => task.id);
    expect(fallbackSorted).toEqual(["invalid-b", "invalid-a"]);
  });

  it("handles server-side viewport detection fallback", () => {
    const originalWindow = globalThis.window;
    delete (globalThis as { window?: Window }).window;
    try {
      expect(isMobileViewport()).toBe(false);
    } finally {
      vi.stubGlobal("window", originalWindow);
    }
  });
});
