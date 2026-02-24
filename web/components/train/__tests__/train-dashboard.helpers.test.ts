import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";

import {
  buildDummyTaskCards,
  clampPassengerCount,
  fetchTasksByStatus,
  formatScheduleDateWithWeekday,
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
  sortActiveTasksByImminence,
  taskDisplayState,
  taskRetryCountdown,
  taskInfoFromSpec,
  taskTicketSeatLabel,
  taskTicketTrainLabel,
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
    const getTaskById = (list: TrainTaskSummary[], id: string): TrainTaskSummary => {
      const task = list.find((item) => item.id === id);
      expect(task).toBeDefined();
      return task as TrainTaskSummary;
    };

    const awaitingPaymentActiveTask = getTaskById(cards.active, "dummy-completed-awaiting-payment");
    expect(awaitingPaymentActiveTask.state).toBe("COMPLETED");
    expect(awaitingPaymentActiveTask.ticket_status).toBe("awaiting_payment");
    expect(awaitingPaymentActiveTask.ticket_paid).toBe(false);
    expect(awaitingPaymentActiveTask.ticket_payment_deadline_at).not.toBeNull();
    expect(awaitingPaymentActiveTask.ticket_reservation_id).toBe("DUMMY-RSV-002");
    const awaitingInfo = taskInfoFromSpec(awaitingPaymentActiveTask);
    expect(awaitingInfo.dep).toBe("수서");
    expect(awaitingInfo.arr).toBe("부산");
    expect(awaitingInfo.scheduleOptionCount).toBe(1);
    expect(awaitingInfo.scheduleOptions).toEqual([{ rank: 1, provider: "KTX", timeLabel: "13:10" }]);
    expect(Math.max(0, awaitingInfo.scheduleOptionCount - 1)).toBe(0);

    const multiScheduleTask = getTaskById(cards.active, "dummy-polling-multi-schedule");
    const ranked = Array.isArray(multiScheduleTask.spec_json?.selected_trains_ranked)
      ? multiScheduleTask.spec_json.selected_trains_ranked
      : [];
    expect(ranked).toHaveLength(3);
    expect(multiScheduleTask.spec_json?.dep).toBe("수서");
    expect(multiScheduleTask.spec_json?.arr).toBe("마산");
    const multiInfo = taskInfoFromSpec(multiScheduleTask);
    expect(multiInfo.scheduleOptionCount).toBe(3);
    expect(multiInfo.scheduleOptions).toEqual([
      { rank: 1, provider: "SRT", timeLabel: "21:24" },
      { rank: 2, provider: "SRT", timeLabel: "21:38" },
      { rank: 3, provider: "SRT", timeLabel: "21:56" },
    ]);
    expect(Math.max(0, multiInfo.scheduleOptionCount - 1)).toBe(2);

    const confirmedCompletedTask = getTaskById(cards.completed, "dummy-completed-confirmed");
    expect(confirmedCompletedTask.state).toBe("COMPLETED");
    expect(confirmedCompletedTask.ticket_status).toBe("ticket_issued");
    expect(confirmedCompletedTask.ticket_paid).toBe(true);
    expect(confirmedCompletedTask.ticket_reservation_id).toBe("DUMMY-RSV-001");
    expect(confirmedCompletedTask.ticket_train_no).toBe("311");
    expect(confirmedCompletedTask.ticket_seat_count).toBe(1);
    expect(confirmedCompletedTask.ticket_seats).toEqual(["8-12A"]);
    expect(taskTicketTrainLabel(confirmedCompletedTask)).toBe("SRT 311");
    expect(taskTicketSeatLabel(confirmedCompletedTask)).toBe("Car 8 - 12A");

    const failedCompletedTask = getTaskById(cards.completed, "dummy-failed");
    expect(failedCompletedTask.state).toBe("FAILED");
    expect(failedCompletedTask.last_attempt_error_code).toBe("provider_transport_error");
    expect(failedCompletedTask.last_attempt_error_message_safe).toContain("Provider transport failed");
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
    expect(getTaskTicketBadge(makeTask("waitlisted", "POLLING", { ticket_status: "waiting", ticket_paid: false }))?.label).toBe(
      "train.badge.waitlisted",
    );
    expect(getTaskTicketBadge(makeTask("paid", "COMPLETED", { ticket_status: "ticket_issued", ticket_paid: true }))?.label).toBe(
      "train.badge.confirmed",
    );
    expect(getTaskTicketBadge(makeTask("other", "COMPLETED", { ticket_status: "custom_state" }))?.label).toBe(
      "Custom State",
    );
    expect(getTaskTicketBadge(makeTask("other-empty", "COMPLETED", { ticket_status: "_" }))?.label).toBe("_");

    expect(taskTicketTrainLabel(makeTask("train-default", "COMPLETED", { ticket_train_no: "301" }))).toBe("301");
    expect(
      taskTicketTrainLabel(
        makeTask("train-with-provider", "COMPLETED", {
          ticket_train_no: "311",
          spec_json: {
            dep: "수서",
            arr: "부산",
            date: "2026-02-22",
            passengers: { adults: 1, children: 0 },
            selected_trains_ranked: [{ rank: 1, departure_at: "2026-02-22T12:30:00+09:00", provider: "SRT" }],
          },
        }),
      ),
    ).toBe("SRT 311");
    expect(taskTicketSeatLabel(makeTask("seat-list", "COMPLETED", { ticket_seats: ["8-12A", "8-12A", "14A"] }))).toBe(
      "Car 8 - 12A, 14A",
    );
    expect(taskTicketSeatLabel(makeTask("seat-list-ko", "COMPLETED", { ticket_seats: ["8-12A", "14A"] }), "ko")).toBe(
      "8호차 - 12A, 14A",
    );
    expect(taskTicketSeatLabel(makeTask("seat-count", "COMPLETED", { ticket_seat_count: 2 }))).toBe("2");
    expect(taskTicketSeatLabel(makeTask("seat-none", "COMPLETED"))).toBeNull();
  });

  it("formats ticket train/seat labels with provider fallback, locale handling, and dedupe", () => {
    expect(taskTicketTrainLabel(makeTask("train-trimmed", "COMPLETED", { ticket_train_no: " 311 " }))).toBe("311");
    expect(taskTicketTrainLabel(makeTask("train-empty", "COMPLETED", { ticket_train_no: "   " }))).toBeNull();
    expect(
      taskTicketTrainLabel(
        makeTask("train-provider-field", "COMPLETED", {
          ticket_train_no: "511",
          spec_json: { provider: "  KTX  " } as unknown as Record<string, unknown>,
        }),
      ),
    ).toBe("KTX 511");
    expect(
      taskTicketTrainLabel(
        makeTask("train-provider-list", "COMPLETED", {
          ticket_train_no: "611",
          spec_json: { providers: [" ", "SRT", "KTX"] } as unknown as Record<string, unknown>,
        }),
      ),
    ).toBe("SRT 611");
    expect(
      taskTicketTrainLabel(
        makeTask("train-provider-ranked", "COMPLETED", {
          ticket_train_no: "711",
          spec_json: {
            selected_trains_ranked: [
              { rank: 3, departure_at: "2026-02-22T12:40:00+09:00", provider: " KTX " },
              { rank: 1, departure_at: "2026-02-22T12:10:00+09:00", provider: " SRT " },
            ],
          },
        }),
      ),
    ).toBe("SRT 711");
    expect(
      taskTicketTrainLabel(
        makeTask("train-provider-invalid-spec", "COMPLETED", {
          ticket_train_no: "811",
          spec_json: "bad" as unknown as Record<string, unknown>,
        }),
      ),
    ).toBe("811");

    expect(
      taskTicketSeatLabel({
        ticket_seat_count: 4,
        ticket_seats: [" 8-12A ", "8-12A", "2 -  3C", "14A", "", "   "],
      }),
    ).toBe("Car 8 - 12A, Car 2 - 3C, 14A");
    expect(
      taskTicketSeatLabel({
        ticket_seat_count: 4,
        ticket_seats: ["2-3C", "2-3C", "10B"],
      }, "ko-KR"),
    ).toBe("2호차 - 3C, 10B");
    expect(
      taskTicketSeatLabel({
        ticket_seat_count: 4,
        ticket_seats: [null, 7, "9-1A"] as unknown as string[],
      }),
    ).toBe("Car 9 - 1A");
    expect(taskTicketSeatLabel({ ticket_seat_count: 2.9, ticket_seats: null })).toBe("2");
    expect(taskTicketSeatLabel({ ticket_seat_count: 0, ticket_seats: null })).toBeNull();
    expect(taskTicketSeatLabel({ ticket_seat_count: Number.NaN, ticket_seats: null })).toBeNull();
  });

  it("maps waitlisted and awaiting-payment cards to pending display state", () => {
    expect(taskDisplayState(makeTask("w", "POLLING", { ticket_status: "waiting", ticket_paid: false }))).toBe("PENDING");
    expect(taskDisplayState(makeTask("a", "COMPLETED", { ticket_status: "awaiting_payment", ticket_paid: false }))).toBe(
      "PENDING",
    );
    expect(taskDisplayState(makeTask("d", "RUNNING", { ticket_status: null, ticket_paid: null }))).toBe("RUNNING");
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
    expect(formatScheduleDateWithWeekday("2026-02-23")).toBe("02/23/2026 (Monday)");
    expect(formatScheduleDateWithWeekday("")).toBe("MM/DD/YYYY (Weekday)");
    expect(formatScheduleDateWithWeekday("bad")).toBe("MM/DD/YYYY (Weekday)");

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
    expect(info.travelDateLabel).toBe("02/22/2026 (Sunday)");
    expect(info.primaryDepartureLabel).toBe("10:30");
    expect(info.primaryArrivalLabel).toBe("-");
    expect(info.scheduleLabel).toContain("02/22/2026");
    expect(info.scheduleOptionCount).toBe(2);
    expect(info.passengerLabel).toBe("1 adult");

    const fallback = taskInfoFromSpec(makeTask("task-2", "POLLING", { spec_json: "invalid" as unknown as Record<string, unknown> }));
    expect(fallback.travelDateLabel).toBe("MM/DD/YYYY (Weekday)");
    expect(fallback.primaryDepartureLabel).toBe("-");
    expect(fallback.primaryArrivalLabel).toBe("-");
    expect(fallback.scheduleLabel).toBe("-");
    expect(fallback.scheduleOptionCount).toBe(0);

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
    expect(dateOnly.travelDateLabel).toBe("02/22/2026 (Sunday)");
    expect(dateOnly.scheduleLabel).toBe("02/22/2026");
    expect(dateOnly.scheduleOptionCount).toBe(0);

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
    expect(fullyFallback.scheduleOptionCount).toBe(0);
    expect(fullyFallback.passengerLabel).toBe("-");

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
    expect(oneChild.passengerLabel).toBe("1 child");

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
    expect(nonStringDateAndRanks.scheduleOptionCount).toBe(0);
    expect(nonStringDateAndRanks.passengerLabel).toBe("-");

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
    expect(nonStringDepartureAt.scheduleOptionCount).toBe(0);

    const unsortedAndMixedRanks = taskInfoFromSpec(
      makeTask("task-7b", "POLLING", {
        spec_json: {
          dep: "수서",
          arr: "진주",
          date: "2026-02-28",
          passengers: { adults: 2, children: 1 },
          selected_trains_ranked: [
            null,
            { rank: 3, departure_at: "2026-02-28T09:30:00+09:00", provider: "  KTX " },
            { rank: 1, departure_at: "2026-02-28T08:10:00+09:00", provider: "  SRT " },
            { rank: 2, departure_at: "", provider: "SRT" },
            { rank: "bad", departure_at: "not-a-date", provider: "   " },
            { rank: 5, departure_at: 9999, provider: "SRT" },
          ],
        } as unknown as Record<string, unknown>,
      }),
    );
    expect(unsortedAndMixedRanks.scheduleLabel).toBe("02/28/2026 08:10");
    expect(unsortedAndMixedRanks.scheduleOptionCount).toBe(3);
    expect(unsortedAndMixedRanks.scheduleOptions).toEqual([
      { rank: 1, provider: "SRT", timeLabel: "08:10" },
      { rank: 3, provider: "KTX", timeLabel: "09:30" },
    ]);
    expect(Math.max(0, unsortedAndMixedRanks.scheduleOptionCount - 1)).toBe(2);
    expect(unsortedAndMixedRanks.passengerLabel).toBe("2 adults, 1 child");
    const unsortedAndMixedRanksKo = taskInfoFromSpec(
      makeTask("task-7c", "POLLING", {
        spec_json: {
          dep: "수서",
          arr: "진주",
          date: "2026-02-28",
          passengers: { adults: 2, children: 1 },
          selected_trains_ranked: [],
        } as unknown as Record<string, unknown>,
      }),
      "ko",
    );
    expect(unsortedAndMixedRanksKo.passengerLabel).toBe("성인 2, 아동 1");

    const multiRank = taskInfoFromSpec(
      makeTask("task-8", "POLLING", {
        spec_json: {
          dep: "수서",
          arr: "마산",
          date: "2026-02-27",
          passengers: { adults: 1, children: 0 },
          selected_trains_ranked: [
            { rank: 1, departure_at: "2026-02-27T19:24:00+09:00", arrival_at: "2026-02-27T21:44:00+09:00" },
            { rank: 2, departure_at: "2026-02-27T19:38:00+09:00", arrival_at: "2026-02-27T21:58:00+09:00" },
            { rank: 3, departure_at: "2026-02-27T19:56:00+09:00", arrival_at: "2026-02-27T22:16:00+09:00" },
          ],
        },
      }),
    );
    expect(multiRank.scheduleLabel).toContain("02/27/2026 19:24");
    expect(multiRank.travelDateLabel).toBe("02/27/2026 (Friday)");
    expect(multiRank.primaryDepartureLabel).toBe("19:24");
    expect(multiRank.primaryArrivalLabel).toBe("21:44");
    expect(multiRank.scheduleTimeOptions).toEqual([
      { rank: 1, provider: null, departureLabel: "19:24", arrivalLabel: "21:44" },
      { rank: 2, provider: null, departureLabel: "19:38", arrivalLabel: "21:58" },
      { rank: 3, provider: null, departureLabel: "19:56", arrivalLabel: "22:16" },
    ]);
    expect(multiRank.scheduleOptionCount).toBe(3);
    expect(multiRank.scheduleOptions).toEqual([
      { rank: 1, provider: null, timeLabel: "19:24" },
      { rank: 2, provider: null, timeLabel: "19:38" },
      { rank: 3, provider: null, timeLabel: "19:56" },
    ]);
    expect(Math.max(0, multiRank.scheduleOptionCount - 1)).toBe(2);
    expect(Math.max(0, dateOnly.scheduleOptionCount - 1)).toBe(0);
    expect(Math.max(0, fallback.scheduleOptionCount - 1)).toBe(0);

    const key1 = taskSummaryRenderKey(task);
    const key2 = taskSummaryRenderKey({ ...task, last_attempt_error_code: "x" });
    expect(key1).not.toBe(key2);

    const ticketKeyBase = makeTask("task-ticket-key", "COMPLETED", {
      ticket_status: "awaiting_payment",
      ticket_paid: false,
      ticket_payment_deadline_at: "2026-02-22T11:00:00+09:00",
      ticket_reservation_id: "RSV-1",
      ticket_train_no: "311",
      ticket_seat_count: 2,
      ticket_seats: ["8-12A", "8-14A"],
    });
    const ticketBaseKey = taskSummaryRenderKey(ticketKeyBase);
    expect(
      taskSummaryRenderKey({
        ...ticketKeyBase,
        ticket_seats: ["8-12A", "8-14A"],
      }),
    ).toBe(ticketBaseKey);
    expect(taskSummaryRenderKey({ ...ticketKeyBase, ticket_status: "ticket_issued" })).not.toBe(ticketBaseKey);
    expect(taskSummaryRenderKey({ ...ticketKeyBase, ticket_paid: true })).not.toBe(ticketBaseKey);
    expect(
      taskSummaryRenderKey({
        ...ticketKeyBase,
        ticket_payment_deadline_at: "2026-02-22T11:30:00+09:00",
      }),
    ).not.toBe(ticketBaseKey);
    expect(taskSummaryRenderKey({ ...ticketKeyBase, ticket_reservation_id: "RSV-2" })).not.toBe(ticketBaseKey);
    expect(taskSummaryRenderKey({ ...ticketKeyBase, ticket_train_no: "312" })).not.toBe(ticketBaseKey);
    expect(taskSummaryRenderKey({ ...ticketKeyBase, ticket_seat_count: 3 })).not.toBe(ticketBaseKey);
    expect(
      taskSummaryRenderKey({
        ...ticketKeyBase,
        ticket_seats: ["8-14A", "8-12A"],
      }),
    ).not.toBe(ticketBaseKey);

    expect(taskListRenderKey([task]).length).toBeGreaterThan(0);
  });

  it("computes functional retry countdown progress from attempt window timestamps", () => {
    const task = makeTask("retry-window", "POLLING", {
      last_attempt_at: "2026-02-22T10:00:00+09:00",
      last_attempt_finished_at: "2026-02-22T10:00:00+09:00",
      next_run_at: "2026-02-22T10:05:00+09:00",
    });

    const nowMid = new Date("2026-02-22T10:02:00+09:00").getTime();
    const mid = taskRetryCountdown(task, nowMid);
    expect(mid).not.toBeNull();
    expect(mid?.remainingMs).toBe(180_000);
    expect(mid?.windowMs).toBe(300_000);
    expect(mid?.elapsedMs).toBe(120_000);
    expect(mid?.progressPercent).toBe(40);
    expect(mid?.isDue).toBe(false);

    const nowLate = new Date("2026-02-22T10:07:00+09:00").getTime();
    const late = taskRetryCountdown(task, nowLate);
    expect(late?.remainingMs).toBe(0);
    expect(late?.progressPercent).toBe(100);
    expect(late?.isDue).toBe(true);

    const noWindowTask = makeTask("retry-no-window", "POLLING", {
      last_attempt_at: null,
      last_attempt_finished_at: null,
      next_run_at: "2026-02-22T10:05:00+09:00",
    });
    const noWindow = taskRetryCountdown(noWindowTask, nowMid);
    expect(noWindow?.progressPercent).toBeNull();
    expect(noWindow?.windowMs).toBeNull();

    const noNextRunTask = makeTask("retry-none", "POLLING", {
      next_run_at: null,
    });
    expect(taskRetryCountdown(noNextRunTask, nowMid)).toBeNull();
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

  it("sorts active tasks by most imminent ranked departure", () => {
    const awaitingLater = makeTask("awaiting-later", "POLLING", {
      created_at: "2026-02-22T07:00:00+09:00",
      ticket_status: "awaiting_payment",
      ticket_paid: false,
      spec_json: {
        selected_trains_ranked: [{ rank: 1, departure_at: "2026-02-22T12:30:00+09:00" }],
      },
    });
    const earliest = makeTask("earliest", "POLLING", {
      created_at: "2026-02-22T09:00:00+09:00",
      spec_json: {
        selected_trains_ranked: [{ rank: 1, departure_at: "2026-02-22T10:30:00+09:00" }],
      },
    });
    const noSchedule = makeTask("none", "POLLING", {
      created_at: "2026-02-22T06:00:00+09:00",
      spec_json: { selected_trains_ranked: [] },
    });

    const sorted = sortActiveTasksByImminence([awaitingLater, noSchedule, earliest]).map((task) => task.id);
    expect(sorted).toEqual(["earliest", "awaiting-later", "none"]);
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
