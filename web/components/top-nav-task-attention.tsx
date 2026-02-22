"use client";

import Link from "next/link";
import { usePathname } from "next/navigation";
import { createPortal } from "react-dom";
import { useCallback, useEffect, useMemo, useRef, useState } from "react";

import { useLocale } from "@/components/locale-provider";
import { clientApiBaseUrl } from "@/lib/api-base";
import { ROUTES } from "@/lib/routes";
import {
  readDummyTaskCards,
  TRAIN_DUMMY_TASKS_ENABLED,
  TRAIN_DUMMY_TASKS_EVENT,
  TRAIN_DUMMY_TASKS_STORAGE_KEY,
} from "@/lib/train/dummy-task-cards";
import type { TrainTaskSummary } from "@/lib/types";

const ATTENTION_POLL_MS = 60000;
const ATTENTION_TASK_FETCH_LIMIT = 200;
const ATTENTION_STORAGE_PREFIX = "bominal_train_attention_seen_v1";
const TERMINAL_TASK_STATES = new Set(["COMPLETED", "CANCELLED", "EXPIRED", "FAILED"]);
const MOBILE_DRAWER_HIDDEN_TRANSLATE = "calc(-100dvh - 24px)";

function isRecord(value: unknown): value is Record<string, unknown> {
  return typeof value === "object" && value !== null && !Array.isArray(value);
}

function isAwaitingPayment(task: Pick<TrainTaskSummary, "ticket_status" | "ticket_paid">): boolean {
  return task.ticket_status === "awaiting_payment" && task.ticket_paid !== true;
}

function needsAttention(task: TrainTaskSummary): boolean {
  if (TERMINAL_TASK_STATES.has(task.state)) return true;
  return isAwaitingPayment(task);
}

function extractTaskIdFromPath(pathname: string | null): string | null {
  if (!pathname) return null;
  const match = pathname.match(/^\/modules\/train\/tasks\/([^/]+)$/);
  return match?.[1] ?? null;
}

function attentionStorageKey(userId: string): string {
  return `${ATTENTION_STORAGE_PREFIX}:${userId}`;
}

function readSeenTaskIds(userId: string): Set<string> {
  if (typeof window === "undefined") return new Set();
  try {
    const raw = window.localStorage.getItem(attentionStorageKey(userId));
    if (!raw) return new Set();
    const parsed = JSON.parse(raw) as unknown;
    if (!Array.isArray(parsed)) return new Set();
    return new Set(parsed.filter((value) => typeof value === "string"));
  } catch {
    return new Set();
  }
}

function formatTaskRoute(task: TrainTaskSummary): string {
  if (!isRecord(task.spec_json)) return "-";
  const dep = typeof task.spec_json.dep === "string" ? task.spec_json.dep : null;
  const arr = typeof task.spec_json.arr === "string" ? task.spec_json.arr : null;
  if (!dep || !arr) return "-";
  return `${dep} -> ${arr}`;
}

function formatTaskSchedule(task: TrainTaskSummary, locale: string): string | null {
  if (!isRecord(task.spec_json)) return null;
  const rankedRaw = Array.isArray(task.spec_json.selected_trains_ranked) ? task.spec_json.selected_trains_ranked : [];
  const ranked = rankedRaw
    .filter(isRecord)
    .map((row) => ({
      rank: typeof row.rank === "number" ? row.rank : Number(row.rank),
      departureAt: typeof row.departure_at === "string" ? row.departure_at : "",
    }))
    .filter((row) => Number.isFinite(row.rank) && row.departureAt.length > 0)
    .sort((a, b) => a.rank - b.rank);
  if (ranked.length === 0) return null;
  const parsed = new Date(ranked[0].departureAt);
  if (Number.isNaN(parsed.getTime())) return null;
  return new Intl.DateTimeFormat(locale === "ko" ? "ko-KR" : "en-US", {
    timeZone: "Asia/Seoul",
    month: "2-digit",
    day: "2-digit",
    hour: "2-digit",
    minute: "2-digit",
    hour12: false,
  }).format(parsed);
}

function formatUpdatedAt(updatedAt: string, locale: string): string {
  const parsed = new Date(updatedAt);
  if (Number.isNaN(parsed.getTime())) return updatedAt;
  return new Intl.DateTimeFormat(locale === "ko" ? "ko-KR" : "en-US", {
    timeZone: "Asia/Seoul",
    month: "2-digit",
    day: "2-digit",
    hour: "2-digit",
    minute: "2-digit",
    hour12: false,
  }).format(parsed);
}

function compareAttentionTasks(a: TrainTaskSummary, b: TrainTaskSummary): number {
  const aUpdated = new Date(a.updated_at).getTime();
  const bUpdated = new Date(b.updated_at).getTime();
  const aa = Number.isNaN(aUpdated) ? 0 : aUpdated;
  const bb = Number.isNaN(bUpdated) ? 0 : bUpdated;
  return bb - aa;
}

export function TopNavTaskAttention({
  userId,
  displayName,
}: {
  userId: string;
  displayName: string;
}) {
  const { locale, t } = useLocale();
  const pathname = usePathname();
  const rootRef = useRef<HTMLDivElement | null>(null);
  const mobilePanelRef = useRef<HTMLDivElement | null>(null);
  const mobileDragStartYRef = useRef<number | null>(null);
  const mobileDragYRef = useRef(0);
  const [menuOpen, setMenuOpen] = useState(false);
  const [portalReady, setPortalReady] = useState(false);
  const [mobileDragY, setMobileDragY] = useState(0);
  const [mobileDragging, setMobileDragging] = useState(false);
  const [mobileDrawerTop, setMobileDrawerTop] = useState(0);
  const [attentionTasks, setAttentionTasks] = useState<TrainTaskSummary[]>([]);
  const [attentionTasksLoaded, setAttentionTasksLoaded] = useState(false);
  const [seenTaskIds, setSeenTaskIds] = useState<Set<string>>(new Set());
  const [selectionMode, setSelectionMode] = useState(false);
  const [selectedTaskIds, setSelectedTaskIds] = useState<Set<string>>(new Set());

  const persistSeenTaskIds = useCallback((next: Set<string>) => {
    if (typeof window === "undefined") return;
    try {
      window.localStorage.setItem(attentionStorageKey(userId), JSON.stringify(Array.from(next)));
    } catch {
      // Best-effort only.
    }
  }, [userId]);

  const markTasksAsSeen = useCallback((taskIds: string[]) => {
    if (taskIds.length === 0) return;
    setSeenTaskIds((current) => {
      const next = new Set(current);
      let changed = false;
      for (const taskId of taskIds) {
        if (!next.has(taskId)) {
          next.add(taskId);
          changed = true;
        }
      }
      if (!changed) return current;
      persistSeenTaskIds(next);
      return next;
    });
  }, [persistSeenTaskIds]);

  const markTaskAsSeen = useCallback((taskId: string) => {
    markTasksAsSeen([taskId]);
  }, [markTasksAsSeen]);

  const loadAttentionTasks = useCallback(async () => {
    let apiTasks: TrainTaskSummary[] = [];
    try {
      const response = await fetch(
        `${clientApiBaseUrl}/api/train/tasks?status=all&limit=${ATTENTION_TASK_FETCH_LIMIT}`,
        {
          credentials: "include",
          cache: "no-store",
        },
      );
      if (response.ok) {
        const payload = (await response.json()) as { tasks: TrainTaskSummary[] };
        apiTasks = payload.tasks;
      }
    } catch {
      // Fall back to local dummy task source (if present).
    }

    const merged = new Map<string, TrainTaskSummary>();
    for (const task of apiTasks) {
      merged.set(task.id, task);
    }

    if (TRAIN_DUMMY_TASKS_ENABLED) {
      for (const task of readDummyTaskCards()) {
        merged.set(task.id, task);
      }
    }

    const next = Array.from(merged.values()).filter(needsAttention).sort(compareAttentionTasks);
    setAttentionTasks(next);
    setAttentionTasksLoaded(true);
  }, []);

  useEffect(() => {
    setSeenTaskIds(readSeenTaskIds(userId));
  }, [userId]);

  useEffect(() => {
    void loadAttentionTasks();
    const interval = window.setInterval(() => {
      void loadAttentionTasks();
    }, ATTENTION_POLL_MS);
    return () => window.clearInterval(interval);
  }, [loadAttentionTasks]);

  useEffect(() => {
    if (!TRAIN_DUMMY_TASKS_ENABLED) return;

    const onDummyTasksUpdated = () => {
      void loadAttentionTasks();
    };
    const onStorage = (event: StorageEvent) => {
      if (event.key !== TRAIN_DUMMY_TASKS_STORAGE_KEY) return;
      void loadAttentionTasks();
    };

    window.addEventListener(TRAIN_DUMMY_TASKS_EVENT, onDummyTasksUpdated);
    window.addEventListener("storage", onStorage);
    return () => {
      window.removeEventListener(TRAIN_DUMMY_TASKS_EVENT, onDummyTasksUpdated);
      window.removeEventListener("storage", onStorage);
    };
  }, [loadAttentionTasks]);

  useEffect(() => {
    const taskId = extractTaskIdFromPath(pathname);
    if (!taskId) return;
    markTaskAsSeen(taskId);
  }, [pathname, markTaskAsSeen]);

  useEffect(() => {
    if (!attentionTasksLoaded) return;
    const activeIds = new Set(attentionTasks.map((task) => task.id));
    setSeenTaskIds((current) => {
      const next = new Set(Array.from(current).filter((id) => activeIds.has(id)));
      if (next.size === current.size) return current;
      persistSeenTaskIds(next);
      return next;
    });
  }, [attentionTasks, attentionTasksLoaded, persistSeenTaskIds]);

  const unseenTasks = useMemo(
    () => attentionTasks.filter((task) => !seenTaskIds.has(task.id)),
    [attentionTasks, seenTaskIds],
  );
  const unseenCount = unseenTasks.length;

  useEffect(() => {
    const unseenIds = new Set(unseenTasks.map((task) => task.id));
    setSelectedTaskIds((current) => {
      const next = new Set(Array.from(current).filter((id) => unseenIds.has(id)));
      if (next.size === current.size) return current;
      return next;
    });
    if (selectionMode && unseenTasks.length === 0) {
      setSelectionMode(false);
    }
  }, [selectionMode, unseenTasks]);

  const toggleSelectedTask = (taskId: string) => {
    setSelectedTaskIds((current) => {
      const next = new Set(current);
      if (next.has(taskId)) {
        next.delete(taskId);
      } else {
        next.add(taskId);
      }
      return next;
    });
  };

  const onClearAll = () => {
    if (unseenTasks.length === 0) return;
    const confirmed = window.confirm(t("nav.taskAttentionClearConfirm"));
    if (!confirmed) return;
    markTasksAsSeen(unseenTasks.map((task) => task.id));
    setSelectionMode(false);
    setSelectedTaskIds(new Set());
  };

  const onConfirmSelection = () => {
    if (selectedTaskIds.size === 0) return;
    markTasksAsSeen(Array.from(selectedTaskIds));
    setSelectionMode(false);
    setSelectedTaskIds(new Set());
  };

  const onCancelSelection = () => {
    setSelectionMode(false);
    setSelectedTaskIds(new Set());
  };

  const closeMenu = useCallback(() => {
    setMenuOpen(false);
    setSelectionMode(false);
    setSelectedTaskIds(new Set());
    setMobileDragY(0);
    mobileDragYRef.current = 0;
    setMobileDragging(false);
    mobileDragStartYRef.current = null;
  }, []);
  const openMenu = useCallback(() => {
    setMenuOpen(true);
  }, []);

  useEffect(() => {
    setPortalReady(true);
  }, []);

  useEffect(() => {
    closeMenu();
  }, [pathname, closeMenu]);

  useEffect(() => {
    if (!menuOpen) return;

    const handleDocumentClick = (event: MouseEvent) => {
      const target = event.target as Node | null;
      if (!target) return;
      const clickedTrigger = rootRef.current?.contains(target) ?? false;
      const clickedMobilePanel = mobilePanelRef.current?.contains(target) ?? false;
      if (!clickedTrigger && !clickedMobilePanel) closeMenu();
    };
    const handleKeyDown = (event: KeyboardEvent) => {
      if (event.key === "Escape") {
        closeMenu();
      }
    };

    document.addEventListener("click", handleDocumentClick);
    document.addEventListener("keydown", handleKeyDown);
    return () => {
      document.removeEventListener("click", handleDocumentClick);
      document.removeEventListener("keydown", handleKeyDown);
    };
  }, [menuOpen, closeMenu]);

  useEffect(() => {
    const updateMobileDrawerTop = () => {
      const header = rootRef.current?.closest("header");
      if (!header) {
        setMobileDrawerTop(0);
        return;
      }
      const nextTop = Math.max(0, Math.round(header.getBoundingClientRect().bottom));
      setMobileDrawerTop(nextTop);
    };

    updateMobileDrawerTop();
    if (!menuOpen) return;

    window.addEventListener("resize", updateMobileDrawerTop);
    window.addEventListener("orientationchange", updateMobileDrawerTop);
    return () => {
      window.removeEventListener("resize", updateMobileDrawerTop);
      window.removeEventListener("orientationchange", updateMobileDrawerTop);
    };
  }, [menuOpen]);

  const attentionMenuContent = (
    <>
      <p className="px-2 pb-1 text-xs font-semibold uppercase tracking-[0.08em] text-blossom-500">
        {t("nav.taskAttention")}
      </p>
      {unseenTasks.length === 0 ? (
        <p className="rounded-xl px-3 py-2 text-sm text-slate-500">{t("nav.taskAttentionEmpty")}</p>
      ) : (
        <>
          <ul className="max-h-80 space-y-2 overflow-auto pr-1">
            {unseenTasks.map((task) => {
              const isSelected = selectedTaskIds.has(task.id);
              return (
                <li key={task.id}>
                  {selectionMode ? (
                    <button
                      type="button"
                      onClick={() => toggleSelectedTask(task.id)}
                      className={`block w-full rounded-xl border px-3 py-2 text-left transition ${
                        isSelected
                          ? "border-blossom-300 bg-blossom-50"
                          : "border-blossom-100 bg-white hover:bg-blossom-50"
                      }`}
                    >
                      <div className="flex items-start justify-between gap-2">
                        <div className="flex items-center justify-start gap-1.5">
                          <span className="text-xs font-semibold text-slate-700">{task.state}</span>
                          {isAwaitingPayment(task) ? (
                            <span className="rounded-full bg-amber-100 px-2 py-0.5 text-[10px] font-medium text-amber-700">
                              {t("train.badge.awaitingPayment")}
                            </span>
                          ) : null}
                        </div>
                        <span
                          className={`inline-flex h-4 w-4 items-center justify-center rounded-full border text-[10px] ${
                            isSelected
                              ? "border-blossom-500 bg-blossom-500 text-white"
                              : "border-slate-300 bg-white text-transparent"
                          }`}
                        >
                          ✓
                        </span>
                      </div>
                      <p className="mt-1 text-xs text-slate-600">{formatTaskRoute(task)}</p>
                      <p className="mt-0.5 text-[11px] text-slate-500">
                        {formatTaskSchedule(task, locale) ?? formatUpdatedAt(task.updated_at, locale)}
                      </p>
                    </button>
                  ) : (
                    <Link
                      href={`${ROUTES.modules.train}/tasks/${task.id}`}
                      onClick={() => {
                        markTaskAsSeen(task.id);
                        closeMenu();
                      }}
                      className="block rounded-xl border border-blossom-100 bg-white px-3 py-2 transition hover:bg-blossom-50"
                    >
                      <div className="flex items-center justify-start gap-1.5">
                        <span className="text-xs font-semibold text-slate-700">{task.state}</span>
                        {isAwaitingPayment(task) ? (
                          <span className="rounded-full bg-amber-100 px-2 py-0.5 text-[10px] font-medium text-amber-700">
                            {t("train.badge.awaitingPayment")}
                          </span>
                        ) : null}
                      </div>
                      <p className="mt-1 text-xs text-slate-600">{formatTaskRoute(task)}</p>
                      <p className="mt-0.5 text-[11px] text-slate-500">
                        {formatTaskSchedule(task, locale) ?? formatUpdatedAt(task.updated_at, locale)}
                      </p>
                    </Link>
                  )}
                </li>
              );
            })}
          </ul>
          <div className="mt-3 border-t border-blossom-100 pb-6 pt-3 md:pb-2">
            {selectionMode ? (
              <div className="flex items-center justify-end gap-3">
                <button
                  type="button"
                  onClick={onConfirmSelection}
                  disabled={selectedTaskIds.size === 0}
                  className={`rounded-lg px-3 py-1.5 text-xs font-medium transition ${
                    selectedTaskIds.size === 0
                      ? "cursor-not-allowed border border-slate-200 bg-slate-100 text-slate-400"
                      : "border border-emerald-200 bg-emerald-50 text-emerald-700 hover:bg-emerald-100"
                  }`}
                >
                  {t("nav.taskAttentionConfirm")}
                </button>
                <button
                  type="button"
                  onClick={onCancelSelection}
                  className="rounded-lg border border-slate-200 bg-white px-3 py-1.5 text-xs font-medium text-slate-700 transition hover:bg-slate-50"
                >
                  {t("nav.taskAttentionCancel")}
                </button>
              </div>
            ) : (
              <div className="flex items-center justify-end gap-3">
                <button
                  type="button"
                  onClick={() => {
                    setSelectionMode(true);
                    setSelectedTaskIds(new Set());
                  }}
                  className="rounded-lg border border-blossom-200 bg-blossom-50 px-3 py-1.5 text-xs font-medium text-blossom-700 transition hover:bg-blossom-100"
                >
                  {t("nav.taskAttentionSelectMarkRead")}
                </button>
                <button
                  type="button"
                  onClick={onClearAll}
                  className="rounded-lg border border-rose-200 bg-rose-50 px-3 py-1.5 text-xs font-medium text-rose-700 transition hover:bg-rose-100"
                >
                  {t("nav.taskAttentionClear")}
                </button>
              </div>
            )}
          </div>
        </>
      )}
    </>
  );

  const onMobileHandleTouchStart = (event: React.TouchEvent<HTMLDivElement>) => {
    if (!menuOpen) return;
    const panel = mobilePanelRef.current;
    if (!panel) return;
    if (panel.scrollTop > 0) return;
    const touch = event.touches[0];
    if (!touch) return;
    const panelRect = panel.getBoundingClientRect();
    // Reserve swipe-close gesture for touches near the bottom handle area.
    if (touch.clientY < panelRect.bottom - 140) return;
    mobileDragStartYRef.current = touch.clientY;
    mobileDragYRef.current = 0;
    setMobileDragging(true);
  };

  const onMobileHandleTouchMove = (event: React.TouchEvent<HTMLDivElement>) => {
    if (!menuOpen) return;
    const startY = mobileDragStartYRef.current;
    if (startY == null) return;
    const touch = event.touches[0];
    if (!touch) return;

    const delta = touch.clientY - startY;
    if (delta >= 0) {
      mobileDragYRef.current = 0;
      setMobileDragY(0);
      return;
    }

    const clamped = Math.max(delta, -360);
    mobileDragYRef.current = clamped;
    setMobileDragY(clamped);
    event.preventDefault();
  };

  const onMobileHandleTouchEnd = () => {
    if (!menuOpen) return;
    if (mobileDragStartYRef.current == null) return;

    const shouldClose = mobileDragYRef.current <= -90;
    mobileDragStartYRef.current = null;
    setMobileDragging(false);
    if (shouldClose) {
      closeMenu();
      return;
    }
    mobileDragYRef.current = 0;
    setMobileDragY(0);
  };

  const mobileDrawer = portalReady
    ? createPortal(
        <>
          <button
            type="button"
            aria-label={t("common.close")}
            onClick={closeMenu}
            className={`fixed inset-x-0 bottom-0 z-[90] bg-slate-900/45 transition-opacity duration-350 ease-out md:hidden ${
              menuOpen ? "pointer-events-auto opacity-100" : "pointer-events-none opacity-0"
            }`}
            style={{ top: mobileDrawerTop, touchAction: "none" }}
            onTouchMove={(event) => {
              event.preventDefault();
            }}
          />

          <div
            className={`fixed inset-x-0 bottom-0 z-[100] will-change-transform md:hidden ${
              menuOpen ? "pointer-events-auto" : "pointer-events-none"
            }`}
            style={{
              top: mobileDrawerTop,
              transform: menuOpen
                ? `translate3d(0, ${mobileDragY}px, 0)`
                : `translate3d(0, ${MOBILE_DRAWER_HIDDEN_TRANSLATE}, 0)`,
              transition: mobileDragging ? "none" : "transform 420ms cubic-bezier(0.22, 1, 0.36, 1)",
            }}
          >
            <div
              ref={mobilePanelRef}
              className="absolute inset-0 overflow-hidden rounded-b-2xl border-b border-blossom-100 bg-white"
              onTouchStart={onMobileHandleTouchStart}
              onTouchMove={onMobileHandleTouchMove}
              onTouchEnd={onMobileHandleTouchEnd}
              onTouchCancel={onMobileHandleTouchEnd}
            >
              <div
                className="min-h-full overflow-y-auto p-2 pb-24 pt-4"
                style={{ overscrollBehavior: "contain" }}
              >
                {attentionMenuContent}
                <div
                  className="absolute inset-x-0 bottom-0 z-10 flex justify-center border-t border-blossom-100/80 bg-white/95 pb-[calc(env(safe-area-inset-bottom,0px)+10px)] pt-3 backdrop-blur"
                  style={{ touchAction: "pan-y" }}
                >
                  <span className="h-1.5 w-12 rounded-full bg-slate-300" />
                </div>
              </div>
            </div>
          </div>
        </>,
        document.body,
      )
    : null;

  return (
    <div ref={rootRef} className="relative">
      <button
        type="button"
        title={displayName}
        aria-label={t("nav.taskAttention")}
        aria-expanded={menuOpen}
        onClick={() => {
          if (menuOpen) {
            closeMenu();
            return;
          }
          openMenu();
        }}
        className="relative inline-flex min-h-11 max-w-[14rem] cursor-pointer list-none items-center justify-center rounded-full border border-blossom-200 bg-white px-3 py-1 text-sm font-medium text-slate-700 shadow-sm transition hover:bg-blossom-50 focus:outline-none focus:ring-2 focus:ring-blossom-100"
      >
        <span className="max-w-full truncate">{displayName}</span>
        {unseenCount > 0 ? (
          <span className="absolute -right-1 -top-1 inline-flex min-h-4 min-w-4 items-center justify-center rounded-full border border-white bg-rose-500 px-1 text-[10px] font-semibold leading-none text-white shadow-sm">
            {unseenCount}
          </span>
        ) : null}
      </button>

      {mobileDrawer}

      <div
        className={`absolute -right-[56px] z-30 mt-2 hidden w-56 origin-top overflow-hidden rounded-2xl border border-blossom-100 bg-white transition-all duration-300 ease-[cubic-bezier(0.22,1,0.36,1)] md:block ${
          menuOpen
            ? "max-h-[32rem] translate-y-0 opacity-100 shadow-[0_14px_28px_-18px_rgba(15,23,42,0.45)]"
            : "pointer-events-none max-h-0 -translate-y-2 opacity-0 shadow-none"
        }`}
      >
        <div className="p-2">{attentionMenuContent}</div>
      </div>
    </div>
  );
}
