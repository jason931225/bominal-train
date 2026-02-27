import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";

describe("train data api reader", () => {
  const originalSupabaseUrl = process.env.NEXT_PUBLIC_SUPABASE_URL;
  const originalSupabaseAnonKey = process.env.NEXT_PUBLIC_SUPABASE_ANON_KEY;

  beforeEach(() => {
    vi.resetModules();
    vi.restoreAllMocks();
    vi.unstubAllGlobals();
    process.env.NEXT_PUBLIC_SUPABASE_URL = "https://test-ref.supabase.co";
    process.env.NEXT_PUBLIC_SUPABASE_ANON_KEY = "anon-key";
  });

  afterEach(() => {
    vi.restoreAllMocks();
    vi.unstubAllGlobals();
    process.env.NEXT_PUBLIC_SUPABASE_URL = originalSupabaseUrl;
    process.env.NEXT_PUBLIC_SUPABASE_ANON_KEY = originalSupabaseAnonKey;
  });

  it("returns null when access token is unavailable", async () => {
    vi.doMock("@/lib/feature-flags", () => ({
      NEXT_PUBLIC_TRAIN_READS_VIA_DATA_API: true,
    }));
    vi.doMock("@/lib/supabase-auth", () => ({
      isSupabaseDirectAuthEnabled: vi.fn().mockReturnValue(true),
      getSupabaseAccessToken: vi.fn().mockResolvedValue(null),
    }));

    const { fetchTrainTaskListViaDataApi } = await import("@/lib/train/data-api");
    const result = await fetchTrainTaskListViaDataApi({ status: "active", limit: 20 });
    expect(result).toBeNull();
  });

  it("fetches and normalizes task list rows", async () => {
    const fetchMock = vi.fn<(input: RequestInfo | URL, init?: RequestInit) => Promise<Response>>(async () =>
      new Response(
        JSON.stringify([
          {
            id: "task-1",
            module: "train",
            state: "polling",
            deadline_at: "2026-02-27T10:00:00Z",
            created_at: "2026-02-27T09:00:00Z",
            updated_at: "2026-02-27T09:30:00Z",
            paused_at: null,
            cancelled_at: null,
            completed_at: null,
            failed_at: null,
            hidden_at: null,
            last_attempt_at: "2026-02-27T09:29:00Z",
            last_attempt_action: "SEARCH",
            last_attempt_ok: false,
            last_attempt_error_code: "seat_unavailable",
            last_attempt_error_message_safe: "No seats",
            last_attempt_finished_at: "2026-02-27T09:29:01Z",
            next_run_at: "2026-02-27T09:30:05Z",
            retry_now_allowed: false,
            retry_now_reason: "cooldown_active",
            retry_now_available_at: "2026-02-27T09:30:15Z",
            spec_json: { dep: "수서", arr: "부산" },
            ticket_status: "waiting",
            ticket_paid: false,
            ticket_payment_deadline_at: null,
            ticket_reservation_id: "RSV-1",
            ticket_train_no: "301",
            ticket_seat_count: 2,
            ticket_seats: ["8-12A", "8-12A"],
            ticket_seat_classes: ["special", "general", "special"],
            list_bucket: "active",
          },
        ]),
        { status: 200, headers: { "Content-Type": "application/json" } },
      ),
    );
    vi.stubGlobal("fetch", fetchMock);

    vi.doMock("@/lib/feature-flags", () => ({
      NEXT_PUBLIC_TRAIN_READS_VIA_DATA_API: true,
    }));
    vi.doMock("@/lib/supabase-auth", () => ({
      isSupabaseDirectAuthEnabled: vi.fn().mockReturnValue(true),
      getSupabaseAccessToken: vi.fn().mockResolvedValue("jwt-token"),
    }));

    const { fetchTrainTaskListViaDataApi } = await import("@/lib/train/data-api");
    const result = await fetchTrainTaskListViaDataApi({ status: "active", limit: 20 });

    expect(fetchMock).toHaveBeenCalledTimes(1);
    const requestedUrl = String(fetchMock.mock.calls[0]?.[0] ?? "");
    expect(requestedUrl).toContain("/rest/v1/v_train_task_list_compact?");
    expect(requestedUrl).toContain("list_bucket=eq.active");
    expect(result).toEqual([
      {
        id: "task-1",
        module: "train",
        state: "POLLING",
        deadline_at: "2026-02-27T10:00:00Z",
        created_at: "2026-02-27T09:00:00Z",
        updated_at: "2026-02-27T09:30:00Z",
        paused_at: null,
        cancelled_at: null,
        completed_at: null,
        failed_at: null,
        hidden_at: null,
        last_attempt_at: "2026-02-27T09:29:00Z",
        last_attempt_action: "SEARCH",
        last_attempt_ok: false,
        last_attempt_error_code: "seat_unavailable",
        last_attempt_error_message_safe: "No seats",
        last_attempt_finished_at: "2026-02-27T09:29:01Z",
        next_run_at: "2026-02-27T09:30:05Z",
        retry_now_allowed: false,
        retry_now_reason: "cooldown_active",
        retry_now_available_at: "2026-02-27T09:30:15Z",
        spec_json: { dep: "수서", arr: "부산" },
        ticket_status: "waiting",
        ticket_paid: false,
        ticket_payment_deadline_at: null,
        ticket_reservation_id: "RSV-1",
        ticket_train_no: "301",
        ticket_seat_count: 2,
        ticket_seats: ["8-12A"],
        ticket_seat_classes: ["special", "general"],
      },
    ]);
  });
});
