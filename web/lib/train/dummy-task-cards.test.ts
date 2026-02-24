import { afterAll, afterEach, beforeEach, describe, expect, it, vi } from "vitest";

import {
  clearStoredDummyTaskCards,
  readDummyTaskCardsModeEnabled,
  readDummyTaskCards,
  setDummyTaskCardsModeEnabled,
  storeDummyTaskCards,
  TRAIN_DUMMY_TASKS_ENABLED,
  TRAIN_DUMMY_TASKS_EVENT,
  TRAIN_DUMMY_TASKS_STORAGE_KEY,
} from "@/lib/train/dummy-task-cards";
import type { TrainTaskSummary } from "@/lib/types";

function createMemoryStorage(): Storage {
  const store = new Map<string, string>();
  return {
    get length() {
      return store.size;
    },
    clear() {
      store.clear();
    },
    getItem(key: string) {
      return store.has(key) ? store.get(key)! : null;
    },
    key(index: number) {
      return Array.from(store.keys())[index] ?? null;
    },
    removeItem(key: string) {
      store.delete(key);
    },
    setItem(key: string, value: string) {
      store.set(String(key), String(value));
    },
  } as Storage;
}

function makeTask(id: string): TrainTaskSummary {
  return {
    id,
    module: "train",
    state: "QUEUED",
    deadline_at: "2026-02-22T12:00:00+09:00",
    created_at: "2026-02-22T11:00:00+09:00",
    updated_at: "2026-02-22T11:01:00+09:00",
    paused_at: null,
    cancelled_at: null,
    completed_at: null,
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
      date: "2026-02-22",
      passengers: { adults: 1, children: 0 },
      selected_trains_ranked: [],
    },
    ticket_status: null,
    ticket_paid: null,
    ticket_payment_deadline_at: null,
    ticket_reservation_id: null,
  };
}

function resetDummyStorage(): void {
  window.localStorage.clear();
  window.sessionStorage.clear();
}

describe("dummy task card storage helpers", () => {
  const originalLocalStorage = Object.getOwnPropertyDescriptor(window, "localStorage");
  const originalSessionStorage = Object.getOwnPropertyDescriptor(window, "sessionStorage");

  const installStorageMocks = () => {
    Object.defineProperty(window, "localStorage", {
      configurable: true,
      value: createMemoryStorage(),
    });
    Object.defineProperty(window, "sessionStorage", {
      configurable: true,
      value: createMemoryStorage(),
    });
  };

  beforeEach(() => {
    installStorageMocks();
    resetDummyStorage();
  });

  afterEach(() => {
    vi.restoreAllMocks();
    resetDummyStorage();
  });

  afterAll(() => {
    if (originalLocalStorage) {
      Object.defineProperty(window, "localStorage", originalLocalStorage);
    }
    if (originalSessionStorage) {
      Object.defineProperty(window, "sessionStorage", originalSessionStorage);
    }
  });

  it("enables dummy mode in non-production test runtime", () => {
    expect(TRAIN_DUMMY_TASKS_ENABLED).toBe(true);
  });

  it("stores and reads valid dummy task rows", () => {
    const task = makeTask("dummy-1");
    storeDummyTaskCards([task]);
    const restored = readDummyTaskCards();
    expect(restored).toHaveLength(1);
    expect(restored[0].id).toBe(task.id);
  });

  it("reads and updates session-scoped dummy mode state", () => {
    expect(readDummyTaskCardsModeEnabled()).toBe(false);
    setDummyTaskCardsModeEnabled(true);
    expect(readDummyTaskCardsModeEnabled()).toBe(true);
    setDummyTaskCardsModeEnabled(false);
    expect(readDummyTaskCardsModeEnabled()).toBe(false);
  });

  it("returns an empty list when storage key is absent", () => {
    expect(readDummyTaskCards()).toEqual([]);
  });

  it("filters out invalid rows and malformed payloads", () => {
    const nonStringIdButOtherwiseValid = {
      ...makeTask("placeholder"),
      id: 123,
    };
    window.localStorage.setItem(
      TRAIN_DUMMY_TASKS_STORAGE_KEY,
      JSON.stringify([
        makeTask("valid"),
        nonStringIdButOtherwiseValid,
        { id: "invalid-no-spec", state: "QUEUED", created_at: "x", updated_at: "y" },
        "bad",
      ]),
    );
    expect(readDummyTaskCards().map((row) => row.id)).toEqual(["valid"]);

    window.localStorage.setItem(TRAIN_DUMMY_TASKS_STORAGE_KEY, "{not-json");
    expect(readDummyTaskCards()).toEqual([]);

    window.localStorage.setItem(TRAIN_DUMMY_TASKS_STORAGE_KEY, JSON.stringify({ not: "an-array" }));
    expect(readDummyTaskCards()).toEqual([]);
  });

  it("dispatches change events on store and clear", () => {
    const listener = vi.fn();
    window.addEventListener(TRAIN_DUMMY_TASKS_EVENT, listener);
    try {
      storeDummyTaskCards([makeTask("dummy-2")]);
      clearStoredDummyTaskCards();
      expect(listener).toHaveBeenCalledTimes(2);
    } finally {
      window.removeEventListener(TRAIN_DUMMY_TASKS_EVENT, listener);
    }
  });

  it("clears both dummy task rows and mode state", () => {
    storeDummyTaskCards([makeTask("dummy-4")]);
    setDummyTaskCardsModeEnabled(true);
    expect(readDummyTaskCards()).toHaveLength(1);
    expect(readDummyTaskCardsModeEnabled()).toBe(true);

    clearStoredDummyTaskCards();

    expect(readDummyTaskCards()).toEqual([]);
    expect(readDummyTaskCardsModeEnabled()).toBe(false);
  });

  it("handles storage failures as best-effort and still emits change events", () => {
    const listener = vi.fn();
    const setItemSpy = vi.spyOn(window.localStorage, "setItem").mockImplementation(() => {
      throw new Error("setItem failed");
    });
    const removeItemSpy = vi.spyOn(window.localStorage, "removeItem").mockImplementation(() => {
      throw new Error("removeItem failed");
    });
    window.addEventListener(TRAIN_DUMMY_TASKS_EVENT, listener);
    try {
      expect(() => storeDummyTaskCards([makeTask("dummy-3")])).not.toThrow();
      expect(() => setDummyTaskCardsModeEnabled(true)).not.toThrow();
      expect(() => clearStoredDummyTaskCards()).not.toThrow();
      expect(listener).toHaveBeenCalledTimes(3);
      expect(setItemSpy).toHaveBeenCalled();
      expect(removeItemSpy).toHaveBeenCalled();
    } finally {
      window.removeEventListener(TRAIN_DUMMY_TASKS_EVENT, listener);
    }
  });

  it("fails closed in non-browser runtime without window", () => {
    const originalWindow = globalThis.window;
    vi.stubGlobal("window", undefined);
    try {
      expect(readDummyTaskCards()).toEqual([]);
      expect(() => storeDummyTaskCards([makeTask("no-window")])).not.toThrow();
      expect(() => setDummyTaskCardsModeEnabled(true)).not.toThrow();
      expect(readDummyTaskCardsModeEnabled()).toBe(false);
      expect(() => clearStoredDummyTaskCards()).not.toThrow();
    } finally {
      vi.stubGlobal("window", originalWindow);
    }
  });
});
