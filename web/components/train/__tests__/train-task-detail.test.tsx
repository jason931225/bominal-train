import React from "react";

import { act, fireEvent, render, screen, waitFor, within } from "@testing-library/react";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";

import { LocaleProvider } from "@/components/locale-provider";
import { TrainTaskDetail } from "@/components/train/train-task-detail";

const pushMock = vi.fn();

vi.mock("next/navigation", async () => {
  const actual = await vi.importActual<typeof import("next/navigation")>("next/navigation");
  return {
    ...actual,
    useRouter: () => ({ push: pushMock }),
  };
});

vi.mock("@/lib/train/task-events", () => ({
  subscribeTrainTaskEvents: () => () => {},
}));

function jsonResponse(payload: unknown, status = 200): Response {
  return new Response(JSON.stringify(payload), {
    status,
    headers: { "Content-Type": "application/json" },
  });
}

function buildTaskDetailPayload() {
  return {
    task: {
      id: "task-1",
      module: "train",
      state: "RUNNING",
      deadline_at: "2026-02-26T23:59:00+09:00",
      created_at: "2026-02-26T10:00:00+09:00",
      updated_at: "2026-02-26T10:02:00+09:00",
      paused_at: null,
      cancelled_at: null,
      completed_at: null,
      failed_at: null,
      hidden_at: null,
      last_attempt_at: "2026-02-26T10:01:00+09:00",
      last_attempt_action: "SEARCH",
      last_attempt_ok: true,
      last_attempt_error_code: null,
      last_attempt_error_message_safe: null,
      last_attempt_finished_at: "2026-02-26T10:01:00+09:00",
      next_run_at: null,
      retry_now_allowed: false,
      retry_now_reason: "task_running",
      retry_now_available_at: null,
      spec_json: {
        dep: "수서",
        arr: "부산",
        date: "2026-02-26",
      },
      ticket_status: "awaiting_payment",
      ticket_paid: false,
      ticket_payment_deadline_at: "2026-02-26T10:20:00+09:00",
      ticket_reservation_id: "PNR-1",
      ticket_train_no: "301",
      ticket_seat_count: 1,
      ticket_seats: ["3-4A"],
    },
    attempts: [],
    artifacts: [
      {
        id: "ticket-1",
        module: "train",
        kind: "ticket",
        data_json_safe: {
          provider: "SRT",
          reservation_id: "PNR-1",
          status: "awaiting_payment",
          paid: false,
          payment_deadline_at: "2026-02-26T10:20:00+09:00",
        },
        created_at: "2026-02-26T10:00:00+09:00",
      },
    ],
  };
}

async function flushAsyncEffects() {
  await act(async () => {
    await Promise.resolve();
    await Promise.resolve();
  });
}

describe("TrainTaskDetail actions", () => {
  beforeEach(() => {
    pushMock.mockReset();
    vi.stubGlobal("confirm", vi.fn(() => true));
    vi.spyOn(window, "confirm").mockReturnValue(true);
  });

  afterEach(() => {
    vi.unstubAllGlobals();
    vi.restoreAllMocks();
  });

  it("supports sync refresh and orders awaiting-payment actions as pay, pause, cancel", async () => {
    const fetchMock = vi.fn(async (input: RequestInfo | URL, init?: RequestInit) => {
      const url = String(input);
      const method = init?.method ?? "GET";
      if (url.endsWith("/api/train/tasks/task-1") && method === "GET") {
        return jsonResponse(buildTaskDetailPayload());
      }
      if (url.endsWith("/api/train/tasks/task-1/refresh") && method === "POST") {
        return jsonResponse(buildTaskDetailPayload());
      }
      if (url.endsWith("/api/train/tasks/task-1/pause") && method === "POST") {
        return jsonResponse({ detail: "paused" });
      }
      if (url.endsWith("/api/train/tasks/task-1/cancel") && method === "POST") {
        return jsonResponse({ detail: "cancelled" });
      }
      if (url.endsWith("/api/train/tasks/task-1/pay") && method === "POST") {
        return jsonResponse({ detail: "paid" });
      }
      return jsonResponse({ detail: "not found" }, 404);
    });
    vi.stubGlobal("fetch", fetchMock);

    render(
      <LocaleProvider initialLocale="en">
        <TrainTaskDetail taskId="task-1" />
      </LocaleProvider>,
    );
    await flushAsyncEffects();

    const taskStatusCard = screen.getByRole("heading", { name: "Task status" }).closest("div");
    expect(taskStatusCard).not.toBeNull();
    const labels = within(taskStatusCard as HTMLElement)
      .getAllByRole("button")
      .map((button) => button.textContent?.trim() ?? "");
    expect(labels.indexOf("Pay")).toBeGreaterThanOrEqual(0);
    expect(labels.indexOf("Pause")).toBeGreaterThan(labels.indexOf("Pay"));
    expect(labels.indexOf("Cancel")).toBeGreaterThan(labels.indexOf("Pause"));

    fireEvent.click(screen.getByRole("button", { name: "Sync refresh" }));
    await waitFor(() =>
      expect(
        fetchMock.mock.calls.some(
          ([request, init]) => String(request).includes("/api/train/tasks/task-1/refresh") && init?.method === "POST",
        ),
      ).toBe(true),
    );
    await waitFor(() => expect(screen.getByRole("button", { name: "Sync refresh" })).toBeEnabled());

    fireEvent.click(screen.getByRole("button", { name: "Cancel" }));
    await waitFor(() =>
      expect(
        fetchMock.mock.calls.some(
          ([request, init]) => String(request).includes("/api/train/tasks/task-1/cancel") && init?.method === "POST",
        ),
      ).toBe(true),
    );
    expect(fetchMock.mock.calls.some(([request]) => String(request).includes("/api/train/tickets/"))).toBe(false);
  });
});
