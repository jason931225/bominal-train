import type { TrainTaskSummary } from "@/lib/types";

export const TRAIN_DUMMY_TASKS_ENABLED = process.env.NODE_ENV !== "production";
export const TRAIN_DUMMY_TASKS_STORAGE_KEY = "bominal_train_dummy_task_cards_v1";
export const TRAIN_DUMMY_TASKS_EVENT = "bominal:train-dummy-task-cards-change";

function isRecord(value: unknown): value is Record<string, unknown> {
  return typeof value === "object" && value !== null && !Array.isArray(value);
}

function isDummyTaskSummary(value: unknown): value is TrainTaskSummary {
  if (!isRecord(value)) return false;
  return (
    typeof value.id === "string" &&
    typeof value.state === "string" &&
    typeof value.created_at === "string" &&
    typeof value.updated_at === "string" &&
    isRecord(value.spec_json)
  );
}

function emitDummyTaskCardsChanged(): void {
  if (typeof window === "undefined") return;
  window.dispatchEvent(new CustomEvent(TRAIN_DUMMY_TASKS_EVENT));
}

export function readDummyTaskCards(): TrainTaskSummary[] {
  if (typeof window === "undefined") return [];
  try {
    const raw = window.localStorage.getItem(TRAIN_DUMMY_TASKS_STORAGE_KEY);
    if (!raw) return [];
    const parsed = JSON.parse(raw) as unknown;
    if (!Array.isArray(parsed)) return [];
    return parsed.filter(isDummyTaskSummary);
  } catch {
    return [];
  }
}

export function storeDummyTaskCards(tasks: TrainTaskSummary[]): void {
  if (typeof window === "undefined") return;
  try {
    window.localStorage.setItem(TRAIN_DUMMY_TASKS_STORAGE_KEY, JSON.stringify(tasks));
  } catch {
    // Best-effort only.
  }
  emitDummyTaskCardsChanged();
}

export function clearStoredDummyTaskCards(): void {
  if (typeof window === "undefined") return;
  try {
    window.localStorage.removeItem(TRAIN_DUMMY_TASKS_STORAGE_KEY);
  } catch {
    // Best-effort only.
  }
  emitDummyTaskCardsChanged();
}
