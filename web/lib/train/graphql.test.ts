import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";

describe("train graphql detail reader", () => {
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
      NEXT_PUBLIC_TRAIN_DETAIL_VIA_GRAPHQL: true,
    }));
    vi.doMock("@/lib/supabase-auth", () => ({
      isSupabaseDirectAuthEnabled: vi.fn().mockReturnValue(true),
      getSupabaseAccessToken: vi.fn().mockResolvedValue(null),
    }));

    const { fetchTrainTaskDetailViaGraphql } = await import("@/lib/train/graphql");
    const result = await fetchTrainTaskDetailViaGraphql("task-1");
    expect(result).toBeNull();
  });

  it("maps task detail payload from graphql response", async () => {
    const fetchMock = vi.fn<(input: RequestInfo | URL, init?: RequestInit) => Promise<Response>>(async () =>
      new Response(
        JSON.stringify({
          data: {
            tasksCollection: {
              edges: [
                {
                  node: {
                    id: "task-1",
                    module: "train",
                    state: "COMPLETED",
                    deadlineAt: "2026-02-27T10:00:00Z",
                    createdAt: "2026-02-27T09:00:00Z",
                    updatedAt: "2026-02-27T09:31:00Z",
                    pausedAt: null,
                    cancelledAt: null,
                    completedAt: "2026-02-27T09:31:00Z",
                    failedAt: null,
                    hiddenAt: null,
                    specJson: { dep: "수서", arr: "부산" },
                    taskAttemptsCollection: {
                      edges: [
                        {
                          node: {
                            id: "attempt-1",
                            action: "SEARCH",
                            provider: "SRT",
                            ok: true,
                            retryable: false,
                            errorCode: null,
                            errorMessageSafe: null,
                            durationMs: 120,
                            metaJsonSafe: { foo: "bar" },
                            startedAt: "2026-02-27T09:29:00Z",
                            finishedAt: "2026-02-27T09:29:02Z",
                          },
                        },
                      ],
                    },
                    artifactsCollection: {
                      edges: [
                        {
                          node: {
                            id: "artifact-1",
                            module: "train",
                            kind: "ticket",
                            dataJsonSafe: {
                              status: "paid",
                              paid: true,
                              reservation_id: "RSV-1",
                              train_no: "301",
                              seat_count: 1,
                              tickets: [{ car_no: "8", seat_no: "12A", seat_class_code: "2" }],
                            },
                            createdAt: "2026-02-27T09:30:00Z",
                          },
                        },
                      ],
                    },
                  },
                },
              ],
            },
          },
        }),
        { status: 200, headers: { "Content-Type": "application/json" } },
      ),
    );
    vi.stubGlobal("fetch", fetchMock);

    vi.doMock("@/lib/feature-flags", () => ({
      NEXT_PUBLIC_TRAIN_DETAIL_VIA_GRAPHQL: true,
    }));
    vi.doMock("@/lib/supabase-auth", () => ({
      isSupabaseDirectAuthEnabled: vi.fn().mockReturnValue(true),
      getSupabaseAccessToken: vi.fn().mockResolvedValue("jwt-token"),
    }));

    const { fetchTrainTaskDetailViaGraphql } = await import("@/lib/train/graphql");
    const result = await fetchTrainTaskDetailViaGraphql("task-1");

    expect(fetchMock).toHaveBeenCalledTimes(1);
    const requestedUrl = String(fetchMock.mock.calls[0]?.[0] ?? "");
    expect(requestedUrl).toContain("/graphql/v1");
    expect(result?.task.id).toBe("task-1");
    expect(result?.task.ticket_status).toBe("paid");
    expect(result?.task.ticket_paid).toBe(true);
    expect(result?.task.ticket_train_no).toBe("301");
    expect(result?.task.ticket_seats).toEqual(["8-12A"]);
    expect(result?.task.ticket_seat_classes).toEqual(["special"]);
    expect(result?.attempts).toHaveLength(1);
    expect(result?.artifacts).toHaveLength(1);
  });
});
